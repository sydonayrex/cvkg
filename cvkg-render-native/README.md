# cvkg-render-native

**cvkg-render-native** provides window management and event loop integration for CVKG on desktop platforms using winit.

## What This Crate Does

- Creates native windows using winit
- Integrates the GPU renderer with the native event loop
- Handles input events (mouse, keyboard, IME)
- Provides application lifecycle management

## What This Crate Does NOT Do

- Does not provide WebGPU rendering (see cvkg-render-gpu)
- Does not provide web bindings (see cvkg-render-web)
- Does not handle HTTP requests

## Public API Overview

### NativeApp

```rust
/// The main application container for native platforms
pub struct NativeApp {
    // private fields
}

impl NativeApp {
    /// Create a new native application with the given dimensions
    pub fn new(width: u32, height: u32, title: &str) -> Self;
    
    /// Run the application with the given view
    pub fn run<V: View>(&mut self, view: V);
    
    /// Set the window title
    pub fn set_title(&self, title: &str);
    
    /// Set whether the window is resizable
    pub fn set_resizable(&self, resizable: bool);
}```

### run_app Function

```rust
/// Convenience function to initialize and run a CVKG application
pub fn run_app<V: View + 'static>(width: u32, height: u32, title: &str, view_factory: impl FnOnce() -> V + 'static);
```

### Event Handling

```rust
/// Input events from the window system
pub enum NativeEvent {
    PointerDown { x: f32, y: f32, button: u32 },
    PointerUp { x: f32, y: f32, button: u32 },
    PointerMove { x: f32, y: f32 },
    KeyDown { key: String, modifiers: ModifiersState },
    KeyUp { key: String, modifiers: ModifiersState },
    Resize { width: u32, height: u32 },
    Close,
}```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | true | Use standard library |
| `x11` | false | Enable X11 support on Linux |
| `wayland` | false | Enable Wayland support on Linux |

## Usage Example

```rust
use cvkg_render_native::run_app;
use cvkg_components::{VStack, Text, Button};

fn main() {
    run_app(800, 600, "My App", || {
        VStack::new(16.0)
            .child(Text::new("Hello, Native!").size(24.0))
            .child(Button::new("Click Me"))
    });
}
```

## Known Limitations

- Window decorations are platform-dependent
- IME support requires platform-specific configuration
- HiDPI scaling may not work correctly on all platforms