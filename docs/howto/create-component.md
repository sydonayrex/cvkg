# How to Create a Component (Manual)

## Goal

Create a custom UI component by implementing the `View` trait manually, without using macros.

## Prerequisites

- Understanding of the `View` trait from `cvkg-core`
- A working CVKG project

## Steps

### 1. Define the Component Struct

```rust
use cvkg_core::{Color, Never, Rect, Renderer, View};

pub struct MyButton {
    label: String,
    color: Color,
}
```

### 2. Implement `View`

```rust
impl View for MyButton {
    type Body = Never; // Primitive view -- no child body

    fn body(self) -> Self::Body {
        unreachable!("Primitive view renders directly via Renderer")
    }
}
```

### 3. Implement Custom Rendering

Use the `Renderer` trait to issue draw commands. The renderer provides methods for drawing rectangles, text, and paths.

### 4. Add to a Parent View

```rust
struct App;

impl View for App {
    type Body = MyButton;

    fn body(self) -> Self::Body {
        MyButton {
            label: "Click me".into(),
            color: Color { r: 0.2, g: 0.6, b: 1.0, a: 1.0 },
        }
    }
}
```

## Expected Output

A rendered button in the application window.

## What Can Go Wrong

- **Forgetting `type Body = Never`**: Primitive views must use `Never` as the body type. Composite views use a concrete child type.
- **State in `body()`**: The `body()` method must be pure and side-effect free. Store state in `State<T>` fields, not in local variables.
- **Not calling renderer methods**: If you implement `body()` without issuing draw commands, nothing appears on screen.
