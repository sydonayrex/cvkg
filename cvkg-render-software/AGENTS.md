# cvkg-render-software AGENTS.md

## Purpose
Own the software renderer backend: CPU-based rendering fallback for environments where GPU acceleration is unavailable.

## Ownership
- `src/lib.rs` — Software renderer implementation
- CPU-based path rendering, blending, compositing

## Local Contracts
- Must produce pixel-identical results to GPU backend (within rounding tolerance).
- Must be performant enough for development and testing use.
- Must implement the full cvkg-core Renderer trait.

## Verification
- Run `cargo test -p cvkg-render-software`
- Run `cargo check --workspace`
