# cvkg-svg-filters AGENTS.md

## Purpose
Own the SVG filter effects: blur, drop-shadow, color matrix, composite filters, and the SVG filter graph evaluation engine.

## Ownership
- `src/lib.rs` — SVG filter definitions, filter graph evaluation
- Integration with cvkg-render-gpu for GPU-accelerated filters

## Local Contracts
- SVG filter semantics must match the SVG 1.1 specification.
- Filter graph must be evaluable on both CPU and GPU backends.
- Must handle filter region clipping correctly.

## Verification
- Run `cargo test -p cvkg-svg-filters`
- Run `cargo check --workspace`
