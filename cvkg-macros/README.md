# cvkg-macros

## Purpose
Procedural macros scaffolding DSL view bodies and reactive state bindings.

## Boundaries
- It does not process dynamic runtime layout constraints.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-macros["cvkg-macros (Focal Crate)"]
    cvkg-core["cvkg-core"]
    cvkg-macros --> cvkg-core
    cvkg-components["cvkg-components"]
    cvkg-macros --> cvkg-components
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-macros
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-macros focal
    class cvkg-test,cvkg-core,cvkg-components sibling
```

## Public API Overview
- `#[derive(View)]` — Macro macro derivation.
- `hamr!` — View composition DSL.

## Usage Example
```rust
use cvkg_macros::View;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
