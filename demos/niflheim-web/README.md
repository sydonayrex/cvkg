# demos/niflheim-web

WebAssembly showcase executing the CVKG component suite.

## Purpose

Runs the standard CVKG components (buttons, sliders, toggles, text) in a browser via WASM. Demonstrates that the component library compiles and renders correctly in a WebAssembly target.

## Boundaries

- Web-only target (WASM via `wasm-pack`).
- Requires `wasm32-unknown-unknown` target installed.

## Usage

```bash
rustup target add wasm32-unknown-unknown
cd demos/niflheim-web
wasm-pack build --target web
python3 -m http.server 8080 --dir pkg
```

## Dependencies

- `cvkg-core` -- View trait, state
- `cvkg-render-gpu` -- GPU rendering
- `cvkg-components` -- Widget library
- `wasm-bindgen` -- WASM glue
