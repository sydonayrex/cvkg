# cvkg-physics

## Purpose
Tyr rigid-body physics engine solving XPBD constraints and broadphase collisions.

## Boundaries
- It does not draw interactive UI controls or shape text spans.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-physics["cvkg-physics (Focal Crate)"]
    cvkg-core["cvkg-core"]
    cvkg-physics --> cvkg-core
    cvkg-scene["cvkg-scene"]
    cvkg-physics --> cvkg-scene
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-physics focal
    class cvkg-scene,cvkg-core sibling
```

## Public API Overview
- `PhysicsWorld` — Rigid-body solver manager.
- `RigidBody` — Mass-point dynamic bodies.

## Usage Example
```rust
use cvkg_physics::PhysicsWorld;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
