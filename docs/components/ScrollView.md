# ScrollView

`cvkg_components::ScrollView<V>` -- scrollable container

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
ScrollView::new(content: V) -> ScrollView<V>
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.scroll_id(u64)` | `u64` | `0` | Unique scroll state identifier |
| `.content_size(f32, f32)` | `(f32, f32)` | `(0, 0)` | Explicit content size |
| `.scroll_speed(f32)` | `f32` | `1.0` | Scroll speed multiplier |
| `.momentum_decay(f32)` | `f32` | `0.95` | Momentum decay factor |
| `.scrollbar_width(f32)` | `f32` | `8.0` | Scrollbar width in pixels |
| `.scrollbar_fade_delay(u32)` | `u32` | `60` | Frames before scrollbar fades |
| `.scrollbar_fade_speed(f32)` | `f32` | `0.1` | Scrollbar fade speed |

## Example

```rust
ScrollView::new(
    VStack::new(8.0)
        .child(Text::new("Item 1"))
        .child(Text::new("Item 2"))
        // ... many items
)
.scroll_id(42)
.scrollbar_width(6.0)
```
