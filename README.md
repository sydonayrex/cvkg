# CVKG: Cyberpunk Viking Knowledge Graph

![CVKG Banner](https://raw.githubusercontent.com/sydonayrex/cvkg/main/docs/static/banner.png)

**CVKG** is a high-fidelity, agentic UI framework for Rust. It is designed to empower developers to build immersive, high-performance interfaces with a "Cyberpunk Viking" aesthetic—combining the raw power of the Norse sagas with the neon-drenched precision of a digital future.

## ⚡ The CVKG Edge

- **Surtr GPU Pipeline**: A high-performance WGPU-based rendering engine featuring the **Muspelheim** multi-pass bloom and **Niflheim** shader suite.
- **Sleipnir Physics**: Native RK4 spring physics for smooth, organic motion that feels alive.
- **Yggdrasil Design System**: A centralized token system that propagates tactical "Cyberpunk Viking" themes across all components.
- **ShieldWall Accessibility**: Native integration with AccessKit to ensure that even the most complex GPU-driven interfaces remain fully accessible.
- **Cross-Platform VDOM**: A high-performance virtual DOM implementation with WebGL/WebGPU fallback for browsers.

## 📦 Workspace Architecture

CVKG is a highly modular ecosystem consisting of 14 specialized crates:

| Crate | Role | Description |
|---|---|---|
| `cvkg` | Facade | The main umbrella crate for the framework. |
| `cvkg-core` | Foundation | Core traits, state graph, and environment tokens. |
| `cvkg-scene` | Engine | Retained scene graph and diffing logic. |
| `cvkg-render-gpu` | Visuals | The **Surtr** high-performance WGPU backend. |
| `cvkg-components` | UI | A library of premium primitive and interactive views. |
| `cvkg-anim` | Motion | The **Sleipnir** spring physics animation solver. |
| `cvkg-layout` | Geometry | Flexbox-inspired layout engine. |
| `cvkg-themes` | Aesthetic | The **Yggdrasil** design system and tactical themes. |
| `cvkg-cli` | Tooling | CLI for scaffolding and managing CVKG projects. |

## 🚀 Quick Start

Add CVKG to your `Cargo.toml`:

```toml
[dependencies]
cvkg = "0.1.0"
```

Create a high-fidelity button with spring physics:

```rust
use cvkg::prelude::*;

fn body() -> impl View {
    VStack::new((
        Text::new("Enter the Void")
            .font_size(24.0)
            .color(Color::CYAN)
            .glow(8.0),
        Button::new("Ascend")
            .on_click(|| println!("Valhalla awaits!"))
            .modifier(Mjolnir::slice(45.0))
    ))
    .padding(20.0)
    .background(Color::VOID)
}
```

## 📖 Documentation

Explore the **CVKG Saga** in our interactive manual:

- [The Interface Atlas](docs/src/SUMMARY.md)
- [Visual Engine Deep-Dive](docs/src/visual/cvkg-render-gpu.md)
- [Core Ecosystem Guide](docs/src/core/README.md)

## 🛡️ License

Licensed under [MIT license](LICENSE-MIT).

---

*Build your saga. Conquer the interface.*
