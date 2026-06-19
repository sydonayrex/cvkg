# cvkg-spatial

## Purpose
Provides spatial indexing algorithms (QuadTree, BVH) for hit-testing.

## Boundaries
- It does not resolve CSS style properties.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-spatial["cvkg-spatial (Focal Crate)"]
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-spatial focal
```

## Public API Overview
- `QuadTree` — Spatial bounding box indexer.

## Usage Example
```rust
use cvkg_spatial::QuadTree;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
