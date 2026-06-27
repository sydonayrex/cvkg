# Error Propagation Implementation Plan

## Purpose

Replace silent panics and unwrapped Results with structured error propagation across CVKG, so that runtime failures produce actionable diagnostic messages instead of segfaults or cryptic backtraces.

## Scope

This plan targets the **4 highest-risk areas** identified in the audit, ordered by blast radius:

1. **cvkg-render-gpu** — material compile failures, surface errors, device loss
2. **cvkg-render-native** — window lifecycle, GPU init, VDom diff unwraps
3. **cvkg-layout** — taffy engine panics on constraint conflicts
4. **cvkg-core** — top-level panic hook, `CvkgError` integration, event dispatch protection

Out of scope (deferred): component prop validation, software renderer warnings, `unreachable!()` cleanup in leaf views.

---

## Phase 1: Define the Error Architecture

### 1.1 New error enums

**`cvkg-render-gpu/src/error.rs`** (new file):

```rust
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("GPU device lost: {0}. Try recreating the renderer.")]
    DeviceLost(String),

    #[error("Surface error: {0}. Check window state and GPU availability.")]
    Surface(#[from] wgpu::SurfaceError),

    #[error("Material compile failed for graph '{name}': {reason}. Validate node connections and types.")]
    MaterialCompile { name: String, reason: String },

    #[error("Shader validation failed: {0}. See inner WGSL error for line/column.")]
    ShaderValidation(String),

    #[error("Surface format {0:?} not supported by this adapter. Try a different backend.")]
    UnsupportedFormat(wgpu::TextureFormat),

    #[error("Vertex buffer overflow: needed {needed} vertices, max is {max}. Batch your geometry or increase pool size.")]
    VertexOverflow { needed: usize, max: usize },

    #[error("Failed to acquire next frame from surface: {0}")]
    FrameAcquire(String),
}
```

**`cvkg-render-native/src/error.rs`** (new file):

```rust
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("Window creation failed: {0}. Check display server connection.")]
    WindowCreation(String),

    #[error("GPU initialization failed: {0}. Verify drivers and GPU availability.")]
    GpuInit(String),

    #[error("VDom diff produced no patches but rebuild was expected. This is a bug in the VDom diff engine.")]
    DiffEmpty,

    #[error("Window {0} was destroyed but events are still being dispatched. Remove event handlers before destroying windows.")]
    WindowDestroyed(WindowId),
}
```

**`cvkg-layout/src/error.rs`** (new file):

```rust
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Layout constraint conflict in node {node_id}: {reason}. Check flex properties for circular or over-constrained layouts.")]
    ConstraintConflict { node_id: u64, reason: String },

    #[error("Layout engine capacity exceeded: {0}. Reduce UI complexity or increase limits.")]
    CapacityExceeded(String),

    #[error("NaN or Inf value propagated through layout calculations at node {node_id}. Check intrinsic_size return values for invalid floats.")]
    InvalidFloat { node_id: u64 },
}
```

### 1.2 Extend `CvkgError` in cvkg-core

Add two variants to the existing enum in `cvkg-core/src/error_types.rs`:

```rust
RendererError {
    backend: String,
    message: String,
    suggestion: String,
},
LayoutError {
    node_id: Option<u64>,
    message: String,
    suggestion: String,
},
```

This unifies all crate-level errors under one enum that can be returned from the Renderer trait and propagated to the application layer.

---

## Phase 2: Wire Errors into the Renderer Trait

### 2.1 Add fallible methods to Renderer trait

In `cvkg-core/src/renderer_trait.rs`, add a new trait for fallible operations:

```rust
pub trait RendererCore: Renderer {
    /// Called when a non-fatal render error occurs.
    /// Default implementation logs the error.
    /// Backends can override to report to app-level error handlers.
    fn on_render_error(&mut self, error: &CvkgError) {
        log::error!("[RenderError] {error}");
    }

    /// Called when a fatal render error occurs that prevents further rendering.
    /// Default implementation logs and requests shutdown.
    fn on_fatal_error(&mut self, error: &CvkgError) {
        log::error!("[Fatal] {error}");
    }

    /// Returns true if the backend is in a recoverable error state.
    fn has_error(&self) -> bool { false }
}
```

### 2.2 Make existing error sites use the trait

Replace `expect()` calls in `init.rs` with error returns that feed `on_fatal_error`:

- Surface creation failure → `on_fatal_error(CvkgError::RendererError { ... })`
- Adapter not found → same
- Device creation failure → same

These remain fatal (app cannot run without GPU), but now produce a structured error message instead of a panic with a string.

