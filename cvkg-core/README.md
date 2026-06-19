# cvkg-core

## Purpose
Defines fundamental traits, shared data structures, state management types, and layout primitives for CVKG.

## Boundaries
- It does not implement layout calculations or drawing operations; those are handled by cvkg-layout and render backends.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-core["cvkg-core (Focal Crate)"]
    cvkg-macros["cvkg-macros"]
    cvkg-macros --> cvkg-core
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-core
    cvkg-physics["cvkg-physics"]
    cvkg-physics --> cvkg-core
    cvkg-icons["cvkg-icons"]
    cvkg-icons --> cvkg-core
    cvkg-render-software["cvkg-render-software"]
    cvkg-render-software --> cvkg-core
    cvkg-telemetry["cvkg-telemetry"]
    cvkg-telemetry --> cvkg-core
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-core focal
    class cvkg-physics,cvkg-render-software,cvkg-test,cvkg-telemetry,cvkg-icons,cvkg-macros sibling
```

## Public API Overview
- `View` — Core view trait.
- `Renderer` — Drawing facade.
- `State` — Reactive state wrapper.

## Usage Example
```rust
use cvkg_core::prelude::*;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
