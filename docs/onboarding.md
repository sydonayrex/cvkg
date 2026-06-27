# Onboarding Guide

This guide walks through cloning, building, and making a change to CVKG.

## 1. Clone

```bash
git clone https://github.com/sydonayrex/cvkg.git
cd cvkg
```

## 2. Install Rust Toolchain

Requires Rust 1.85.0 or later (Edition 2024):

```bash
rustup toolchain install stable
rustup default stable
rustup target add wasm32-unknown-unknown
```

## 3. Install System Dependencies (Linux)

```bash
sudo apt-get update
sudo apt-get install -y libwayland-dev libx11-dev libxkbcommon-dev \
    libasound2-dev libfontconfig1-dev pkg-config
```

## 4. Build

```bash
cargo build --workspace
```

## 5. Run the Demo Application

```bash
cargo run -p demos/berserker
```

## 6. Run Tests

Full workspace:

```bash
cargo test --workspace
```

Single crate:

```bash
cargo test -p cvkg-layout
```

Single test by name:

```bash
cargo test -p cvkg-layout tests::test_hstack_basic
```

## 7. Verify a Change

After modifying code, run this sequence:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo check --workspace
cargo test --workspace
```

## Source Layout

```
cvkg/                  -- Umbrella facade crate
cvkg-core/             -- View trait, state, geometry types
cvkg-vdom/             -- Virtual DOM (not a workspace member)
cvkg-scene/            -- Retained scene graph with AABB culling
cvkg-spatial/          -- QuadTree, BVH, SpatialHash
cvkg-layout/           -- Taffy flexbox/grid layout
cvkg-anim/             -- Spring physics, particles
cvkg-render-gpu/       -- WGPU render graph
cvkg-compositor/       -- Layer tree, damage tracking
cvkg-render-native/    -- Desktop windowing via winit
cvkg-render-software/  -- CPU fallback renderer
cvkg-runic-text/       -- HarfBuzz text shaper, BiDi
cvkg-svg-filters/      -- GPU SVG filter primitives
cvkg-svg-serialize/    -- SVG XML write
cvkg-components/       -- Widget library
cvkg-themes/           -- OKLCH color tokens
cvkg-flow/             -- Node graph editor
cvkg-cli/              -- Dev server, asset pipeline
cvkg-webkit-server/    -- axum HTTP/WS server
cvkg-physics/          -- Rigid body simulation
cvkg-scheduler/        -- Frame update ordering
cvkg-test/             -- Visual regression testing
cvkg-macros/           -- hamr! proc macro
cvkg-reflect/          -- Runtime type reflection
cvkg-materials/        -- Glass, Mica, Acrylic data
cvkg-accessibility/    -- Accessibility tree, focus
cvkg-certification/    -- Cross-crate test suites
cvkg-telemetry/        -- Opt-in metrics
cvkg-icons/            -- Icon registry
cvkg-gallery/           -- Component gallery demo
cvkg-game-hud/         -- Game HUD components
cvkg-export-raster/    -- PNG/GIF export
berserker/             -- Native tactical HUD demo
demos/adele-web/       -- Web design explorer
demos/niflheim-web/    -- WASM component suite
demos/niflheim-wasi/   -- WASI headless target
demos/berserker-fire-web/ -- WASM stress test
```

## Troubleshooting

See [troubleshooting.md](./troubleshooting.md) for common build errors, runtime crashes, and visual artifacts.

## Maintainer Contact

TODO: add maintainer contact
