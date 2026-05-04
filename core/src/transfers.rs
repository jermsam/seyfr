pub use crate::walker::collect_files;

use std::path::PathBuf;

use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{BlobsProtocol, BlobFormat, ticket::BlobTicket};
use iroh_blobs::format::collection::Collection;
use iroh_blobs::hashseq::HashSeq;
use crate::errors::SeyfrError;
use crate::progress::ProgressSink;
use futures_util::stream::{self, StreamExt};
use path_jail::Jail;

// Security constants
/// Maximum size for single file download (1 GB)
const MAX_FILE_SIZE: u64 = 1_073_741_824;

/// Maximum size for collection download (10 GB)
const MAX_COLLECTION_SIZE: u64 = 10_737_418_240;

/// Maximum number of files in a collection
const MAX_FILES_IN_COLLECTION: u64 = 10_000;

/// Extract progress bytes from DownloadProgressItem::Progress(u64)
fn parse_progress_bytes(event_str: &str) -> Option<u64> {
    // Format: "Progress(12345)"
    if !event_str.starts_with("Progress(") {
        return None;
    }
    
    let bytes_str = event_str
        .strip_prefix("Progress(")?
        .strip_suffix(")")?
        .trim();
    
    bytes_str.parse::<u64>().ok()
}

/// Get the total size of a blob from the store
async fn get_blob_size(
    blobs: &iroh_blobs::api::blobs::Blobs,
    hash: iroh_blobs::Hash,
) -> Option<u64> {
    match blobs.status(hash).await {
        Ok(iroh_blobs::api::blobs::BlobStatus::Complete { size }) => Some(size),
        Ok(iroh_blobs::api::blobs::BlobStatus::Partial { size }) => size,
        _ => None,
    }
}

/// Validate and create a secure jail for the destination directory
/// 
/// Uses path_jail to prevent path traversal attacks and ensure files
/// are written within the intended destination.
fn validate_dest_dir(dest_dir: &str) -> Result<Jail, SeyfrError> {
    // Create jail - this validates the path and resolves symlinks
    Jail::new(dest_dir).map_err(|e| SeyfrError::InvalidPath {
        message: format!("invalid destination directory: {}", e),
    })
}

/// Shared inner state for the transfer engine.
pub struct TransferEngine {
    pub endpoint: Endpoint,
    pub router: Router,
    pub blobs: BlobsProtocol,
}

/// Validates that `path` points to a regular file and extracts its display name.
///
/// Returns `(PathBuf, display_name)` on success, or a `SeyfrError` if the path
/// is missing, not a file, or otherwise inaccessible.
fn resolve_file_path(path: &str) -> Result<(PathBuf, String), SeyfrError> {
    let src = PathBuf::from(path);
    let metadata = std::fs::metadata(&src).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SeyfrError::FileNotFound {
                path: path.to_string(),
            }
        } else {
            SeyfrError::from(e)
        }
    })?;
    if !metadata.is_file() {
        return Err(SeyfrError::Io {
            message: format!("expected a file, got non-file path: {}", path),
        });
    }
    let file_name = src
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".to_string());
    Ok((src, file_name))
}

impl TransferEngine {
    /// Send a single file. Returns a compact `BlobTicket` (Raw format).
    pub async fn send_file(
        &self,
        path: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<String, SeyfrError> {
        let (src, file_name) = resolve_file_path(path)?;

        if let Some(p) = progress {
            p.on_file_start(file_name.clone(), 1, 1);
        }

        let tag = self.blobs.add_path(src).await.map_err(|e| SeyfrError::Store {
            message: format!("failed to import file: {}", e),
        })?;

        let ticket = BlobTicket::new(
            self.router.endpoint().addr(),
            tag.hash,
            BlobFormat::Raw,
        );

        if let Some(p) = progress {
            p.on_file_complete(file_name, 1, 1);
            p.on_complete("File shared successfully".to_string());
        }

        Ok(ticket.to_string())
    }

    /// Unified send: auto-detects file vs. folder and delegates.
    pub async fn send(
        &self,
        path: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<String, SeyfrError> {
        let src = PathBuf::from(path);
        let metadata = std::fs::metadata(&src).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SeyfrError::FileNotFound {
                    path: path.to_string(),
                }
            } else {
                SeyfrError::from(e)
            }
        })?;

