# Button

`cvkg_components::Button` (English alias; canonical type `BifrostButton`)

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Button::new(label: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Button
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.variant(ButtonVariant)` | `ButtonVariant` | `Default` | Visual variant (Default, Destructive, Secondary, Ghost, Link, Glass, TintedGlass, Capsule) |
| `.size(ButtonSize)` | `ButtonSize` | `Default` | Size variant (Small, Default, Large, Icon) |
| `.disabled(bool)` | `bool` | `false` | Disables interaction and dims via theme token |
| `.loading(bool)` | `bool` | `false` | Shows spinner, disables interaction |

## Example

```rust
Button::new("Save", || println!("saved"))
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Default)
```

## Accessibility

- Minimum touch target: 44x45px (enforced by theme defaults).
- Focus ring drawn via `draw_focus_ring()`; do not override focus styling
  without an APCA-validated replacement.
