# cvkg

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg` is the primary entry point and public facade for the Cyber Viking Kvasir Graph framework. It unifies the modular workspace crates into a single, cohesive API.

## Boundaries and Responsibilities

This crate does NOT contain implementation logic. Instead, it:
- Orchestrates the feature-gated selection of rendering backends (`gpu`, `native`, `web`).
- Re-exports core components, layout engines, and animation solvers.
- Provides a `prelude` for streamlined application development.

## Public API Overview

### Feature Flags
You MUST select exactly one rendering feature for your application:
- `gpu`: Enables direct `wgpu` rendering (Surtr).
- `native`: Enables `winit` windowing and desktop integration.
- `web`: Enables WASM/Browser deployment.

### Re-exported Modules
- `core`: Fundamental traits and types.
- `layout`: Stacking and grid containers.
- `anim`: Physics-based animation solvers.
- `components`: Reusable UI elements.
- `scene`: Retained scene graph management.
- `themes`: Semantic design tokens.

### The Prelude
```rust
use cvkg::prelude::*;
// Includes: View, State, Binding, Rect, view_component macro, etc.
```

## Usage Example

```rust
// Cargo.toml
// cvkg = { version = "0.1.18", features = ["native"] }

use cvkg::prelude::*;

#[view_component]
fn App() {
    VStack::new(20.0, Alignment::Center, Distribution::Center) {
        Text::new("Skål, Cyber Viking!")
            .gungnir([1.0, 0.5, 0.0, 1.0], 10.0, 2.0)
    }
}
```

## Known Limitations
- Mixing backend features (e.g., `native` and `web`) in a single target is unsupported and will lead to compilation conflicts.
- Always check the root README for system-level prerequisites for your chosen backend.