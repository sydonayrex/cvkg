# How to Run a Demo

## Goal

Run one of the pre-built demo applications to see CVKG in action.

## Prerequisites

- Rust toolchain installed
- `cargo build --workspace` succeeds

## Steps

### Native Demo (berserker)

```bash
cargo run -p demos/berserker
```

This launches the native tactical HUD application. A window should open showing the demo UI.

### Web Demo (niflheim-web)

```bash
cd demos/niflheim-web
wasm-pack build --target web
# Serve with any static file server:
python3 -m http.server 8080 --dir pkg/
```

Open `http://localhost:8080` in a browser with WebGPU or WebGL2 support.

### Web Stress Test (berserker-fire-web)

```bash
cd demos/berserker-fire-web
wasm-pack build --target web
python3 -m http.server 8080 --dir pkg/
```

## Expected Output

- `berserker`: Native window with rendered UI components.
- `niflheim-web`: Browser page displaying the CVKG component suite.
- `berserker-fire-web`: Browser page with procedural fire and lightning rendering.

## What Can Go Wrong

- **"GPU adapter not found"**: Your system lacks Vulkan/Metal/DX12 drivers. Install Mesa drivers (`sudo apt-get install mesa-utils`) or update GPU drivers.
- **WASM build fails**: Ensure `wasm-pack` is installed: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`.
- **Blank page in browser**: Check browser console for WebGPU/WebGL2 errors. Try Chrome or Firefox with WebGPU enabled.
