# Slider

`cvkg_components::Slider`

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Slider::new(value: f32, range: RangeInclusive<f32>, on_change: impl Fn(f32) + Send + Sync + 'static) -> Slider
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.step(f32)` | `f32` | `None` | Step increment for discrete values |

## Example

```rust
Slider::new(0.5, 0.0..=1.0, |val| state.volume = val)
    .step(0.1)
```

## Accessibility

- ARIA role: `slider`
- Sets `aria-valuemin`, `aria-valuemax`, `aria-valuenow`
