pub use crate::walker::collect_files;

use std::path::PathBuf;
use std::time::SystemTime;

use bytes::Bytes;
use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{BlobsProtocol, BlobFormat, ticket::BlobTicket};
use iroh_blobs::hashseq::HashSeq;
use crate::errors::SeyfrError;
use crate::progress::ProgressSink;
use futures_util::stream::{self, StreamExt};
use path_jail::Jail;

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
        details: format!("invalid destination directory: {}", e),
    })
}

// ── Seyfr metadata format ─────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeyfrKind {
    File,
    Folder,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeyfrItem {
    pub name: String,
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeyfrMetadata {
    pub version: String,
    pub kind: SeyfrKind,
    pub items: Vec<SeyfrItem>,
}

/// Store raw bytes in the blob store and return the content hash.
async fn store_bytes(
    store: &iroh_blobs::api::Store,
    bytes: Vec<u8>,
) -> Result<iroh_blobs::Hash, SeyfrError> {
    let import = store.blobs().add_bytes(Bytes::from(bytes));
    let mut stream = import.stream().await;
    while let Some(item) = stream.next().await {
        match item {
            iroh_blobs::api::blobs::AddProgressItem::Done(mut tag) => {
                let hash = tag.hash();
                tag.leak();
                return Ok(hash);
            }
            iroh_blobs::api::blobs::AddProgressItem::Error(e) => {
                return Err(SeyfrError::Store {
                    details: format!("add bytes error: {}", e),
                });
            }
            _ => {}
        }
    }
    Err(SeyfrError::Store {
        details: "add_bytes stream ended without Done".to_string(),
    })
}

/// Apply timestamps to a file from a SeyfrItem.
fn apply_timestamps(path: &std::path::Path, item: &SeyfrItem) {
    let mtime = filetime::FileTime::from_unix_time(item.modified_at as i64, 0);
    let atime = filetime::FileTime::from_unix_time(item.modified_at as i64, 0);
    if let Err(e) = filetime::set_file_times(path, atime, mtime) {
        eprintln!("Warning: could not set timestamps for {}: {}", item.name, e);
    }
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
            details: format!("expected a file, got non-file path: {}", path),
        });
    }
    let file_name = src
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".to_string());
    Ok((src, file_name))
}