---

## Phase 3: Fix the GPU Renderer (Highest Risk)

### 3.1 Replace `unwrap()` in material compilation

In `cvkg-render-gpu/src/material.rs`, lines 1065-1106:

**Before:**
```rust
let compiled = MaterialCompiler::compile(&graph).unwrap();
```

**After:**
```rust
let compiled = MaterialCompiler::compile(&graph)
    .map_err(|e| RenderError::MaterialCompile {
        name: format!("graph_{i}"),
        reason: e.to_string(),
    })?;
```

This requires changing the function signature of the callers to return `Result<_, RenderError>`. The callers are in the material validation test runner and the material cache. The cache entry can be `Result<CompiledMaterial, RenderError>` so that a failed compile is cached as an error and doesn't re-panic on every frame.

### 3.2 Handle surface errors

In `cvkg-render-gpu/src/renderer/draw.rs`, the frame acquisition:

**Before:**
```rust
let frame = surface.get_current_texture().expect("Failed to acquire frame");
```

**After:**
```rust
let frame = surface.get_current_texture()
    .map_err(|e| RenderError::Surface(e))?;
```

### 3.3 Handle device lost

In `cvkg-render-gpu/src/renderer/mod.rs`, add a device lost watcher:

```rust
fn check_device_lost(&mut self) -> Result<(), RenderError> {
    // Poll device for lost state
    // Return RenderError::DeviceLost if detected
}
```

### 3.4 Replace `unwrap()` in draw call state

In `cvkg-render-gpu/src/api/shapes.rs`, lines 74-75:

**Before:**
```rust
|| self.draw_calls.last().unwrap().scissor_rect != self.clip_stack.last().copied()
```

**After:**
```rust
|| self.draw_calls.last().map_or(false, |dc| {
    self.clip_stack.last().map_or(false, |&clip| dc.scissor_rect == clip)
})
```

This silently skips the optimization rather than panicking when state is corrupted, and logs via `on_render_error`.

### 3.5 Replace `expect()` in frame.rs

**Before:**
```rust
.expect("No target window set for frame. Call set_target_window first.")
```

**After:**
```rust
.map_err(|_| RenderError::FrameAcquire("No target window set. Call set_target_window before rendering.".into()))?
```

---

## Phase 4: Fix the Native Renderer

### 4.1 Replace VDom diff unwraps

In `cvkg-render-native/src/main_loop.rs`, lines 334, 361, 380:

**Before:**
```rust
let patches = diff_patches.as_ref().unwrap();
```

**After:**
```rust
let patches = diff_patches.as_ref()
    .ok_or(NativeError::DiffEmpty)
    .map_err(|e| CvkgError::RendererError {
        backend: "native".into(),
        message: e.to_string(),
        suggestion: "Report this as a bug with the UI state that triggered it.".into(),
    })?;
```

### 4.2 Replace window-not-found panic

**Before:**
```rust
panic!("winit_id not found for window handle: window may have been destroyed")
```

**After:**
```rust
return Err(NativeError::WindowDestroyed(self.winit_id).into());
```

### 4.3 Replace window creation expect

**Before:**
```rust
.expect("failed to create native window")
```

**After:**
```rust
.map_err(|e| NativeError::WindowCreation(e.to_string()))?
```

---

## Phase 5: Fix the Layout Engine

### 5.1 Wrap taffy calls

In `cvkg-layout/src/taffy_engine.rs`, wrap the 6 highest-risk `unwrap()` calls:

**Before:**
```rust
let new_node = engine.tree.new_leaf(style).unwrap();
```

**After:**
```rust
let new_node = engine.tree.new_leaf(style)
    .map_err(|e| LayoutError::ConstraintConflict {
        node_id,
        reason: format!("{e}"),
    })?;
```

### 5.2 Handle layout computation

**Before:**
```rust
let layout = engine.tree.layout(node).unwrap();
```

**After:**
```rust
let layout = engine.tree.layout(node)
    .map_err(|e| LayoutError::ConstraintConflict {
        node_id,
        reason: format!("{e}"),
    })?;
```

### 5.3 Add NaN detection

After computing layout rects, add a validation pass:

```rust
if !rect.x.is_finite() || !rect.y.is_finite() || !rect.width.is_finite() || !rect.height.is_finite() {
    return Err(LayoutError::InvalidFloat { node_id });
}
```

---

## Phase 6: Add Top-Level Panic Hook

### 6.1 Install panic hook in main entry points

In `cvkg-render-native/src/main_loop.rs` and any binary entry point:

