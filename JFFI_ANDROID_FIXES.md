# JFFI Android Production Fixes Required

## Issue 1: 16 KB Page Size Alignment (Android 15+ / API 35+)

### Problem
Android 15+ devices require native libraries (`.so` files) to be 16 KB page-aligned. Apps with unaligned native libraries show a runtime compatibility warning dialog and may fail to build with `integer overflow` during `packageDebug`.

### Affected Libraries
- `libseyfr_core.so` (our Rust core)
- `libjnidispatch.so` (JNA)
- ML Kit / CameraX native libraries

### Proper Fix (Required for Production)

JFFI should:

1. **Auto-generate `.cargo/config.toml`** with linker flags for all Android targets:
   ```toml
   [target.aarch64-linux-android]
   rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]

   [target.armv7-linux-androideabi]
   rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]

   [target.x86_64-linux-android]
   rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]

   [target.i686-linux-android]
   rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]
   ```

2. **Place the file at** `<project-root>/core/.cargo/config.toml` before running `cargo-ndk`.

3. **Validate alignment** post-build by checking ELF headers or using Android Studio's APK analyzer.

### Current Workaround (DO NOT use in production)
Adding `useLegacyPackaging = true` to `build.gradle.kts` suppresses the error by extracting `.so` files instead of memory-mapping them. This has performance implications and may break in future Android versions.

---

## Issue 2: UniFFI JNA + ndk-context Systematic Crash

### Problem
Every JFFI Android app that uses a Rust library with `ndk-context` (directly or transitively) will crash on startup with:
```
uniffi.<namespace>.InternalException: android context was not initialized
```

### Root Cause Chain
```
Core::new() → iroh::Endpoint::bind() → hickory-resolver → ndk_context::android_context() → PANIC
```

UniFFI uses JNA, which does **not** call `JNI_OnLoad`. `ndk-context` requires initialization via `JNI_OnLoad` or an explicit call to `initialize_android_context()`.

### Why This Is JFFI's Responsibility
- UniFFI explicitly uses JNA by design — [issue #1778](https://github.com/mozilla/uniffi-rs/issues/1778) has been open since 2023 with no resolution
- JFFI is the abstraction layer over UniFFI — it should paper over platform-specific gaps
- This is a **systematic failure mode**, not an app-specific issue

### What JFFI Should Do (Auto-Detection + Code Generation)

#### Step 1: Detect ndk-context in Dependency Tree
```bash
cargo tree -i ndk-context --target aarch64-linux-android --depth 10
```

If `ndk-context` is found anywhere in the dependency tree, generate the bridge.

#### Step 2: Generate Rust JNI Bridge

Create `<project>/core/src/android.rs` (gated by `#[cfg(target_os = "android")]`):

```rust
use jni::objects::{JClass, JObject};

/// # Safety
/// Called from JNI. Must only be called once before any code that uses ndk-context.
#[no_mangle]
pub unsafe extern "C" fn Java_<pkg>_JffiAndroidInit_initNdkContext(
    env: jni::JNIEnv,
    _class: JClass,
    context: JObject,
) {
    // Get the JavaVM pointer
    let vm = match env.get_java_vm() {
        Ok(vm) => vm,
        Err(e) => {
            eprintln!("JFFI: Failed to get JavaVM: {}", e);
            return;
        }
    };
    let vm_ptr = vm.get_java_vm_pointer() as *mut std::ffi::c_void;

    // Create a global reference to the context so it persists
    let global_context = match env.new_global_ref(context) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("JFFI: Failed to create global ref: {}", e);
            return;
        }
    };
    let ctx_ptr = global_context.as_raw() as *mut std::ffi::c_void;

    // Initialize ndk-context with raw pointers
    ndk_context::initialize_android_context(vm_ptr, ctx_ptr);

    // Leak the GlobalRef — ndk-context owns the raw pointer for the app lifetime
    std::mem::forget(global_context);
}
```

**CRITICAL**: Both arguments to `initialize_android_context()` must be `*mut c_void` raw pointers, not `&mut JNIEnv` or `&JObject` references. This is the bug in JFFI's current generator.

#### Step 3: Register Module in lib.rs

Add to `<project>/core/src/lib.rs`:
```rust
#[cfg(target_os = "android")]
mod android;
```

#### Step 4: Generate Kotlin Helper

Create a Kotlin file (e.g., `JffiAndroidInit.kt`) in the app's package:

```kotlin
package <pkg> // e.g., com.example.seyfr

import android.content.Context

object JffiAndroidInit {
    init {
        System.loadLibrary("seyfr_core")
    }

    @JvmStatic
    external fun initNdkContext(context: Context)
}
```

#### Step 5: Inject Initialization Before First UniFFI Call

Generate documentation or code comments instructing the user to add to `MainActivity.onCreate()`:

```kotlin
override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    // Initialize ndk-context before any Rust code that needs it
    // (e.g., DNS resolution via hickory-resolver used by iroh)
    JffiAndroidInit.initNdkContext(applicationContext)

    // ... rest of onCreate, including UniFFI Core initialization
}
```

### JFFI Implementation Checklist

- [ ] Run `cargo tree -i ndk-context --target <arch>` to detect dependency
- [ ] Add `ndk-context = "0.1"` and `jni = "0.21"` to `Cargo.toml` under `[target.'cfg(target_os = "android")'.dependencies]`
- [ ] Generate `android.rs` with correct raw pointer types
- [ ] Add `#[cfg(target_os = "android")] mod android;` to `lib.rs`
- [ ] Generate `JffiAndroidInit.kt` with correct package name (read from `AndroidManifest.xml` or `build.gradle.kts`)
- [ ] Print clear message after build: `Add to MainActivity: JffiAndroidInit.initNdkContext(applicationContext)`
- [ ] Document the requirement in JFFI's README / Android setup guide
- [ ] Add `--no-android-bridge` flag for advanced users who want to provide their own init

### Verification

After implementing, verify:
1. App launches without `android context was not initialized` crash
2. `iroh::Endpoint::bind()` succeeds on Android
3. DNS resolution works (test by connecting to a remote peer)
4. No `16 KB alignment` warnings on Android 15+ devices
