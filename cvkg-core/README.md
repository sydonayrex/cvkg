# cvkg-core

**cvkg-core** is the foundation of the CVKG framework. it provides the essential traits, types, and state management logic that power high-fidelity agentic interfaces.

## Key Features

- **The View Trait**: A declarative, functional interface for defining UI structures.
- **State & Bindings**: A reactive state graph with support for local and environmental values.
- **Environment Tokens**: A type-safe way to propagate context (themes, localizations) through the view tree.
- **Geometry Primitives**: High-performance representations of Rects, Sizes, and Offsets.

## Usage

This crate is typically consumed via the main `cvkg` facade, but can be used independently for building custom CVKG backends.

```rust
use cvkg_core::{View, Rect, Renderer};

pub struct MyPrimitive;

impl View for MyPrimitive {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Direct drawing implementation
    }
}
```
