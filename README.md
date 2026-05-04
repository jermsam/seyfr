# Seyfr

**Secure, peer-to-peer file transfer powered by [iroh](https://iroh.computer)**

Seyfr is a cross-platform file transfer application built with Rust and [UniFFI](https://mozilla.github.io/uniffi-rs/), enabling fast, secure, and direct file sharing between devices without intermediary servers.

At its core, Seyfr leverages [**iroh**](https://iroh.computer) and [**iroh-blobs**](https://github.com/n0-computer/iroh-blobs) - a next-generation networking stack that provides efficient peer-to-peer connectivity and content-addressed blob storage. 

**iroh-blobs** is the content layer that handles:
- Content-addressed storage using BLAKE3 cryptographic hashes
- Efficient blob streaming with progress tracking
- Collection management for multi-file transfers (HashSeq format)
- Local storage with filesystem or in-memory backends

**iroh** provides the networking foundation:
- P2P connectivity with automatic NAT traversal and hole-punching
- Relay fallback for difficult network conditions
- Direct peer-to-peer connections when possible

Together, they handle all the complex networking and storage details, allowing Seyfr to focus on the security layer (path validation, size limits) and user experience.

The project uses [JFFI](https://github.com/jffi-dev/jffi) (Just FFI) as the build framework to seamlessly generate platform-specific bindings and manage the multi-platform development workflow.

### 🎯 Right Tool for the Right Job

This architecture follows a clear separation of concerns:

- **Rust** (via `core/`) handles the business logic: P2P networking, file transfers, security validations, and data integrity. Rust's memory safety, performance, and robust ecosystem make it ideal for these critical operations.

- **Native Languages** (Swift for iOS, Kotlin for Android, TypeScript for Web) handle UI and state management. Each platform uses its native tools and patterns, providing the best user experience without compromise.

**JFFI + UniFFI** bridges these worlds automatically, generating type-safe bindings so platform developers can call Rust functions as if they were native APIs. This means:
- ✅ Write complex logic once in Rust, use everywhere
- ✅ No compromises on native UI/UX patterns
- ✅ Each platform feels truly native
- ✅ Shared security and business logic across all platforms

## ✨ Features

- 🔒 **Secure by Design**: Path traversal protection, file size limits, and validated destinations
- 🚀 **Peer-to-Peer**: Direct transfers using iroh's efficient P2P protocol
- 📦 **Smart Transfer**: Auto-detects files vs folders, handles collections seamlessly
- 📊 **Progress Tracking**: Real-time byte-level progress reporting
- 🎯 **Simple API**: Single `send()` and `receive()` methods handle everything
- 🌍 **Cross-Platform**: iOS ready, with architecture for Android and Web

## 🔐 Security Features

- **Path Traversal Protection**: Uses `path_jail` to prevent malicious paths
- **File Size Limits**: 1 GB per file, 10 GB per collection
- **Collection Limits**: Maximum 10,000 files per transfer
- **Destination Validation**: Ensures files only write to intended directories
- **Symlink Safety**: Skips symlinks by default to prevent escapes

## 🏗️ Architecture

```
seyfr/
├── core/                    # Rust business logic
│   ├── src/
│   │   ├── lib.rs          # UniFFI exports & Core API
│   │   ├── transfers.rs    # P2P transfer engine (iroh-blobs)
│   │   ├── progress.rs     # Progress callback trait
│   │   ├── errors.rs       # Error types
│   │   ├── walker.rs       # File collection utilities
│   │   └── test_utils.rs   # Shared test helpers
│   └── Cargo.toml
├── platforms/
│   └── ios/                # Swift UI
└── README.md
```

## 🚀 Quick Start

### iOS

```bash
# Build the project
jffi build --platform ios

# Run on simulator/device
jffi run --platform ios

# Development mode with auto-rebuild
jffi dev --platform ios
```

### For Rust Developers

See [`core/README.md`](./core/README.md) for detailed API documentation, testing instructions, and development guidelines.

## 🧪 Testing

```bash
# Run unit tests (in-memory, fast)
cargo test --package seyfr-core

# Run network integration tests (requires --ignored)
cargo test --package seyfr-core network_integration -- --ignored --test-threads=1
```

## 🛠️ Development

### Adding Platforms

```bash
# Add Android support
jffi add android

# Add Web support  
jffi add web
```

## 📦 Dependencies

- **[iroh](https://github.com/n0-computer/iroh)**: P2P networking and blob transfer
- **[iroh-blobs](https://github.com/n0-computer/iroh-blobs)**: Content-addressed blob storage
- **[UniFFI](https://mozilla.github.io/uniffi-rs/)**: Rust → Swift/Kotlin bindings
- **[path_jail](https://github.com/tenuo-ai/path_jail)**: Secure path validation
- **[tokio](https://tokio.rs/)**: Async runtime

## 📄 License

[Add your license here]

## 🤝 Contributing

Contributions welcome! Please ensure:
- All tests pass (`cargo test`)
- Security validations remain intact
- Follow existing code patterns (DRY, SOLID principles)
