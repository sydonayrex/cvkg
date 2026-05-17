# berserker-fire-web-demo

![Berserker Hero HUD](../../docs/images/berserker_hero_hud.png)

`berserker-fire-web-demo` is a WebAssembly-based stress test that showcases highly visual interactive particles, floating background texts, glassmorphic panels, and procedural lightning bolt drawing inside a web browser.

## Boundaries and Responsibilities

This crate acts as a visual stress test. It is responsible for:
- Initializing the browser `WebRenderer` and binding a high-polling render cycle to standard window animation ticks.
- Driving a highly dense custom particle system simulating fireballs, embers, and lightning arcs.
- Managing interactive coordinate-based click boundaries to map mouse inputs to reactive state counters.

This crate does NOT:
- Implement general-purpose flexbox UI layouts (it relies on direct coordinate drawings for particle rendering efficiency).
- Compile for native OS desktop windows directly (compiled to WASM).

## Public API Overview

### Entry Points
- `start()`: Prepares the rendering engine, binds standard DOM event listeners for mouse events, and fires up the custom request animation frame loop.

## Usage Example

```rust
// Compiled and initialized in browser contexts:
// import init from './berserker_fire_web_demo.js';
// init();
```

## Platform & Build Flags

- Target: `wasm32-unknown-unknown`
- Features: Specifically requires high-fidelity WebGL2 or WebGPU features for custom gradient and procedural mesh generation.
