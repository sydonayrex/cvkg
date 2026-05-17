# cvkg-render-native

![Berserker Hero HUD](../docs/images/berserker_hero_hud.png)

`cvkg-render-native` provides platform-native windowing and event loop integration for CVKG desktop applications using `winit` and `AccessKit`.

## Boundaries and Responsibilities

This crate acts as the host environment for native applications. It does NOT implement low-level GPU drawing (delegated to `cvkg-render-gpu`). Its responsibilities include:
- Managing the OS window lifecycle and event loop via `winit`.
- Bridging the CVKG VDOM to the platform accessibility tree using `AccessKit`.
- Dispatching native input events (Keyboard, Mouse, IME) into the CVKG event system.
- Providing high-resolution frame timing and jitter telemetry for performance monitoring.
- Managing "Berserker Mode" OS-level scheduler priorities for high-priority rendering.

## Public API Overview

### Entry Points
- `NativeRenderer::run<V: View>(view: V)`: The primary entry point for launching a CVKG desktop application.

### Key Types
- `NativeRenderer`: Implements the `Renderer` trait by wrapping a GPU-accelerated Surtr instance.
- `App`: The internal `winit` application handler managing windows and GPU contexts.
- `NativeAssetManager`: A concrete asset loader for the local filesystem using `arc-swap` for lock-free reads.

### Critical Features
- **Kinetic Injection**: Translates window movement into "Rage" telemetry for dynamic UI effects.
- **ShieldWall Integration**: Automatic generation of accessibility trees from VNodes.

## Usage Example

```rust
use cvkg_render_native::NativeRenderer;
use cvkg_core::View;

fn main() {
    let app_view = MyApp::new();
    NativeRenderer::run(app_view);
}
```

## Known Limitations
- Multi-window support is implemented but experimental; focus management across windows is handled via the VDOM bridge.
- Wayland support requires specific system dependencies (see root README).
- Hardware verification is required; do not rely on mocks for this crate.