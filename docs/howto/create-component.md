# How to Create a Component

Goal: Implement a custom UI component in CVKG.

## Prerequisites

- Understanding of the `View` trait
- Basic Rust knowledge

## Steps

### 1. Define the component struct

```rust
use cvkg_core::{View, Rect, Renderer, Size, SizeProposal};

pub struct MyComponent {
    label: String,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl MyComponent {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), on_click: None }
    }
    
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}```

### 2. Implement the View trait

```rust
impl View for MyComponent {
    type Body = Self;
    
    fn body(self) -> Self::Body { self }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, [0.2, 0.6, 1.0, 1.0]);
        renderer.draw_text(&self.label, rect.x + 16.0, rect.y + 16.0, 16.0, [1.0, 1.0, 1.0, 1.0]);
    }
    
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: 120.0, height: 44.0 }
    }
}
```

### 3. Use the component

```rust
use cvkg_components::{VStack, Text};
use cvkg_core::View;

let view = VStack::new(16.0)
    .child(MyComponent::new("Click me").on_click(|| println!("clicked")))
    .child(Text::new("Static text"));
```

## Expected Output

The custom component renders with the specified appearance and handles click events.

## Recovery

If the component does not appear:

1. Verify `intrinsic_size()` returns non-zero dimensions
2. Check that `render()` is actually drawing content
3. Ensure parent container has proper layout constraints