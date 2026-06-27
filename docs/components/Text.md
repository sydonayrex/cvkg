# Text

`cvkg_components::Text`

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Text::new(content: impl Into<String>) -> Text
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.font_size(f32)` | `f32` | `14.0` | Font size in logical pixels |
| `.color([f32; 4])` | `[f32; 4]` | `[1,1,1,1]` | RGBA color array |
| `.theme_color(&str)` | `&str` | - | Use a theme token by name |
| `.font_weight(FontWeight)` | `FontWeight` | `Regular` | Font weight (Regular, Bold, Italic) |
| `.bold()` | - | - | Shorthand for `.font_weight(FontWeight::Bold)` |

## Example

```rust
Text::new("Hello, World!")
    .font_size(24.0)
    .color([0.0, 0.8, 1.0, 1.0])
    .bold()
```
