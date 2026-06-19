# cvkg-icons

## Purpose
Vector icon SVG asset storage, retrieval, and cache registration.

## Boundaries
- It does not calculate visual constraints or apply text styling rules.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-icons["cvkg-icons (Focal Crate)"]
    cvkg-core["cvkg-core"]
    cvkg-icons --> cvkg-core
    cvkg-components["cvkg-components"]
    cvkg-icons --> cvkg-components
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-icons focal
    class cvkg-core,cvkg-components sibling
```

## Public API Overview
- `IconRegistry` — Dynamic icon registration.

## Usage Example
```rust
use cvkg_icons::IconRegistry;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
