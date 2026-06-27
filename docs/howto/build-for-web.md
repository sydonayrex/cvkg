# How to Build for Web

## Goal

Compile CVKG crates to WebAssembly for browser execution.

## Prerequisites

- `rustup target add wasm32-unknown-unknown`
- `wasm-pack` installed
- A web server for serving the output

## Steps

### Build a Web Demo

```bash
cd demos/niflheim-web
wasm-pack build --target web --out-dir pkg
```

### Serve Locally

```bash
cd pkg
python3 -m http.server 8080
```

Open `http://localhost:8080` in a browser.

### Build for Production

```bash
wasm-pack build --target web --release --out-dir pkg
```

The `--release` flag enables optimizations. The output includes a `.wasm` file and JavaScript glue code.

## Expected Output

- `pkg/` directory with `.wasm`, `.js`, and `.d.ts` files.
- Browser page rendering CVKG components via WebGL2 or WebGPU.

## What Can Go Wrong

- **`wasm-pack` not found**: Install it: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`.
- **`web-sys` feature errors**: Ensure you are targeting `wasm32-unknown-unknown`, not `wasm32-wasi`.
- **Blank canvas**: Check browser console. Enable WebPU or WebGL2 in browser flags if needed.
- **Large `.wasm` file**: Use `wasm-opt` for further optimization: `wasm-opt -Oz -o output.wasm input.wasm`.
