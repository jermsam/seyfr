/// Progress callback.
///
/// The UI receives:
/// - `current_bytes` / `total_bytes` for the overall transfer
/// - `current_file` name for the file currently in flight
/// - `file_index` / `total_files` when transferring folders
#[uniffi::export(callback_interface)]
pub trait ProgressSink: Send + Sync {
    /// Called when a new file starts processing.
    fn on_file_start(&self, file_name: String, file_index: u64, total_files: u64);

    /// Called with byte-level progress for the current file.
    fn on_file_progress(&self, file_name: String, current_bytes: u64, total_bytes: u64);

    /// Called when a file completes.
    fn on_file_complete(&self, file_name: String, file_index: u64, total_files: u64);

    /// Called when the entire transfer completes.
    fn on_complete(&self, details: String);

    /// Called on any error with a human-readable message.
    fn on_error(&self, details: String);
}

