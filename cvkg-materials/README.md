# cvkg-materials

## Purpose
Defines materials configs like Glass, Mica, and Acrylic profiles.

## Boundaries
- It does not render shapes or execute GPU fragment shaders.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-materials["cvkg-materials (Focal Crate)"]
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-materials focal
```

## Public API Overview
- `GlassMaterial` — blur/tint settings.
- `MicaMaterial` — backdrop characteristics.

## Usage Example
```rust
use cvkg_materials::GlassMaterial;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
