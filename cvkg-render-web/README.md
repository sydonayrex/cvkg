# cvkg-render-web

**cvkg-render-web** provides the WASM-based rendering bridge for running CVKG applications in the browser.

## Features
*   **HTML Canvas Integration**: Bridges the CVKG `Renderer` trait to a WebGL2 or WebGPU context in the browser.
*   **DOM Event Mapping**: Translates browser DOM events (PointerEvents, KeyboardEvents) into CVKG `Event` variants.
*   **WASM Orchestration**: Provides the entry point for compiled CVKG applications.
