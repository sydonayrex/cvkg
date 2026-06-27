# Select

`cvkg_components::Select<V>` -- dropdown select

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Select::new(placeholder: impl Into<String>) -> Select<V>
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.option(impl Into<String>, V)` | `(String, V)` | - | Add an option with label and value |
| `.selected(usize)` | `usize` | `None` | Pre-select an option by index |

## Example

```rust
Select::new("Choose option")
    .option("Option A", "a")
    .option("Option B", "b")
    .selected(0)
```

## Note

`Select<V>` requires `V: Clone + View`. This is an unusual constraint for a
value type. Consider using a newtype wrapper that implements `View` if you
need a functional dropdown.
