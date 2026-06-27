# demos/berserker-fire-web

Web stress-test drawing procedural fires and lightning.

## Purpose

Stress-test rendering of procedural fire, lightning, and particle effects via WASM. Measures frame times and GPU throughput under heavy draw-call load.

## Boundaries

- WASM-only target.
- Demands WebGPU or WebGL2 capable browser.
- Not a UI demo -- a rendering benchmark.

## Usage

```bash
cd demos/berserker-fire-web
wasm-pack build --target web
python3 -m http.server 8080 --dir pkg
```

## Dependencies

- `cvkg-core` -- View trait, geometry
- `cvkg-render-gpu` -- GPU rendering
- `glam` -- Vector math