```rust
std::panic::set_hook(Box::new(|info| {
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    };
    log::error!("[CVKG PANIC] {msg}");
    log::error!("[CVKG PANIC] Backtrace:\n{}", std::backtrace::Backtrace::force_capture());
    // Write crash dump to disk for post-mortem debugging
    if let Ok(mut file) = std::fs::File::create("cvkg-crash.log") {
        let _ = writeln!(file, "CVKG Panic Dump");
        let _ = writeln!(file, "Message: {msg}");
        let _ = writeln!(file, "Backtrace:\n{}", std::backtrace::Backtrace::force_capture());
    }
}));
```

### 6.2 Add event dispatch panic protection

In `cvkg-vdom/src/lib.rs`, wrap event dispatch in `catch_unwind`:

```rust
fn dispatch_event(&self, event: Event) {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        // existing dispatch logic
    }));
    if let Err(panic) = result {
        log::error!("[VDom] Event handler panicked: {panic:?}");
        // Continue processing other events rather than crashing
    }
}
```

---

## Phase 7: Tests

### 7.1 Error propagation tests

For each new error variant, write a test that:
1. Triggers the error condition
2. Asserts the error message contains the expected context
3. Asserts the error chain (if `thiserror` is used) is correct

```rust
#[test]
fn material_compile_failure_returns_error_not_panic() {
    let graph = make_cyclic_graph(); // guaranteed to fail compilation
    let result = MaterialCompiler::compile(&graph);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("cycle"), "error should mention cycle");
}
```

### 7.2 Error recovery tests

```rust
#[test]
fn device_lost_triggers_on_fatal_error() {
    let mut mock = MockRenderer::new();
    mock.simulate_device_lost();
    assert!(mock.has_error());
    // Verify the error was reported, not panicked
}
```

### 7.3 Panic hook test

```rust
#[test]
fn panic_hook_writes_crash_log() {
    // Spawn a subprocess that panics
    // Assert cvkg-crash.log exists and contains the panic message
}
```

### 7.4 Layout NaN detection test

```rust
#[test]
fn layout_nan_returns_error_not_panic() {
    let mut engine = LayoutEngine::new();
    engine.set_style(Style::default().width(NAN));
    let result = engine.compute_layout(root_node);
    assert!(matches!(result, Err(LayoutError::InvalidFloat { .. })));
}
```

---

## Phase 8: Documentation

### 8.1 Error catalog

Create `docs/error-catalog.md` listing every error variant with:
- When it occurs
- What the user should do
- What the programmer should check

### 8.2 Migration guide

Document the API changes:
- `Renderer` trait now has `on_render_error` / `on_fatal_error`
- Functions that previously panicked now return `Result`
- How to install the panic hook in your own binary

---

## Implementation Order

| Order | Phase | Crates | Est. LOC | Risk |
|-------|-------|--------|----------|------|
| 1 | Error enums | core, render-gpu, render-native, layout | ~120 | Low |
| 2 | Renderer trait extension | core | ~25 | Low |
| 3 | GPU renderer fixes | render-gpu | ~200 | High |
| 4 | Native renderer fixes | render-native | ~80 | High |
| 5 | Layout engine fixes | layout | ~100 | Medium |
| 6 | Panic hook | core, render-native | ~40 | Low |
| 7 | Tests | all | ~300 | Medium |
| 8 | Documentation | docs | ~150 | Low |

**Total: ~1015 LOC changed across 4 crates.**

## Verification

After each phase:
1. `cargo check --workspace` — zero errors
2. `cargo test -p <crate>` — all existing tests pass
3. `cargo test -p <crate> -- new_error_tests` — new tests pass
4. Manual: trigger each error condition in the gallery demo, verify the error message is logged (not a panic)

## Design Decisions

1. **Why `thiserror` instead of manual `Display`?** — `thiserror` generates `Display`, `Error`, `From` impls with zero boilerplate. It's already a dependency in cvkg-core.

2. **Why `on_render_error` trait method instead of returning `Result` from `render()`?** — The `render()` method is called in a hot loop across hundreds of views. Returning `Result` from every draw call would require unwrapping at every call site or propagating through the entire View tree. The trait-method approach lets backends decide how to handle errors (log, count, abort) without breaking the View trait signature.

3. **Why not use `anyhow`?** — `anyhow` is for applications, not libraries. CVKG is a library; it should return structured errors so applications can match on specific variants. `thiserror` is the right choice here.

4. **Why keep some `expect()` calls?** — Calls that are genuinely "impossible by construction" (e.g., `NonZeroUsize::new(1024)` where the value is a constant > 0) are kept. Only calls where the value comes from runtime data or external state are converted.
