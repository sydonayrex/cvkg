# cvkg-render-web

**cvkg-render-web** provides the WASM-based rendering bridge for running CVKG applications in the browser.

## Features
*   **HTML Canvas Integration**: Bridges the CVKG `Renderer` trait to a WebGL2 or WebGPU context in the browser.
*   **DOM Event Mapping**: Translates browser DOM events (PointerEvents, KeyboardEvents, IME) into CVKG `Event` variants.
*   **Telemetry Passthrough**: Maps platform-specific performance metrics to the unified `TelemetryData` format.
*   **WASM Orchestration**: Provides the entry point for compiled CVKG applications.

---

# cvkg-scene

**cvkg-scene** manages 3D coordinate systems and scene graphs for hybrid 2D/3D CVKG applications.

## Features
*   **Perspective Projection**: Handles 3D-to-2D projection for UI elements.
*   **Scene Graph**: Tree-based management of 3D objects and lighting.

---

# cvkg-macros

**cvkg-macros** contains procedural macros for the CVKG framework.

## Features
*   `#[derive(View)]`: Automatically implements the `View` trait for custom structs.
*   `view! { ... }`: DSL-like macro for declarative UI definition.

---

# cvkg-cli

**cvkg-cli** is the development command-line tool for CVKG.

## Features
*   `cvkg new`: Scaffolds a new project.
*   `cvkg dev`: Hot-reloading development server.

---

# cvkg-webkit-server

**cvkg-webkit-server** provides a bridge for embedding CVKG views inside web-based environments or headless testing.
