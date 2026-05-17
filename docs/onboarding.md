# Onboarding Guide

Welcome to the Cyber Viking Kvasir Graph (CVKG) development environment. This guide will walk you through the setup and workflow to compile and test the project on your local system.

## Getting Started

Follow these steps in order to download, install dependencies, and build the workspace.

### 1. Clone the Repository
Clone the repository to your local workspace and navigate to the directory:
```bash
git clone https://github.com/sydonayrex/cvkg.git
cd cvkg
```

### 2. Install Rust Toolchain
Ensure you have the Rust toolchain installed (minimum version 1.85.0) which supports the 2024 Edition:
```bash
rustup toolchain install stable
rustup default stable
rustup update
```

### 3. Install System Dependencies
On Linux systems, specific packages are required to support windowing, font configuration, and the WGPU graphics pipeline:
```bash
sudo apt-get update
sudo apt-get install -y libwayland-dev libx11-dev libxkbcommon-dev libasound2-dev libfontconfig1-dev pkg-config
```

### 4. Build the Workspace
Build all crates and binaries in the workspace:
```bash
cargo build --workspace
```

### 5. Run the Reference Application
Execute the native tactical HUD launcher application:
```bash
cargo run -p berserker
```

---

## Workspace Directory Layout

The workspace is organized into modular tiers to separate core application logic from platform-specific integration.

- `cvkg/` — Main public facade and backend selector.
- `cvkg-core/` — Fundamental geometry types, environment maps, and rendering traits.
- `cvkg-vdom/` — Stateless Virtual DOM manager, tree diffing, and accessibility bridges.
- `cvkg-scene/` — Retained scene graph with spatial subdivision and drawing buffers.
- `cvkg-layout/` — Flexbox positioning engines and stack containers.
- `cvkg-anim/` — Sleipnir spring-physics and RK4 animation solvers.
- `cvkg-render-gpu/` — Surtr GPU renderer implementing WGPU pipelines and textures.
- `cvkg-render-native/` — Desktop OS windowing support using `winit`.
- `cvkg-render-web/` — Browser canvas and WebGPU/WebGL2 integration.
- `cvkg-components/` — Standard widget library (Buttons, Sliders, Editors, diagnostic panels).
- `cvkg-themes/` — Semantic design tokens for colors, typography, and spaces.
- `cvkg-macros/` — Declarative syntax macros for views and state values.
- `cvkg-runic-text/` — Text shaper, word-wrapping engine, and Swash bitmap generator.
- `cvkg-cli/` — Scaffold and development tool for building and packing.
- `cvkg-webkit-server/` — WebSocket host and server for local hot-reloading.
- `cvkg-flow/` — Interactive node graphs and canvas grids.
- `cvkg-test/` — Quality assurance suite and visual comparison regression tools.
- `berserker/` — Native launcher bin showcase app.
- `demos/` — Browser-facing showcase crates including Adele catalog and Niflheim views.

---

## Running Tests

Testing is divided into library unit tests, component integrations, and headless visual comparisons.

### Run the Full Test Suite
To run all tests in the workspace:
```bash
cargo test --workspace
```

### Run a Single Crate's Tests
To test a specific crate, use the `-p` parameter:
```bash
cargo test -p cvkg-layout
```

### Run a Specific Test by Name
To isolate a single test execution:
```bash
cargo test -p cvkg-layout tests::test_hstack_basic
```

---

## Modifying Code & Verifying Changes

When adding a feature or resolving an issue, use this checklist to ensure stability:

1. **Verify Formatting**: Ensure all files conform to the standard style:
   ```bash
   cargo fmt --all --check
   ```
2. **Execute Lints**: Run compiler static analysis checks:
   ```bash
   cargo clippy --workspace --all-targets
   ```
3. **Verify Build**: Run target checks to ensure compile pipeline passes:
   ```bash
   cargo check --workspace
   ```
4. **Run Unit Tests**: Confirm that the change has not introduced regressions:
   ```bash
   cargo test --workspace
   ```

---

## Support & Contacts

If you experience unexpected build or runtime failures, consult [troubleshooting.md](./troubleshooting.md).

For additional inquiries or maintainer feedback:
- TODO: add maintainer contact