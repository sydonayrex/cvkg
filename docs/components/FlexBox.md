# FlexBox

`cvkg_components::FlexBox` -- flexible box layout

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
FlexBox::new(orientation: Orientation, spacing: f32) -> FlexBox
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.gap(f32)` | `f32` | `0.0` | Gap between children |
| `.child(V: View + Clone + 'static)` | `View` | - | Add a child view |

## Example

```rust
FlexBox::new(Orientation::Horizontal, 12.0)
    .child(Text::new("Left"))
    .child(Text::new("Right"))
```

## Notes

- `HStack` and `VStack` are convenience wrappers around `FlexBox` with
  horizontal and vertical orientation respectively.
- For most use cases, prefer `HStack`/`VStack` for readability.
