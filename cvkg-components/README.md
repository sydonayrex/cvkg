# cvkg-components

## Purpose
Tahoe component library containing base inputs, buttons, and custom layout controls.

## Boundaries
- It does not write GPU hardware drivers or compute text metrics directly.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-components["cvkg-components (Focal Crate)"]
    cvkg-macros["cvkg-macros"]
    cvkg-macros --> cvkg-components
    cvkg-scene["cvkg-scene"]
    cvkg-scene --> cvkg-components
    cvkg-test["cvkg-test"]
    cvkg-test --> cvkg-components
    cvkg-icons["cvkg-icons"]
    cvkg-icons --> cvkg-components
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-components focal
    class cvkg-macros,cvkg-scene,cvkg-test,cvkg-icons sibling
```

## Public API Overview
- `Button` — Native-drawn click component.
- `PhoneInput` / `MentionInput` — Custom input editors.

## Usage Example
```rust
use cvkg_components::Button;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
