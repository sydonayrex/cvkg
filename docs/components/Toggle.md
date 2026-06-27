# Toggle

`cvkg_components::Toggle`

## Import

```rust
use cvkg::prelude::*;
// Toggle is not in the prelude; import explicitly:
use cvkg_components::Toggle;
```

## Signature

```rust
Toggle::new(label: impl Into<String>, is_on: bool, on_change: impl Fn(bool) + Send + Sync + 'static) -> Toggle
```

## Modifiers

Toggle has no additional modifiers beyond the constructor parameters.

## Example

```rust
Toggle::new("Enable notifications", false, |val| {
    state.notifications = val;
})
```

## Accessibility

- ARIA role: `switch`
- Keyboard: toggles on Space/Enter
