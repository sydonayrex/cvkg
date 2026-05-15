# cvkg-render-web

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-render-web` provides a multi-tier rendering strategy for CVKG applications running in the browser via WebAssembly, supporting everything from WebGPU to Canvas2D fallbacks.

## Boundaries and Responsibilities

This crate manages the browser execution environment. It focuses on:
- Providing three rendering tiers: WebGPU (Tier 1), WebGL2 (Tier 2), and Canvas2D (Tier 3).
- Bridging the CVKG VDOM to the browser's DOM for accessibility and developer tooling.
- Managing asynchronous asset loading via browser Fetch APIs.
- Supporting hot-module replacement (HMR) and snapshots via the WebKit server.

## Public API Overview

### Core Types
- `WebRenderer`: The unified entry point for web-based rendering, automatically selecting the best available tier.
- `RenderTier`: An enum representing the active rendering capability (Tier1GPU, Tier2GPU, Tier3Fallback).
- `WebKitBridge`: Handles WebSocket communication with the development server for HMR.

### Web Tiers
- **Tier 1 (WebGPU)**: Direct GPU access for full feature parity with native rendering (if supported by browser).
- **Tier 2 (WebGL2)**: Hardware-accelerated fallback using GLSL-compatible shaders.
- **Tier 3 (Canvas2D)**: Software-simulated fallback for maximum compatibility.

## Usage Example

```rust
use cvkg_render_web::WebRenderer;

#[wasm_bindgen(start)]
pub async fn start() {
    let mut renderer = WebRenderer::new();
    renderer.forge().await.expect("Failed to initialize web renderer");
    
    // Start the render loop...
}
```

## Known Limitations
- WebGPU support is currently limited to browsers with the feature enabled (e.g., Chrome with Unsafe WebGPU).
- Tier 3 rendering does not support complex Mjolnir shattering effects due to CPU bottlenecks.
- Text rendering in WASM relies on the browser's canvas text measurement APIs.