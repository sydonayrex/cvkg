# berserker

![Berserker Hero HUD](../docs/images/berserker_hero_hud.png)

**berserker** is the gaming/UI application built with CVKG framework featuring the Cyber Viking aesthetic.

## 🚀 Quick Start

```bash
# Run the Berserker application
cd berserker
cargo run

# Or use the CVKG CLI
cvkg run --target berserker
```

## 🎮 Features

| Feature | Description |
|---------|-------------|
| **Cyber Viking UI** | Immersive gaming interface with Norse mythology themes |
| **Real-time Effects** | Mjöllnir lightning, fire, shatter animations |
| **HUD System** | Runic text display and performance monitoring |
| **Interactive Demo** | Hit-test demo for HID validation |
| **Asset Pipeline** | Integrated theme and asset management |

## 📚 Examples

### Fire Demo

```rust
// Run the fire demo
cargo run --example berserker_fire_demo
```

### Shatter Demo

```rust
// Run the shatter demo
cargo run --example shatter_demo
```

### Hit Test Demo

```rust
// Run the hit test demo for HID validation
cargo run --example hit_test_demo
```

## 🛠️ Configuration

### Themes

Located in `themes/default.rs`:

```rust
pub struct BerserkerTheme {
    pub fire_colors: [f32; 4],
    pub ice_colors: [f32; 4],
    pub lightning_colors: [f32; 4],
}
```

### Assets

Place assets in the `assets/` directory:
- Textures
- Models
- Sounds
- Fonts

## 🎨 Customization

### Colors

```rust
use berserker::themes::BerserkerTheme;

let theme = BerserkerTheme {
    fire: [1.0, 0.3, 0.0, 1.0],    // Orange-red
    ice: [0.0, 0.8, 1.0, 1.0],     // Cyan
    lightning: [0.8, 0.8, 1.0, 1.0], // Light blue
};
```

## 📖 Related Documentation

- [cvkg-components](../cvkg-components/README.md) - UI components
- [cvkg-render-gpu](../cvkg-render-gpu/README.md) - GPU renderer
- [cvkg-cli](../cvkg-cli/README.md) - CLI tool
- [Main CVKG README](../README.md) - Project overview

## 📜 License

MIT License - see [LICENSE-MIT](../LICENSE-MIT)