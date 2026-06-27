# Progress

`cvkg_components::Progress` (English alias for `SkollProgress`)

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Progress::new(value: f32) -> Progress
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.variant(ProgressVariant)` | `ProgressVariant` | `Linear` | Visual variant (Linear, Segmented) |
| `.height(f32)` | `f32` | `8.0` | Track height in logical pixels |
| `.fill([f32; 4])` | `[f32; 4]` | theme accent | Override fill color |

## Example

```rust
Progress::new(0.7)
    .variant(ProgressVariant::Linear)
    .height(8.0)
```

## Accessibility

- ARIA role: `progressbar`
- Sets `aria-valuenow` to the current value
