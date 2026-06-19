# cvkg-runic-text

## Purpose
Text shaping, layout, and font rasterization coordinates engine using HarfBuzz and Swash.

## Boundaries
- It does not allocate GPU memory or run desktop event loop windows.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-runic-text["cvkg-runic-text (Focal Crate)"]
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-runic-text
    cvkg-render-software["cvkg-render-software"]
    cvkg-render-software --> cvkg-runic-text
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-runic-text focal
    class cvkg-test,cvkg-render-software sibling
```

## Public API Overview
- `RunicTextEngine` — Main shaper logic.

## Usage Example
```rust
use cvkg_runic_text::RunicTextEngine;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
