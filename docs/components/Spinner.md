# Spinner

`cvkg_components::Spinner` (English alias for `HatiSpinner`)

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Spinner::new() -> Spinner
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.size(f32)` | `f32` | `24.0` | Spinner diameter in logical pixels |
| `.variant(SpinnerVariant)` | `SpinnerVariant` | `Determinate` | Animation variant (Determinate, Indeterminate) |
| `.color([f32; 4])` | `[f32; 4]` | theme accent | Override spinner color |

## Example

```rust
HStack::new(8.0)
    .child(Spinner::new().size(16.0))
    .child(Text::new("Loading..."))
```
