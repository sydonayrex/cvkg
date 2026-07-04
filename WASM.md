# CVKG WebAssembly (WASM) Guide

CVKG fully supports compiling to `wasm32-unknown-unknown` for running in the
browser. The framework uses `wasm-bindgen` / `wasm-pack` for JS interop and
supports three rendering backends in the browser: **WebGPU**, **WebGL2**, and
**headless** (SVG / canvas-2d fallback).

## Prerequisites

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (one-time)
cargo install wasm-pack
```

## Quick Start

### 1. Minimal headless WASM app (no GPU)

Create a `lib.rs`:

```rust
use cvkg::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn render(width: f32, height: f32) -> String {
    let view = Button::new("Click me")
        .on_click(|| web_sys::console::log_1(&"Hello!".into()));
    let mut headless = CvkgHeadless::new(view, Rect::sized(width, height));
    let frame = headless.render_frame();
    frame.svg  // or serialize via a canvas backend
}
```

Build:

```bash
cargo build --target wasm32-unknown-unknown
wasm-bindgen --target web \
    target/wasm32-unknown-unknown/debug/my_app.wasm \
    --out-dir pkg
```

### 2. WebGPU WASM app

Use the `cvkg-render-gpu` backend with `wgpu` targeting WebGPU:

```rust
use cvkg::prelude::*;
use cvkg_render_gpu::GpuRenderer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let view = App::new();
    let renderer = GpuRenderer::forge(canvas, view, Default::default());
    renderer.run();  // enters the render loop

    Ok(())
}
```

Build:

```bash
cd demos/adele-web
wasm-pack build --target web
python3 -m http.server 8080 --dir pkg
```

### 3. Using CSS variables from a theme

The `ThemeBuilder::to_css_variables()` method exports the full semantic palette
as CSS custom properties on `:root`. Use this in WASM to align CSS-styled
elements with the CVKG theme:

```rust
use cvkg_themes::ThemeBuilder;

let css = ThemeBuilder::dark().to_css_variables();
// css => ":root {\n  --cvkg-primary: #ffd700;\n  ...\n}"
```

Inject into the page:

```rust
let style = web_sys::window()
    .unwrap()
    .document()
    .unwrap()
    .create_element("style")?;
style.set_inner_html(&css);
web_sys::window()
    .unwrap()
    .document()
    .unwrap()
    .head()
    .unwrap()
    .append_child(&style)?;
```

## Demos

| Demo | Description | Backend |
|------|-------------|---------|
| `demos/adele-web` | Design system explorer | WebGPU |
| `demos/berserker-fire-web` | Procedural fire/lightning stress test | WebGPU |
| `demos/niflheim-web` | Multi-backend WASM demo (wasm/webgl2/wgpu) | All three |

## WASM-Specific Dependencies

These crates are conditionally compiled for `wasm32-unknown-unknown`:

- `wasm-bindgen` — JS-Rust interop
- `wasm-bindgen-futures` — Async on WASM
- `js-sys` / `web-sys` — Browser API bindings
- `getrandom` (wasm_js feature) — Random number support
- `console_error_panic_hook` — Better panic messages

All WASM gates use `#[cfg(target_arch = "wasm32")]` or Cargo `[target.'cfg(target_arch = "wasm32")'.dependencies]`.

## Cargo.toml Setup

```toml
[dependencies]
cvkg = { version = "0.3", features = ["render-gpu"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = [
    "Document", "Element", "HtmlCanvasElement", "Window",
    "console", "performance",
] }
js-sys = "0.3"
console_error_panic_hook = "0.1"
getrandom = { version = "0.4", features = ["wasm_js"] }
```

## Build Configuration

For production, optimise the wasm binary:

```bash
wasm-pack build --target web --release
# Binary size options:
#   wasm-opt -O4 pkg/*.wasm -o pkg/*.wasm     # aggressive
#   wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm     # size-optimised
```

Expected sizes (release):
- Minimal headless app: ~200 KB (wasm) + ~20 KB (JS glue)
- WebGPU app with components: ~1.5 MB (wasm) + ~40 KB (JS glue)

## Known Limitations

- **No native file dialogs** on WASM (`rfd` is gated off).
- **No clipboard** (`arboard` is gated off).
- **WebGL2 backend**: some `wgpu` shader features may be unavailable;
  fall back to WebGPU or headless SVG if needed.
- **`wasm-pack` required**: plain `cargo build --target wasm32` produces
  `.wasm` files but you need `wasm-bindgen` (via wasm-pack or manually) to
  generate the JS glue layer.

## Bundle Size Analysis

