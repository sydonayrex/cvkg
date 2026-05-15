# CVKG: Cyber Viking Kvasir Graph

![CVKG Hero HUD](docs/images/cvkg_hero.png)

CVKG is a high-fidelity, agentic UI framework for Rust designed for building visually stunning native and web applications.

## Problem and Audience

CVKG provides a declarative, functional-reactive UI system that delivers high-fidelity visual effects and smooth animations without sacrificing performance or developer ergonomics. This framework is for UI developers and creative technologists who need to build complex, high-performance interfaces with advanced graphical features like translucent frosting, glowing outlines, and physics-based animations.

## Prerequisites

- **Rust Toolchain**: Rust 1.85.0 or later (Edition 2024).
- **System Dependencies**: Vulkan, Metal, or DX12 capable GPU. Linux users require `libfontconfig1-dev` and `pkg-config`.
- **WASM Support**: `wasm-pack` is required for web builds.

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/sydonayrex/cvkg.git && cd cvkg

# 2. Build the workspace
cargo build --workspace

# 3. Run the shatter demo (requires GPU)
cargo run --example shatter_demo -p cvkg --features gpu

# 4. Run the full test suite
cargo test --workspace
```

## Workspace Crate Map

| Crate | Role |
| :--- | :--- |
| `cvkg` | Entry point crate providing the public facade and feature-gated backend selection. |
| `cvkg-core` | Core traits and types defining the View, Renderer, and fundamental geometry. |
| `cvkg-vdom` | Virtual DOM implementation for stateless UI reconciliation and event handling. |
| `cvkg-scene` | Retained scene graph for efficient rendering, culling, and batching. |
| `cvkg-layout` | Flexbox-inspired layout engine supporting stacks and flexible positioning. |
| `cvkg-anim` | RK4 physics-based animation solver for smooth, realistic UI transitions. |
| `cvkg-render-gpu` | WGPU-based renderer implementation with advanced shader-based effects. |
| `cvkg-render-native` | Native windowing and event loop integration using `winit`. |
| `cvkg-render-web` | WASM and WebGPU/WebGL2 bindings for browser-based deployment. |
| `cvkg-components` | Library of reusable UI components built on the CVKG core. |
| `cvkg-themes` | Semantic styling system for consistent color, typography, and spacing. |
| `cvkg-macros` | Procedural macros for simplifying view definitions and state management. |
| `cvkg-runic-text` | High-performance text shaping and font rendering engine with runic support. |
| `cvkg-cli` | Command-line tool for project scaffolding, building, and serving. |
| `cvkg-webkit-server` | Development server providing WebSocket-based hot-reloading for web builds. |
| `cvkg-flow` | Node-based graph UI components for building interactive data flows. |
| `cvkg-test` | Specialized utilities for visual regression testing and UI benchmarking. |
| `berserker` | Reference implementation of a tactical HUD application showcasing project capabilities. |

## Documentation Index

- [Onboarding Guide](docs/onboarding.md) - Step-by-step setup and development workflow.
- [Architecture](docs/architecture.md) - Deep dive into the framework design and crate relationships.
- [How-To: Creating Components](docs/howto/creating_components.md) - Task-specific instructions for building custom UI elements.
- [Troubleshooting](docs/troubleshooting.md) - Solutions for common build and runtime issues.

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT).