# cvkg-render-gpu

![GPU Shader Pipeline](../docs/images/gpu_shader_pipeline.png)

`cvkg-render-gpu` is the "Surtr" pipeline: a high-performance, `wgpu`-powered GPU renderer designed for "Berserker" aesthetics and complex visual effects.

## Boundaries and Responsibilities

This crate is the authoritative drawing backend for CVKG. It does NOT handle windowing or higher-level layout. Its responsibilities include:
- Managing the Muspelheim multi-pass pipeline (Bloom, Blur, Composite).
- Tessellating 2D primitives and SVG strokes into GPU-ready meshes using `lyon`.
- Managing a Mega-Atlas for high-efficiency text and image batching.
- Executing real-time parsing and evaluation of Animated SVGs via `roxmltree` and `usvg`.
- Propagating full 3x3 affine transformation matrices for complex nested scaling, skew, and rotation.
- Implementing advanced shader effects: Bifrost (frost), Gungnir (glow), and Mjolnir (geometric clipping/shattering).
- Optimizing VRAM usage via LRU-based cache eviction and atlas recycling.

## Public API Overview

### Core Types
- `SurtrRenderer`: The central GPU controller managing device state, pipelines, and buffers.
- `Vertex`: The unified vertex format supporting position, color, radius, and effect-specific metadata.
- `DrawCall`: Represents a batched GPU operation organized by texture and transparency layer.

### Systems
- **Muspelheim Passes**: Specialized pipelines for Gaussian blur and bloom extraction.
- **ShelfPacker**: A high-speed atlas packing algorithm for real-time UI textures.
- **RunicTextEngine**: Integration with `cvkg-runic-text` for GPU-accelerated glyph rendering.

### Key Methods
- `SurtrRenderer::forge()`: Initializes the GPU device and pipelines for a target window.
- `SurtrRenderer::begin_frame()` / `SurtrRenderer::end_frame()`: Manages the command encoder lifecycle.

## Known Limitations
- Requires a GPU supporting `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`.
- Texture atlas size is currently fixed at 4096x4096px.
- Culling is hierarchical but relies on the `cvkg-scene` crate for complex queries.