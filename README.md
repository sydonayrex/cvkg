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

## 📦 Workspace Crates Overview

| Crate | Purpose | Key Functions |
| :--- | :--- | :--- |
| `cvkg` | Main entry point and orchestration | `run_app()`, state management, view composition |
| `cvkg-core` | Core traits, types, and fluent `ViewExt` API | `View`, `Renderer`, `Rect`, `ViewExt` trait |
| `cvkg-vdom` | Stateless Virtual DOM and event dispatcher | `VNode`, `VDiff`, event handling, patching |
| `cvkg-render-gpu` | The **Surtr** GPU renderer (WGPU) | `SurtrRenderer`, `render_frame()`, shader pipeline |
| `cvkg-render-native` | Native OS windowing and event loop integration | `run_app()`, window management, input handling |
| `cvkg-render-web` | WASM/Web platform integration | `WasmRenderer`, context creation, web bindings |
| `cvkg-runic-text` | Native text shaping, layout, and font fallback engine | `Shaper`, `runic_shape()`, font loading |
| `cvkg-components` | High-level interactive UI component library | `Button`, `Slider`, `VStack`, `HStack`, `Text` |
| `cvkg-layout` | Flexbox-inspired geometric layout engine | `VBoxLayout`, `HBoxLayout`, spacing, flex distribution |
| `cvkg-themes` | Cyber Viking color themes and design tokens | `Theme`, `Color`, `Spacing`, semantic colors |
| `cvkg-anim` | **Sleipnir** RK4 physics-based animation solver | `Animation`, `Sleipnir`, spring physics |
| `cvkg-scene` | 3D scene management and coordinate systems | `Scene`, `Node`, transform hierarchy |
| `cvkg-macros` | Procedural macros for view generation | `#[view]`, `view!` macro |
| `cvkg-cli` | Development tools for the CVKG ecosystem | `scaffold`, `build`, `serve` commands |
| `cvkg-test` | Visual testing and benchmarking utilities | Test harness, snapshot comparison, benchmarks |
| `cvkg-flow` | Node-based graph UI components | `Graph`, `Node`, `Edge`, interaction handling |
| `cvkg-webkit-server` | Development server with WebSocket support | `serve()`, hot-reload, asset serving |
| `temp-server` | Temporary HTTP server for development | `Server`, static file serving, hot-reload |
| `berzerker` | Demo application showcasing CVKG capabilities | Example UI patterns, integration demo |

## 📚 Documentation & Follow-On Resources

### Getting Started Guides
- **[Getting Started - Novice](docs/getting_started_novice.md)** - Copy-paste examples for beginners
- **[Getting Started - Developer](docs/getting_started_developer.md)** - Architecture overview for developers
- **[Getting Started - Designer](docs/getting_started_designer.md)** - Visual effects guide for UI/UX designers
- **[Getting Started - Expert](docs/getting_started_expert.md)** - ADRs and integration patterns for experts

### Migration & Troubleshooting
- **[Migration Guide](docs/migration_guide.md)** - Migration from SwiftUI/Jetpack Compose
- **[Troubleshooting Guide](docs/troubleshooting_guide.md)** - Common issues and solutions

### Advanced Topics
- **[Asgard Mode Tutorial](docs/asgard_mode_tutorial.md)** - God-tier visual effects guide

### Development & Planning
- **[Improvement Plan](fix_plans/CVKG_IMPROVEMENT_PLAN.md)** - Development roadmap and status
- **[Acceptance Test Framework](fix_plans/CVKG_ACCEPTANCE_TEST_FRAMEWORK.md)** - Testing methodology
- **[SUPPORT.md](SUPPORT.md)** - Support channels and contribution guidelines

### Crate-Specific Documentation
Each crate includes its own README.md with specific API references:
- `cvkg-core/README.md` - Core trait documentation
- `cvkg-components/README.md` - Component library API
- `cvkg-render-gpu/README.md` - GPU rendering documentation
- `cvkg-themes/README.md` - Theme system and design tokens

## 🛠️ Getting Started

### Prerequisites

*   Rust (latest stable)
*   GPU with Vulkan, Metal, or DX12 support (for native GPU rendering)

### Run a Demo

```bash
# Feature-gated examples (require renderer feature)
# GPU renderer examples (requires --features gpu)
cargo run --example shatter_demo -p cvkg --features gpu
cargo run --example hit_test_demo -p cvkg --features gpu
cargo run --example berserker_fire_demo -p cvkg --features gpu

# Web renderer examples (requires --features web)
cargo run --example niflheim_demo -p cvkg-components --features web

# View all available examples
cargo run --example --list
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.