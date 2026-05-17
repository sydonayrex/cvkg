# cvkg-webkit-server

![VDOM Agent Graph](../docs/images/vdom_agent_graph.png)

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