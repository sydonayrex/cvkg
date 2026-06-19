# cvkg-svg-filters

## Purpose
Implements SVG filter primitives (blur, morphology, displacement) for visual effects.

## Boundaries
- It does not compute layout margins or compile final rendering buffers.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-svg-filters["cvkg-svg-filters (Focal Crate)"]
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-svg-filters focal
```

## Public API Overview
- `FilterPrimitive` — Base effect definition.
- `BlurEffect` — Box/Gaussian blur parameters.

## Usage Example
```rust
use cvkg_svg_filters::FilterPrimitive;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
