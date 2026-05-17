# niflheim-web-demo

![VDOM Agent Graph](../../docs/images/vdom_agent_graph.png)

`niflheim-web-demo` is a WebAssembly-based target package that compiles and runs the Niflheim UI system showcase in the browser, powered by WebGL2 or WebGPU.

## Boundaries and Responsibilities

This crate acts as a deployment bundle. It is responsible for:
- Initializing the browser `WebRenderer` and executing the rendering pipeline initialization (Forge).
- Setting up the global application environment keys (`YggdrasilKey` for style tokens, `AppearanceKey` for theme selection).
- Orchestrating the browser render loop via `request_animation_frame` calls to tick the VDOM and canvas.

This crate does NOT:
- Define new UI components or custom layouts (delegates entirely to the shared library).
- Implement platform-level event listeners for native OS windows.

## Public API Overview

### Entry Points
- `start()`: The primary asynchronous WASM start function. It prepares the canvas, updates the Virtual DOM tree with the core Niflheim view components, and establishes the frame ticking loop.
- `get_render_tier_name()`: Re-exported utility from the WebRenderer backend to identify runtime hardware capabilities.

## Usage Example

```rust
// The entry point is driven automatically by the WASM bundle loading inside the HTML page:
// import init from './niflheim_web_demo.js';
// init();
```

## Platform & Build Flags

- Target: `wasm32-unknown-unknown`
- Features: Gated behind WASM browser deployment, requiring target-specific browser capabilities (WebGL2 or WebGPU).
