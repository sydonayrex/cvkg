# Troubleshooting

This document lists common failure modes, their causes, and fixes.

## Build Failures

### Symptom
`error: failed to run custom build command for 'cvkg-render-gpu'`

### Cause
Missing Vulkan SDK or `vulkan.h` header.

### Fix

```bash
# Ubuntu/Debian
sudo apt-get install -y libvulkan-dev vulkan-sdk

# macOS
brew install vulkan-sdk
```

---

### Symptom
`error: no renderer feature enabled`

### Cause
Building without specifying a renderer feature.

### Fix

```bash
cargo build --features gpu    # For GPU renderer
cargo build --features web    # For web renderer
cargo build --features native   # For native renderer
```

---

### Symptom
`error: linking with `cc` failed` on Windows

Cause
Missing Visual Studio build tools or Windows SDK.

Fix
Install Visual Studio with "Desktop development with C++" workload.

---
## Runtime Failures

### Symptom
Application panics with `no adapter found` or `no suitable GPU`

Cause
No compatible GPU or drivers.

Fix
Ensure Vulkan/Metal/DX12 capable GPU is available. For headless, use software adapter:

```bash
export WGPU_ADAPTER=mesa # Linux software rendering
```

---
### Symptom
`panic!: layout cache overflow`

Cause
Too many unique layout configurations in a single frame.

Fix
Reduce the number of distinct sized views or increase the cache size in `cvkg-core`.

---
### Symptom
Text appears as boxes or question marks

Cause
Missing glyph in loaded fonts.

Fix
Register additional fonts with the shaper or verify font data is valid.

---
## Panic Messages in Source

The following panic!() calls are documented from the source code:

### `View::body()` panic
Symptom: `body must return Self::Body`
Cause: Incorrect Body type implementation
Fix: Ensure `body()` returns the correct associated type

### Renderer initialization panic
Symptom: `renderer not initialized`
Cause: Calling render before setup complete
Fix: Ensure window/surface creation succeeded before rendering

---
## Environment Variables

| Variable | Expected | Failure Mode if Missing |
|----------|----------|------------------------|
| `WGPU_ADAPTER` | `discrete`, `integrated`, or `mesa` | Runtime selects wrong GPU |
| `RUST_BACKTRACE` | `1` | No stack trace on panic |
| `WGPU_LOG` | `info` or `debug` | No GPU diagnostic output |

---
## Error Types

### `StateUpdateError`
Cause: Concurrent state mutation during render
Fix: Ensure state updates happen on main thread only

### `FontLoadError`
Cause: Invalid font data or unsupported format
Fix: Verify font files are valid TTF/OTF

### `ShaderCompileError`
Cause: Invalid WGSL shader code
Fix: Check shader syntax and feature support

---
## Getting Help

If the above does not resolve the issue:
1. Run with `RUST_BACKTRACE=full` and include output
2. File an issue at https://github.com/sydonayrex/cvkg/issues
3. Include your GPU model and driver version