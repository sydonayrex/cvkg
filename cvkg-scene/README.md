# cvkg-scene

## Purpose
Retained scene graph with spatial partitioning (QuadTree/BVH) for accelerated culling and hit-testing.

## Boundaries
- It does not compute layout dimensions or flexbox constraints.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-scene["cvkg-scene (Focal Crate)"]
    cvkg-components["cvkg-components"]
    cvkg-scene --> cvkg-components
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene --> cvkg-vdom
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-scene
    cvkg-physics["cvkg-physics"]
    cvkg-physics --> cvkg-scene
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-scene focal
    class cvkg-test,cvkg-vdom,cvkg-physics,cvkg-components sibling
```

## Public API Overview
- `SceneGraph` — Main visual tree buffer.
- `SceneNode` — Render-ready geometry nodes.

## Usage Example
```rust
use cvkg_scene::SceneGraph;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
