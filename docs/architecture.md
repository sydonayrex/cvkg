# Architecture Overview

This document describes the structure, data flow, and subsystems of the Cyber Viking Kvasir Graph (CVKG) framework.

---

## Crate Dependency Topology

The CVKG framework is divided into independent crates to separate compositional logic, platform adapters, and visual drawing pipelines. The dependency flow is strictly directed downward toward the core.

```mermaid
graph TD
    %% Subgraphs for visual structure and tiering
    subgraph Core ["Core Foundations"]
        cvkg-core["cvkg-core<br/>(Core traits, state, telemetry)"]
        cvkg-vdom["cvkg-vdom<br/>(Virtual DOM & diffing)"]
        cvkg-scene["cvkg-scene<br/>(Scene graph & spatial partitioning)"]
        cvkg-layout["cvkg-layout<br/>(Constraint layout & Taffy wrapper)"]
    end

    subgraph Graphics ["Graphics & Shaping Layer"]
        cvkg-render-gpu["cvkg-render-gpu<br/>(wgpu rendering engine)"]
        cvkg-compositor["cvkg-compositor<br/>(Compositor, layers, damage)"]
        cvkg-runic-text["cvkg-runic-text<br/>(Text shaping & BiDi engine)"]
        cvkg-svg-filters["cvkg-svg-filters<br/>(SVG filter effects)"]
        cvkg-svg-serialize["cvkg-svg-serialize<br/>(SVG serialization)"]
    end

    subgraph Platform ["Platform Integration"]
        cvkg-render-native["cvkg-render-native<br/>(Native backend, windowing)"]
    end

    subgraph Presentation ["UI & Interaction Layer"]
        cvkg-themes["cvkg-themes<br/>(OKLCH color, premium materials)"]
        cvkg-anim["cvkg-anim<br/>(Spring dynamics, particles)"]
        cvkg-flow["cvkg-flow<br/>(Visual node graph engine)"]
        cvkg-components["cvkg-components<br/>(Tahoe component library)"]
    end

    subgraph Infra ["Infrastructure & Tooling"]
        cvkg-physics["cvkg-physics<br/>(XPBD physics solver)"]
        cvkg-macros["cvkg-macros<br/>(hamr! DSL macro)"]
        cvkg-cli["cvkg-cli<br/>(Dev Server & asset pipeline)"]
        cvkg-webkit-server["cvkg-webkit-server<br/>(axum HTTP/WS server)"]
        cvkg-test["cvkg-test<br/>(Visual regression comparator)"]
    end

    subgraph Entry ["Umbrella Crate"]
        cvkg["cvkg<br/>(Top-level umbrella)"]
    end

    %% Dependency Connections
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

    %% Visual Styling Classes for Premium Design (High-Contrast, Harmonious)
    classDef core fill:#1a1a2e,stroke:#1e293b,color:#e2e8f0,stroke-width:1px
    classDef render fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:1.5px
    classDef ui fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    classDef infra fill:#1c1917,stroke:#78716c,color:#d6d3d1,stroke-width:1px
    classDef platform fill:#1e1b4b,stroke:#6366f1,color:#a5b4fc,stroke-width:1.5px
    classDef umbrella fill:#064e3b,stroke:#10b981,color:#a7f3d0,stroke-width:2px

    class cvkg-core,cvkg-vdom,cvkg-scene,cvkg-layout core
    class cvkg-render-gpu,cvkg-compositor,cvkg-runic-text,cvkg-svg-filters,cvkg-svg-serialize render
    class cvkg-render-native platform
    class cvkg-components,cvkg-themes,cvkg-anim,cvkg-flow ui
    class cvkg-macros,cvkg-cli,cvkg-webkit-server,cvkg-test,cvkg-physics infra
    class cvkg umbrella
```


---

## Subsystems Reference

The framework isolates functional responsibilities into distinct modules. Below is a description of the key types and traits that define each major subsystem.

### 1. View Composition & Reactivity
Defines the declarative building blocks of the interface and manages data bindings.
- **Primary Crate**: `cvkg-core`
- **Key Traits**:
  - `View` — The primary unit of UI composition. Every component implements this trait and returns a hierarchical tree via its `body()` method.
  - `Renderer` — The drawing interface exposed to primitive views for geometric commands.
  - `ViewModifier` — Trait for extending views with styling, positioning, or custom shader effects.
- **Key Structs**:
  - `Binding<T>` — Read/write pointer to a state location that triggers view updates on modification.
  - `State<T>` — Authoritative storage container for reactive variables.
  - `YggdrasilTokens` — Container for global styling parameters.

### 2. State Reconciliation
Manages UI tree transformations, tracking nodes and calculating visual updates.
- **Primary Crate**: `cvkg-vdom`
- **Key Structs**:
  - `VNode` — A stateless representation of a view node including properties, geometry, and event handler keys.
  - `VDom` — The hierarchical Virtual DOM tree.
  - `VDomPatch` — Mutations (Create, Update, Delete, Move) computed by tree diffing.
  - `VNodeRenderer` — Driver that evaluates a composed `View` tree to populate a logical `VDom`.

### 2.5. Layer Compositor
Retained-mode layer orchestration engine that routes draw calls into multi-pass GPU buckets.
- **Primary Crate**: `cvkg-compositor`
- **Key Structs**:
  - `CompositorEngine` — Evaluates damage tracking and routes layer rendering by material type.
  - `LayerTree` — Z-sorted hierarchy of visual layers.
  - `Material` — Determines GPU passes (e.g., Scene, Glass, Overlay).

