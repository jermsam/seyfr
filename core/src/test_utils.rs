//! Shared test utilities for seyfr-core tests

use std::sync::{Arc, Mutex};
use crate::progress::ProgressSink;

/// Test progress tracker for verifying progress callbacks
#[derive(Debug, Clone, Default)]
pub struct TestProgress {
    pub file_starts: Arc<Mutex<Vec<(String, u64, u64)>>>,
    pub file_completes: Arc<Mutex<Vec<(String, u64, u64)>>>,
    pub completes: Arc<Mutex<Vec<String>>>,
    pub errors: Arc<Mutex<Vec<String>>>,
}

impl ProgressSink for TestProgress {
    fn on_file_start(&self, name: String, current: u64, total: u64) {
        self.file_starts.lock().unwrap().push((name, current, total));
    }

    fn on_file_progress(&self, _name: String, _bytes: u64, _total: u64) {
        // Not tracked in tests by default
    }

    fn on_file_complete(&self, name: String, current: u64, total: u64) {
        self.file_completes.lock().unwrap().push((name, current, total));
    }

    fn on_complete(&self, details: String) {
        self.completes.lock().unwrap().push(details);
    }

    fn on_error(&self, details: String) {
        self.errors.lock().unwrap().push(details);
    }
}