        if metadata.is_dir() {
            self.send_folder(path, progress).await
        } else {
            self.send_file(path, progress).await
        }
    }
}

/// Validates that `path` points to an existing directory.
///
/// Returns `PathBuf` on success, or a `SeyfrError` if the path is missing,
/// not a directory, or otherwise inaccessible.
fn resolve_folder_path(path: &str) -> Result<PathBuf, SeyfrError> {
    let src = PathBuf::from(path);
    let metadata = std::fs::metadata(&src).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SeyfrError::FileNotFound {
                path: path.to_string(),
            }
        } else {
            SeyfrError::from(e)
        }
    })?;
    if !metadata.is_dir() {
        return Err(SeyfrError::NotADirectory {
            path: path.to_string(),
        });
    }
    Ok(src)
}

impl TransferEngine {
    /// Send a folder. Returns a `BlobTicket` (HashSeq / Collection format).
    pub async fn send_folder(
        &self,
        path: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<String, SeyfrError> {
        let src = resolve_folder_path(path)?;

        let files = collect_files(&src).await?;
        if files.is_empty() {
            return Err(SeyfrError::EmptyFolder { path: path.to_string() });
        }

        let total = files.len() as u64;

        // Fire all file-start notifications upfront since imports will run in parallel.
        if let Some(p) = progress {
            for (idx, file_path) in files.iter().enumerate() {
                let rel = file_path
                    .strip_prefix(&src)
                    .expect("collect_files only returns paths under the root");
                let rel_str = rel.to_string_lossy().to_string();
                p.on_file_start(rel_str, (idx + 1) as u64, total);
            }
        }

        let mut collection = Collection::default();

        let results: Vec<_> = stream::iter(files)
            .map(|file_path| async {
                let rel = file_path
                    .strip_prefix(&src)
                    .expect("collect_files only returns paths under the root");
                let rel_str = rel.to_string_lossy().to_string();

                let tag = self.blobs.add_path(file_path).await.map_err(|e| SeyfrError::Store {
                    message: format!("failed to import '{}': {}", rel_str, e),
                })?;

                Ok::<_, SeyfrError>((rel_str, tag.hash))
            })
            .buffer_unordered(4)
            .collect()
            .await;

        let mut completed = 0u64;
        for res in results {
            let (rel_str, hash) = res?;
            completed += 1;
            collection.push(rel_str.clone(), hash);

            if let Some(p) = progress {
                p.on_file_complete(rel_str, completed, total);
            }
        }

        let store = self.blobs.store();
        let mut root_tag = collection.store(&store).await.map_err(|e| SeyfrError::Store {
            message: format!("failed to store collection: {}", e),
        })?;
        root_tag.leak();

        let ticket = BlobTicket::new(
            self.router.endpoint().addr(),
            root_tag.hash(),
            BlobFormat::HashSeq,
        );

        if let Some(p) = progress {
            p.on_complete(format!("Folder shared: {} items", total));
        }

        Ok(ticket.to_string())
    }

    /// Receive from a ticket. Supports both Raw (single file) and HashSeq (folder).
    /// 
    /// # Progress Reporting
    /// - **File-level**: `on_file_start`, `on_file_complete`, `on_complete` 
    /// - **Byte-level**: `on_file_progress(name, current_bytes, total_bytes)` - full progress
    /// 
    /// Before each download, queries `store.blobs().status(hash)` to get total size,
    /// then uses `DownloadProgressItem::Progress(u64)` stream for current bytes.
    /// This enables proper progress bars with percentages in the UI.
    pub async fn receive(
        &self,
        ticket_str: &str,
        dest_dir: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<(), SeyfrError> {
        let ticket: BlobTicket = ticket_str.parse().map_err(|e| SeyfrError::InvalidTicket {
            message: format!("{}", e),
        })?;

        // Validate destination directory and create secure jail
        let jail = validate_dest_dir(dest_dir)?;
        
        // Ensure destination exists
        tokio::fs::create_dir_all(jail.root()).await.map_err(SeyfrError::from)?;

        let store = self.blobs.store();
        let downloader = store.downloader(&self.endpoint);

        match ticket.format() {
            BlobFormat::HashSeq => {
                // Folder transfer: first download the collection hash sequence
                // Query size before downloading for proper progress reporting
                let collection_size = get_blob_size(store.blobs(), ticket.hash()).await.unwrap_or(0);
                
                // Validate collection size
                if collection_size > MAX_COLLECTION_SIZE {
                    return Err(SeyfrError::FileTooLarge {
                        size: collection_size,
                        max: MAX_COLLECTION_SIZE,
                    });
                }
                
                let download_progress = downloader.download(ticket.hash(), Some(ticket.addr().id));
                let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                    message: format!("failed to start download: {}", e),
                })?;
                
                while let Some(event) = stream.next().await {
                    let event_str = format!("{:?}", event);
                    
                    if event_str.starts_with("Progress(") {
                        // Progress event - extract bytes downloaded and report with total
                        if let Some(p) = progress {
                            if let Some(current) = parse_progress_bytes(&event_str) {
                                p.on_file_progress("collection".to_string(), current, collection_size);
                            }
                        }
                    } else if event_str.contains("AllDone") || event_str.contains("Done") {
                        break;
                    } else if event_str.contains("Error") || event_str.contains("Abort") {
                        return Err(SeyfrError::Network {
                            message: format!("download failed: {}", event_str),
                        });
                    }
                }

                // Read the hash sequence bytes
                let hs_bytes = store.blobs().get_bytes(ticket.hash()).await.map_err(|e| SeyfrError::Store {
                    message: format!("failed to read collection: {}", e),
                })?;

                let hash_seq = HashSeq::try_from(hs_bytes).map_err(|e| SeyfrError::Store {
                    message: format!("invalid hash sequence: {}", e),
                })?;

                // First hash is the metadata
                let mut hashes = hash_seq.iter();
                let meta_hash = hashes.next().ok_or_else(|| SeyfrError::Store {
                    message: "empty collection".to_string(),
                })?;

                // Download and parse metadata
                let meta_size = get_blob_size(store.blobs(), meta_hash).await.unwrap_or(0);
                
                let download_progress = downloader.download(meta_hash, Some(ticket.addr().id));
                let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                    message: format!("failed to start metadata download: {}", e),
                })?;
                
                while let Some(event) = stream.next().await {
                    let event_str = format!("{:?}", event);
                    
                    if event_str.starts_with("Progress(") {
                        // Metadata download progress with proper total
                        if let Some(p) = progress {
                            if let Some(current) = parse_progress_bytes(&event_str) {
                                p.on_file_progress("metadata".to_string(), current, meta_size);
                            }
                        }
                    } else if event_str.contains("AllDone") || event_str.contains("Done") {
                        break;
                    } else if event_str.contains("Error") || event_str.contains("Abort") {
                        return Err(SeyfrError::Network {
                            message: format!("failed to download collection metadata: {}", event_str),
                        });
                    }
                }

                let meta_bytes = store.blobs().get_bytes(meta_hash).await.map_err(|e| SeyfrError::Store {
                    message: format!("failed to read metadata: {}", e),
                })?;

                let meta: iroh_blobs::format::collection::CollectionMeta = postcard::from_bytes(&meta_bytes).map_err(|e| SeyfrError::Store {
                    message: format!("failed to parse metadata: {}", e),
                })?;

                let total = meta.names().len() as u64;
                
                // Validate file count
                if total > MAX_FILES_IN_COLLECTION {
                    return Err(SeyfrError::TooManyFiles {
                        count: total,
                        max: MAX_FILES_IN_COLLECTION,
                    });
                }

                // Download and export each file
                for (idx, (name, hash)) in meta.names().iter().zip(hashes).enumerate() {
                    // Use jail.join() for secure path validation - prevents path traversal
                    let file_dest = jail.join(name).map_err(|e| SeyfrError::PathTraversal {
                        path: name.clone(),
                        message: format!("{}", e),
                    })?;
                    
                    if let Some(parent) = file_dest.parent() {
                        tokio::fs::create_dir_all(parent).await.map_err(SeyfrError::from)?;
                    }

                    if let Some(p) = progress {
                        p.on_file_start(name.clone(), (idx + 1) as u64, total);
                    }

                    // Download the blob with progress
                    // Query size first for proper progress reporting
                    let file_size = get_blob_size(store.blobs(), hash).await.unwrap_or(0);
                    
                    // Validate file size
                    if file_size > MAX_FILE_SIZE {
                        return Err(SeyfrError::FileTooLarge {
                            size: file_size,
                            max: MAX_FILE_SIZE,
                        });
                    }
                    
                    let download_progress = downloader.download(hash, Some(ticket.addr().id));
                    let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                        message: format!("failed to start download for '{}': {}", name, e),
                    })?;
                    
