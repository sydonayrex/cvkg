# cvkg-core

**cvkg-core** contains the fundamental traits, types, and modifiers that define the CVKG framework. It is the "glue" that allows the VDOM, renderers, and components to interoperate.

## What This Crate Does

- Defines the `View` trait for UI composition
- Provides the `Renderer` trait for drawing operations
- Contains geometric types (`Rect`, `Size`, `EdgeInsets`)
- Implements fluent modifiers for visual effects
- Provides reactive state management (`State`, `Binding`)
## What This Crate Does NOT Do

- Does not provide concrete rendering implementations (see cvkg-render-gpu, cvkg-render-native)
- Does not provide layout algorithms (see cvkg-layout)
- Does not provide UI components (see cvkg-components)

## Public API Overview

### Core Traits

```rust
// View trait - the primary building block of UI
pub trait View: Sized + Send {
    type Body: View;
    fn body(self) -> Self::Body;
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect);
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size;
}
```

### Renderer Trait

```rust
pub trait Renderer: ElapsedTime + Send {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]);
    fn draw_image(&mut self, image_name: &str, rect: Rect);
    fn push_clip_rect(&mut self, rect: Rect);
    fn pop_clip_rect(&mut self);
}
```

### Geometric Types

```rust
pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }
pub struct Size { pub width: f32, pub height: f32 }
pub struct EdgeInsets { pub top: f32, pub leading: f32, pub bottom: f32, pub trailing: f32 }
```

### State Management

```rust
pub struct State<T: Clone + Send + Sync + 'static> { /* ... */ }
impl<T> State<T> {
    pub fn new(initial: T) -> Self;
    pub fn get(&self) -> T;
    pub fn set(&self, value: T);
}
pub struct Binding<T> { /* ... */ }
```

### Fluent Modifiers

```rust
// Visual modifiers
view.padding(16.0)
view.background([0.0, 0.0, 0.0, 1.0])
view.opacity(0.5)
view.frame(Some(100.0), Some(50.0))
view.clip_to_bounds()
view.border([1.0, 1.0, 1.0, 1.0], 2.0)

// Event modifiers
view.on_click(|| println!("clicked"))
view.on_pointer_enter(|| println!("enter"))
view.on_pointer_leave(|| println!("leave"))
view.on_appear(|| println!("appear"))

// Cyber Viking visual effects
view.bifrost(blur: 10.0, saturation: 1.5, opacity: 0.8)
view.gungnir(color: "cyan", radius: 10.0, intensity: 0.8)
view.mjolnir_slice(angle: 45.0, offset: 10.0)
view.magnetic(radius: 100.0, intensity: 0.5)
```

### Enums and Constants

```rust
pub enum MemoryLayer { Episodic, Semantic, Procedural }
pub enum Realm { Midgard, Asgard }
pub enum RenderTier { Tier1GPU, Tier2GPU, Tier3Fallback }
pub enum Alignment { Top, Center, Bottom, Leading, Trailing, Fill }
pub enum Distribution { Leading, Center, Trailing, Fill, SpaceBetween }
pub enum Orientation { Horizontal, Vertical }
```

### Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | true | Use standard library (disable for no_std) |

## Known Limitations

- The `Renderer` trait uses `&mut dyn Renderer` for object safety; concrete types should use `&mut Self` for performance
- State updates trigger full re-renders by default; use `MemoView` for optimization
- Some visual effects may not be supported by all renderer backends

## Usage Example

```rust
use cvkg_core::{View, Rect, Renderer};

struct MyButton {
    label: String,
}

impl View for MyButton {
    type Body = Self;
    
    fn body(self) -> Self::Body { self }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, [0.2, 0.6, 1.0, 1.0]);
        renderer.draw_text(&self.label, rect.x + 16.0, rect.y + 16.0, 16.0, [1.0, 1.0, 1.0, 1.0]);
    }
    
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size { width: 120.0, height: 44.0 }
    }
}
```