# niflheim-wasi-demo

![VDOM Agent Graph](../../docs/images/vdom_agent_graph.png)

`niflheim-wasi-demo` provides a server-side, headless WebAssembly component conforming to the WebAssembly System Interface (WASI) standard. It is designed to verify UI view-tree composition and compile-time integrity in headless execution environments.

## Boundaries and Responsibilities

This crate acts as a headless validation package. Its responsibilities are:
- Exposing stable C-style foreign function interface (FFI) bindings for orchestration by host servers or testing runtimes.
- Checking that the core component definitions can be instantiated within standard sandboxed environments.

This crate does NOT:
- Integrate with high-fidelity GPU backends (no direct WebGPU/Vulkan hooks).
- Manage high-frequency frame drawing loops or capture physical input events.

## Public API Overview

### FFI Exports
- `cvkg_init()`: Initializes environmental structures or registers headless diagnostic hooks.
- `cvkg_update()`: Simulates logical or reactive ticks within the sandboxed context.
- `cvkg_render()`: Headless rendering driver that verifies the Niflheim view tree can be composed without errors.

## Usage Example

```rust
// Invoked by the WASI runtime or host execution environment:
// cvkg_init();
// cvkg_update();
// cvkg_render();
```

## Platform & Build Flags

- Target: `wasm32-wasip1`
- Features: Configured for compilation into stand-alone `.wasm` components using standard WASI tooling.
