# How-to: Creating Components

Creating reusable components is the core workflow in CVKG. This guide covers how to use procedural macros and the modifier system to build everything from simple buttons to complex dashboards.

## 1. Using `#[view_component]`

The simplest way to create a component is by decorating a function with `#[view_component]`. This macro transforms the function into a `View` struct and handles the boilerplate.

```rust
use cvkg::prelude::*;

#[view_component]
fn StatusBadge(label: String, is_active: bool) {
    HStack::new(4.0) {
        Circle::new(4.0)
            .foregroundColor(if is_active { Color::GREEN } else { Color::RED });
            
        Text::new(label)
            .caption()
            .bold();
    }
    .padding(8.0)
    .background(Color::TACTICAL_OBSIDIAN)
    .cornerRadius(4.0)
}
```

### Key Rules:
- The function name becomes the component name (e.g., `StatusBadge`).
- Arguments become public fields on the generated struct.
- The return type is automatically inferred as `impl View`.

## 2. Using State and Bindings

To make components interactive, use the `#[state]` and `#[binding]` macros.

```rust
#[state]
struct AppState {
    count: i32,
}

#[view_component]
fn Counter(state: Binding<AppState>) {
    VStack::new(10.0) {
        Text::new(format!("Count: {}", state.count.get()));
        
        Button::new("Increment", || {
            state.count.set(state.count.get() + 1);
        });
    }
}
```

## 3. Applying Modifiers

Modifiers are chained methods that transform a view. They are applied **from top to bottom**.

```rust
Text::new("Bifrost Effect")
    .padding(20.0)             // 1. Add space
    .background(Color::CLEAR)  // 2. Set background
    .bifrost(10.0, 1.0, 0.5)   // 3. Apply frosted glass
    .frame(width: 200.0)       // 4. Constrain size
```

## 4. Primitive Views

If you need to implement a low-level primitive (like a custom shader effect), you can implement the `View` trait manually:

```rust
struct MyPrimitive;

impl View for MyPrimitive {
    type Body = Never; // Primitive views have no children
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [1.0, 0.0, 0.0, 1.0]);
    }
}
```

## 5. Best Practices

- **Atomic Components**: Keep components small. If a component grows past 50 lines, break it into sub-components.
- **Prop Drilling**: Avoid passing deep state; use `EnvironmentValue` for global resources.
- **Performance**: Use `.memoize()` for expensive drawing operations that don't change every frame.
