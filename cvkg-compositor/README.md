# cvkg-compositor

`cvkg-compositor` is a high-performance, node-based visual compositor crate for the Computer Vision Knowledge Graph (CVKG) ecosystem. It provides the core engine for composing complex scenes, layouts, and node graphs efficiently.

```mermaid
graph TD
    cvkg-core["cvkg-core"]
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene["cvkg-scene"]
    cvkg-layout["cvkg-layout"]
    cvkg-render-gpu["cvkg-render-gpu"]
    cvkg-render-native["cvkg-render-native"]
    cvkg-compositor["cvkg-compositor"]
    cvkg-themes["cvkg-themes"]
    cvkg-anim["cvkg-anim"]
    cvkg-flow["cvkg-flow"]
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-svg-filters["cvkg-svg-filters"]
    cvkg-svg-serialize["cvkg-svg-serialize"]
    cvkg-components["cvkg-components"]
    cvkg-macros["cvkg-macros"]
    cvkg-cli["cvkg-cli"]
    cvkg-webkit-server["cvkg-webkit-server"]
    cvkg-test["cvkg-test"]
    cvkg-physics["cvkg-physics"]
    cvkg["cvkg (umbrella)"]

    cvkg-vdom --> cvkg-core
    cvkg-vdom --> cvkg-scene
    cvkg-layout --> cvkg-core
    cvkg-layout --> cvkg-anim
    cvkg-scene --> cvkg-core

    cvkg-render-gpu --> cvkg-core
    cvkg-render-gpu --> cvkg-compositor
    cvkg-render-gpu --> cvkg-svg-filters
    cvkg-render-gpu --> cvkg-svg-serialize
    cvkg-render-gpu --> cvkg-runic-text

    cvkg-render-native --> cvkg-core
    cvkg-render-native --> cvkg-render-gpu
    cvkg-render-native --> cvkg-vdom
    cvkg-render-native --> cvkg-themes

    cvkg-compositor --> cvkg-core

    cvkg-themes --> cvkg-core
    cvkg-themes --> cvkg-anim
    cvkg-anim --> cvkg-core
    cvkg-flow --> cvkg-core
    cvkg-flow --> cvkg-scene
    cvkg-flow --> cvkg-themes

    cvkg-runic-text --> cvkg-core
    cvkg-svg-filters --> cvkg-core

    cvkg-components --> cvkg-core
    cvkg-components --> cvkg-vdom
    cvkg-components --> cvkg-layout
    cvkg-components --> cvkg-themes
    cvkg-components --> cvkg-anim
    cvkg-components --> cvkg-runic-text

    cvkg-macros --> cvkg-core
    cvkg-cli --> cvkg-core
    cvkg-cli --> cvkg-physics
    cvkg-cli --> cvkg-anim
    cvkg-cli --> cvkg-macros
    cvkg-webkit-server --> cvkg-cli
    cvkg-physics --> cvkg-core
    cvkg-physics --> cvkg-scene

    cvkg --> cvkg-core
    cvkg --> cvkg-vdom
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
```

## Features

- **Advanced Node Compositing**: Combine, transform, and layer diverse visual elements.
- **Continuous Proximity & Kinematic Fields (Vili Paradigm)**: The UI is driven by continuous proximity fields and dynamic attention scaling rather than static rectangular hitboxes.
- **Hardware Acceleration Ready**: Architected to plug seamlessly into GPU-accelerated rendering pipelines (`cvkg-render-gpu`).
- **Reactive State Management**: Uses a robust virtual DOM (`cvkg-vdom`) under the hood to efficiently track diffs and apply layout patches.

## Integration

`cvkg-compositor` integrates seamlessly with:
- `cvkg-flow` for graph-based node visualization and wiring.
- `cvkg-components` for the UI shell, docking panels, and interactive property inspectors.
- `cvkg-scene` to inject composite results into the final 2D/3D render context.

## Usage

Add `cvkg-compositor` to your `Cargo.toml`:

```toml
[dependencies]
cvkg-compositor = { path = "../cvkg-compositor", version = "0.1.21" }
```

Initialize the compositor within your primary app configuration, leveraging the Vili Interaction Paradigm for fluid, dynamic pointer interaction and kinematics.

## License

MIT / Apache-2.0
