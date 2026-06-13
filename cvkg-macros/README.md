# cvkg-macros

```mermaid
graph TD
    cvkg-core["cvkg-core"]
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene["cvkg-scene"]
    cvkg-layout["cvkg-layout"]
    cvkg-render-gpu["cvkg-render-gpu"]
    cvkg-render-native["cvkg-render-native"]
    cvkg-compositor["cvkg-compositor"]
    cvkg-themes["cvkg-themes"]
    cvkg-anim["cvkg-anim"]
    cvkg-flow["cvkg-flow"]
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-svg-filters["cvkg-svg-filters"]
    cvkg-svg-serialize["cvkg-svg-serialize"]
    cvkg-components["cvkg-components"]
    cvkg-macros["cvkg-macros"]
    cvkg-cli["cvkg-cli"]
    cvkg-webkit-server["cvkg-webkit-server"]
    cvkg-test["cvkg-test"]
    cvkg-physics["cvkg-physics"]
    cvkg["cvkg (umbrella)"]

    cvkg-vdom --> cvkg-core
    cvkg-vdom --> cvkg-scene
    cvkg-layout --> cvkg-core
    cvkg-layout --> cvkg-anim
    cvkg-scene --> cvkg-core

    cvkg-render-gpu --> cvkg-core
    cvkg-render-gpu --> cvkg-compositor
    cvkg-render-gpu --> cvkg-svg-filters
    cvkg-render-gpu --> cvkg-svg-serialize
    cvkg-render-gpu --> cvkg-runic-text

    cvkg-render-native --> cvkg-core
    cvkg-render-native --> cvkg-render-gpu
    cvkg-render-native --> cvkg-vdom
    cvkg-render-native --> cvkg-themes

    cvkg-compositor --> cvkg-core

    cvkg-themes --> cvkg-core
    cvkg-themes --> cvkg-anim
    cvkg-anim --> cvkg-core
    cvkg-flow --> cvkg-core
    cvkg-flow --> cvkg-scene
    cvkg-flow --> cvkg-themes

    cvkg-runic-text --> cvkg-core
    cvkg-svg-filters --> cvkg-core

    cvkg-components --> cvkg-core
    cvkg-components --> cvkg-vdom
    cvkg-components --> cvkg-layout
    cvkg-components --> cvkg-themes
    cvkg-components --> cvkg-anim
    cvkg-components --> cvkg-runic-text

    cvkg-macros --> cvkg-core
    cvkg-cli --> cvkg-core
    cvkg-cli --> cvkg-physics
    cvkg-cli --> cvkg-anim
    cvkg-cli --> cvkg-macros
    cvkg-webkit-server --> cvkg-cli
    cvkg-physics --> cvkg-core
    cvkg-physics --> cvkg-scene

    cvkg --> cvkg-core
    cvkg --> cvkg-vdom
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
```

`cvkg-macros` provides the procedural macros that power the CVKG developer experience, automating boilerplate and enabling a declarative, SwiftUI-like syntax in Rust.

## Boundaries and Responsibilities

This crate provides compile-time transformations. It does NOT contain runtime logic. Its responsibilities include:
- Transforming functions into `View` structs via `#[view_component]`.
- Automating state management boilerplate with `#[state]` and `#[binding]`.
- Providing the `view! { ... }` DSL for hierarchical UI definition.
- Deriving the `View` trait for structs to enable modifier-based composition.
- Generating VDOM metadata and serialization logic for data models.

## Public API Overview

### Attribute Macros
- `#[view_component]`: The primary macro for creating UI components. It generates a struct and implements the `View` trait for the decorated function.
- `#[state]`: Automatically derives `Clone`, `Debug`, `Default`, and `Serde` traits for state containers.
- `#[binding]`: Marks a struct as a reactive read/write reference to parent state.

### Derive Macros
- `#[derive(View)]`: Implements `cvkg_core::View` for a struct, defaulting to a primitive view unless a `body` method is present.

### Function-like Macros
- `view! { ... }`: A DSL for nesting components and applying modifiers in a readable hierarchy.
- `cvkg_model! { ... }`: Generates data models with unique VDOM identifiers for efficient reconciliation.

## Usage Example

```rust
use cvkg::prelude::*;

#[view_component]
fn Profile(name: String, rank: u32) {
    HStack::new(8.0) {
        Image::new("avatar_placeholder")
            .frame(40.0, 40.0);
            
        VStack::new(2.0, Alignment::Leading) {
            Text::new(name).bold();
            Text::new(format!("Rank: {}", rank)).caption();
        }
    }
}
```

## Known Limitations
- Macro expansion can significantly increase compile times for extremely large view trees; use `view_component` to break down complex UIs.
- Error messages from inside macros can sometimes be opaque; always check the generated code if debugging complex transformations.
