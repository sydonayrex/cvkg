# cvkg-components

![CVKG Hero HUD](../docs/images/cvkg_hero.png)

`cvkg-components` is the standard library for the CVKG framework, providing a vast collection of atomic primitives and complex "Berserker" widgets.

## Boundaries and Responsibilities

This crate implements concrete UI elements using the `View` trait. It does NOT handle low-level rendering (delegated to `cvkg-core` and the active backend). Its responsibilities include:
- Providing atomic building blocks (`Button`, `Toggle`, `Input`, `Slider`).
- Implementing complex, stateful widgets like `RunestoneEditor`, `MimirSpotlight`, and `OracleOrb`.
- Managing layout containers (`VStack`, `HStack`, `ZStack`, `Grid`).
- Offering high-fidelity tactical HUD elements (`TacticalGauge`, `Vegvísir`, `WyrdHUD`).
- Exposing the `ViewExt` trait for sheet and modal presentation.

## Public API Overview

### Atomic Primitives
- `Button`, `Toggle`, `Slider`, `Input`: Standard interactive controls.
- `Text`, `Image`, `Shape`: Basic content display.
- `Progress`, `Gauge`, `StatusBar`: Visualization components.

### Tactical & Agentic Widgets
- `MimirSpotlight`: A keyboard-driven command palette.
- `OracleOrb`: An interactive, state-aware agentic status indicator.
- `RunestoneEditor`: A high-performance code and text editor.
- `VölvaScan`: Advanced diagnostic and telemetry visualization.

### Layout & Containers
- `NavigationStack`, `Tabview`, `ScrollView`: Structural navigation.
- `GjallarSplitter`: Resizable split views.
- `MjolnirFrame`: Specialized decorative and functional container frames.

## Usage Example

```rust
use cvkg_components::prelude::*;

fn MyComponent() -> impl View {
    VStack::new(10.0) {
        Text::new("Agentic Status: Active")
            .foregroundColor(Color::VIKING_GOLD);
            
        OracleOrb::new(AgentState::Thinking);
        
        Button::new("Execute Protocol", || {
            println!("Protocol Initialized");
        });
    }
}
```

## Known Limitations
- Complex widgets like `RunestoneEditor` require a GPU-accelerated backend for full feature support (e.g., syntax highlighting).
- Component state is typically managed via the `Binding` system defined in `cvkg-core`.