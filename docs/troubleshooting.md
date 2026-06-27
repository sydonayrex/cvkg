# Troubleshooting

This guide covers common compilation, runtime, and rendering failures sourced from the codebase.

## 1. Compilation Failures

### Target 'wasm32-unknown-unknown' Missing

- **Symptom**: `error[E0463]: can't find crate for 'core'`
- **Cause**: The WASM target is not installed.
- **Fix**:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### wasm-pack Not Found

- **Symptom**: `Failed to execute wasm-pack: No such file or directory (os error 2)`
- **Cause**: `wasm-pack` is not installed.
- **Fix**:
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```

### Cyclic Dependency

- **Symptom**: Cargo reports circular dependency warnings.
- **Cause**: Multiple mutually-exclusive features enabled simultaneously (e.g., `native` and `web`).
- **Fix**: Check for duplicate features in your Cargo.toml. Enable only one backend:
  ```toml
  cvkg = { version = "0.2.15", features = ["native"] }
  ```

### "File Too Large" / Bus Error in CI

- **Symptom**: `Bus error (core dumped)` during `cargo test` in CI.
- **Cause**: Debug info exhausts memory in constrained environments.
- **Fix**: The workspace sets `debug = 0` in `[profile.dev]` and `debug = 1` in `[profile.test]`. Do not override these in member crates.

## 2. Runtime Crashes

### GPU Device Not Found

- **Symptom**: `[GPU] Fatal error: RequestAdapter failed`
- **Cause**: WGPU cannot find a compatible hardware adapter.
- **Fix**:
  1. Verify Vulkan: `vulkaninfo`
  2. Update GPU drivers (NVIDIA proprietary or Mesa).
  3. Force software rasterization:
     ```bash
     export WGPU_ADAPTER=mesa
     cargo run
     ```

### Wayland Event Loop Crash

- **Symptom**: Application halts immediately on Wayland.
- **Cause**: Mismatch between X11 and Wayland environments.
- **Fix**:
  ```bash
  export WINIT_UNIX_BACKEND=x11
  cargo run
  ```

### Panic: "Cannot start runtime from within a runtime"

- **Symptom**: Panic when calling `tokio::runtime::Runtime::new()` inside an async context.
- **Cause**: `cvkg-render-native` and `cvkg-cli` create their own tokio runtimes. Calling them from within an existing async context triggers this panic.
- **Fix**: Use `tokio::task::block_in_place` or restructure to avoid nested runtimes.

## 3. Visual Artifacts

### Text Flickers or Missing Glyphs

- **Symptom**: Labels blink or display as empty rectangles during scroll.
- **Cause**: Texture atlas is full, causing cache eviction thrashing.
- **Fix**:
  1. Reduce the number of active font sizes.
  2. Increase texture dimension limits in `RendererConfig`.
  3. Embed custom fonts directly to prevent system fallback.

### Glow Misaligned With Element Bounds

- **Symptom**: Glow shadows are displaced relative to parent components.
- **Cause**: Mismatched coordinate mappings or subpixel rounding on High-DPI screens.
- **Fix**: Check `scale_factor` configuration. Use float coordinates for layout, snap to pixel grid only when motion stops.

### Blank Window

- **Symptom**: Window opens but shows no content.
- **Cause**: The render graph has no passes, or the scene graph is empty.
- **Fix**: Verify that at least one `View` is returning a non-empty body. Check that `Renderer::submit()` is called.

## 4. Performance

### Low Frame Rate

- **Symptom**: Frames exceeding 16.6ms rendering window.
- **Cause**: Heavy synchronous work inside `body()` composition routines.
- **Fix**:
  1. Move data loading out of `body()`.
  2. Use `State::memoize()` for views that change infrequently.
  3. Build in release mode: `cargo run --release`.

### Render Graph Cache Thrashing

- **Symptom**: Frequent graph sorting cycles in telemetry logs.
- **Cause**: Dynamic node/pass additions on every frame prevent cache hits.
- **Fix**: Compile conditional passes as static branches rather than rebuilding dynamically.

### Color Distortion in Highlights

- **Symptom**: Bright colored zones shift toward white or blue.
- **Cause**: AgX tonemapper is bypassed or HDR color spaces are not declared.
- **Fix**: Ensure the WGPU target format matches sRGB. Confirm AgX is active in shader configuration.

## 5. Diagnostic Data

To collect diagnostic logs:

```bash
export RUST_LOG=debug
cargo run > cvkg_diagnostics.log 2>&1
```

The log contains adapter selection, font mapping, and layout proposal data.
