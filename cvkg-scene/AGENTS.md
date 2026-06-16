# cvkg-scene AGENTS.md

## Purpose
Own the 3D scene graph: node hierarchy, transforms, cameras, lights, and the scene rendering pipeline.

## Ownership
- `src/lib.rs` — Scene graph, NodeId, Transform3D, Camera3D, Light types
- Scene graph traversal and culling

## Local Contracts
- Scene graph must support arbitrary depth without stack overflow.
- Transform propagation must be efficient (dirty-flag based).
- Must integrate with cvkg-render-gpu for 3D rendering.

## Verification
- Run `cargo test -p cvkg-scene`
- Run `cargo check --workspace`
