# seyfr

Cross-platform app built with Rust + UniFFI

## Platforms

- ios

## Quick Start

```bash
# Build for your platform
jffi build --platform ios

# Run the app
jffi run --platform ios

# Development mode (auto-rebuild)
jffi dev --platform ios
```

## Project Structure

- `core/` - Business logic + UniFFI exports
- `ffi-web/` - WASM FFI layer (present when web platform is enabled)
- `platforms/` - Platform-specific UIs
- `Cargo.toml` - Workspace manifest
- `jffi.toml` - Framework configuration

## Add or Remove Platforms

```bash
# Add another platform to this project
jffi add <platform>

# Remove a platform
jffi remove <platform>
```

## Development

Edit your business logic in `core/src/lib.rs`. The FFI bindings will be automatically regenerated.

## Adding Features

1. Add logic to `core/src/lib.rs`
2. Expose via `#[uniffi::export]`
3. Rebuild: `jffi build --platform <platform>`
4. Update UI in `platforms/<platform>/`
# seyfr
