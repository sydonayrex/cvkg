# cvkg-cli

![VDOM Agent Graph](../docs/images/vdom_agent_graph.png)

`cvkg-cli` is the authoritative command-line toolchain for the CVKG ecosystem, managing the lifecycle of applications from scaffolding to production deployment.

## Boundaries and Responsibilities

This crate provides the developer interface to the framework. Its responsibilities include:
- **Project Scaffolding**: Creating new workspaces with `cvkg new`.
- **Development Engine**: Orchestrating hot-reloading dev servers with `cvkg dev`.
- **Build Orchestration**: Compiling for native and web targets via `cvkg build`.
- **Quality Assurance**: Running lints, audits, and visual regression tests with `cvkg check` and `cvkg test`.
- **Observability**: Providing a real-time telemetry inspector via `cvkg inspect`.
- **Asset Management**: Processing and bundling assets through the `asset_pipeline`.

## Public API Overview

### Primary Commands
- `cvkg new <NAME>`: Scaffolds a new project with optional templates and git initialization.
- `cvkg dev --target <PLATFORM>`: Launches the development server with state-preserving hot reload.
- `cvkg build --target <PLATFORM>`: Compiles the project for the specified hardware/runtime.
- `cvkg serve`: Starts a high-performance preview server for web targets.
- `cvkg export`: Bundles a project into a production-ready static WASM distribution.

### Advanced Tooling
- `cvkg inspect`: Connects to a running application to stream real-time FPS, VRAM, and VDOM metrics.
- `cvkg theme`: Generates type-safe Rust themes from design token JSON files.

## Usage Example

```bash
# Start a new project
cvkg new my-tactical-app

# Run in development mode with the inspector enabled
cvkg dev --target native --inspector

# Build for web deployment
cvkg build --target wasm --release
```

## Known Limitations
- `cvkg export` requires `wasm-pack` to be installed on the system path for web targets.
- Hot reload stability is dependent on the complexity of the state graph being preserved.
- The asset pipeline currently assumes an `assets/` directory at the project root.