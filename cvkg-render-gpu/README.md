# cvkg-render-gpu

**cvkg-render-gpu** (Project Surtr) is the primary high-performance GPU renderer for CVKG, built on `wgpu`.

## Features

### `SurtrRenderer`
The "Forge" that translates CVKG drawing commands into GPU draw calls.
*   **Muspelheim Pipeline**: A multi-pass rendering pipeline supporting advanced effects.
*   **Bifrost (Frosted Glass)**: Real-time backdrop blur for refractive UI elements.
*   **Gungnir (Neon Glow)**: Additive bloom pass for emissive borders and text.
*   **Mjolnir (SDF Clipping)**: Sharp, geometric clipping for Cyberpunk aesthetics.
*   **Vector Iconography (Lyon)**: Direct GPU tessellation of SVG paths for resolution-independent icons.
*   **Multi-Window Architecture**: Shared `wgpu::Device` and `Queue` across multiple window surfaces for resource efficiency.
*   **Batching**: Efficiently batches primitives into high-throughput vertex buffers.
*   **Telemetry**: Built-in tracking for frame times, draw calls, and vertex throughput.
*   **Headless Support**: Hardened GPU pipeline for off-screen rendering and frame capture (Project Niflheim companion).

### Typography (Runic-Text)
*   Integrates `cvkg-runic-text` for high-fidelity text layout.
*   Maintains a dynamic GPU texture atlas for glyph caching.
*   Supports subpixel positioning and custom font embedding.

### Integration
*   Implements the `Renderer` trait from `cvkg-core`.
*   Supports `accesskit_winit` for integrated accessibility.
*   Handles native window surface management and configuration.