### 3. Layout Engine
Calculates spatial positions and dimensions.
- **Primary Crate**: `cvkg-layout`
- **Key Structs**:
  - `SizeProposal` — Dimensions proposed by a parent container to its children.
  - `Size` — Bounding dimensions resolved by a child view.
  - `HStack` / `VStack` / `Grid` — Primary layout containers that distribute viewport coordinates.

### 4. Retained Scene Graph
Accelerates frame rendering, culling, and interactive hit testing.
- **Primary Crate**: `cvkg-scene`
- **Key Structs**:
  - `SceneGraph` — Retained visual tree holding pre-tessellated geometries.
  - `SceneNode` — Individual spatial node containing absolute canvas bounds.

### 5. Animation Solver
Calculates motion transitions using mathematical solvers.
- **Primary Crate**: `cvkg-anim`
- **Key Structs**:
  - `SleipnirSolver` — A fourth-order Runge-Kutta (RK4) numerical integrator for spring physics.
  - `SleipnirParams` — Configuration storing mass, stiffness, and damping coefficients.
  - `RubberBand` — Logarithmic boundary resistance solver for scrolling or dragging overflow.

### 6. Text Shaping & Layout
Translates unicode characters into positioned, renderable glyph instances.
- **Primary Crate**: `cvkg-runic-text`
- **Key Structs**:
  - `RunicTextEngine` — High-performance shaper wrapping HarfBuzz and Swash rasterizers.
  - `ShapedText` — A fully wrapped, positioned set of glyph outputs.
  - `GlyphInstance` — Position offset and font index mapping for a single character glyph.

### 7. Graphics Pipeline (Surtr)
Draws geometric meshes and compiles GPU rendering programs.
- **Primary Crate**: `cvkg-render-gpu`
- **Key Structs**:
  - `SurtrRenderer` — WGPU pipeline manager driving command buffers, multi-pass filters, and render targets.
  - `Vertex` — Vertex description including position coordinates, color vectors, and effect parameters.
  - `DrawCall` — GPU execution request batched by texture and transparency level.

---

## Design Decisions

Several architectural choices separate CVKG from conventional UI systems.

1. **Separation of VDOM (`cvkg-vdom`) and Scene Graph (`cvkg-scene`)**:
   Reconciliation operates on logical state diffs to determine which views have mutated. However, redrawing, hierarchical AABB culling, and sub-pixel event hit-testing require spatial indexing. Keeping these two structures distinct prevents state management code from polluting hardware-accelerated spatial calculations.
2. **Spring Physics Solver Over Pre-baked Splines**:
   Animation in CVKG is computed iteratively at runtime using `SleipnirSolver`'s RK4 integration. Pre-baked cubic Bezier curves cannot handle mid-motion interruptions gracefully. Spring solvers allow animations to change targets instantly while retaining velocity, eliminating visual hitching.
3. **Stand-alone Text shaping (`cvkg-runic-text`)**:
   Text shaping is computationally heavy and interacts with volatile operating system font databases. Isolate this complexity to prevent OS font-linking quirks and external library compilation cycles from impacting core framework compilation speed.
4. **Vili Interaction Paradigm**:
   Standard UI frameworks rely on discrete rectangular hitboxes (AABB) for mouse interactions. CVKG instead relies on continuous mathematical fields—evaluating exact distance metrics using Signed Distance Fields (SDFs). This approach unlocks rich, dynamic feedback where pointer velocity (`mimir_intent`), proximity (`mani_glow`), and layout adaptation (`fafnir_evolve`) directly influence visual elements before a click ever occurs.
5. **AgX Tonemapping in Shader Pipelines**:
   Standard color space mapping models cause color distortion (hue shifting/saturation loss) in bright highlights. CVKG incorporates logarithmic color conversions and cubic contrast curves to map wide-dynamic-range HDR spaces into sRGB target viewports, keeping color fidelity intact.
6. **Render Graph Execution Plan Cache**:
   Topological sorting of multi-pass pipelines using Kahn's algorithm is CPU-intensive. Since UI node connections and draw orders do not change on every frame, CVKG caches compiled `PassId`/`NodeKey` sequences in `CachedGraphPlan` structures, bypassing the sorting stage.
7. **Temporal Sub-Pixel Snapping**:
   Traditional UI toolkits snap element coordinates to integer pixels on every layout pass, creating visual jitter during slow spring animations. CVKG queries mass/spring velocities from `cvkg-anim` during layout passes to allow sub-pixel rendering on active motion, snapping to the physical pixel grid only when movement stops.

---

## Out of Scope

The CVKG project is focused strictly on highly interactive, custom graphic user interfaces. The following items are out of scope:

- **HTML/CSS Browser Emulation**: CVKG does not parse CSS stylesheets or compile standard HTML structures. It is a direct GPU UI engine.
- **Database Management & SQL Systems**: Crate libraries do not provide ORM layers or SQL drivers. Developers leverage standard database connectors.
- **Operating System Control Wrappers**: The engine does not hook into platform-native widgets (like Cocoa NSView or Win32 Buttons). To guarantee visual identity across targets, all widgets are drawn procedurally from scratch on the GPU canvas.