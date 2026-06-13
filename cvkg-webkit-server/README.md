# cvkg-webkit-server

```mermaid
graph TD
    cvkg-core["cvkg-core"]
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene["cvkg-scene"]
    cvkg-layout["cvkg-layout"]
    cvkg-render-gpu["cvkg-render-gpu"]
    cvkg-render-native["cvkg-render-native"]
    cvkg-compositor["cvkg-compositor"]
    cvkg-themes["cvkg-themes"]
    cvkg-anim["cvkg-anim"]
    cvkg-flow["cvkg-flow"]
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-svg-filters["cvkg-svg-filters"]
    cvkg-svg-serialize["cvkg-svg-serialize"]
    cvkg-components["cvkg-components"]
    cvkg-macros["cvkg-macros"]
    cvkg-cli["cvkg-cli"]
    cvkg-webkit-server["cvkg-webkit-server"]
    cvkg-test["cvkg-test"]
    cvkg-physics["cvkg-physics"]
    cvkg["cvkg (umbrella)"]

    cvkg-vdom --> cvkg-core
    cvkg-vdom --> cvkg-scene
    cvkg-layout --> cvkg-core
    cvkg-layout --> cvkg-anim
    cvkg-scene --> cvkg-core

    cvkg-render-gpu --> cvkg-core
    cvkg-render-gpu --> cvkg-compositor
    cvkg-render-gpu --> cvkg-svg-filters
    cvkg-render-gpu --> cvkg-svg-serialize
    cvkg-render-gpu --> cvkg-runic-text

    cvkg-render-native --> cvkg-core
    cvkg-render-native --> cvkg-render-gpu
    cvkg-render-native --> cvkg-vdom
    cvkg-render-native --> cvkg-themes

    cvkg-compositor --> cvkg-core

    cvkg-themes --> cvkg-core
    cvkg-themes --> cvkg-anim
    cvkg-anim --> cvkg-core
    cvkg-flow --> cvkg-core
    cvkg-flow --> cvkg-scene
    cvkg-flow --> cvkg-themes

    cvkg-runic-text --> cvkg-core
    cvkg-svg-filters --> cvkg-core

    cvkg-components --> cvkg-core
    cvkg-components --> cvkg-vdom
    cvkg-components --> cvkg-layout
    cvkg-components --> cvkg-themes
    cvkg-components --> cvkg-anim
    cvkg-components --> cvkg-runic-text

    cvkg-macros --> cvkg-core
    cvkg-cli --> cvkg-core
    cvkg-cli --> cvkg-physics
    cvkg-cli --> cvkg-anim
    cvkg-cli --> cvkg-macros
    cvkg-webkit-server --> cvkg-cli
    cvkg-physics --> cvkg-core
    cvkg-physics --> cvkg-scene

    cvkg --> cvkg-core
    cvkg --> cvkg-vdom
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
```

`cvkg-webkit-server` is a professional-grade development and preview server for CVKG applications, providing state-preserving hot reload, SEO pre-rendering, and real-time observability.

## Boundaries and Responsibilities

This crate implements the backend for the development toolchain. Its responsibilities include:
- **Universal Serving**: Hosting WASM, static assets, and package artifacts via high-performance `axum` routes.
- **State-Preserving HMR**: Managing WebSocket connections for instant UI updates without losing application state.
- **SEO Pre-rendering**: Capturing VDOM snapshots to serve meaningful HTML before WASM initialization.
- **Observability**: Exposing Prometheus-compatible metrics and health check endpoints.
- **Security**: Enforcing strict Content Security Policies (CSP) and security headers for dev environments.

## Public API Overview

### Server Components
- `BuildOrchestrator`: Manages background compilation tasks with automatic retry logic.
- `WebKitBridge`: The server-side implementation of the CVKG developer protocol.
- `AppState`: Shared state for managing VDOM snapshots and server configuration.

### Observability Endpoints
- `/health/liveness`: Returns `OK` when the server is running.
- `/health/readiness`: Returns `READY` when assets and pkg directories are successfully pivoted.
- `/metrics`: Standard Prometheus metrics output.
- `/api/system/time`: Provides a synchronized timestamp for distributed state logs.

## Usage Example

```bash
# Usually launched via cvkg-cli, but can be run directly:
cargo run -p cvkg-webkit-server -- --addr 0.0.0.0:8080 --pkg-dir ./dist/pkg
```

## Known Limitations
- The server requires absolute paths for reliable execution across different working directories.
- SEO snapshots are currently stored in-memory; high-frequency snapshotting may impact server performance in resource-constrained environments.
