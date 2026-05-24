# CVKG: Cyber Viking Kvasir Graph

![CVKG Hero HUD](docs/images/cvkg_hero.png)

CVKG is a high-fidelity graphic user interface framework for Rust, enabling developers to build visually intense, hardware-accelerated desktop and web applications.

## Problem and Target Audience

Modern application developers frequently face a choice between high-performance but low-fidelity native GUI tools, or heavy web-tech runtimes. CVKG addresses this challenge by providing a declarative UI system that compiles directly to GPU pipelines (Vulkan, Metal, DirectX 12) and browser WebGPU/WebGL canvases, delivering sub-millisecond drawing times and spring-physics animations without sacrificing performance. This framework is tailored for creative engineers and system designers building custom interfaces, tactical widgets, and node-based dashboards.

---

## Prerequisites

- **Rust Compiler**: Rust 1.85.0 or later (Edition 2024).
- **GPU Drivers**: Vulkan, Metal, or DX12 compatible hardware.
- **Linux Tools**: System packages `libfontconfig1-dev`, `pkg-config`, and windowing libraries (`libx11-dev`, `libwayland-dev`) are required.
- **WebAssembly Compiler**: `wasm-pack` is required for web build pipelines.

---

## Quick Start (Five Commands)

```bash
# 1. Clone the project repository
git clone https://github.com/sydonayrex/cvkg.git && cd cvkg

# 2. Add WASM target library
rustup target add wasm32-unknown-unknown

# 3. Compile the workspace packages
cargo build --workspace

# 4. Execute the unit testing suite
cargo test --workspace

# 5. Run the native tactical HUD launcher application
cargo run -p berserker
```

---

## Workspace Crate Map

| Crate Path | Role / Responsibility |
| :--- | :--- |
| [cvkg](file:///D/rex/projects/cvkg/cvkg) | Main public entry point facade selecting native or web backends. |
| [cvkg-core](file:///D/rex/projects/cvkg/cvkg-core) | Core traits defining view composition, renderers, and geometry types. |
| [cvkg-vdom](file:///D/rex/projects/cvkg/cvkg-vdom) | Stateless Virtual DOM implementation managing tree diffs and updates. |
| [cvkg-compositor](file:///D/rex/projects/cvkg/cvkg-compositor) | Retained-mode layer orchestration engine routing UI to GPU passes. |
| [cvkg-scene](file:///D/rex/projects/cvkg/cvkg-scene) | Retained scene graph utilizing bounding box acceleration for culling. |
| [cvkg-layout](file:///D/rex/projects/cvkg/cvkg-layout) | Coordinate layout engines distributing spacer proposed bounds. |
| [cvkg-anim](file:///D/rex/projects/cvkg/cvkg-anim) | Physics-based RK4 Sleipnir spring motion solver system. |
| [cvkg-render-gpu](file:///D/rex/projects/cvkg/cvkg-render-gpu) | Surtr graphics pipeline rendering custom GPU shader pipelines. |
| [cvkg-render-native](file:///D/rex/projects/cvkg/cvkg-render-native) | Desktop platform windowing and event loops wrapping `winit`. |
| [cvkg-render-web](file:///D/rex/projects/cvkg/cvkg-render-web) | Browser canvas drawing wrapper executing on WebGPU or WebGL2. |
| [cvkg-components](file:///D/rex/projects/cvkg/cvkg-components) | Base widget library housing inputs, sliders, and advanced AI workflow components. |
| [cvkg-themes](file:///D/rex/projects/cvkg/cvkg-themes) | OKLCH-based system token catalog managing semantic color and typography mappings. |
| [cvkg-macros](file:///D/rex/projects/cvkg/cvkg-macros) | Procedural compiler macros scaffolding DSL views and reactive bindings. |
| [cvkg-runic-text](file:///D/rex/projects/cvkg/cvkg-runic-text) | Font-discovery, word-wrapping, and HarfBuzz text shaper. |
| [cvkg-cli](file:///D/rex/projects/cvkg/cvkg-cli) | Scaffolding command line interface managing development pipelines and AI templates. |
| [cvkg-webkit-server](file:///D/rex/projects/cvkg/cvkg-webkit-server) | Headless WebSocket dev server handling local bundle reloading. |
| [cvkg-flow](file:///D/rex/projects/cvkg/cvkg-flow) | Interactive node and flow-chart visual editor component. |
| [cvkg-test](file:///D/rex/projects/cvkg/cvkg-test) | Pixel comparison engine executing visual regression testing. |
| [berserker](file:///D/rex/projects/cvkg/berserker) | Native tactical HUD application showcasing layout and graphics. |
| [demos/adele-web](file:///D/rex/projects/cvkg/demos/adele-web) | Web design system explorer and matrix comparison layout. |
| [demos/niflheim-web](file:///D/rex/projects/cvkg/demos/niflheim-web) | WebAssembly showcase executing the standard components suite. |
| [demos/niflheim-wasi](file:///D/rex/projects/cvkg/demos/niflheim-wasi) | Headless server-side WASI target checking view validation. |
| [demos/berserker-fire-web](file:///D/rex/projects/cvkg/demos/berserker-fire-web) | Highly visual web stress-test drawing procedural fires and lightning. |

---

## Documentation Index

Explore our guides to understand CVKG's capabilities:

- [Onboarding Guide](docs/onboarding.md) — Step-by-step setup and local development workflow.
- [Architecture Guide](docs/architecture.md) — System topology, subsystem specs, and crate graph.
- [Troubleshooting Guide](docs/troubleshooting.md) — Compilation errors, runtime crashes, and graphics resolution.

### How-To Guides

- [How to Run a Demo](docs/howto/run-demo.md) — Run native and web-based graphic previews.
- [How to Run Tests](docs/howto/run-tests.md) — Run workspace tests, single packages, or visual regressions.
- [How to Build for Web](docs/howto/build-for-web.md) — Bundle, target, and serve WebAssembly applications.
- [How to Create a Component (Manual)](docs/howto/create-component.md) — Write custom primitive drawings implementing `View` manually.
- [How to Create Components (Macros)](docs/howto/creating_components.md) — Author interactive components utilizing state macros.
- [How to Use the CVKG CLI](docs/howto/using-cli.md) — Scaffold projects, start dev servers, and run telemetry streams.
- [How to Generate a Theme](docs/howto/generate-theme.md) — Compile Rust style constants from JSON color tokens.

---

## License

Mozilla Public License 2.0 - see [LICENSE](LICENSE).