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

### WGSL Parse Failure (build.rs)

- **Symptom**: Compile error from `naga` during `cargo build` even though your Rust source is correct.
- **Cause**: `naga` is a build dependency of `cvkg-render-gpu` used to validate WGSL at compile time. Errors in `.wgsl` shader files surface as build failures.
- **Fix**: Check the shader file mentioned in the error output. If you added or modified WGSL, run `cargo build -p cvkg-render-gpu 2>&1 | head -80` to see the full naga diagnostic.

## 2. Runtime Crashes

### GPU Device Not Found

- **Symptom**: `Failed to find a suitable GPU for Surtr` or `[GPU] Fatal error: RequestAdapter failed`
- **Cause**: WGPU cannot find a compatible hardware adapter.
- **Fix**:
  1. Verify Vulkan: `vulkaninfo`
  2. Update GPU drivers (NVIDIA proprietary or Mesa).
  3. Force a specific adapter by name:
     ```bash
     export WGPU_ADAPTER_NAME=nvidia
     cargo run
     ```
  4. Force a specific backend:
     ```bash
     export WGPU_BACKEND=vulkan
     cargo run
     ```

### No Compatible Adapter Found (Headless/CI)

- **Symptom**: `No compatible adapter found` — `cvkg-render-gpu` fails during `enumerate_adapters`.
- **Cause**: Running in a container or remote environment with no GPU and no software rasterizer.
- **Fix**: Install Mesa's LLVMpipe: `sudo apt-get install libegl1-mesa`. Or use `WGPU_BACKEND=llvmpipe`.

### Wayland Event Loop Crash

- **Symptom**: Application halts immediately on Wayland.
- **Cause**: Mismatch between X11 and Wayland environments.
- **Fix**:
  ```bash
  export WINIT_UNIX_BACKEND=x11
  cargo run
  ```

### "winit_id not found for window handle"

- **Symptom**: Panic in `cvkg-render-native/main_loop.rs` with message "winit_id not found for window handle: window may have been destroyed".
- **Cause**: The window was destroyed (e.g. closed) but the event loop still has stale references.
- **Fix**: Do not manually destroy winit windows. Let `WindowManager` handle lifecycle.

### "Failed to load background image"

- **Symptom**: `panic!("Failed to load background image '{}': {}", ...)` from `cvkg-render-native/renderer.rs`.
- **Cause**: The image path passed to `run_with_background()` does not exist or is not a valid PNG/JPEG.
- **Fix**: Use an absolute path or verify the asset exists relative to the working directory.

### "Failed to create native window"

- **Symptom**: `panic!("failed to create native window")` from `cvkg-render-native/window.rs`.
- **Cause**: Display server connection failed (no X11/Wayland, or DISPLAY/WAYLAND_DISPLAY not set).
- **Fix**: Verify display server is running; on headless systems use a virtual framebuffer:
  ```bash
  export DISPLAY=:99
  Xvfb :99 -screen 0 1024x768x24 &
  ```

### "Failed to create winit event loop"

- **Symptom**: Panic from `cvkg-render-native/renderer.rs` with message "failed to create winit event loop: platform initialization failed".
- **Cause**: No display server available (common in Docker or SSH sessions).
- **Fix**: Ensure `DISPLAY` (X11) or `WAYLAND_DISPLAY` (Wayland) is set and valid.

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

## 5. Environment Variables Reference

| Variable | Crate | Effect |
|---|---|---|
| `WGPU_ADAPTER_NAME` | cvkg-render-gpu | Case-insensitive substring match against GPU adapter/driver name. Forces selection of a specific adapter. |
| `WGPU_BACKEND` | cvkg-render-gpu | Force a specific wgpu backend (`vulkan`, `metal`, `dx12`, `opengl`, `llvmpipe`). |
| `RUST_LOG` | All crates | Standard `log` filter. Use `cvkg_render_gpu=debug` for renderer diagnostics, `wgpu=error` for GPU validation. |
| `CVKG_THEME` | cvkg-core | Override system theme detection. Set to `dark` or `light` to force a theme. |
| `CVKG_CONFIG` | cvkg-cli | Path to config file. Defaults to `.cvkg.toml` in the current directory. |
| `UPDATE_GOLDEN` | cvkg-test | Set to `"1"` to update golden image files on disk instead of comparing. |
| `GTK_A11Y` | cvkg-core | Set to enable reduced-motion preferences (GNOME accessibility). |
| `GTK_THEME` | cvkg-core, cvkg-components | Read to detect system dark/high-contrast theme preferences. |
| `MACOS_REDUCED_MOTION` | cvkg-components | macOS only. Set to enable reduced-motion preference detection. |
| `ACCESSIBILITY_REDUCED_MOTION` | cvkg-components | Cross-platform. Set to enable reduced-motion preference detection. |
| `NO_ANIMATIONS` | cvKG-components | Set to disable all animations globally. |

