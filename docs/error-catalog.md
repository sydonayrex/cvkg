# CVKG Error Catalog

Alphabetical reference of every structured error type in the CVKG workspace.
For each variant: when it occurs, what the user should do, what the programmer should check.

---

## CvkgError (cvkg-core)

Top-level error enum. All crate-level errors can be converted to this type for unified handling.

### `InvalidGeometry { rect, reason, suggestion }`

| | |
|---|---|
| **When** | A view is rendered with invalid rectangle dimensions (negative width/height, NaN, zero-size). |
| **User action** | Check the reported `rect` values. Usually a layout bug returning negative sizes. |
| **Programmer check** | Verify `intrinsic_size()` and layout computations return positive finite values. |

### `MissingFeature { feature, crate_name, suggestion }`

| | |
|---|---|
| **When** | A feature is used that requires a Cargo feature flag not enabled in this build. |
| **User action** | Enable the required feature in `Cargo.toml` (suggestion provides exact syntax). |
| **Programmer check** | Ensure `cfg(feature = "...")` gates match the Cargo.toml features. |

### `InvalidViewComposition { view_type, parent_type, suggestion }`

| | |
|---|---|
| **When** | A view is placed inside an incompatible parent view. |
| **User action** | Check the view hierarchy in the reported code. Usually a wrong nesting. |
| **Programmer check** | The View trait has conformance rules — verify parent-child compatibility. |

### `RendererInitFailed { backend, reason, suggestion }`

| | |
|---|---|
| **When** | The GPU renderer cannot be initialized (no GPU, driver issue, surface creation failure). |
| **User action** | Update GPU drivers. Check if the GPU is available. Try a different backend. |
| **Programmer check** | The renderer falls back gracefully via `on_fatal_error` — ensure the app handles this callback. |

### `RendererError { backend, message, suggestion }`

| | |
|---|---|
| **When** | A runtime rendering error occurs during drawing (device lost, surface error, vertex overflow). |
| **User action** | Usually transient — restart the application. If persistent, check GPU health. |
| **Programmer check** | Received via `on_render_error` trait method. Can be logged, counted, or used to trigger recovery. |

### `LayoutError { node_id, message, suggestion }`

| | |
|---|---|
| **When** | The taffy layout engine encounters constraint conflicts or invalid float values. |
| **User action** | Usually not actionable — this is a UI layout bug. |
| **Programmer check** | Check flex properties around the reported `node_id`. Look for circular constraints or NaN intrinsic sizes. |

---

## RenderError (cvkg-render-gpu)

Errors from the GPU rendering backend.

### `DeviceLost(String)`

| | |
|---|---|
| **When** | The GPU device is removed, reset, or becomes unavailable (e.g., GPU sleep, driver crash, laptop dock/undock). |
| **User action** | Recreate the renderer. If persistent, restart the application. |
| **Programmer check** | Implement `on_fatal_error` to show a "Recreating renderer…" UI state. |

### `Surface(String)`

| | |
|---|---|
| **When** | The wgpu surface cannot produce a frame (window minimized, surface lost, outdated). |
| **User action** | Resize or restore the window. |
| **Programmer check** | The renderer handles this automatically; use `on_render_error` to log frequency. |

### `MaterialCompile { name, reason }`

| | |
|---|---|
| **When** | A material graph fails to compile to WGSL (type mismatch, cycle, too many nodes). |
| **User action** | If using custom materials, validate node connections in the material editor. |
| **Programmer check** | Run `Material::validate()` before compiling. The `reason` field provides specific diagnostic info from the taffy-like graph validator. |

### `ShaderValidation(String)`

| | |
|---|---|
| **When** | WGSL shader compilation fails (syntax error, type mismatch, unsupported feature). |
| **User action** | Check if the GPU supports the required shader features. Update drivers. |
| **Programmer check** | The inner string contains the WGSL validation error with line/column info. Log it faithfully. |

### `UnsupportedFormat(wgpu::TextureFormat)`

