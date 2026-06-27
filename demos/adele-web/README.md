# demos/adele-web

Web design system explorer built on CVKG with WebGPU/WASM.

## Purpose

Demonstrates CVKG component rendering in a browser via WebGPU. Serves as a design system explorer and matrix comparison layout.

## Boundaries

- Web-only target (WASM via `wasm-pack`).
- Does NOT run natively -- requires a browser with WebGL2 or WebGPU support.

## Usage

```bash
cd demos/adele-web
wasm-pack build --target web
# Serve the pkg/ directory with any static server
```

## Dependencies

- `cvkg-core` -- View trait, color, rect
- `cvkg-render-gpu` -- GPU rendering in browser
- `cvkg-vdom` -- Virtual DOM diffing
- `cvkg-components` -- Widget library
- `cvkg-themes` -- Design tokens
- `wasm-bindgen`, `wasm-bindgen-futures` -- WASM glue
- `web-sys` -- Browser API bindings
