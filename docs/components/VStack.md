# VStack

`cvkg_components::VStack` -- vertical stack layout

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
VStack::new(spacing: f32) -> VStack
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.alignment(Alignment)` | `Alignment` | `Start` | Horizontal alignment of children |
| `.distribution(Distribution)` | `Distribution` | `Start` | How space is distributed |
| `.wrap(bool)` | `bool` | `false` | Whether children wrap to next line |
| `.gap(f32)` | `f32` | `0.0` | Gap between children (alias for `.spacing()`) |
| `.child(V: View + Clone + 'static)` | `View` | - | Add a child view |

## Example

```rust
VStack::new(12.0)
    .alignment(Alignment::Center)
    .child(Text::new("Title").font_size(24.0))
    .child(Text::new("Subtitle").font_size(14.0))
    .child(Button::new("Action", || {}))
```
