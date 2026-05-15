# Architecture Overview

CVKG is a modular, high-performance UI framework built on the "Cyber-Viking" design philosophy. It separates logic, layout, and rendering into distinct stages to ensure maximum efficiency on both native and web targets.

## The Rendering Pipeline

The transformation from code to pixels follows a five-stage "Forge" process:

### 1. The Body (Composition)
You define UI using the `View` trait. This is a declarative stage where views are composed hierarchically.
- **Primary Crate**: `cvkg-core`
- **Mechanism**: The `body()` method returns a type-erased `impl View` tree.

### 2. The VDOM (Reconciliation)
The view tree is captured into a stateless Virtual DOM. This stage handles state diffing and event mapping.
- **Primary Crate**: `cvkg-vdom`
- **Output**: A `VNode` tree and a set of `VDomPatch` mutations.

### 3. The Layout (Spatial Distribution)
The layout engine computes absolute bounds for every node based on flexbox-inspired rules.
- **Primary Crate**: `cvkg-layout`
- **Mechanism**: `HStack`, `VStack`, and `Grid` containers distribute space and resolve proposals.

### 4. The Scene Graph (Retained Geometry)
Retained geometry is stored in a scene graph for efficient temporal updates and spatial queries.
- **Primary Crate**: `cvkg-scene`
- **Optimization**: Hierarchical AABB culling and dirty-region tracking.

### 5. Surtr (GPU Rasterization)
The final stage tessellates geometry and submits commands to the GPU.
- **Primary Crate**: `cvkg-render-gpu`
- **Mechanism**: `wgpu` pipelines, Muspelheim multi-pass effects, and Mega-Atlas batching.

## State Management

CVKG uses a reactive `Binding` system:
- **State**: The authoritative source of truth.
- **Binding**: A read/write reference to state that triggers a redraw on mutation.
- **Environment**: Dependency injection for global resources like themes and asset managers.

## Modular Crate Map

| Tier | Crates | Responsibility |
| :--- | :--- | :--- |
| **Logic** | `cvkg-core`, `cvkg-macros` | Traits, state, and DSL. |
| **UI** | `cvkg-components`, `cvkg-flow` | Widgets and node-graph tools. |
| **Geometry** | `cvkg-layout`, `cvkg-scene`, `cvkg-anim` | Positioning and motion. |
| **Platform** | `cvkg-render-native`, `cvkg-render-web` | OS/Browser integration. |
| **Backend** | `cvkg-render-gpu`, `cvkg-runic-text` | Shaders and typography. |

## Design Philosophy: "Berserker"

The architecture is built for speed and visual "rage":
- **Deterministic**: Frame timings are strictly monitored for jitter.
- **Retained**: Only changed regions are updated.
- **Shader-First**: Effects like Bifrost (frosted glass) are first-class primitives, not expensive post-processes.