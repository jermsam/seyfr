mod errors;
mod progress;
mod transfers;
mod walker;

#[cfg(test)]
pub mod test_utils;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use uniffi;

use crate::errors::SeyfrError;
use crate::progress::ProgressSink;
use crate::transfers::TransferEngine;

struct Seyfr {
    runtime: Runtime,
    engine: Arc<TransferEngine>,
}

#[derive(uniffi::Object)]
pub struct Core {
    inner: Arc<Seyfr>,
}

#[uniffi::export]
impl Core {
    /// Construct the core with a persistent on-disk store.
    /// `data_dir` is the iOS app sandbox path (e.g. `.../Library/Application Support/seyfr`).
    #[uniffi::constructor]
    pub fn new(data_dir: String) -> Result<Arc<Self>, SeyfrError> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| SeyfrError::Internal {
                message: format!("failed to create Tokio runtime: {}", e),
            })?;

        let engine = runtime.block_on(async {
            let data_path = PathBuf::from(data_dir);
            tokio::fs::create_dir_all(&data_path)
                .await
                .map_err(|e| SeyfrError::Io { message: e.to_string() })?;

            let store = iroh_blobs::store::fs::FsStore::load(&data_path)
                .await
                .map_err(|e| SeyfrError::Store {
                    message: format!("failed to load FsStore: {}", e),
                })?;

            let endpoint = iroh::Endpoint::bind(iroh::endpoint::presets::N0)
                .await
                .map_err(|e| SeyfrError::Network { message: e.to_string() })?;

            let blobs = iroh_blobs::BlobsProtocol::new(&store, None);

            let router = iroh::protocol::Router::builder(endpoint.clone())
                .accept(iroh_blobs::ALPN, blobs.clone())
                .spawn();

            Ok::<_, SeyfrError>(Arc::new(TransferEngine {
                endpoint,
                router,
                blobs,
            }))
        })?;

        Ok(Arc::new(Self {
            inner: Arc::new(Seyfr { runtime, engine }),
        }))
    }

    /// Send a file or folder. Auto-detects type and returns a compact ticket string.
    pub fn send(&self, path: String, progress: Option<Box<dyn ProgressSink>>) -> Result<String, SeyfrError> {
        self.inner.runtime.block_on(async {
            self.inner.engine.send(&path, progress.as_deref()).await
        })
    }

    /// Receive from a ticket into `dest_dir`.
    /// Works for both single-file (Raw) and folder (HashSeq) tickets.
    pub fn receive(&self, ticket: String, dest_dir: String, progress: Option<Box<dyn ProgressSink>>) -> Result<(), SeyfrError> {
        self.inner.runtime.block_on(async {
            self.inner.engine.receive(&ticket, &dest_dir, progress.as_deref()).await
        })
    }

    /// Human-readable node ID. Useful for debugging / logging.
    pub fn node_id(&self) -> String {
        self.inner.runtime.block_on(async {
            self.inner.engine.endpoint.addr().id.to_string()
        })
    }
}

uniffi::setup_scaffolding!();
