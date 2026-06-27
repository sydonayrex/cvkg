# iOS Platform Expansion Feasibility

## Current State

CVKG targets desktop (via `winit`) and web (via WASM/WebGL). There is no iOS
rendering backend, no `MTKView` integration, and no `#[cfg(target_os = "ios")]`
conditional compilation anywhere in the workspace.

## Blocker Analysis

The primary blockers for iOS support are:

### 1. Windowing / Event Loop Ownership

`cvkg-render-native` wraps `winit` directly and owns the event loop. On iOS,
the event loop is owned by the host application (via `UIApplicationMain` /
`CADisplayLink`). CVKG would need a "subrenderer" mode where it renders into
a caller-provided `wgpu::Surface` backed by an `MTKView` or `CAMetalLayer`.

**Affected crates:** `cvkg-render-native` (confirmed)

**Platform-agnostic crates (no changes needed):**
- `cvkg-render-gpu` -- uses `wgpu` which already supports Metal on iOS
- `cvkg-vdom` -- pure Rust, no platform assumptions
- `cvkg-components` -- pure Rust view tree, no platform assumptions
- `cvkg-core` -- core traits are platform-agnostic
- `cvkg-themes` -- OKLCH color math is platform-agnostic

### 2. Surface Creation

`wgpu::Surface` on iOS requires a `CAMetalLayer` pointer from the host Swift/ObjC
code. CVKG's current surface creation in `cvkg-render-native` uses `winit`'s
window handle. A new iOS backend would need a `SurfaceConfig` struct that accepts
a raw `CAMetalLayer` pointer or a platform-specific surface descriptor.

### 3. Input Handling

Desktop uses `winit` events (keyboard, mouse, touch). iOS uses `UIKit` touch
events, gesture recognizers, and the iOS input system. A new input abstraction
layer would be needed to translate iOS touch events into CVKG's `InputEvent` type.

### 4. Accessibility

Desktop uses `AccessKit`. iOS uses `UIAccessibility`. A new accessibility bridge
would be needed to map CVKG's accessibility tree to `UIAccessibilityElement`.

## Proposed Approach

### Phase 1: Subrenderer Crate (`cvkg-render-subview`)

Create a new crate that provides a render mode where CVKG draws into a
caller-provided `wgpu::Surface` instead of owning the window:

```rust
pub struct SubviewRenderer {
    // Uses existing wgpu pipeline, but surface is external
}

impl SubviewRenderer {
    pub fn new(device: wgpu::Device, surface: wgpu::Surface) -> Self { ... }
    pub fn render(&mut self, view: &impl View) { ... }
    pub fn resize(&mut self, width: u32, height: u32) { ... }
}
```

This crate depends on `cvkg-render-gpu` and `cvkg-core` only.

### Phase 2: iOS Platform Crates

- `cvkg-render-ios` -- Surface creation from `CAMetalLayer`, touch input translation
- `cvkg-window-ios` -- `CADisplayLink`-based frame timer, `UIViewController` integration

### Phase 3: Swift Bindings

A Swift package that wraps `cvkg-render-ios` via C FFI, exposing a
`UIView` subclass that hosts the CVKG rendering surface.

## Dependencies

The good news: `wgpu` already supports Metal on iOS. No new GPU dependencies
needed. The work is entirely in the platform integration layer.

## Estimated Effort

- Subrenderer crate: 2-3 weeks
- iOS platform crate: 2-3 weeks
- Swift bindings: 1-2 weeks
- Testing and polish: 2-4 weeks

**Total estimate: 7-12 weeks** for a production-ready iOS backend.

## Risks

- `winit` may add iOS support in the future, which would simplify the windowing story
- `wgpu`'s Metal backend on iOS may have platform-specific bugs not encountered on macOS
- App Store review may reject apps with custom rendering pipelines (unlikely but possible)
