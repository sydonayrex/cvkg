# HStack

`cvkg_components::HStack` -- horizontal stack layout

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
HStack::new(spacing: f32) -> HStack
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.alignment(Alignment)` | `Alignment` | `Start` | Vertical alignment of children |
| `.distribution(Distribution)` | `Distribution` | `Start` | How space is distributed |
| `.wrap(bool)` | `bool` | `false` | Whether children wrap to next line |
| `.gap(f32)` | `f32` | `0.0` | Gap between children (alias for `.spacing()`) |
| `.child(V: View + Clone + 'static)` | `View` | - | Add a child view |

## Example

```rust
HStack::new(8.0)
    .alignment(Alignment::Center)
    .child(Spinner::new())
    .child(Text::new("Loading..."))
```
