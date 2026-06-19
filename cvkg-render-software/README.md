# cvkg-render-software

## Purpose
Provides a CPU-based software rendering fallback using standard text layouts.

## Boundaries
- It does not run wgpu bindings or compile pipeline graphics shaders.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-render-software["cvkg-render-software (Focal Crate)"]
    cvkg-core["cvkg-core"]
    cvkg-render-software --> cvkg-core
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-render-software --> cvkg-runic-text
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-render-software focal
    class cvkg-runic-text,cvkg-core sibling
```

## Public API Overview
- `SoftwareRenderer` — Software drawing interface.

## Usage Example
```rust
use cvkg_render_software::SoftwareRenderer;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