impl TransferEngine {
    /// Send a single file. Returns a `BlobTicket` (HashSeq format with metadata).
    pub async fn send_file(
        &self,
        path: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<String, SeyfrError> {
        let (src, file_name) = resolve_file_path(path)?;
        let file_meta = std::fs::metadata(&src)?;

        if let Some(p) = progress {
            p.on_file_start(file_name.clone(), 1, 1);
        }

        // Compute metadata before add_path consumes src
        let mime_type = mime_guess::from_path(&src).first_or_octet_stream().to_string();

        // 1. Import file into blob store
        let file_tag = self.blobs.add_path(src).await.map_err(|e| SeyfrError::Store {
            details: format!("failed to import file: {}", e),
        })?;
        let file_hash = file_tag.hash;

        // 2. Build metadata JSON blob
        let created_at = file_meta
            .created()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let modified_at = file_meta
            .modified()
            .unwrap_or_else(|_| SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let seyfr_meta = SeyfrMetadata {
            version: "seyfr-v1".to_string(),
            kind: SeyfrKind::File,
            items: vec![SeyfrItem {
                name: file_name.clone(),
                size: file_meta.len(),
                created_at,
                modified_at,
                mime_type,
            }],
        };
        let meta_json = serde_json::to_vec(&seyfr_meta).map_err(|e| SeyfrError::Store {
            details: format!("failed to serialize metadata: {}", e),
        })?;

        // 3. Store metadata blob
        let store = self.blobs.store();
        let meta_hash = store_bytes(&store, meta_json).await?;

        // 4. Build HashSeq: [metadata_hash, file_hash]
        let hash_seq: HashSeq = std::iter::once(meta_hash)
            .chain(std::iter::once(file_hash))
            .collect();
        let hash_seq_bytes = hash_seq.into_inner().to_vec();

        // 5. Store HashSeq and create ticket
        let root_hash = store_bytes(&store, hash_seq_bytes).await?;
        let ticket = BlobTicket::new(
            self.router.endpoint().addr(),
            root_hash,
            BlobFormat::HashSeq,
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

        // Collect file hashes and metadata
        let mut items = Vec::new();
        let mut file_hashes = Vec::new();

        let results: Vec<_> = stream::iter(files)
            .map(|file_path| async {
                let rel = file_path
                    .strip_prefix(&src)
                    .expect("collect_files only returns paths under the root");
                let rel_str = rel.to_string_lossy().to_string();

                let file_meta = std::fs::metadata(&file_path)?;
                let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream().to_string();
                let created_at = file_meta
                    .created()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let modified_at = file_meta
                    .modified()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let tag = self.blobs.add_path(file_path).await.map_err(|e| SeyfrError::Store {
                    details: format!("failed to import '{}': {}", rel_str, e),
                })?;

                Ok::<_, SeyfrError>((rel_str, tag.hash, file_meta.len(), created_at, modified_at, mime_type))
            })
            .buffer_unordered(4)
            .collect()
            .await;

        let mut completed = 0u64;
        for res in results {
            let (rel_str, hash, size, created_at, modified_at, mime_type) = res?;
            completed += 1;
            items.push(SeyfrItem {
                name: rel_str.clone(),
                size,
                created_at,
                modified_at,
                mime_type,
            });
            file_hashes.push(hash);

            if let Some(p) = progress {
                p.on_file_complete(rel_str, completed, total);
            }
        }

        // Build metadata JSON blob
        let seyfr_meta = SeyfrMetadata {
            version: "seyfr-v1".to_string(),
            kind: SeyfrKind::Folder,
            items,
        };
        let meta_json = serde_json::to_vec(&seyfr_meta).map_err(|e| SeyfrError::Store {
            details: format!("failed to serialize metadata: {}", e),
        })?;

        // Store metadata blob
        let store = self.blobs.store();
        let meta_hash = store_bytes(&store, meta_json).await?;

        // Build HashSeq: [metadata_hash, file1_hash, file2_hash, ...]
        let hash_seq: HashSeq = std::iter::once(meta_hash)
            .chain(file_hashes.into_iter())
            .collect();
        let hash_seq_bytes = hash_seq.into_inner().to_vec();

        // Store HashSeq and create ticket
        let root_hash = store_bytes(&store, hash_seq_bytes).await?;
        let ticket = BlobTicket::new(
            self.router.endpoint().addr(),
            root_hash,
            BlobFormat::HashSeq,
        );

        if let Some(p) = progress {
            p.on_complete(format!("Folder shared: {} items", total));
        }

        Ok(ticket.to_string())
    }

    /// Receive from a ticket. Only HashSeq with metadata is supported.
    /// 
    /// # Progress Reporting
    /// - **File-level**: `on_file_start`, `on_file_complete`, `on_complete` 
    /// - **Byte-level**: `on_file_progress(name, current_bytes, total_bytes)` - full progress
    /// 
    /// Preserves original filenames, timestamps, and MIME types when available.
    pub async fn receive(
        &self,
        ticket_str: &str,
        dest_dir: &str,
        progress: Option<&dyn ProgressSink>,
    ) -> Result<(), SeyfrError> {
        let ticket: BlobTicket = ticket_str.parse().map_err(|e| SeyfrError::InvalidTicket {
            details: format!("{}", e),
        })?;

        // Validate destination directory and create secure jail
        let jail = validate_dest_dir(dest_dir)?;
        
        // Ensure destination exists
        tokio::fs::create_dir_all(jail.root()).await.map_err(SeyfrError::from)?;

        let store = self.blobs.store();
        let downloader = store.downloader(&self.endpoint);

        match ticket.format() {
            BlobFormat::HashSeq => {
                // Download HashSeq root (contains metadata hash + file hashes)
                let mut collection_size = get_blob_size(store.blobs(), ticket.hash()).await.unwrap_or(0);
                
                let download_progress = downloader.download(ticket.hash(), Some(ticket.addr().id));
                let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                    details: format!("failed to start download: {}", e),
                })?;
                
                while let Some(event) = stream.next().await {
                    let event_str = format!("{:?}", event);
                    
                    if event_str.starts_with("Progress(") {
                        if collection_size == 0 {
                            collection_size = get_blob_size(store.blobs(), ticket.hash()).await.unwrap_or(0);
                        }
                        if let Some(p) = progress {
                            if let Some(current) = parse_progress_bytes(&event_str) {
                                p.on_file_progress("metadata".to_string(), current, collection_size);
                            }
                        }
                    } else if event_str.contains("AllDone") || event_str.contains("Done") {
                        break;
                    } else if event_str.contains("Error") || event_str.contains("Abort") {
                        return Err(SeyfrError::Network {
                            details: format!("download failed: {}", event_str),
                        });
                    }
                }

                // Read the hash sequence bytes
                let hs_bytes = store.blobs().get_bytes(ticket.hash()).await.map_err(|e| SeyfrError::Store {
                    details: format!("failed to read hash sequence: {}", e),
                })?;

                let hash_seq = HashSeq::try_from(hs_bytes).map_err(|e| SeyfrError::Store {
                    details: format!("invalid hash sequence: {}", e),
                })?;

                // First hash is the metadata blob
                let mut hashes = hash_seq.iter();
                let meta_hash = hashes.next().ok_or_else(|| SeyfrError::Store {
                    details: "empty hash sequence".to_string(),
                })?;

                // Download metadata blob
                let mut meta_size = get_blob_size(store.blobs(), meta_hash).await.unwrap_or(0);
                
                let download_progress = downloader.download(meta_hash, Some(ticket.addr().id));
                let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                    details: format!("failed to start metadata download: {}", e),
                })?;
                
                while let Some(event) = stream.next().await {
                    let event_str = format!("{:?}", event);
                    
                    if event_str.starts_with("Progress(") {
                        if meta_size == 0 {
                            meta_size = get_blob_size(store.blobs(), meta_hash).await.unwrap_or(0);
                        }
                        if let Some(p) = progress {
                            if let Some(current) = parse_progress_bytes(&event_str) {
                                p.on_file_progress("metadata".to_string(), current, meta_size);
                            }
                        }
                    } else if event_str.contains("AllDone") || event_str.contains("Done") {
                        break;
                    } else if event_str.contains("Error") || event_str.contains("Abort") {
                        return Err(SeyfrError::Network {
                            details: format!("failed to download metadata: {}", event_str),
                        });
                    }
                }

                let meta_bytes = store.blobs().get_bytes(meta_hash).await.map_err(|e| SeyfrError::Store {
                    details: format!("failed to read metadata: {}", e),
                })?;

                // Parse our SeyfrMetadata JSON format
                let seyfr_meta: SeyfrMetadata = serde_json::from_slice(&meta_bytes).map_err(|e| SeyfrError::Store {
                    details: format!("invalid metadata format: {}", e),
                })?;
                let items = seyfr_meta.items;

                let total = items.len() as u64;

                // Download and export each file
                for (idx, item) in items.iter().enumerate() {
                    let hash = hashes.next().ok_or_else(|| SeyfrError::Store {
                        details: format!("missing hash for file: {}", item.name),
                    })?;

                    // Use jail.join() for secure path validation
                    let file_dest = jail.join(&item.name).map_err(|e| SeyfrError::PathTraversal {
                        path: item.name.clone(),
                        details: format!("{}", e),
                    })?;
                    
                    if let Some(parent) = file_dest.parent() {
                        tokio::fs::create_dir_all(parent).await.map_err(SeyfrError::from)?;
                    }

                    if let Some(p) = progress {
                        p.on_file_start(item.name.clone(), (idx + 1) as u64, total);
                    }

                    // Download the blob with progress — use known size from metadata
                    let file_size = if item.size > 0 {
                        item.size
                    } else {
                        get_blob_size(store.blobs(), hash).await.unwrap_or(0)
                    };
                    
                    let download_progress = downloader.download(hash, Some(ticket.addr().id));
                    let mut stream = download_progress.stream().await.map_err(|e| SeyfrError::Network {
                        details: format!("failed to start download for '{}': {}", item.name, e),
                    })?;
                    
                    while let Some(event) = stream.next().await {
                        let event_str = format!("{:?}", event);
                        
                        if event_str.starts_with("Progress(") {
                            if let Some(p) = progress {
                                if let Some(current) = parse_progress_bytes(&event_str) {
                                    p.on_file_progress(item.name.clone(), current, file_size);
                                }
                            }
                        } else if event_str.contains("AllDone") || event_str.contains("Done") {
                            break;
                        } else if event_str.contains("Error") || event_str.contains("Abort") {
                            return Err(SeyfrError::Network {
                                details: format!("download failed for '{}': {}", item.name, event_str),
                            });
                        }
                    }

                    // Export to destination
                    store.blobs().export(hash, &file_dest).await.map_err(|e| SeyfrError::Store {
                        details: format!("export failed for '{}': {}", item.name, e),
                    })?;

                    // Apply timestamps if available (our format).
                    // Note: only file timestamps are preserved. Directory timestamps
                    // are not captured because they change automatically when files
                    // are written, making them unreliable to preserve.
                    if item.modified_at > 0 {
                        apply_timestamps(&file_dest, item);
                    }

                    if let Some(p) = progress {
                        p.on_file_complete(item.name.clone(), (idx + 1) as u64, total);
                    }
                }

                if let Some(p) = progress {
                    let details = if total == 1 {
                        "File received successfully".to_string()
                    } else {
                        format!("Folder received: {} items", total)
                    };
                    p.on_complete(details);
                }
            }
            BlobFormat::Raw => {
                return Err(SeyfrError::InvalidTicket {
                    details: "Raw format tickets are not supported — use HashSeq with metadata".to_string(),
                });
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
                details: format!("failed to bind endpoint: {}", e),
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
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::HashSeq);
            
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
            
            // Verify received file content (original name preserved from metadata)
            let received_file = recv_dir.join("test.txt");
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
            assert_eq!(parsed.format(), iroh_blobs::BlobFormat::HashSeq);
            
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
                details: format!("failed to create store: {}", e),
            })?;
            
            let endpoint = Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Disabled) // Disable relay for local testing
                .address_lookup(lookup)
                .bind()
                .await
                .map_err(|e| SeyfrError::Network {
                    details: format!("failed to bind endpoint: {}", e),
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
            
            // Verify received file (original name preserved from metadata)
            let received_file = recv_dir.join("test.txt");
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
            
            let received_file = recv_dir.join("large.bin");
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