| | |
|---|---|
| **When** | The requested surface texture format is not supported by the GPU adapter. |
| **User action** | Try a different GPU or backend. |
| **Programmer check** | Query `surface.get_capabilities(&adapter)` for supported formats before configuring. |

### `VertexOverflow { needed, max }`

| | |
|---|---|
| **When** | More vertices are submitted in a single frame than the vertex buffer pool can hold. |
| **User action** | Reduce scene complexity. |
| **Programmer check** | Increase `max_vertices` in `GeometryBuffers` capacity, or batch draw calls across frames. |

### `FrameAcquire(String)`

| | |
|---|---|
| **When** | `surface.get_current_texture()` fails (surface lost, out of memory, outdated). |
| **User action** | Restart the application. If persistent, reduce GPU memory usage. |
| **Programmer check** | The renderer will attempt recreation; log frequency via `on_render_error`. |

---

## NativeError (cvkg-render-native)

Errors from the native windowing backend.

### `WindowCreation(String)`

| | |
|---|---|
| **When** | The native window cannot be created (display server connection failed, invalid monitor). |
| **User action** | Check display server status (Wayland/X11). Try a different monitor configuration. |
| **Programmer check** | Verify winit event loop initializes correctly. The inner string provides OS-level error details. |

### `GpuInit(String)`

| | |
|---|---|
| **When** | GPU initialization fails during application startup. |
| **User action** | Update GPU drivers. Verify the GPU supports Vulkan/Metal/DX12. |
| **Programmer check** | The inner string provides wgpu adapter init error details. |

### `DiffEmpty`

| | |
|---|---|
| **When** | The VDom diff engine returned `None` when a patch set was expected (logic bug). |
| **User action** | Report this as a bug with the UI state that triggered it. |
| **Programmer check** | This is now handled gracefully (skips patching for that frame). Check VDom diff logic. |

### `WindowDestroyed(WindowId)`

| | |
|---|---|
| **When** | Events are dispatched to a window after it has been destroyed. |
| **User action** | Report this as a bug. |
| **Programmer check** | Event processing should stop when a window is destroyed. The `WindowId` identifies which window. |

### `EventLoop(String)`

| | |
|---|---|
| **When** | The event loop encounters an error. |
| **User action** | Restart the application. |
| **Programmer check** | The inner string provides winit event loop error details. |

---

## LayoutError (cvkg-layout)

Errors from the layout engine.

### `ConstraintConflict { node_id, reason }`

| | |
|---|---|
| **When** | The taffy solver cannot resolve flex constraints (circular dependencies, over-constrained). |
| **User action** | Usually not actionable — this is a UI layout bug. |
| **Programmer check** | Inspect flex properties around `node_id`. Check for conflicting `width`/`min_width`/`max_width` or circular flex nesting. |

### `CapacityExceeded(String)`

| | |
|---|---|
| **When** | The layout engine exceeds configured capacity limits (node count, edge count). |
| **User action** | Simplify the UI. Reduce deeply nested views. |
| **Programmer check** | Increase `max_nodes` / `max_edges` in `MaterialValidationConfig` or layout engine capacity. |

### `InvalidFloat { node_id }`

| | |
|---|---|
| **When** | A NaN or Inf value propagates through layout calculations. |
| **User action** | Report this as a bug. |
| **Programmer check** | Verify that `intrinsic_size()` returns finite values. Any `f32::NAN` or `f32::INFINITY` from size calculations will trigger this. |

### `Internal(String)`

| | |
|---|---|
| **When** | An internal error occurs in the layout engine that doesn't fit other categories. |
| **User action** | Report this as a bug with the UI state that triggered it. |
| **Programmer check** | The inner string provides context about what went wrong. |

---

## Writing Good Error Messages (Convention)

All CVKG error messages follow this format:

```
<What happened>. <What the user should do or check>.
```

Rules:
1. Start with the specific failure, not a generic prefix.
2. Include relevant values (node IDs, vertex counts, format types).
3. End with a concrete suggestion or link to what to investigate.
4. Never use words like "unexpected", "invalid", "failed" without context — say *what* failed, *why*, and *where*.
