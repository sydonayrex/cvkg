# cvkg-render-gpu

**cvkg-render-gpu** (Project Surtr) is the primary high-performance GPU renderer for CVKG, built on `wgpu`.

## 🚀 Quick Start

```rust
use cvkg_render_gpu::SurtrRenderer;
use cvkg_components::{Text, VStack, Button};
use cvkg_core::View;

fn main() {
    // 1. Create renderer
    let mut renderer = SurtrRenderer::new(800, 600);
    
    // 2. Build UI
    let app = VStack::new(16.0)
        .child(Text::new(