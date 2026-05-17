# Troubleshooting Guide

This guide provides diagnostics, symptoms, and resolution procedures for common compilation, runtime, and rendering failures encountered when building or running CVKG applications.

---

## 1. Compilation Failures

### Target 'wasm32-unknown-unknown' Missing
- **Symptoms**: The compiler throws errors during WASM compilation stating that target core libraries are missing:
  ```
  error[E0463]: can't find crate for `core` which `cvkg_render_web` depends on
  ```
- **Cause**: The standard library is not installed for the WebAssembly target.
- **Resolution**:
  Run the target addition tool via `rustup`:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### Tool 'wasm-pack' is Not Found
- **Symptoms**: Export scripts fail with:
  ```
  Failed to execute wasm-pack: No such file or directory (os error 2)
  ```
- **Cause**: The CLI requires `wasm-pack` for compiling, optimization, and generation of browser FFI bindings.
- **Resolution**:
  Install `wasm-pack` globally on your workstation:
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```

### Cyclic Dependency Between Rendering Backend and Core
- **Symptoms**: Cargo compilation stops with cyclical link warnings.
- **Cause**: An application manifest has selected multiple mutually-exclusive rendering features (e.g. both `native` and `web` simultaneously).
- **Resolution**:
  Audit the application dependency tree using `cargo tree` to identify package mismatches:
  ```bash
  cargo tree -d
  ```
  Ensure only **one** backend feature is selected in your final manifest:
  ```toml
  cvkg = { version = "0.1.20", features = ["native"] }
  ```

---

## 2. Runtime Crashes

### "GPU Device Not Found" (Native Desktop)
- **Symptoms**: The application crashes during start with:
  ```
  [GPU] Fatal error: RequestAdapter failed
  ```
- **Cause**: WGPU cannot locate a compatible hardware adapter (Vulkan/Metal/DX12) or the required system drivers are missing.
- **Resolution**:
  1. On Linux systems, verify that Vulkan works correctly:
     ```bash
     vulkaninfo
     ```
  2. Confirm your drivers are updated (e.g. proprietary NVIDIA or Mesa drivers).
  3. Force software rasterization on headless environments for basic verification:
     ```bash
     export WGPU_ADAPTER=mesa
     cargo run
     ```

### Winit Event Loop Collision (Wayland Linux)
- **Symptoms**: The application halts immediately with Wayland environment link crashes.
- **Cause**: Mismatches between operating system server environments (X11 vs Wayland).
- **Resolution**:
  Force the application to utilize XWayland compatibility mapping:
  ```bash
  export WINIT_UNIX_BACKEND=x11
  cargo run
  ```

---

## 3. Visual Artifacts

### Text Flickers or Missing Glyphs
- **Symptoms**: Procedural runes or labels blink or display as empty rectangles during high-frequency scrolls.
- **Cause**: The texture atlas is full, resulting in cache eviction thrashing.
- **Resolution**:
  1. Simplify the variety of font sizes active in the view.
  2. Increase the texture dimension limits inside the WGPU renderer configurations.
  3. Ensure that custom fonts are embedded and loaded directly to prevent system fallback discrepancies.

### Glowing Outlines Do Not Align With Element Bounds
- **Symptoms**: Glow shadows are displaced or warped relative to their parent components.
- **Cause**: Mismatched coordinate proposal mappings or subpixel rounding issues on High-DPI screens.
- **Resolution**:
  Check high-DPI scaling factor configurations and lock view dimensions to float coordinates:
  ```rust
  // Force integer boundary rounding for high-DPI displays
  let aligned_rect = rect.align_to_pixels(scale_factor);
  ```

---

## 4. Performance Bottlenecks

### Low Frame Rate or Frame Jitter
- **Symptoms**: Telemetry indicates frequent frames exceeding the 16.6ms rendering window.
- **Cause**: Heavy synchronous database operations or file I/O executed inside reactive `body()` composition routines.
- **Resolution**:
  1. Move all synchronous data loading out of component composition loops.
  2. Apply `.memoize()` on views that change infrequently to bypass full tree reconstruction.
  3. Run the compiler in release mode:
     ```bash
     cargo run --release
     ```

---

## 5. Diagnostic Data Collection

To gather diagnostic logs for maintainer review, execute the application with detailed tracing active:
```bash
# Activate debug tracing
export RUST_LOG=debug
cargo run > cvkg_diagnostics.log 2>&1
```
The resulting `cvkg_diagnostics.log` contains detailed reports on adapter selection, font mapping pipelines, and layout proposals.