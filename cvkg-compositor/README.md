# cvkg-compositor

`cvkg-compositor` is a high-performance, node-based visual compositor crate for the Computer Vision Knowledge Graph (CVKG) ecosystem. It provides the core engine for composing complex scenes, layouts, and node graphs efficiently.

![Compositor Engine Capabilities](/home/nelson/.gemini/antigravity/brain/bf1b2c15-7683-4950-8fe1-c14440a259c8/compositor_engine_diagram_1779592343685.png)

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
