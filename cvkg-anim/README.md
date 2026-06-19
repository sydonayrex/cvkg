# cvkg-anim

## Purpose
Solves spring-physics motion transitions using RK4 numerical integration solvers.

## Boundaries
- It does not manage visual layers or rasterize character fonts.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-anim["cvkg-anim (Focal Crate)"]
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-anim
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-anim focal
    class cvkg-test sibling
```

## Public API Overview
- `SleipnirSolver` — RK4 spring motion solver.
- `RubberBand` — Scroll overflow damping resolver.

## Usage Example
```rust
use cvkg_anim::SleipnirSolver;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
