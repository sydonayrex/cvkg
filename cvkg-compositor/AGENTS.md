# cvkg-compositor AGENTS.md

## Purpose
Own the compositor: layer management, z-ordering, blend modes, and the final compositing pipeline that combines all visual layers into the output frame.

## Ownership
- `src/lib.rs` — Compositor state, layer stack, blend modes
- Integration with cvkg-render-gpu for GPU-accelerated compositing

## Local Contracts
- Compositor must handle arbitrary layer depth without overflow.
- Blend modes must be composable and order-independent where possible.
- Must integrate with the Kvasir render graph for GPU compositing.

## Verification
- Run `cargo test -p cvkg-compositor`
- Run `cargo check --workspace`
