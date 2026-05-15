# Troubleshooting Guide

This guide addresses common issues encountered when building or running CVKG applications.

## 1. Rendering Issues

### "GPU Device Not Found"
- **Cause**: No compatible GPU adapter was found.
- **Fix**: 
  - Ensure your drivers are updated.
  - On Linux, check if `vulkan-utils` reports a valid device.
  - On Windows/macOS, ensure you are not running in a VM without GPU passthrough.

### "Black Screen" (Native)
- **Cause**: Often a Wayland/X11 mismatch or a missing font.
- **Fix**:
  - Try running with `WINIT_UNIX_BACKEND=x11` or `wayland`.
  - Check the logs for `[GPU] Selected adapter`. If it shows `Software`, performance will be poor and some shaders may fail.

### "Flickering Text"
- **Cause**: Mega-Atlas exhaustion or subpixel rounding issues.
- **Fix**:
  - Increase the atlas size in `cvkg-render-gpu` (requires rebuild).
  - Ensure you are using a consistent `scale_factor` (check HighDPI settings).

## 2. Compilation Issues

### "Cyclic Dependency"
- **Cause**: Usually a circular link between `cvkg-core` and a rendering backend.
- **Fix**:
  - Ensure you only select **ONE** rendering feature in your `Cargo.toml`.
  - Use `cargo tree -d` to identify the cycle.

### "Missing Feature: 'native'"
- **Cause**: The main `cvkg` crate was included without specifying a backend.
- **Fix**:
  ```toml
  cvkg = { version = "0.1.18", features = ["native"] }
  ```

## 3. Platform Specifics

### Linux (Wayland)
If the application crashes on startup:
```bash
# Force X11 backend if Wayland drivers are unstable
export WINIT_UNIX_BACKEND=x11
cargo run
```

### WASM (Web)
If the page is stuck on "Loading...":
- Open Browser DevTools (F12).
- Check for `WebGPU not supported` errors.
- Ensure the `cvkg-webkit-server` is pointing to the correct `pkg/` directory.

## 4. Performance Degressions

### High Jitter / Frame Drops
- Check `cvkg inspect` for `VDom Diff` times. If high, simplify your view hierarchy.
- Ensure you are not performing heavy I/O or calculations in the `body()` method.
- Activate `GodMode` in the telemetry dashboard to prioritize the render thread.

## 5. Reporting New Issues

If your problem is not listed:
1. Run with `RUST_LOG=debug`.
2. Capture a screenshot or trace.
3. Open a GitHub Issue with your OS, GPU, and Rust version.