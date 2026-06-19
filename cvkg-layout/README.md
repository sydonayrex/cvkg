# cvkg-layout

## Purpose
Computes spatial bounds and flexbox positioning using Taffy constraints.

## Boundaries
- It does not draw vector lines or compile wgpu shader programs.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-layout["cvkg-layout (Focal Crate)"]
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-layout
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-layout focal
    class cvkg-test sibling
```

## Public API Overview
- `SizeProposal` — proposed layout bounds.
- `HStack` / `VStack` — layout containers.

## Usage Example
```rust
use cvkg_layout::{HStack, SizeProposal};
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
