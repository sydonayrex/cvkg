# cvkg-physics TLDR.md

## Purpose
Own the physics simulation engine: collision detection, rigid body dynamics, and physics-based interactions for UI elements.

## Ownership
- `src/lib.rs` — Physics world, bodies, colliders, constraints
- Integration with cvkg-anim for physics-driven animations

## Local Contracts
- Physics simulation must be deterministic given the same inputs.
- Time step must be configurable but default to stable values.
- Must not panic on degenerate geometry (zero-size, NaN).

## Verification
- Run `cargo test -p cvkg-physics`
- Run `cargo check --workspace`
