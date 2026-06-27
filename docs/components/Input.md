# Input

`cvkg_components::Input`

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Input::new(placeholder: impl Into<String>) -> Input
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.value(impl Into<String>)` | `String` | `""` | Initial text value |
| `.on_change(impl Fn(String))` | `Fn(String)` | noop | Called on every keystroke |
| `.on_commit(impl Fn(String))` | `Fn(String)` | noop | Called on Enter/blur |
| `.focused(bool)` | `bool` | `false` | Whether the input has focus |
| `.error(impl Into<String>)` | `String` | `""` | Error message (shows error styling) |
| `.success()` | - | - | Marks input with success styling |

## Example

```rust
Input::new("Enter your name")
    .value(&state.name)
    .on_change(|new_name| state.name = new_name)
    .on_commit(|final_name| save_name(final_name))
```

## Accessibility

- ARIA role: `textbox`
- Error state sets `aria-invalid="true"`
- Minimum touch target: 44px height
