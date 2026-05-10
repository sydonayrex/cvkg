# CVKG: Cyber Viking Kvasir Graph

**CVKG** is a high-fidelity agentic UI framework for building native and web applications with a Cyberpunk/Viking aesthetic using Rust.

## What Problem It Solves

CVKG provides a declarative, functional-reactive UI system for Rust with:
- GPU-accelerated rendering with advanced visual effects (Bifrost frosting, Gungnir glow, Mjolnir clipping)
- Stateless Virtual DOM for efficient updates
- Multi-platform support (Native desktop, Web/WASM)
- Agent-optimized architecture for AI-assisted development

This framework is for UI developers building high-performance applications who need both visual fidelity and developer ergonomics.

## Prerequisites

- **Rust toolchain**: Rust 1.81.0 or later (edition 2024)
- **System dependencies**: GPU with Vulkan, Metal, or DX12 support for native GPU rendering
- **Optional**: `wasm-pack` for web target builds

## Quick Start (5 Commands)

```bash
# 1. Clone the repository
git clone https://github.com/sydonayrex/cvkg.git && cd cvkg

# 2. Build the workspace
cargo build --workspace

# 3. Run a demo (requires GPU support)
cargo run --example shatter_demo -p cvkg --features gpu

# 4. Run tests
cargo test --workspace

# 5. Build for web (optional)
cargo build --target wasm32-unknown-unknown --features web
```

## Workspace Crate Map

| Crate | Role |
|-------|------|
| `cvkg-core` | Core traits (`View`, `Renderer`), types (`Rect`, `Size`), and fluent modifier API |
| `cvkg-vdom` | Virtual DOM implementation (`VNode`, `VDiff`) and event system |
| `cvkg-render-gpu` | WGPU-based GPU renderer (`SurtrRenderer`) with shader effects |
| `cvkg-render-native` | winit window integration and native event loop |
| `cvkg-render-web` | WASM/WebGPU bindings for browser deployment |
| `cvkg-layout` | Flexbox-inspired layout engine (`HStack`, `VStack`) |
| `cvkg-anim` | RK4 physics-based animation solver (`SleipnirSolver`) |
| `cvkg-scene` | Retained scene graph with culling and batching |
| `cvkg-components` | Reusable UI components (`Button`, `Text`, `Slider`) |
| `cvkg-themes` | Semantic color and typography themes |
| `cvkg-runic-text` | Text shaping and font fallback engine |
| `cvkg-macros` | Procedural macros for view generation |
| `cvkg-cli` | Development tools (`scaffold`, `build`, `serve`) |
| `cvkg-test` | Visual regression and benchmarking utilities |
| `cvkg-flow` | Node-based graph UI components |
| `cvkg-webkit-server` | Development server with WebSocket hot-reload |

## Documentation

- [Onboarding Guide](docs/onboarding.md) - Clone to running tests
- [Architecture](docs/architecture.md) - How crates fit together
- [How-To Guides](docs/howto/) - Task-specific instructions
- [Troubleshooting](docs/troubleshooting.md) - Common issues and fixes

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT).