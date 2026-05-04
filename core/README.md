# seyfr-core

**Rust library for secure peer-to-peer file transfers using iroh**

This is the core business logic library for Seyfr, providing a simple API for P2P file transfers with built-in security validations.

## 🚀 Quick Start

```rust
use seyfr_core::Core;

// Initialize with data directory
let core = Core::new("/path/to/data".to_string())?;

// Send a file or folder (auto-detects)
let ticket = core.send("/path/to/file.txt".to_string(), None)?;

// Receive using the ticket
core.receive(ticket, "/path/to/destination".to_string(), None)?;

// Get node ID for debugging
let node_id = core.node_id();
```

## 📡 How It Works

1. **Send**: Files are added to a local iroh-blobs store and a compact `BlobTicket` is generated
2. **Share**: The ticket (containing peer address + content hash) is shared with the recipient
3. **Receive**: Recipient uses the ticket to establish a direct P2P connection and download
4. **Verify**: Content is verified using cryptographic hashes (BLAKE3)

### Transfer Formats

- **Single File**: `BlobFormat::Raw` - Direct blob transfer
- **Folders**: `BlobFormat::HashSeq` - Collection with metadata + individual file blobs

## 🔐 Security Features

All security validations are enforced at the core level:

- **Path Traversal Protection**: Uses `path_jail` to validate all file paths
- **File Size Limits**: 1 GB per file, 10 GB per collection
- **Collection Limits**: Maximum 10,000 files per transfer
- **Destination Validation**: Ensures files only write to intended directories
- **Symlink Safety**: Skips symlinks by default to prevent escapes

### Security Constants

```rust
const MAX_FILE_SIZE: u64 = 1_073_741_824;           // 1 GB
const MAX_COLLECTION_SIZE: u64 = 10_737_418_240;    // 10 GB
const MAX_FILES_IN_COLLECTION: u64 = 10_000;
```

## 📊 Progress Tracking

Implement the `ProgressSink` trait to receive progress callbacks:

```rust
pub trait ProgressSink {
    fn on_file_start(&self, name: String, current: u64, total: u64);
    fn on_file_progress(&self, name: String, bytes: u64, total: u64);
    fn on_file_complete(&self, name: String, current: u64, total: u64);
    fn on_complete(&self, message: String);
    fn on_error(&self, message: String);
}
```

## 🏗️ Architecture

```
core/
├── src/
│   ├── lib.rs          # Public API (Core struct)
│   ├── transfers.rs    # TransferEngine (iroh-blobs integration)
│   ├── progress.rs     # ProgressSink trait
│   ├── errors.rs       # SeyfrError enum
│   ├── walker.rs       # File collection utilities
│   └── test_utils.rs   # Shared test helpers
├── Cargo.toml
└── uniffi.toml         # UniFFI configuration
```

### Key Components

#### `Core` (lib.rs)
- Public API exposed via UniFFI
- Manages Tokio runtime
- Wraps `TransferEngine`

#### `TransferEngine` (transfers.rs)
- Core P2P transfer logic
- Integrates with iroh-blobs
- Implements security validations
- Methods: `send()`, `send_file()`, `send_folder()`, `receive()`

#### `ProgressSink` (progress.rs)
- Callback trait for progress reporting
- Byte-level and file-level progress

#### `SeyfrError` (errors.rs)
- Comprehensive error types
- UniFFI-compatible error variants

## 🧪 Testing

### Unit Tests (Fast, In-Memory)

```bash
cargo test --package seyfr-core
```

Tests use in-memory stores and mock progress tracking for fast, reliable testing.

### Network Integration Tests

```bash
cargo test --package seyfr-core network_integration -- --ignored --test-threads=1
```

These tests verify real P2P transfers between separate nodes using filesystem stores.

### Test Structure

```rust
#[cfg(test)]
mod unit_tests {
    // Fast tests with MemStore
}

#[cfg(test)]
mod network_integration {
    #[tokio::test]
    #[ignore]  // Run with --ignored flag
    async fn test_network_single_file_transfer() { ... }
}
```

## 🛠️ Development

### Adding Features

1. Implement logic in `transfers.rs` or create new modules
2. Expose public API in `lib.rs` via `#[uniffi::export]`
3. Add error variants to `errors.rs` if needed
4. Write tests (unit + integration)
5. Update this README

### Code Guidelines

- **No bandaids**: Fix root causes, not symptoms
- **DRY principles**: Extract shared logic
- **SOLID principles**: Single responsibility, clear interfaces
- **Security first**: All paths validated, all sizes checked
- **Test coverage**: Unit tests for logic, integration tests for network

## 📦 Dependencies

- **[iroh](https://github.com/n0-computer/iroh)** (0.98.2): P2P networking
- **[iroh-blobs](https://github.com/n0-computer/iroh-blobs)** (0.100.0): Content-addressed storage
- **[path_jail](https://crates.io/crates/path_jail)** (0.2): Secure path validation
- **[tokio](https://tokio.rs/)** (1.52.1): Async runtime
- **[uniffi](https://mozilla.github.io/uniffi-rs/)** (0.31.1): FFI bindings generation

## 🔄 UniFFI Integration

This library is designed to be used via UniFFI bindings. The `Core` struct and all public types are automatically exposed to Swift, Kotlin, and other languages.

### Generated Bindings

```bash
# Generate Swift bindings
cargo run --bin uniffi-bindgen generate src/seyfr_core.udl --language swift

# Generate Kotlin bindings  
cargo run --bin uniffi-bindgen generate src/seyfr_core.udl --language kotlin
```

Or use JFFI which handles this automatically:

```bash
jffi build --platform ios
```

## 📄 License

[Add your license here]
