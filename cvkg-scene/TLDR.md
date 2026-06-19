# cvkg-scene TLDR.md

## Purpose
Own the 3D scene graph: node hierarchy, transforms, cameras, lights, and the scene rendering pipeline.

## Ownership
- `src/lib.rs` — Scene graph, NodeId (type alias for KvasirId), Transform3D, Camera3D, Light types
- Scene graph traversal and culling

## Local Contracts
- Scene graph must support arbitrary depth without stack overflow.
- Transform propagation must be efficient (dirty-flag based).
- Must integrate with cvkg-render-gpu for 3D rendering.
- NodeId is a type alias for cvkg_core::KvasirId — never define a separate identity struct.

## Verification
- Run `cargo test -p cvkg-scene`
- Run `cargo check --workspace`
