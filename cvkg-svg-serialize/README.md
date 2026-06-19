# cvkg-svg-serialize

## Purpose
Formats and writes raw SVG XML files from geometric vector structures.

## Boundaries
- It does not parse or draw SVGs; it is strictly a write-path serializer.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-svg-serialize["cvkg-svg-serialize (Focal Crate)"]
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-svg-serialize focal
```

## Public API Overview
- `SvgSerializer` — Serializes graphics to raw XML.

## Usage Example
```rust
use cvkg_svg_serialize::SvgSerializer;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
