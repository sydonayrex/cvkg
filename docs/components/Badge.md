# Badge

`cvkg_components::Badge`

## Import

```rust
use cvkg::prelude::*;
// Badge is not in the prelude; import explicitly:
use cvkg_components::Badge;
```

## Signature

```rust
Badge::new(text: impl Into<String>) -> Badge
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.variant(BadgeVariant)` | `BadgeVariant` | `Default` | Visual variant (Default, Secondary, Outline, Destructive, Success) |
| `.size(BadgeSize)` | `BadgeSize` | `Md` | Size (Sm, Md, Lg) |
| `.on_click(impl Fn())` | `Fn()` | `None` | Optional click callback |
| `.dot_indicator(bool)` | `bool` | `false` | Show status dot before text |
| `.count_only(bool)` | `bool` | `false` | Render as number-only circle |

## Example

```rust
HStack::new(8.0)
    .child(Badge::new("Default"))
    .child(Badge::new("Info").variant(BadgeVariant::Secondary))
    .child(Badge::new("Outline").variant(BadgeVariant::Outline))
```
