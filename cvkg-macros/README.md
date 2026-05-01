# cvkg-macros

**cvkg-macros** contains procedural macros for the CVKG framework.

## Features
*   `#[derive(View)]`: Automatically implements the `View` trait for custom structs, handling state tracking and body composition.
*   `view! { ... }`: A DSL-like macro for declarative UI definition, inspired by Swift's Result Builders, supporting conditional rendering and list expansion.
*   `cvkg_model!`: Macro for generating data models compatible with the CVKG VDOM diffing engine.
