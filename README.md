# CVKG: Cyber Viking Kvasir Graph

**CVKG** is a high-fidelity, agentic UI framework for Rust, designed for building stunning native and web applications with a Cyberpunk/Viking aesthetic ("Berserker Mode"). It leverages a custom GPU-accelerated rendering pipeline, a stateless Virtual DOM, and a fluent modifier-based API.

## 🚀 Key Features

*   **Stateless UI**: Declarative, functional-reactive UI definition inspired by SwiftUI and Dioxus.
*   **Muspelheim Rendering**: High-performance GPU renderer (WGPU) with advanced shaders (Bifrost frosting, Gungnir glow, Mjolnir clipping).
*   **Runic-Text Engine**: Natively integrated text shaping (rustybuzz) and layout (swash) with Global Font Fallback and BiDi support.
*   **Intrinsic Sizing**: Sophisticated content-aware layout negotiation system allowing components to define their natural dimensions.
*   **Vector Iconography**: GPU-accelerated SVG tessellation engine (via lyon) for resolution-independent icons and paths.
*   **Performance Telemetry**: Real-time monitoring of frame times, draw calls, and vertex counts directly in the UI.
*   **Headless Rendering**: Hardened GPU pipeline for automated visual regression testing and server-side frame capture.
*   **Cross-Platform**: Seamlessly targets Native (Desktop) and Web (WASM) via unified rendering traits.
*   **Agent-Optimized**: Designed to be easily navigable and manipulatable by AI agents.
*   **Accessibility**: Integrated Screen Reader support (AccessKit/Section 508) and IME functionality.
*   **SecOps Ready**: Production-hardened with capability-based security, WASM sandboxing, and resource metering.

## 📦 Workspace Crates

| Crate | Purpose |
| :--- | :--- |
| `cvkg` | Main entry point and orchestration. |
| `cvkg-core` | Core traits, types, and the fluent `ViewExt` API. |
| `cvkg-vdom` | Stateless Virtual DOM and event dispatcher. |
| `cvkg-render-gpu` | The **Surtr** GPU renderer (WGPU). |
| `cvkg-render-native` | Native OS windowing and event loop integration. |
| `cvkg-render-web` | WASM/Web platform integration. |
| `cvkg-runic-text` | Native text shaping, layout, and font fallback engine. |
| `cvkg-components` | High-level interactive UI component library. |
| `cvkg-layout` | Flexbox-inspired geometric layout engine. |
| `cvkg-themes` | Cyber Viking color themes and design tokens. |
| `cvkg-anim` | **Sleipnir** RK4 physics-based animation solver. |
| `cvkg-scene` | 3D scene management and coordinate systems. |
| `cvkg-macros` | Procedural macros for view generation. |
| `cvkg-cli` | Development tools for the CVKG ecosystem. |

## 🛠️ Getting Started

### Prerequisites

*   Rust (latest stable)
*   GPU with Vulkan, Metal, or DX12 support (for native GPU rendering)

### Run a Demo

### Run a Demo

```bash
# Feature-gated examples (require renderer feature)
# GPU renderer examples (requires --features gpu)
# cargo run --example shatter_demo -p cvkg --features gpu
# cargo run --example hit_test_demo -p cvkg --features gpu
# cargo run --example berserker_fire_demo -p cvkg --features gpu
# cargo run --example forge_effects_demo -p cvkg-components --features gpu
# cargo run --example memory_system_demo -p cvkg-components --features gpu
# cargo run --example interactive_demo -p cvkg-components --features native

# Non-feature-gated examples (work with default features)
# cargo run --example error_boundary_demo -p cvkg-components --no-default-features
# cargo run --example niflheim_demo -p cvkg-components
# cargo run --example component_feature_showcase -p cvkg-components

# View all available examples
cargo run --example --list

## 📜 License

This project is licensed under the MIT License.
