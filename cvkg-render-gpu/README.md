# cvkg-render-gpu

![GPU Shader Pipeline](../docs/images/gpu_shader_pipeline.png)

**cvkg-render-gpu** (Project Surtr) is the primary GPU-accelerated renderer for CVKG, built on `wgpu`.

## What This Crate Does

- Implements the `Renderer` trait using WebGPU (wgpu)
- Provides high-performance drawing operations with shader effects
- Supports Bifrost (frosted glass), Gungnir (neon glow), and Mjolnir (geometric effects)
- Implements accessibility via AccessKit integration

## What This Crate Does NOT Do

- Does not provide window management (see cvkg-render-native)
- Does not provide web bindings (see cvkg-render-web)
- Does not provide text shaping (see cvkg-runic-text)

## Public API Overview

### SurtrRenderer

```rust
/// The main GPU renderer for CVKG
pub struct SurtrRenderer {
    // private fields
}

impl SurtrRenderer {
    /// Create a new renderer with the given dimensions
    pub fn new(width: u32, height: u32) -> Self;
    
    /// Resize the renderer surface
    pub fn resize(&mut self, width: u32, height: u32);
    
    /// Render a view with the given bounds
    pub fn render<V: View>(&mut self, view: &V, rect: Rect);
    
    /// Capture the current frame as PNG bytes
    pub fn capture_png(&mut self) -> Vec<u8>;
}```

### Renderer Trait Implementation

```rust
// Core drawing operations
fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]);
fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]);
fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]);
fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], stroke_width: f32);
```

### Visual Effects

```rust
// Bifrost (frosted glass)
fn bifrost(&mut self, rect: Rect, blur: f32, saturation: f32, opacity: f32);

// Gungnir (neon glow)
fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32);

// Mjolnir (geometric effects)
fn push_mjolnir_slice(&mut self, angle: f32, offset: f32);
fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]);
```

### Re-exports

```rust
pub use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
pub use cvkg_core::{ColorTheme, SceneUniforms};
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | true | Use standard library |
| `shader-validation` | false | Validate WGSL shaders at compile time |

## Usage Example

```rust
use cvkg_render_gpu::SurtrRenderer;
use cvkg_core::{View, Rect};
use cvkg_components::{VStack, Text, Button};
use cvkg_core::View;

fn main() {
    let mut renderer = SurtrRenderer::new(800, 600);
    
    let app = VStack::new(16.0)
        .child(Text::new("GPU Rendered UI").size(24.0))
        .child(Button::new("Click Me"));
    
    // Render the app
    renderer.render(&app, Rect::new(0.0, 0.0, 800.0, 600.0));
    
    // Capture for testing or export
    let png = renderer.capture_png();n}
```

## Known Limitations

- Requires GPU with Vulkan, Metal, or DirectX 12 support
- Headless rendering requires appropriate WGPU backend features
- Large texture atlases may hit GPU memory limits on integrated graphics