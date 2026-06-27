# Web Bundle Size Baseline

## Current State

The existing WASM bundle (`cvkg-webkit-server/static/pkg/ulfhednar_bg.wasm`) is
approximately **1.3 MB** (1,305,401 bytes).

## Target

For ad-network compatibility, the IAB recommends **<200 KB** initial load. This
means the CVKG WASM bundle needs to be reduced by approximately **85%**.

## Bundle Composition

The current bundle includes the entire CVKG workspace compiled to WebAssembly:

- `cvkg-core` -- Core traits, state management, layout
- `cvkg-vdom` -- Virtual DOM diffing
- `cvkg-render-gpu` -- wgpu-based GPU rendering (WebGL backend)
- `cvkg-components` -- 80+ UI components
- `cvkg-themes` -- OKLCH color system
- `cvkg-anim` -- Spring physics animation
- `cvkg-runic-text` -- HarfBuzz text shaping
- `cvkg-layout` -- Taffy layout engine
- `cvkg-svg-filters` -- SVG filter effects
- `cvkg-svg-serialize` -- SVG serialization
- ... and all transitive dependencies

## Reduction Strategies

### 1. Feature Flags (estimated 40-60% reduction)

Add a `web-ad` feature flag that strips out non-essential crates:

```toml
[features]
web-ad = [
    "cvkg-core/minimal",
    "cvkg-vdom/minimal",
    "cvkg-render-gpu/webgl-only",
    # Exclude: cvkg-physics, cvkg-svg-filters, cvkg-runic-text, cvkg-layout
]
```

### 2. Component Tree Shaking (estimated 20-30% reduction)

Use `wasm-bindgen`'s `--weak-refs` and `--reference-types` flags. Ensure
`cargo build --target wasm32-unknown-unknown -Z build-std=std,panic_abort`
is used with `opt-level = "z"`.

### 3. Text Shaping Simplification (estimated 10-15% reduction)

Replace `cvkg-runic-text` (HarfBuzz) with a simpler CPU-based text shaper for
ad units that don't need complex script support.

### 4. Layout Engine Swap (estimated 5-10% reduction)

Replace Taffy with a minimal flexbox-only layout engine for ad units.

## Recommended Approach

1. Add `web-ad` feature flag to all crates
2. Create a `cvkg-ad` umbrella crate that depends only on essential crates
3. Build with `wasm-opt -Oz` (Binaryen)
4. Measure bundle size at each step

## Measurement Command

```bash
cargo build -p cvkg-ad --target wasm32-unknown-unknown --release --features web-ad
wasm-opt -Oz target/wasm32-unknown-unknown/release/cvkg_ad.wasm -o optimized.wasm
ls -la optimized.wasm
```