                    while let Some(event) = stream.next().await {
                        let event_str = format!("{:?}", event);
                        
                        if event_str.starts_with("Progress(") {
                            // File download progress with proper total size
                            if let Some(p) = progress {
                                if let Some(current) = parse_progress_bytes(&event_str) {
                                    p.on_file_progress(name.clone(), current, file_size);
                                }
                            }
                        } else if event_str.contains("AllDone") || event_str.contains("Done") {
                            break;
                        } else if event_str.contains("Error") || event_str.contains("Abort") {
                            return Err(SeyfrError::Network {
                                message: format!("download failed for '{}': {}", name, event_str),
                            });
                        }
                    }

                    // Export to destination
                    store.blobs().export(hash, &file_dest).await.map_err(|e| SeyfrError::Store {
                        message: format!("export failed for '{}': {}", name, e),
                    })?;

                    if let Some(p) = progress {
                        p.on_file_complete(name.clone(), (idx + 1) as u64, total);
                    }
                }

                if let Some(p) = progress {
                    p.on_complete(format!("Folder received: {} items", total));
                }
            }
            BlobFormat::Raw => {
                // Single file transfer
                let hash = ticket.hash();

                if let Some(p) = progress {
                    p.on_file_start("file".to_string(), 1, 1);
                }

                // Query size first for proper progress reporting
                let file_size = get_blob_size(store.blobs(), hash).await.unwrap_or(0);
                
                // Validate file size
                if file_size > MAX_FILE_SIZE {
                    return Err(SeyfrError::FileTooLarge {
                        size: file_size,
                        max: MAX_FILE_SIZE,
                    });
                }
                
                let download_progress = downloader.download(hash, Some(ticket.addr().id));
                let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                    message: format!("failed to start download: {}", e),
                })?;
                
                while let Some(event) = stream.next().await {
                    let event_str = format!("{:?}", event);
                    
                    if event_str.starts_with("Progress(") {
                        // Single file download progress with proper total size
                        if let Some(p) = progress {
                            if let Some(current) = parse_progress_bytes(&event_str) {
                                p.on_file_progress("file".to_string(), current, file_size);
                            }
                        }
                    } else if event_str.contains("AllDone") || event_str.contains("Done") {
                        break;
                    } else if event_str.contains("Error") || event_str.contains("Abort") {
                        return Err(SeyfrError::Network {
                            message: format!("download failed: {}", event_str),
                        });
                    }
                }

                // Use jail.join() for secure path - even for default filename
                let file_dest = jail.join("received_file").map_err(|e| SeyfrError::PathTraversal {
                    path: "received_file".to_string(),
                    message: format!("{}", e),
                })?;
                store.blobs().export(hash, &file_dest).await.map_err(|e| SeyfrError::Store {
                    message: format!("export failed: {}", e),
                })?;

                if let Some(p) = progress {
                    p.on_file_complete("file".to_string(), 1, 1);
                    p.on_complete("File received successfully".to_string());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_file_path_accepts_regular_file() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("data.txt");
        fs::write(&file, "hello").unwrap();
        let (path, name) = resolve_file_path(file.to_str().unwrap()).unwrap();
        assert_eq!(path, file);
        assert_eq!(name, "data.txt");
    }

    #[test]
    fn resolve_file_path_rejects_missing_file() {
        let result = resolve_file_path("/nonexistent/path/to/file.txt");
        assert!(matches!(result, Err(SeyfrError::FileNotFound { .. })));
    }

    #[test]
    fn resolve_file_path_rejects_directory() {
        let temp = tempfile::tempdir().unwrap();
        let result = resolve_file_path(temp.path().to_str().unwrap());
        assert!(matches!(result, Err(SeyfrError::Io { .. })));
    }

    #[test]
    #[cfg(unix)]
    fn resolve_file_path_follows_symlink_to_file() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let real = temp.path().join("real.txt");
        let link = temp.path().join("link.txt");
        fs::write(&real, "data").unwrap();
        symlink(&real, &link).unwrap();
        let (path, name) = resolve_file_path(link.to_str().unwrap()).unwrap();
        assert_eq!(path, link);
        assert_eq!(name, "link.txt");
    }

    #[test]
    #[cfg(unix)]
    fn resolve_file_path_rejects_broken_symlink() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("gone.txt");
        let link = temp.path().join("broken.txt");
        symlink(&target, &link).unwrap();
        let result = resolve_file_path(link.to_str().unwrap());
        assert!(matches!(result, Err(SeyfrError::FileNotFound { .. })));
    }

    #[test]
    fn resolve_folder_path_accepts_directory() {
        let temp = tempfile::tempdir().unwrap();
        let path = resolve_folder_path(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(path, temp.path());
    }

    #[test]
    fn resolve_folder_path_rejects_missing() {
        let result = resolve_folder_path("/nonexistent/path/to/folder");
        assert!(matches!(result, Err(SeyfrError::FileNotFound { .. })));
    }

    #[test]
    fn resolve_folder_path_rejects_file() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("data.txt");
        fs::write(&file, "hello").unwrap();
        let result = resolve_folder_path(file.to_str().unwrap());
        assert!(matches!(result, Err(SeyfrError::NotADirectory { .. })));
    }

    #[test]
    #[cfg(unix)]
    fn resolve_folder_path_rejects_broken_symlink() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("gone");
        let link = temp.path().join("broken");
        symlink(&target, &link).unwrap();
        let result = resolve_folder_path(link.to_str().unwrap());
        assert!(matches!(result, Err(SeyfrError::FileNotFound { .. })));
    }

    /// Unit tests for send/receive logic using in-memory stores.
    /// These tests verify the core logic without network I/O for fast, reliable testing.
    /// 
    /// For end-to-end network tests, see tests/e2e/ directory.
    mod unit_tests {
        use super::*;
        use iroh::{Endpoint, protocol::Router, endpoint::presets};
        use iroh_blobs::store::mem::MemStore;

        /// Helper to create a test TransferEngine with in-memory store (no network I/O)
        async fn create_test_engine_mem() -> Result<TransferEngine, SeyfrError> {
            let store = MemStore::new();
            
            let endpoint = Endpoint::bind(presets::N0).await.map_err(|e| SeyfrError::Network {
                message: format!("failed to bind endpoint: {}", e),
            })?;
            
            let blobs = BlobsProtocol::new(&store, None);
            let router = Router::builder(endpoint.clone())
                .accept(iroh_blobs::ALPN, blobs.clone())
                .spawn();
            
            Ok(TransferEngine {
                endpoint,
                router,
                blobs,
            })
        }

        use crate::test_utils::TestProgress;

        /// Test send_file creates valid ticket and stores blob in memory
        #[tokio::test]
        async fn test_send_file_creates_ticket() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            // Create test file
            let test_file = testdir.path().join("test.txt");
            let test_data = b"Hello, World!";
            fs::write(&test_file, test_data)?;
            
            // Send file with progress tracking
            let progress = TestProgress::default();
            let ticket = engine.send_file(test_file.to_str().unwrap(), Some(&progress)).await?;
            
            // Verify progress callbacks
            assert_eq!(progress.file_starts.lock().unwrap().len(), 1);
            assert_eq!(progress.file_completes.lock().unwrap().len(), 1);
            assert_eq!(progress.completes.lock().unwrap().len(), 1);
            
            // Verify ticket is valid
            let parsed: iroh_blobs::ticket::BlobTicket = ticket.parse()?;
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::Raw);
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test send/receive roundtrip with same engine (in-memory store)
        #[tokio::test]
        async fn test_send_receive_single_file_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            // Create and send test file
            let test_file = testdir.path().join("test.txt");
            let test_data = b"Hello, World! This is a test file.";
            fs::write(&test_file, test_data)?;
            
            let ticket = engine.send_file(test_file.to_str().unwrap(), None).await?;
            
            // Receive to different directory (data already in store)
            let recv_dir = testdir.path().join("recv");
            fs::create_dir_all(&recv_dir)?;
            
            let recv_progress = TestProgress::default();
            engine.receive(&ticket, recv_dir.to_str().unwrap(), Some(&recv_progress)).await?;
            
            // Verify receive progress
            assert_eq!(recv_progress.file_starts.lock().unwrap().len(), 1);
            assert_eq!(recv_progress.file_completes.lock().unwrap().len(), 1);
            assert_eq!(recv_progress.completes.lock().unwrap().len(), 1);
            
            // Verify received file content
            let received_file = recv_dir.join("received_file");
            assert!(received_file.exists());
            let received_data = fs::read(&received_file)?;
            assert_eq!(received_data, test_data);
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test send_folder creates collection and stores all blobs
        #[tokio::test]
        async fn test_send_folder_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            // Create test folder structure
            let send_dir = testdir.path().join("send_folder");
            fs::create_dir_all(&send_dir)?;
            fs::write(send_dir.join("file1.txt"), b"Content of file 1")?;
            fs::write(send_dir.join("file2.txt"), b"Content of file 2")?;
            
            let subdir = send_dir.join("subdir");
            fs::create_dir_all(&subdir)?;
            fs::write(subdir.join("file3.txt"), b"Content of file 3")?;
            
            // Send folder with progress tracking
            let send_progress = TestProgress::default();
            let ticket = engine.send_folder(send_dir.to_str().unwrap(), Some(&send_progress)).await?;
            
            // Verify send progress (3 files)
            assert_eq!(send_progress.file_starts.lock().unwrap().len(), 3);
            assert_eq!(send_progress.file_completes.lock().unwrap().len(), 3);
            assert_eq!(send_progress.completes.lock().unwrap().len(), 1);
            
            // Verify ticket format
            let parsed: iroh_blobs::ticket::BlobTicket = ticket.parse()?;
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::HashSeq);
            
            // Receive folder (data already in store)
            let recv_dir = testdir.path().join("recv_folder");
            fs::create_dir_all(&recv_dir)?;
            let recv_progress = TestProgress::default();
            engine.receive(&ticket, recv_dir.to_str().unwrap(), Some(&recv_progress)).await?;
            
            // Verify receive progress
            assert_eq!(recv_progress.file_starts.lock().unwrap().len(), 3);
            assert_eq!(recv_progress.file_completes.lock().unwrap().len(), 3);
            assert_eq!(recv_progress.completes.lock().unwrap().len(), 1);
            
            // Verify received files
            assert_eq!(fs::read(recv_dir.join("file1.txt"))?, b"Content of file 1");
            assert_eq!(fs::read(recv_dir.join("file2.txt"))?, b"Content of file 2");
            assert_eq!(fs::read(recv_dir.join("subdir/file3.txt"))?, b"Content of file 3");
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test unified send auto-detects file type
        #[tokio::test]
        async fn test_send_unified_detects_file() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            let test_file = testdir.path().join("test.txt");
            fs::write(&test_file, b"test content")?;
            
            let ticket = engine.send(test_file.to_str().unwrap(), None).await?;
            let parsed: iroh_blobs::ticket::BlobTicket = ticket.parse()?;
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::Raw);
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test unified send auto-detects folder type
        #[tokio::test]
        async fn test_send_unified_detects_folder() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            let test_dir = testdir.path().join("test_folder");
            fs::create_dir_all(&test_dir)?;
            fs::write(test_dir.join("file.txt"), b"test")?;
            
            let ticket = engine.send(test_dir.to_str().unwrap(), None).await?;
            let parsed: iroh_blobs::ticket::BlobTicket = ticket.parse()?;
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::HashSeq);
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test receive with invalid ticket format
        #[tokio::test]
        async fn test_receive_invalid_ticket() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            let recv_dir = testdir.path().join("recv");
            fs::create_dir_all(&recv_dir)?;
            
            let result = engine.receive("invalid_ticket", recv_dir.to_str().unwrap(), None).await;
            assert!(matches!(result, Err(SeyfrError::InvalidTicket { .. })));
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }

        /// Test send_folder rejects empty folders
        #[tokio::test]
        async fn test_send_empty_folder_fails() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            let engine = create_test_engine_mem().await?;
            
            let empty_dir = testdir.path().join("empty");
            fs::create_dir_all(&empty_dir)?;
            
            let result = engine.send_folder(empty_dir.to_str().unwrap(), None).await;
            assert!(matches!(result, Err(SeyfrError::EmptyFolder { .. })));
            
            engine.router.shutdown().await?;
            engine.endpoint.close().await;
            Ok(())
        }
    }

    /// Network integration tests - real P2P transfers between separate nodes.
    /// These tests verify the complete network stack works correctly.
    /// 
    /// Run with: cargo test --package seyfr-core network_integration -- --ignored --test-threads=1
    #[cfg(test)]
    mod network_integration {
        use super::*;
        use crate::test_utils::TestProgress;
        use iroh::{Endpoint, protocol::Router, endpoint::presets, RelayMode, address_lookup::MemoryLookup};
        use iroh_blobs::store::fs::FsStore;
        use std::time::Duration;
        
        // Test configuration constants
        const ENDPOINT_READY_DELAY_MS: u64 = 100;
        const SMALL_TRANSFER_TIMEOUT_SECS: u64 = 10;
        const LARGE_TRANSFER_TIMEOUT_SECS: u64 = 30;

        /// Helper to create a test TransferEngine with network capabilities
        async fn create_network_engine(
            store_path: std::path::PathBuf,
            lookup: MemoryLookup,
        ) -> Result<TransferEngine, SeyfrError> {
            let store = FsStore::load(&store_path).await.map_err(|e| SeyfrError::Store {
                message: format!("failed to create store: {}", e),
            })?;
            
            let endpoint = Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Disabled) // Disable relay for local testing
                .address_lookup(lookup)
                .bind()
                .await
                .map_err(|e| SeyfrError::Network {
                    message: format!("failed to bind endpoint: {}", e),
                })?;
            
            let blobs = BlobsProtocol::new(&store, None);
            let router = Router::builder(endpoint.clone())
                .accept(iroh_blobs::ALPN, blobs.clone())
                .spawn();
            
            Ok(TransferEngine {
                endpoint,
                router,
                blobs,
            })
        }

        /// Test real network transfer of a single file between two nodes
        #[tokio::test]
        #[ignore] // Run with --ignored flag
        async fn test_network_single_file_transfer() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            
            // Setup two nodes with shared address lookup
            let lookup = MemoryLookup::new();
            let sender_store = testdir.path().join("sender_store");
            let sender = create_network_engine(sender_store, lookup.clone()).await?;
            
            let receiver_store = testdir.path().join("receiver_store");
            let receiver = create_network_engine(receiver_store, lookup.clone()).await?;
            
            // Share endpoint addresses for peer discovery
            lookup.add_endpoint_info(sender.endpoint.addr());
            lookup.add_endpoint_info(receiver.endpoint.addr());
            
            // Small delay to ensure endpoints are ready
            tokio::time::sleep(Duration::from_millis(ENDPOINT_READY_DELAY_MS)).await;
            
            // Create test file
            let send_dir = testdir.path().join("send");
            fs::create_dir_all(&send_dir)?;
            let test_file = send_dir.join("test.txt");
            let test_data = b"Hello, World! This is a network test.";
            fs::write(&test_file, test_data)?;
            
            // Send file
            let ticket = sender.send_file(test_file.to_str().unwrap(), None).await?;
            
            // Receive file with timeout
            let recv_dir = testdir.path().join("recv");
            fs::create_dir_all(&recv_dir)?;
            
            let recv_future = receiver.receive(&ticket, recv_dir.to_str().unwrap(), None);
            tokio::time::timeout(Duration::from_secs(SMALL_TRANSFER_TIMEOUT_SECS), recv_future).await??;
            
            // Verify received file
            let received_file = recv_dir.join("received_file");
            assert!(received_file.exists());
            let received_data = fs::read(&received_file)?;
            assert_eq!(received_data, test_data);
            
            // Cleanup
            sender.router.shutdown().await?;
            receiver.router.shutdown().await?;
            sender.endpoint.close().await;
            receiver.endpoint.close().await;
            
            Ok(())
        }

        /// Test real network transfer of a folder between two nodes
        #[tokio::test]
        #[ignore] // Run with --ignored flag
        async fn test_network_folder_transfer() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            
            // Setup two nodes
            let lookup = MemoryLookup::new();
            let sender_store = testdir.path().join("sender_store");
            let sender = create_network_engine(sender_store, lookup.clone()).await?;
            
            let receiver_store = testdir.path().join("receiver_store");
            let receiver = create_network_engine(receiver_store, lookup.clone()).await?;
            
            // Share endpoint addresses
            lookup.add_endpoint_info(sender.endpoint.addr());
            lookup.add_endpoint_info(receiver.endpoint.addr());
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Create test folder
            let send_dir = testdir.path().join("send_folder");
            fs::create_dir_all(&send_dir)?;
            fs::write(send_dir.join("file1.txt"), b"Content 1")?;
            fs::write(send_dir.join("file2.txt"), b"Content 2")?;
            
            let subdir = send_dir.join("subdir");
            fs::create_dir_all(&subdir)?;
            fs::write(subdir.join("file3.txt"), b"Content 3")?;
            
            // Send folder with progress tracking
            let send_progress = TestProgress::default();
            let ticket = sender.send_folder(send_dir.to_str().unwrap(), Some(&send_progress)).await?;
            
            assert_eq!(send_progress.file_starts.lock().unwrap().len(), 3);
            assert_eq!(send_progress.file_completes.lock().unwrap().len(), 3);
            
            // Receive folder with timeout
            let recv_dir = testdir.path().join("recv_folder");
            fs::create_dir_all(&recv_dir)?;
            
            let recv_progress = TestProgress::default();
            let recv_future = receiver.receive(&ticket, recv_dir.to_str().unwrap(), Some(&recv_progress));
            tokio::time::timeout(Duration::from_secs(SMALL_TRANSFER_TIMEOUT_SECS), recv_future).await??;
            
            // Verify progress
            assert_eq!(recv_progress.file_starts.lock().unwrap().len(), 3);
            assert_eq!(recv_progress.file_completes.lock().unwrap().len(), 3);
            
            // Verify files
            assert_eq!(fs::read(recv_dir.join("file1.txt"))?, b"Content 1");
            assert_eq!(fs::read(recv_dir.join("file2.txt"))?, b"Content 2");
            assert_eq!(fs::read(recv_dir.join("subdir/file3.txt"))?, b"Content 3");
            
            // Cleanup
            sender.router.shutdown().await?;
            receiver.router.shutdown().await?;
            sender.endpoint.close().await;
            receiver.endpoint.close().await;
            
            Ok(())
        }

        /// Test network transfer with large file (1MB)
        #[tokio::test]
        #[ignore] // Run with --ignored flag
        async fn test_network_large_file() -> Result<(), Box<dyn std::error::Error>> {
            let testdir = tempfile::tempdir()?;
            
            let lookup = MemoryLookup::new();
            let sender_store = testdir.path().join("sender_store");
            let sender = create_network_engine(sender_store, lookup.clone()).await?;
            
            let receiver_store = testdir.path().join("receiver_store");
            let receiver = create_network_engine(receiver_store, lookup.clone()).await?;
            
            lookup.add_endpoint_info(sender.endpoint.addr());
            lookup.add_endpoint_info(receiver.endpoint.addr());
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Create 1MB file
            let send_dir = testdir.path().join("send");
            fs::create_dir_all(&send_dir)?;
            let test_file = send_dir.join("large.bin");
            let test_data = vec![0xAB; 1024 * 1024];
            fs::write(&test_file, &test_data)?;
            
            let ticket = sender.send_file(test_file.to_str().unwrap(), None).await?;
            
            let recv_dir = testdir.path().join("recv");
            fs::create_dir_all(&recv_dir)?;
            
            let recv_future = receiver.receive(&ticket, recv_dir.to_str().unwrap(), None);
            tokio::time::timeout(Duration::from_secs(LARGE_TRANSFER_TIMEOUT_SECS), recv_future).await??;
            
            let received_file = recv_dir.join("received_file");
            let received_data = fs::read(&received_file)?;
            assert_eq!(received_data.len(), test_data.len());
            assert_eq!(received_data, test_data);
            
            sender.router.shutdown().await?;
            receiver.router.shutdown().await?;
            sender.endpoint.close().await;
            receiver.endpoint.close().await;
            
            Ok(())
        }
    }
}