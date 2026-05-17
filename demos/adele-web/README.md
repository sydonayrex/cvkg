# adele-web-demo

![CVKG Component Showcase](../../docs/images/cvkg_component_showcase.png)

`adele-web-demo` is a WebAssembly-based design system catalog explorer that allows side-by-side comparison and filtering of major design systems, reading data dynamically from a JSON schema.

## Boundaries and Responsibilities

This crate serves as a reference application. It focuses on:
- High-level layout composition using `View` trait implementations.
- UI state management representing a multi-view catalog explorer.
- Client-side data parsing of design system catalog datasets.

This crate does NOT:
- Implement any low-level graphic rasterization or event loop integration (delegated to the rendering and platform backends).
- Perform custom server-side database querying.

## Public API Overview

### Core Types
- `AdeleApp`: The main application view controller that manages systems list, filter parameters, and view state.
- `ViewMode`: Navigation state enum representing the current active panel:
  - `Catalog`
  - `Detail(String)`
  - `Comparison`

### Entry Points
- `main()`: The WASM entry point that registers the panic hook and initializes console logs.

## Usage Example

```rust
use cvkg_core::{View, Rect};

// Instantiate the application
let app = AdeleApp::new();

// Render using the target frame renderer
let viewport = Rect { x: 0.0, y: 0.0, width: 1200.0, height: 800.0 };
app.render(&mut renderer, viewport);
```

## Platform & Build Flags

- Target: `wasm32-unknown-unknown`
- Features: Gated behind WASM standard browser deployment configurations.