The following are approximate **release** `.wasm` binary sizes for each usage
profile. Actual sizes depend on which cvkg feature flags are enabled and how
aggressively the linker strips unused code (LTO + `wasm-opt`).

### Measurement Methodology

Three representative apps are measured for accurate sizing:

| Target | Description | Expected Use Case |
|--------|-------------|-------------------|
| **Minimal** | One `Button` + one `Text` ("Hello, CVKG!") | Smallest possible CVKG app |
| **Typical** | `BentoGrid` + `Carousel` + `Card` grid + `Form` | Persona 5 landing page |
| **Full** | Dashboard with `DataGrid` + `GpuCharts` + `Navigation` + `ThemeSwitch` | App with rich UI patterns |

**Build commands:**
```bash
# Build for WASM
cargo build -p adele-web-demo --target wasm32-unknown-unknown --release

# Measure wasm binary size
stat --format=%s target/wasm32-unknown-unknown/release/adele_web_demo.wasm

# Optimize with wasm-opt
wasm-opt -O4 -o pkg/optimized.wasm pkg/adele_web_demo_bg.wasm

# Gzipped transfer size
gzip -c pkg/*.wasm | wc -c
```

**Cold-start measurement:**
- Desktop Chrome: Use Performance DevTools → FCP marker
- Mobile Chrome (Moto G throttle): Network throttling + CPU 4x slowdown

### Reference Sizes

| Profile  | Deps linked | wasm size | JS glue | LTO+opt |
|----------|------------|----------:|--------:|:--------|
| Headless (no GPU) | core, vdom, layout, styles | ~250 KB | ~15 KB | 180 KB |
| With interactive components | +buttons, inputs, selects | ~400 KB | ~20 KB | 280 KB |
| Full component suite | +tree, tabs, tables, animation | ~600 KB | ~25 KB | 380 KB |
| WebGPU renderer | +wgpu (WebGPU backend) | ~1.5 MB | ~40 KB | 800 KB |

### CI Lint

To prevent accidental bloat, add a CI step that checks the wasm binary stays
under a threshold:

```yaml
# .github/workflows/ci.yml (excerpt)
wasm-size-check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: rustup target add wasm32-unknown-unknown
    - run: cargo build -p adele-web-demo --target wasm32-unknown-unknown --release
    - run: |
        WASM=$(ls target/wasm32-unknown-unknown/release/*.wasm | head -1)
        SIZE=$(stat --format=%s "$WASM")
        echo "WASM size: $SIZE bytes"
        test "$SIZE" -le 2000000  # ≤ 2 MB
```

### wasm-opt Integration

Run [`wasm-opt`](https://github.com/WebAssembly/binaryen) from the
[binaryen](https://github.com/WebAssembly/binaryen) toolchain to shrink the
binary after wasm-bindgen:

```bash
# Install
apt install binaryen          # Debian/Ubuntu
brew install binaryen          # macOS
cargo install wasm-opt         # via Rust

# Optimise
wasm-opt -O4 -o output.wasm input.wasm
# -O4  = aggressive (best size, may be slow)
# -Oz  = size-optimised (balance)
# -O   = default optimisations
```

### wasm-pack Build

The recommended production build:

```bash
# Compile + bindgen + optimise in one step
wasm-pack build --target web --release

# Then strip debug symbols (optional)
wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm
```

### Known Build Issues

The following were discovered when validating WASM compilation in CI:

- **`getrandom` on wasm32**: requires the `wasm_js` feature. Add to
  `Cargo.toml`:
  ```toml
  [target.'cfg(target_arch = "wasm32")'.dependencies]
  getrandom = { version = "0.4", features = ["wasm_js"] }
  ```
- **`cvkg-core::knowledge`**: `fallback_runtime()` is gated with
  `#[cfg(not(target_arch = "wasm32"))]`. Ensure no code path that touches
  async knowledge accesses this function in WASM builds.
- **WebGL2 backend**: `wgpu` may fall back to the software renderer on older
  mobile browsers. Prefer WebGPU where available.

## Headless Testing (CI)

CVKG's `CvkgHeadless` backend works on WASM — useful for CI snapshot testing:...

```rust
use cvkg::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn headless_snapshot_matches() {
    let view = Button::new("Test");
    let mut h = CvkgHeadless::new(view, Rect::sized(200, 60));
    let frame = h.render_frame();
    assert!(frame.telemetry.contains_key("phases_flushed"));
}
```

## Further Reading

- `COMPONENTS.md` — complete component index
- Theme docs in `cvkg-themes/src/lib.rs`
- Demo source under `demos/adele-web/` and `demos/berserker-fire-web/`
