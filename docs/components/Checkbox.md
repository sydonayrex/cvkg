# Checkbox

`cvkg_components::Checkbox`

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Checkbox::new(is_checked: bool, on_change: impl Fn(bool) + Send + Sync + 'static) -> Checkbox
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.indeterminate(bool)` | `bool` | `false` | Shows indeterminate state (dash instead of check) |
| `.label(impl Into<String>)` | `String` | `""` | Label text displayed next to the checkbox |

## Example

```rust
Checkbox::new(false, |checked| println!("checked: {}", checked))
    .label("Enable notifications")
```

## Accessibility

- ARIA role: `checkbox`
- Keyboard: toggles on Space/Enter
- Focus ring drawn via `draw_focus_ring()`