## 6. Common Error Types

| Error | Crate | Cause |
|---|---|---|
| `MaterialError::NoOutput` | cvkg-render-gpu | Material graph has no output node defined. |
| `MaterialError::Cycle` | cvkg-render-gpu | Material graph contains a cycle. |
| `MaterialError::DisconnectedInput { node, socket }` | cvkg-render-gpu | A node input socket is not connected. |
| `MaterialError::TypeMismatch { from, to }` | cvkg-render-gpu | Connected sockets have incompatible types. |
| `MaterialError::CompileError(String)` | cvkg-render-gpu | WGSL code generation failed. |
| `MaterialError::TooManyNodes(usize, usize)` | cvkg-render-gpu | Graph exceeds configured `max_nodes` limit. |
| `MaterialError::TooManyEdges(usize, usize)` | cvkg-render-gpu | Graph exceeds configured `max_edges` limit. |
| `MaterialError::UnreachableNode(MatNodeId)` | cvkg-render-gpu | A node exists but is not reachable from the output. |
| `FilterError::Cycle` | cvkg-svg-filters | SVG filter graph contains a cycle. |
| `FilterError::UnresolvedFilterInput(String)` | cvkg-svg-filters | Filter primitive references an input that does not exist. |
| `FilterError::WgpuError(wgpu::Error)` | cvkg-svg-filters | WGPU operation failed inside a filter pass. |
| `FilterError::InvalidFilterRegion(w, h)` | cvkg-svg-filters | Filter region has zero or negative dimensions. |
| `FilterError::TextureAllocationFailed(String)` | cvkg-svg-filters | Texture allocation for filter output failed. |
| `SvgSerializeError::SerializationFailed(String)` | cvkg-svg-serialize | `usvg::Tree` to XML serialization failed. |
| `SvgSerializeError::OutputExceededMaxSize(max, actual)` | cvkg-svg-serialize | Serialized output exceeds the configured byte limit. |
| `TemplateError::VersionMismatch { expected, found }` | cvkg-compositor | Loading a template with an incompatible version. |
| `FontLoadError::Io(std::io::Error)` | cvkg-runic-text | Font file could not be read. |
| `FontLoadError::Parse(String)` | cvkg-runic-text | Font file passed header check but failed during parsing. |
| `ShapingError` | cvkg-runic-text | Text shaping failed (malformed input or unsupported script). |
| `CliError` | cvkg-cli | Command-line operation failed (build, serve, scaffold, etc.). |
| `KvasirError` | cvkg-render-gpu | Kvasir render graph internal error. |
| `DocumentError::Io / Parse / Serialize` | cvkg-core | Document read/write/parse failure. |
| `FileDialogError::Cancelled` | cvkg-core | User cancelled the native file dialog. |
| `NotificationError::PermissionDenied` | cvkg-core | Notification permission not granted. |
| `SecurityError` | cvkg-core | Security boundary violation. |
| `ValidationError` | cvkg-core | Form field validation failed. |
| `ReflectError` | cvkg-reflect | Type reflection operation failed. |
| `SnapshotError` | cvkg-physics | Physics snapshot serialization/deserialization failed. |

## 7. Diagnostic Data

To collect diagnostic logs:

```bash
export RUST_LOG=debug
cargo run > cvkg_diagnostics.log 2>&1
```

The log contains adapter selection, font mapping, and layout proposal data. For GPU-specific diagnostics:

```bash
export RUST_LOG=cvkg_render_gpu=debug,wgpu=error
cargo run -p berserker 2>&1 | tee gpu_debug.log
```
