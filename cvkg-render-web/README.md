# cvkg-render-web

**cvkg-render-web** provides WebGPU/WebAssembly bindings for CVKG to run in web browsers.

## What This Crate Does

- Implements the `Renderer` trait using WebGPU via wasm-bindgen
- Provides WASM bindings for browser deployment
- Handles web-specific input events (touch, mouse, keyboard)
- Integrates with web page lifecycle events

## What This Crate Does NOT Do

- Does not provide native window management (see cvkg-render-native)
- Does not provide desktop GPU rendering (see cvkg-render-gpu)
- Does not handle web server functionality

## Public API Overview

### WasmRenderer

```rust
/// The WebGPU renderer for WASM targets
pub struct WasmRenderer {
    // private fields
}

impl WasmRenderer {
    /// Create a new WASM renderer for a canvas element
    pub fn new(canvas_id: &str) -> Self;
    
    /// Render a view with the given bounds
    pub fn render<V: View>(&mut self, view: &V, rect: Rect);
}
```

### WebAssembly Exports

```rust
/// Initialize the renderer for the given canvas
#[wasm_bindgen]
pub fn init(canvas_id: &str) -> WasmRenderer;

/// Render a frame
#[wasm_bindgen]
pub fn render_frame();
```

### Re-exports

```rust
pub use cvkg_core::{Rect, Size, Color};
pub use wasm_bindgen::prelude::*;
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | true | Use standard library |
| `web` | false | Enable web-specific features |

## Building for Web

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build for web
cargo build --target wasm32-unknown-unknown --features web
```

## Usage Example

```rust
use cvkg_render_web::WasmRenderer;
use cvkg_components::{VStack, Text};
use cvkg_core::View;

#[wasm_bindgen]
pub fn run_app() {
    let mut renderer = WasmRenderer::new("canvas");
    let app = VStack::new(16.0).child(Text::new("Hello, Web!"));
    renderer.render(&app, cvkg_core::Rect::new(0.0, 0.0, 800.0, 600.0));
}
```

## Known Limitations

- Requires browser with WebGPU support (Chrome 113+, Firefox 115+)
- WebGL fallback not implemented
- Canvas size changes require manual resize handling
- Performance may be lower than native GPU rendering