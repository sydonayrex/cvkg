# cvkg-render-software

CPU-based software rendering fallback for CVKG. Used for headless and test environments where GPU access is unavailable.

## Purpose

Provides a minimal CPU rasterizer that can render CVKG view trees without a GPU. Useful for CI, headless servers, and fallback when no GPU adapter is available.

## Boundaries

- Does NOT implement full GPU feature set (no compute shaders, no multi-pass pipeline).
- Does NOT provide windowing or event loop -- combine with a software surface library if you need display.
- Text rendering requires the `text` feature (enables `cvkg-runic-text`).

## Dependency Graph

```mermaid
graph TD
    cvkg-render-software["cvkg-render-software<br/>(CPU fallback)"]
    cvkg-render-software --> cvkg-core
    cvkg-render-software -.->|feature=text| cvkg-runic-text

    classDef render fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:1.5px
    classDef core fill:#1a1a2e,stroke:#1e293b,color:#e2e8f0,stroke-width:1px
    classDef text fill:#1c1917,stroke:#78716c,color:#d6d3d1,stroke-width:1px
    class cvkg-render-software render
    class cvkg-core core
    class cvkg-runic-text text
```

## Public API

- `Renderer` -- software rasterizer that processes draw commands into pixel buffers.
- Re-exports from `cvkg-core`: `Color`, `Rect`, `Renderer as RendererTrait` (the trait).

## Features

| Flag | Default | Effect |
|---|---|---|
| `text` | yes | Enables text shaping via `cvkg-runic-text` |

## Usage

```toml
[dependencies]
cvkg-render-software = { path = "../cvkg-render-software" }
```

```rust
use cvkg_render_software::Renderer;
use cvkg_core::{Color, Rect};

// Create a software renderer and issue draw commands.
```

## Use Cases

- Headless rendering in CI or server environments.
- Fallback when GPU adapter is unavailable.
- Visual regression testing (cvkg-test uses this path).

## Edge Cases

- Performance is significantly lower than GPU renderer. Not suitable for real-time UI.
- Without the `text` feature, text rendering is unavailable.
- Some advanced visual effects (blur, glass, glow) may not have CPU implementations.
