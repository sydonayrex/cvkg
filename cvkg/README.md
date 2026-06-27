# cvkg

Umbrella facade crate for the CVKG framework. Selects the native or web backend based on feature flags.

## Purpose

Re-exports the core CVKG surface so consumers depend on a single crate. Feature flags select the rendering backend.

## Boundaries

This crate does NOT implement any rendering, layout, logic, or components. It is a facade that delegates to workspace crates.

## Dependency Graph

```mermaid
graph TD
    cvkg["cvkg<br/>(Umbrella facade)"]
    cvkg --> cvkg-core
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
    berserker --> cvkg
    berserker --> cvkg-core
    berserker --> cvkg-physics
    berserker --> cvkg-anim
    berserker --> cvkg-components
    berserker --> cvkg-themes
    berserker --> cvkg-vdom
    adele-web --> cvkg-core
    adele-web --> cvkg-render-gpu
    adele-web --> cvkg-components
    adele-web --> cvkg-themes
    adele-web --> cvkg-vdom
    adele-web --> cvkg-layout

    classDef entry fill:#064e3b,stroke:#10b981,color:#a7f3d0,stroke-width:2px
    classDef core fill:#1a1a2e,stroke:#1e293b,color:#e2e8f0,stroke-width:1px
    classDef ui fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    classDef meta fill:#3f3f46,stroke:#a1a1aa,color:#d4d4d8,stroke-width:1px
    class cvkg entry
    class cvkg-core,cvkg-scene,cvkg-vdom core
    class cvkg-layout,cvkg-themes,cvkg-anim ui
    class cvkg-macros,cvkg-components meta
```

## Public API

The crate re-exports key types from its dependencies:

- `cvkg-core`: `View`, `Renderer`, `State`, `Binding`, `Color`, `Rect`, `KvasirId`
- `cvkg-layout`: `HStack`, `VStack`, `ZStack`, `Grid`, `TaffyLayoutEngine`
- `cvkg-anim`: `SpringParams`, `SpringSolver`, `RubberBand`
- `cvkg-components`: Full widget library
- `cvkg-themes`: `Theme`, `OklchColor`, `ThemeBuilder`

Examples: `berserker_fire_demo`, `hit_test_demo`, `shatter_demo`, `physics_3d_demo`, `declarative_dashboard`, `interactive_daw`, `daw_perf`.

## Usage

```toml
# Cargo.toml
[dependencies]
cvkg = { path = "../cvkg", features = ["native"] }
```

```rust
use cvkg::prelude::*;
use cvkg::{Color, HStack, VStack, State, View};

struct App;

impl View for App {
    type Body = VStack;
    fn body(self) -> Self::Body {
        VStack::new()
            .child(Color::RED)
            .child(Color::BLUE)
    }
}
```

## Feature Flags

| Flag | Effect |
|---|---|
| `native` | Enables `cvkg-render-native` backend |
| `gpu` | Enables `cvkg-render-gpu` |
| `web` | Enables WebGPU backend via `cvkg-render-gpu` |

## Use Cases

- Top-level application depends on `cvkg` with `features = ["native"]`.
- Library crates depend on specific sub-crates (`cvkg-core`, `cvkg-layout`, etc.) rather than the umbrella.

## Edge Cases

- Enabling both `native` and `web` simultaneously may cause conflicts. Choose one backend.
- The `gpu` and `web` features both enable `cvkg-render-gpu`. The difference is the backend integration layer.
