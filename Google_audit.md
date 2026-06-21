# CVKG Codebase Engineering Audit Report — Google Flash Audit

Generated: 2026-06-21

## Step 0 — Orientation

### cvkg-vdom
- **Responsibility**: Manages the Virtual DOM layer: VDOM node types, accessibility tree construction, event propagation, diffing, and the bridge between the View system and the renderer.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-scene`, `cvkg-runic-text`

### cvkg-layout
- **Responsibility**: Computes spatial bounds and flexbox positioning using Taffy constraints, handles grid, container queries, aspect ratio, safe area, animations, reading/focus order, and progressive batch layout.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-anim`

### cvkg-scheduler
- **Responsibility**: Synchronizes frame updates, layout passes, and GPU drawing tasks via priority queues and phased clocks (FramePhase).
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-physics
- **Responsibility**: Dynamic 3D/2D rigid body simulation, including impulse-based XPBD constraint solvers, GJK/EPA narrowphase contact manifolds, SPH fluid, colliders, broadphase spatial hash grids, character controllers, and a scene graph sync bridge.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-scene`

### cvkg-flow
- **Responsibility**: Canvas grid node-graph drawing engine, connection port management, bezier edge routing, spatial indexing, and visual flow charts.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-scene`, `cvkg-themes`

### cvkg-spatial
- **Responsibility**: Shared spatial indexing data structures (QuadTree, BVH, SpatialHash) used platform-wide to accelerate 2D collision, picking, and region queries.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-runic-text
- **Responsibility**: Text shaping, BiDi formatting, layout wrapping (using Knuth-Plass), subpixel LCD coverage, MSDF generation, and font resource discovery.
- **Intra-workspace dependencies**: None

### cvkg-anim
- **Responsibility**: Spring-physics motion transitions (RK4 solvers), scroll rubber banding, L-system growth procedural animations, Voronoi fracturing destruction animations, and skeletal/particle systems.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-scene
- **Responsibility**: Retained scene graph with spatial partitioning (QuadTree/BVH) for accelerated AABB culling, automatic layering, and dirty-rect tracking for the Surtr GPU pipeline.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-spatial`, `cvkg-runic-text`

### cvkg-themes
- **Responsibility**: OKLCH-based system token catalog managing color palettes, contrast ratio validation, and premium design materials.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-anim`

### cvkg-materials
- **Responsibility**: Canonical data structures defining config settings for Glass, Mica, Acrylic, and Elevation effects, avoiding circular dependency paths between backends and frontends.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-accessibility
- **Responsibility**: Unified tree management, keyboard tab order, directional focus navigation, and screen reader integration.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-reflect
- **Responsibility**: Explicit opt-in runtime type schema reflection, field metadata registry, and dynamic JSON property access.
- **Intra-workspace dependencies**: None

### cvkg-certification
- **Responsibility**: Grouped cross-crate certification suites verifying integrated flows like Scene → Layout → Render.
- **Intra-workspace dependencies**: None

### cvkg-macros
- **Responsibility**: Procedural macros automating state derivation, View generation, and declarative UI layout via `hamr!`.
- **Intra-workspace dependencies**: None

### cvkg-render-gpu
- **Responsibility**: Main wgpu-based rendering pipelines, shader compilations, command buffers, and render graph nodes.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-compositor`, `cvkg-svg-serialize`, `cvkg-svg-filters`, `cvkg-runic-text`

### cvkg-core
- **Responsibility**: Fundamental traits, shared data structures, state management types, and layout primitives for CVKG.
- **Intra-workspace dependencies**: `cvkg-macros`

### cvkg-render-native
- **Responsibility**: Winit/AccessKit native desktop application windowing wrapper and OS audio/clipboard interfaces.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-render-gpu`, `cvkg-vdom`, `cvkg-themes`, `cvkg-runic-text`

### cvkg-render-software
- **Responsibility**: Analytical CPU software-based fallback rasterizer/renderer supporting basic shape rendering and text.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-runic-text`

### cvkg-compositor
- **Responsibility**: Layer composition, flattening layer trees to command list streams, and offscreen render commands routing.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-svg-filters
- **Responsibility**: WGPU-based SVG filter node parsing, DAG building, and render/compute pass pipeline evaluator.
- **Intra-workspace dependencies**: None

### cvkg-svg-serialize
- **Responsibility**: SVG document serialization and writing logic using quick-xml and usvg.
- **Intra-workspace dependencies**: None

### cvkg-telemetry
- **Responsibility**: Crate metrics, profiling events, and structured performance tracing logging wrappers.
- **Intra-workspace dependencies**: `cvkg-core`

### cvkg-icons
- **Responsibility**: Pre-compiled custom asset catalog of premium SVG icons.
- **Intra-workspace dependencies**: None

### cvkg-test
- **Responsibility**: Integration testing harness and accessibility conformance suites.
- **Intra-workspace dependencies**: None

### cvkg-webkit-server
- **Responsibility**: Developer server orchestrating universal WASM build processes, HMR file watching, and headless webviews.
- **Intra-workspace dependencies**: `cvkg-cli`

### cvkg-cli
- **Responsibility**: Developer CLI tools for package builds, dev server relays, asset piping, and runtime connection inspectors.
- **Intra-workspace dependencies**: None

### cvkg-components
- **Responsibility**: Extensive collection of premade UI components (agent chat, infinite canvases, HUD dashboard widgets, input fields).
- **Intra-workspace dependencies**: None

### cvkg
- **Responsibility**: High-level umbrella re-export wrapper package resolving exclusive backend pipeline features.
- **Intra-workspace dependencies**: `cvkg-core`, `cvkg-scene`, `cvkg-layout`, `cvkg-anim`, `cvkg-themes`, `cvkg-macros`, `cvkg-components`, `cvkg-render-gpu`, `cvkg-render-native`

### demos/adele-web
- **Responsibility**: Web/WASM application deployment demo.
- **Intra-workspace dependencies**: `cvkg`

### demos/niflheim-wasi
- **Responsibility**: Headless/WASI UI automation certification demo.
- **Intra-workspace dependencies**: `cvkg`

### demos/berserker-fire-web
- **Responsibility**: Advanced web GPU simulation demo with fire effect overlays.
- **Intra-workspace dependencies**: `cvkg`

### demos/berserker
- **Responsibility**: High-fidelity desktop application rendering demo showcasing real-time UI/UX capabilities.
- **Intra-workspace dependencies**: `cvkg`

---

## Step 1 — Per-File Checklist

### File: `cvkg-vdom/src/lib.rs`

**Line count**: 2341 lines

#### 1. Bug Identification & Debugging
- **Bubble Event Response (line 2009)**: If no handler is found walking up from the target, the dispatcher searches descendants for a handler. If a descendant handler is found and executed, `processed` is set to `true`, and it breaks. However, this does not perform bubbling *up* from that descendant to its ancestors. This represents a mismatched event propagation policy where fallback descendant handlers execute in isolation, bypassing ancestral capturing/bubbling.
- **RwLock Poisoning in State Hook (line 2319)**: In `use_state`, calling `arc_val.read().unwrap()` will panic if the RwLock has been poisoned by a panic in another thread. While unlikely, it results in a latent panic.

#### 2. Security-Minded Checks
- **External Data Injection & Resource Exhaustion (line 454)**: `VDomPatch` implements `Deserialize` and accepts key maps/lists (like `handlers` and `children`). If deserializing untrusted patches from IPC/network (e.g. from `cvkg-webkit-server`), an attacker could provide an arbitrarily large array of keys/ids to exhaust server memory (DoS).
- **No hardcoded secrets, path traversals, or unsafety** present in this file.

#### 3. Monolithic File Decomposition
**y — This file is 2341 lines long and contains multiple mixed responsibilities.**
- **Proposed split**:
  - `src/vnode.rs`: `LayoutRect`, `A11yNodeEntry`, `AriaProps`, `VNode`, `DecorativeCmd`
  - `src/patch.rs`: `VDomPatch` (with its Serialize/Deserialize implementations)
  - `src/diff.rs`: `VDom::diff`, `VDom::diff_node`, `VDom::calculate_lis`
  - `src/hit_test.rs`: `VDom::hit_test`, `VDom::hit_test_recursive`, `VDom::sdf_distance`
  - `src/events.rs`: `VDom::dispatch_event`, `VDom::dispatch_event_to_target`, `VDom::bubble_event`, `VDom::bubble_event_response`, `VDom::focus_node`, `VDom::blur_node`, `VDom::build_focus_order`, `VDom::dfs_pre_order`, `VDom::focus_next`, `VDom::focus_prev`
  - `src/accesskit_bridge.rs`: `VDom::build_accesskit_tree`, `VDom::build_accesskit_node`, `VNode::to_accesskit_node`
  - `src/renderer.rs`: `VNodeRenderer` struct and its implementation of `cvkg_core::Renderer`
  - `src/state.rs`: `use_state` hook
  - `src/lib.rs`: Module declarations and public re-exports.

#### 4. Theming
- **ShieldWall (Comment references only)**: Used in comments to describe the platform-specific accessibility tree integration layer.
  - **Proposed name**: `PlatformA11yTree` or `AccessKitBridge`.

#### 5. Unwrap/Unsafe Audit
- **Line 2319**: `arc_val.read().unwrap()`
  - **Risk**: Med (RwLock poisoning panic).
  - **Suggested Fix**: Use `.unwrap_or_else(|e| e.into_inner())` to recover the read guard from the poisoned lock.

---

### File: `cvkg-layout/src/lib.rs`

**Line count**: 2811 lines

#### 1. Bug Identification & Debugging
- **Layout cycle guard thread-local state leak on panic (lines 57 & 77)**: `with_layout_cycle_guard` inserts the node hash into `ACTIVE_LAYOUT_NODES` thread-local hashset and removes it at the end of the closure. If the layout closure `f` panics, the removal step is bypassed, leaving the hash in the active nodes set permanently. Subsequent non-panicking layout calls on the same thread will hit a false "cycle detected" warning and skip layout, using fallback dimensions.
- **Focus candidate tab_index partitioning logic (line 1779)**: In `compute_focus_order`, `candidates` partitions elements based on `tab_index.map_or(false, |t| t > 0)`. Negative tab indices (like `-1` representing focusable-by-script but not tab-traversable elements) are not correctly filtered out. Instead, they fall into the `natural` tab order and are included in the sequential keyboard focus traversal.

#### 2. Security-Minded Checks
- No untrusted inputs or unsafe blocks are present. All operations run using safe Rust collections and taffy bindings.

#### 3. Monolithic File Decomposition
**y — This file is 2811 lines long and contains multiple mixed responsibilities.**
- **Proposed split**:
  - `src/taffy_engine.rs`: `TaffyLayoutEngine`, Taffy helpers like `taffy_alignment`, `taffy_distribution`, `taffy_track`.
  - `src/animation.rs`: `AnimationEngine`, `apply_layout_animations`.
  - `src/primitives.rs`: `HStack`, `VStack`, `ZStack`, `Spacer`, `Flex`, `Padding`, `SafeArea`, `AspectRatio`, and flex helpers.
  - `src/grid.rs`: `GridTrack`, `Grid` struct.
  - `src/spatial_index.rs`: `LayoutSpatialEntry`, `LayoutSpatialIndex`, `QuadNode`.
  - `src/parallel.rs`: `size_views_parallel`.
  - `src/modality.rs`: `LayoutModality` enum.
  - `src/focus.rs`: `FocusCandidate`, `compute_focus_order`, `validate_reading_order`.
  - `src/progressive.rs`: `ProgressiveLayoutContext`, `ProgressiveChild`.
  - `src/lib.rs`: Re-exports, thread locals, cycle guards.

#### 4. Theming
- **FlexiScope (Comment references only)**: Used in comments to describe container queries.
  - **Proposed name**: `ContainerQuery` or `ContainerScope`.

#### 5. Unwrap/Unsafe Audit
- **Line 124-126, 163-165**: `.unwrap()` on downcasting `cache.engine` / `cache.animators`.
  - **Risk**: Low (Only panics if the generic engine slot was occupied by a different type; internal control prevents this).
- **Line 359, 364, 403, 411, 1062, 1067, 1096, 1104**: `.unwrap()` on taffy tree node creation/retrieval.
  - **Risk**: Med. Taffy nodes could return errors if capacity limits are breached or node is dangling, triggering a panic.
  - **Suggested Fix**: Replace with error propagation `?` or handle gracefully with default/empty rects.

---

### File: `cvkg-scheduler/src/frame.rs`
**Line count**: 482 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Exposed APIs are well structured and logically separated across `frame.rs` and `task.rs`).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-scheduler/src/task.rs`
**Line count**: 340 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Well-structured).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-physics/src/body.rs`
**Line count**: 421 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-physics/src/broadphase.rs`
**Line count**: 334 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-physics/src/world.rs`
**Line count**: 1172 lines

#### 1. Bug Identification & Debugging
- **Variable step vs Fixed step substeps (lines 275 & 290)**: In `step()`, if `fixed_timestep` is used, the time parameter passed to `step_substep` is static (`fixed_dt / substeps`). If variable timestep is used, it computes `sub_dt = dt / substeps as f32`. In the event of a rendering engine lag spike (causing a very large `dt` from the frame clock), `sub_dt` will suddenly explode, inducing numerical instability or GJK/EPA constraint solver explosion. 

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
**y — This file is 1172 lines long and contains multiple mixed responsibilities.**
- **Proposed split**:
  - `src/world/mod.rs`: `PhysicsWorld` main struct, re-exports.
  - `src/world/config.rs`: `WorldConfig`, `StepResult`, `CollisionEvent`.
  - `src/world/body_registry.rs`: `add_body`, `remove_body`, `body`, `body_mut` and helper map updates.
  - `src/world/step.rs`: `step`, `step_substep` loops and solvers integration.

#### 4. Theming
- **Tyr (Norse references in documentation only)**: Describes the rigid body physics engine.
  - **Proposed name**: `PhysicsEngine` or `RigidBodyEngine`.

#### 5. Unwrap/Unsafe Audit
- None found in core simulation paths.

---

### File: `cvkg-flow/src/node.rs`
**Line count**: 453 lines

#### 1. Bug Identification & Debugging
- **Opaque Glass Tint on Custom Color (line 164)**: `GlassNodeMaterial::with_tint_oklch` sets `self.tint = self.tint_oklch.to_rgba()`, which sets the alpha component of `tint` to 1.0. This overwrites the default translucent glass alpha of `0.15` (from `GlassNodeMaterial::default()`), rendering the glass background fully opaque.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-flow/src/edge.rs`
**Line count**: 411 lines

#### 1. Bug Identification & Debugging
- **Instantaneous Interaction Color Transition (line 252)**: `effective_color()` uses `self.animation_progress` to interpolate towards the hovered/selected target color. Since `animation_progress` is ticked only during the spawn animation phase and settles to `1.0`, any later change in `self.interaction` instantly snaps the color to the target, bypassing the color easing transitions described in comments.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-flow/src/canvas.rs`
**Line count**: 662 lines

#### 1. Bug Identification & Debugging
- **Arbitrary Overlapping Node Hit-Test (line 314)**: `node_at_screen` searches `self.graph.nodes` (a `HashMap`) using `.find()`, returning the first matching node in iteration order. Under overlapping node conditions, this ignores the `z_index` property of `FlowNode`, leading to arbitrary hit testing instead of prioritizing the topmost node.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **y** — `Camera` and `FlowCanvas` share a file.
  - **Proposed split**:
    - `src/camera.rs`: `Camera` struct and its operations.
    - `src/canvas.rs`: `FlowCanvas` struct.

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-flow/src/graph.rs`
**Line count**: 338 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-flow/src/ribbon.rs`
**Line count**: 526 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-spatial/src/quadtree.rs`
**Line count**: 251 lines

#### 1. Bug Identification & Debugging
- None found (duplicate overlapping rect storage in leaves is by design).

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-spatial/src/bvh.rs`
**Line count**: 373 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-spatial/src/spatial_hash.rs`
**Line count**: 222 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-runic-text/src/lib.rs`
**Line count**: 4037 lines

#### 1. Bug Identification & Debugging
- **UTF-8 Byte Offset Character Index Confusion (line 1980)**: `char_at_cluster` calls `text.chars().nth(glyph.cluster as usize)`. However, `glyph.cluster` is a byte offset, while `nth()` expects a character index. For non-ASCII text, this causes layout wrapping to look up incorrect characters, leading to broken wrapping behavior.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **y** — This file is 4037 lines long, mixing shaper, layouter, styles, path sampling, boundaries, and accessibility run structures.
  - **Proposed split**:
    - `src/style.rs`: `TextStyle`, `LineHeight`, `TextDecorations`, `TextOverflow`, `TextAlign`, `RenderMode`, `VariableAxis`.
    - `src/path.rs`: `TextPath`, `LayoutBoundary` structs.
    - `src/span.rs`: `TextSpan`, `TextSpanKind`, `PortalAlignment`.
    - `src/run.rs`: `TextRun`, `SemanticRange`, `Paragraph`.
    - `src/engine.rs`: `RunicTextEngine` struct and its main implementation methods.
    - `src/layout_engine.rs`: Layout calculation loops, line formatting, and word wrap logic.
    - `src/lib.rs`: Public exports and core types setup.

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- **Line 381**: `intersections.sort_by(|a, b| a.partial_cmp(b).unwrap())`. Can panic if `partial_cmp` returns `None` (e.g. on `NaN` floats). Replace with `total_cmp()`.

---

### File: `cvkg-runic-text/src/global_cache.rs`
**Line count**: 70 lines

#### 1. Bug Identification & Debugging
- **LRU Stale Ordering Duplicate Entries (line 31)**: `global_cache_insert` pushes keys onto `cache_order` without checking for duplicates or removing old entries when updating a key. This causes `cache_order` length to inflate and results in stale keys pointing to already-removed cache elements.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- **Line 23**: `unwrap()` on `GLOBAL_SHAPE_CACHE` lock. Can panic if mutex is poisoned.

---

### File: `cvkg-runic-text/src/msdf.rs`
**Line count**: 319 lines

#### 1. Bug Identification & Debugging
- **Missing Out-of-Bounds Packed Height Verification (line 183)**: `pack_atlas` checks if single glyph dimensions exceed `max_size`, but does not verify if the total accumulated packed height `total_height` exceeds `max_size`, allowing the generation of oversized/unallocatable texture bounds.
- **Division by Zero on Zero-Radius Spread (line 121)**: `generate_sdf` does not validate if the `spread` parameter is greater than `0.0`. A zero or negative spread leads to division by zero or NaN results.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-runic-text/src/subpixel.rs`
**Line count**: 499 lines

#### 1. Bug Identification & Debugging
- **Constant Zero Subpixel Phase / Opaque Channel Fallback (line 290)**: In `render_lcd`, `subpixel_phase` is computed as `(local_x * 3.0) as i32 % 3`. Since `local_x` is an integer, the float multiplication and integer conversion result in multiples of 3, meaning the modulo operation *always* yields `0`. This forces every pixel to evaluate with phase-0 coverage (`[255, 0, 0]`), turning off the Green and Blue channels, causing severe color fringing and breaking subpixel LCD filtering.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-runic-text/src/knuth_plass.rs`
**Line count**: 504 lines

#### 1. Bug Identification & Debugging
- **UTF-8 Byte Count Character Width Confusion (line 351)**: `break_text_simple` computes line length as `char_end - line_start`, which is a byte count in UTF-8. It then scales this directly by `char_width`, which counts each byte as a full character, causing non-ASCII lines to wrap prematurely.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-anim/src/lib.rs`
**Line count**: 717 lines

#### 1. Bug Identification & Debugging
- **Inertial/Physics Solver Explosion on Scalar Timeline (line 386)**: If `ProgressDriver::Scalar(t)` is passed, it represents the absolute timeline scroll offset. However, `dt.delta_time_secs()` returns `t` directly as the delta time instead of computing the difference. For non-linear/physics animations like `MjolnirShatter` or `Momentum` that still use `.tick(dt_secs)`, they receive this absolute offset as a delta, causing the physics integration to explode or behave erratically.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- **Line 599**: `child.solver.as_mut().unwrap()`. Low risk but can panic if child states are initialized or processed incorrectly.

---

### File: `cvkg-anim/src/spring_snap.rs`
**Line count**: 283 lines

#### 1. Bug Identification & Debugging
- **Repeated Overshoot Haptic Trigger (line 122)**: `SnapTracker::track` calculates `overshooting = curr_dist > prev_dist && dist_to_target > self.threshold`. When a spring oscillates past target, it remains in `overshooting` for multiple frames until it reaches peak amplitude. This causes `track` to continuously return `SpringSnapEvent::Overshoot` on every frame, resulting in rapid back-to-back haptic triggers instead of a single trigger at the start of the overshoot.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-anim/src/physics.rs`
**Line count**: 1456 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **y** — 1456 lines, contains multiple independent solvers.
  - **Proposed split**:
    - `src/physics/mod.rs`: Primitives (`Vec3`, `Quat`) and exports.
    - `src/physics/rigid.rs`: `RigidBody`, `RigidConstraint`, `RigidBodyWorld`.
    - `src/physics/soft.rs`: `SoftParticle`, `DistanceConstraint`, `VolumeConstraint`, `SoftBody`.
    - `src/physics/cloth.rs`: `ClothParticle`, `ClothConstraint`, `Cloth`.
    - `src/physics/fluid.rs`: `FluidParticle`, `FluidSimulation`.

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-anim/src/growth.rs`
**Line count**: 1231 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **y** — 1231 lines, contains L-system and Voronoi fracture solvers.
  - **Proposed split**:
    - `src/growth/lsystem.rs`: `LRule`, `LineSegment`, `LSystem`, `SimpleRng` (LSystem specific).
    - `src/growth/voronoi.rs`: `Bounds`, `VoronoiCell`, `Fragment`, `VoronoiFracture`.
    - `src/growth/mod.rs`: Exports.

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-scene/src/lib.rs`
**Line count**: 835 lines

#### 1. Bug Identification & Debugging
- **Quadratic Rect Merging / Performance Bottleneck (line 390)**: `merge_dirty_regions` allocates and rebuilds a `Quadtree` from scratch on *every single* inner loop iteration while merging overlapping dirty regions. Under high dirty-rect counts, this results in a severe $O(N^2 \log N)$ performance bottleneck.
- **Deserialized next_id Collisions (line 361)**: `deserialize_binary` resets `next_id` to 0. A subsequent call to `next_id()` starts allocating IDs from 0, resulting in ID collisions with nodes loaded from the deserialized graph. It should compute the maximum ID in the loaded nodes and set `next_id` to `max + 1`.
- **Orphan File**: `src/quadtree.rs` in `cvkg-scene` is an unused duplicate file that was replaced by imports from `cvkg-spatial` but left in the source directory.

#### 2. Security-Minded Checks
- **Untrusted Deserialization**: `deserialize_binary` uses raw `bincode::deserialize` which parses untrusted binary buffers without bounds checks or sizing limits, risking panics or OOM.

#### 3. Monolithic File Decomposition
- **y** (Moderate split makes things cleaner).
  - **Proposed split**:
    - `src/node.rs`: `VNode` struct, `NodeId` alias, `Change` and `Patch` enums.
    - `src/lib.rs`: `SceneGraph` struct and methods.

#### 4. Theming
- Norse theme comments: "Surtr GPU pipeline".
  - **Proposed name**: `GpuPipeline` or `GPURenderer`.

#### 5. Unwrap/Unsafe Audit
- **Line 176, 183**: `.unwrap()` on retrieving nodes during recursive transform propagation. Can panic if graph structure is malformed or has dangling parent relationships.

---

### File: `cvkg-scene/src/test_renderer.rs`
**Line count**: 366 lines

#### 1. Bug Identification & Debugging
- **Monospace Text Measurement Multi-Byte Overestimation (line 230)**: `measure_text` estimates width using `text.len()`, which returns byte length instead of character count. For multi-byte non-ASCII characters, this causes layout wrapping to overestimate text bounds.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- Norse-themed renderer methods: `Bifrost` (glass blur), `Gungnir` (glow), `push_mjolnir_slice` (shatter).
  - **Proposed names**: `BackdropFilter`, `BloomFilter`, `ShatterSlice`.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-themes/src/lib.rs`
**Line count**: 1309 lines

#### 1. Bug Identification & Debugging
- **NaN / Division by Zero on from_rgb (line 35)**: `from_rgb` accepts negative or unbounded RGB values. If input components are `< -0.055`, `to_linear` attempts `powf(2.4)` on negative bases, yielding `NaN` color outputs. Input parameters must be clamped.
- **Grayscale Contrast Search Failure (line 1085)**: In `compute_contrasting_text`, if both black and white fall below the target `Lc >= 60` threshold, the binary search iterates 20 times to find a middle gray contrast, but middle gray will always have even lower contrast than the extremes. It should immediately return the best-performing extreme instead of searching.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **y** — 1309 lines of colors, APCA logic, themes, states, and radius/motion scales.
  - **Proposed split**:
    - `src/color.rs`: `OklchColor` and models.
    - `src/material.rs`: `GlassMaterial`.
    - `src/contrast.rs`: APCA and accessibility checks.
    - `src/theme.rs`: `Theme`, `RadiusScale`, `SpacingScale`, `MotionScale`.
    - `src/builder.rs`: `ThemeBuilder`.
    - `src/states.rs`: `InteractiveState`, `StateColors`.

#### 4. Theming
- Norse identifiers: `SleipnirParams`, `Mani glow`, `Viking Gold`, `Tactical Obsidian`.
  - **Proposed names**: `SpringParams`, `MoonGlow`, `Gold`, `Obsidian`.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-materials/src/glass.rs`
**Line count**: 190 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Small and clean).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-accessibility/src/focus.rs`
**Line count**: 356 lines

#### 1. Bug Identification & Debugging
- **Sequential ID Sorting Stub (line 179)**: `rebuild_from_tree` sorts node focus targets by their raw u64 `KvasirId` values rather than document layout hierarchy. This causes a randomized tab-stop ordering based on allocation times. Visual x/y bounds must be used.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Well-structured).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-accessibility/src/tree.rs`
**Line count**: 483 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None found.

#### 5. Unwrap/Unsafe Audit
- None found.

---

### File: `cvkg-reflect/src/lib.rs`
**Line count**: 640 lines

#### 1. Bug Identification & Debugging
- **Input Validation Omission (line 355)**: `ColorStop::set_field` parses values from JSON but performs no range validation. The normalized `position` float field can be set to negative, NaN, or excessively large values, violating the `[0, 1]` gradient constraint.

#### 2. Security-Minded Checks
- **Omission of Limits**: Reflection structures should restrict max sizes for incoming dynamic strings to prevent memory abuse.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-certification/src/lib.rs`
**Line count**: 483 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-macros/src/lib.rs`
**Line count**: 292 lines

#### 1. Bug Identification & Debugging
- **Component Builder Build Panic (line 191)**: `cvkg_component` generates a `build()` method that executes `.expect("missing required field ")` on *all* struct fields. This makes all fields mandatory and triggers a runtime panic if any field lacks an explicit builder call, rather than leveraging structural or fallback defaults. It also hides the field name from the error string.
- **Stateless/Duplicate vdom_id Generation (line 284)**: `cvkg_model` generates a `vdom_id` method by instantiating an empty `DefaultHasher` and instantly calling `finish()`. Since no data is hashed, it constantly yields the same hash suffix, generating identical IDs for all model nodes.

#### 2. Security-Minded Checks
- None found.

#### 3. Monolithic File Decomposition
- **n** (proc-macro).

#### 4. Theming
- Norse macros: `hamr!` (after Mjolnir).
  - **Proposed name**: `markup!` or `ui!`.

#### 5. Unwrap/Unsafe Audit
- Output code contains `.expect("missing required field ")` which creates a runtime crash risk if component values are omitted.

---

### File: `cvkg-render-gpu/src/color_blindness.rs`
**Line count**: 249 lines

#### 1. Bug Identification & Debugging
- **Degenerate Full-Screen Triangle Vertex Shader (line 122)**: `fs_main_vs` returns a degenerate flat line for vertex index 2 because both components are hardcoded to `-1.0`. The position for `vid == 2u` should map to `(-1.0, 3.0)` to form a valid triangle covering the viewport. This makes color blindness simulation completely invisible.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-render-gpu/src/heim.rs`
**Line count**: 140 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- Norse terminology: `SundrPacker`, `Mega-Heim`.
  - **Proposed names**: `SkylinePacker`, `MegaAtlas`.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-render-gpu/src/pyramid.rs`
**Line count**: 88 lines

#### 1. Bug Identification & Debugging
- **Mismatched Luminance Dimension (line 50)**: `pyramid_luminance` allocates with bounds `mip_w` and `mip_h` after the mip calculation loop completes. Because the loop performs one final division step, the luminance texture is allocated at half the size of the final mip level rather than matching it.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Sized correctly).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-render-gpu/src/renderer.rs`
**Line count**: 6637 lines

#### 1. Bug Identification & Debugging
- **Dangling Window / Context Panic (line 4382)**: If `current_window` is present but missing from the surfaces registry, or if headless context setup fails, the renderer triggers an unrecoverable `unwrap()` panic in `render_frame`.

#### 2. Security-Minded Checks
- **Unsafe Pipeline Cache Corruption**: Passing unverified binary cache bytes from the filesystem directly to `create_pipeline_cache` can trigger memory corruption inside graphics drivers. The binary data must be checksummed before compilation.

#### 3. Monolithic File Decomposition
- **y** — This file is 6637 lines long and handles device setup, asset loading, staging belts, drawing command records, pipelines, passes scheduling, and shader bindings.
  - **Proposed split**: Move passes setup, subsystems (Geometry, Text, SVG) compilation, and staging command buffering to separate files under `src/subsystems/` and `src/pipelines/`.

#### 4. Theming
- Norse structures: `SurtrRenderer`, `Kvasir Graph`.
  - **Proposed names**: `GpuRenderer`, `RenderGraph`.

#### 5. Unwrap/Unsafe Audit
- **Line 424, 426**: `unsafe impl Send/Sync for SurtrRenderer {}` on WASM.
- **Line 1138**: `unsafe { device.create_pipeline_cache(&cache_data) }`. Risk of loading tampered cache files.
- **Multiple `.unwrap()` calls** on internal registry reads.

---

### File: `cvkg-core/src/lib.rs`
**Line count**: 6611 lines

#### 1. Bug Identification & Debugging
- **Float Sorting Panic on NaN (line 3703)**: `sort_by` parses float values and calls `.unwrap()` on `partial_cmp`. If `NaN` values are present, the comparator returns `None`, inducing a thread panic. `total_cmp` should be used instead.

#### 2. Security-Minded Checks
- **Thread Locals and State Exposure**: Sharing global state hooks across multiple tasks without mutex poison propagation safeguards could leak state information.

#### 3. Monolithic File Decomposition
- **y** — 6611 lines long. Mixing primitive geometry, color models, views routing, signals, undo stacks, error boundary containers, and task systems.
  - **Proposed split**: Separate into `src/signals.rs`, `src/view.rs`, `src/color.rs`, `src/layout_types.rs`, `src/state_management.rs`.

#### 4. Theming
- Norse terminology in colors: `MuspelMagenta`, `NiflheimNavy`.

#### 5. Unwrap/Unsafe Audit
- **Multiple `.unwrap()`** on Mutex locks and float conversions.

---

### File: `cvkg-render-native/src/lib.rs`
**Line count**: 4277 lines

#### 1. Bug Identification & Debugging
- **Dangling Thread-Local Pointer on Panic (line 1248)**: Inside the locked render pass, `GPU_FRAME_PTR` is set to the raw pointer of the `SurtrRenderer` and reset to null at the end of the block. If `self.view.render` panics, unwinding skips the resetting line, leaving a dangling pointer in thread-local storage that will cause undefined behavior/crashes when subsequent frames are rendered.
- **Unchecked setpriority Call (line 2581)**: Inside `set_berserker_mode`, the code Elevates process priority using `libc::setpriority` under Linux. Although permissions might fail and return an error (which is ignored), calling `setpriority` with a negative increment (`-10`) is a privileged operation and could fail silently.

#### 2. Security-Minded Checks
- **Privileged Priority Elevation**: Attempting to raise process priority could be flagged by system security sandboxes/policies.

#### 3. Monolithic File Decomposition
- **y** — 4277 lines long, mixing winit event loops, raw pointer bindings, clipboard integration, audio playback engines, and window management.
  - **Proposed split**:
    - `src/audio.rs`: `RodioAudioEngine` and audio playback structures.
    - `src/clipboard.rs`: Clipboard and primary selection integration wrappers.
    - `src/window.rs`: Window creation, surface registration, resizing, and input event translation.
    - `src/main_loop.rs`: Main event loop run method and state orchestrator.
    - `src/lib.rs`: Exports and thread locals.

#### 4. Theming
- Norse indicators: `RodioAudioEngine` uses `mimer!` macro naming, and Berserker Mode uses `GodMode`.

#### 5. Unwrap/Unsafe Audit
- **Line 362 & 381**: Dereferences raw pointers `*mut SurtrRenderer` in `GpuRef` mapping helper without verifying structure alignment or lifespans.
- **Line 2581 & 2586**: Invokes raw `libc::setpriority` syscall.

---

### File: `cvkg-render-software/src/lib.rs`
**Line count**: 748 lines

#### 1. Bug Identification & Debugging
- **Rounded Rectangle SDF Calculation Error (line 224)**: The math `(fx - rect.x).max(rect.x + rect.width - fx).max(0.0) - rect.width * 0.5` simplifies to `|fx - center_x|` because it doesn't subtract the half-dimensions before clamping to zero. This computes distance to the center point rather than the borders minus radius, causing the rasterizer to draw a tiny central circle instead of the rounded rectangle.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-compositor/src/engine.rs`
**Line count**: 347 lines

#### 1. Bug Identification & Debugging
- **Stack Overflow on Cyclic References (line 248)**: `CompositorEngine::flatten_layer` uses deep recursion (DFS traversal) to flatten child layers without maintaining a set of visited layer IDs. If a cyclic reference is introduced into the `LayerTree`, it results in an immediate stack overflow.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Sized appropriately).

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-svg-filters/src/lib.rs`
**Line count**: 4021 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **y** — 4021 lines long, handles usvg filters parsing, ping-pong textures management, and shader layout creation.
  - **Proposed split**: Move DAG parsing logic to `src/parser.rs` and pipeline building to `src/pipeline.rs`.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-svg-serialize/src/lib.rs`
**Line count**: 496 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-telemetry/src/lib.rs`
**Line count**: 215 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-icons/src/lib.rs`
**Line count**: 150 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-test/src/lib.rs`
**Line count**: 120 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-webkit-server/src/main.rs`
**Line count**: 680 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- **Static Assets Directory Traversal**: `ServeDir` is used to serve pkg, assets, and static directories. Ensure that proper routing limits are maintained to prevent path traversal.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- Norse titles/labels: "Agent Ulfhednar - Tactical Dashboard", "Loading Agent Ulfhednar...".

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-cli/src/main.rs`
**Line count**: 890 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **y** — Large CLI interface containing build pipeline runners, config parsers, devtools relays, and scaffolding.
  - **Proposed split**: Move commands into sub-modules under `src/commands/`.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg-components/src/lib.rs`
**Line count**: 310 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- Uses names like Sleipnir transitions and Bragi creative modes.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `cvkg/src/lib.rs`
**Line count**: 52 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- Comments referencing Surtr, Muspelheim, and Bifrost.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `demos/berserker/src/main.rs`
**Line count**: 1120 lines

#### 1. Bug Identification & Debugging
- None found.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n** (Standard demo entrypoint).

#### 4. Theming
- Berserker UI theme configuration.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `demos/adele-web/src/lib.rs`
**Line count**: 100 lines

#### 1. Bug Identification & Debugging
- None.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- None.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `demos/niflheim-wasi/src/main.rs`
**Line count**: 140 lines

#### 1. Bug Identification & Debugging
- None.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- Uses Niflheim themes.

#### 5. Unwrap/Unsafe Audit
- None.

---

### File: `demos/berserker-fire-web/src/lib.rs`
**Line count**: 110 lines

#### 1. Bug Identification & Debugging
- None.

#### 2. Security-Minded Checks
- None.

#### 3. Monolithic File Decomposition
- **n**.

#### 4. Theming
- Berserker theme styling.

#### 5. Unwrap/Unsafe Audit
- None.

---

## Step 2 — Summary Table

| File | Bugs Found | Security Issues | Decomposition Needed | Theming Issues | Unwrap/Unsafe Issues |
|------|------------|-----------------|---------------------|--------------|-------------------|
| `cvkg-vdom/src/lib.rs` | y (bubbling skip, lock panic) | y (DoS/exhaustion) | y (2341 lines) | none | y (med) |
| `cvkg-layout/src/lib.rs` | y (cycle guard leak, focus tab index) | none | y (2811 lines) | none | y (med) |
| `cvkg-scheduler/src/frame.rs` | none | none | n | none | none |
| `cvkg-scheduler/src/task.rs` | none | none | n | none | none |
| `cvkg-physics/src/body.rs` | none | none | n | none | none |
| `cvkg-physics/src/broadphase.rs` | none | none | n | none | none |
| `cvkg-physics/src/world.rs` | y (variable dt spike) | none | y (1172 lines) | none | none |
| `cvkg-flow/src/node.rs` | y (opaque tint) | none | n | none | none |
| `cvkg-flow/src/edge.rs` | y (interaction color snap) | none | n | none | none |
| `cvkg-flow/src/canvas.rs` | y (arbitrary overlap selection) | none | y (662 lines) | none | none |
| `cvkg-flow/src/graph.rs` | none | none | n | none | none |
| `cvkg-flow/src/ribbon.rs` | none | none | n | none | none |
| `cvkg-spatial/src/quadtree.rs` | none | none | n | none | none |
| `cvkg-spatial/src/bvh.rs` | none | none | n | none | none |
| `cvkg-spatial/src/spatial_hash.rs` | none | none | n | none | none |
| `cvkg-runic-text/src/lib.rs` | y (UTF-8 nth char wrap) | none | y (4037 lines) | none | y (low) |
| `cvkg-runic-text/src/global_cache.rs` | y (LRU stale duplicates) | none | n | none | y (med) |
| `cvkg-runic-text/src/msdf.rs` | y (atlas wrap height, div 0) | none | n | none | none |
| `cvkg-runic-text/src/subpixel.rs` | y (subpixel phase constant 0) | none | n | none | none |
| `cvkg-runic-text/src/knuth_plass.rs` | y (byte count simple wrap) | none | n | none | none |
| `cvkg-anim/src/lib.rs` | y (scalar dt physics solver explosion) | none | n | none | y (low) |
| `cvkg-anim/src/spring_snap.rs` | y (repeated overshoot haptics) | none | n | none | none |
| `cvkg-anim/src/physics.rs` | none | none | y (1456 lines) | none | none |
| `cvkg-anim/src/growth.rs` | none | none | y (1231 lines) | none | none |
| `cvkg-scene/src/lib.rs` | y (quadtree merge bottleneck, next_id collision) | y (bincode deserialization) | y (835 lines) | none | y (med) |
| `cvkg-scene/src/test_renderer.rs` | y (byte measurement count) | none | n | none | none |
| `cvkg-themes/src/lib.rs` | y (from_rgb NaN, grayscale search loop) | none | y (1309 lines) | none | none |
| `cvkg-materials/src/glass.rs` | none | none | n | none | none |
| `cvkg-accessibility/src/focus.rs` | y (ID sorting stub) | none | n | none | none |
| `cvkg-accessibility/src/tree.rs` | none | none | n | none | none |
| `cvkg-reflect/src/lib.rs` | y (set_field bounds check) | none | n | none | none |
| `cvkg-certification/src/lib.rs` | none | none | n | none | none |
| `cvkg-macros/src/lib.rs` | y (builder panic, empty hash vdom_id) | none | n | none | y (low) |
| `cvkg-render-gpu/src/color_blindness.rs` | y (degenerate full-screen triangle) | none | n | none | none |
| `cvkg-render-gpu/src/heim.rs` | none | none | n | none | none |
| `cvkg-render-gpu/src/pyramid.rs` | y (luminance dimension mismatch) | none | n | none | none |
| `cvkg-render-gpu/src/renderer.rs` | y (context panic) | y (unsafe pipeline cache) | y (6637 lines) | none | y (high) |
| `cvkg-core/src/lib.rs` | y (float sorting NaN panic) | none | y (6611 lines) | none | y (med) |
| `cvkg-render-native/src/lib.rs` | y (TLS leak on panic, setpriority) | none | y (4277 lines) | y (GodMode, mimer!) | y (high) |
| `cvkg-render-software/src/lib.rs` | y (rounded rect SDF) | none | n | none | none |
| `cvkg-compositor/src/engine.rs` | y (cycle recursion) | none | n | none | none |
| `cvkg-svg-filters/src/lib.rs` | none | none | y (4021 lines) | none | none |
| `cvkg-svg-serialize/src/lib.rs` | none | none | n | none | none |
| `cvkg-telemetry/src/lib.rs` | none | none | n | none | none |
| `cvkg-icons/src/lib.rs` | none | none | n | none | none |
| `cvkg-test/src/lib.rs` | none | none | n | none | none |
| `cvkg-webkit-server/src/main.rs` | none | y (ServeDir traversal) | n | y (Ulfhednar) | none |
| `cvkg-cli/src/main.rs` | none | none | y (890 lines) | none | none |
| `cvkg-components/src/lib.rs` | none | none | n | y (Sleipnir, Bragi) | none |
| `cvkg/src/lib.rs` | none | none | n | y (Surtr, Bifrost) | none |
| `demos/berserker/src/main.rs` | none | none | n | y (Berserker UI) | none |
| `demos/adele-web/src/lib.rs` | none | none | n | none | none |
| `demos/niflheim-wasi/src/main.rs` | none | none | n | y (Niflheim) | none |
| `demos/berserker-fire-web/src/lib.rs` | none | none | n | y (Berserker UI) | none |

---

## Step 3 — Aggregate Plan

### Prioritized Bug/Security Fix List
1. **P0**: Dangling thread-local pointer on panic in `GPU_FRAME_PTR` (cvkg-render-native/src/lib.rs:1248). Wrap the rendering pass in a custom RAII drop guard to ensure the thread-local is safely cleared even if `self.view.render()` panics.
2. **P0**: Unsafe pipeline cache corruption in `SurtrRenderer` (cvkg-render-gpu/src/renderer.rs:1138). Verify cache bytes with SHA-256 before invoking graphics API.
3. **P1**: Software renderer rounded rectangle SDF layout bug (cvkg-render-software/src/lib.rs:224). Correct math computing distance to boundaries to avoid collapsing the width.
4. **P1**: Degenerate color blindness full-screen triangle (cvkg-render-gpu/src/color_blindness.rs:122). Fix indices and mapping coordinates to resolve output coverage.
5. **P1**: Constant Zero Subpixel Phase / Opaque Channel Fallback (cvkg-runic-text/src/subpixel.rs:290). Fix subpixel phase computation from pixel offset.
6. **P1**: UTF-8 Byte Offset Character Index Confusion (cvkg-runic-text/src/lib.rs:1980). Use character parsing/iterating instead of `.nth(byte_offset)`.
7. **P1**: Inertial/Physics Solver Explosion on Scalar Timeline (cvkg-anim/src/lib.rs:386). Calculate and track proper delta step for scroll drive.
8. **P2**: Stack overflow on cyclic layer references (cvkg-compositor/src/engine.rs:248). Track visited layer IDs during DFS traversal in `flatten_layer` to detect cycles.
9. **P2**: RwLock Poisoning in `use_state` (cvkg-vdom/src/lib.rs:2319). Replace `unwrap()` with `.unwrap_or_else(|e| e.into_inner())`.
10. **P2**: Cycle guard leak on panic (cvkg-layout/src/lib.rs:57, 77). Use a custom RAII drop guard for `ACTIVE_LAYOUT_NODES`.
11. **P2**: next_id collisions after deserialization (cvkg-scene/src/lib.rs:361). Update `deserialize_binary` to calculate next unique ID from parsed nodes map.
12. **P2**: NaN results on OKLCH conversions (cvkg-themes/src/lib.rs:35). Add boundary clamps on sRGB inputs to prevent taking powf fractional exponents on negative values.
13. **P2**: Component builder panic on missing default fields (cvkg-macros/src/lib.rs:191). Avoid unconditional runtime `.expect()` inside generated macro build methods, and insert correct field names into panics.
14. **P2**: Duplicate empty hash vdom_ids (cvkg-macros/src/lib.rs:284). Feed actual unique attributes to hasher to prevent identical IDs.
15. **P2**: Float sorting panic on NaN (cvkg-core/src/lib.rs:3703 and cvkg-runic-text/src/lib.rs:381). Sort using `total_cmp` instead of `partial_cmp().unwrap()`.
16. **P3**: Unchecked priority elevation `setpriority` syscall (cvkg-render-native/src/lib.rs:2581). Ensure that scheduler hints are validated and checked gracefully.
17. **P3**: Static assets directory traversal potential (cvkg-webkit-server/src/main.rs:587-589). Ensure proper path sanitization limits are maintained on Axum ServeDir routing.
18. **P3**: Mismatched luminance mip sizes (cvkg-render-gpu/src/pyramid.rs:50). Store actual bounds before division step inside ImagePyramid instantiation loop.
19. **P3**: Quadratic dirty regions merging bottleneck (cvkg-scene/src/lib.rs:390). Re-use or rebuild spatial index efficiently instead of allocating new quadtrees in loop.
20. **P3**: Repeated Overshoot Haptic Trigger (cvkg-anim/src/spring_snap.rs:122). Track whether overshoot event was already fired during current target crossing cycle.
21. **P3**: Arbitrary Overlapping Node Hit-Test (cvkg-flow/src/canvas.rs:314). Sort candidate nodes by `z_index` descending before matching.
22. **P3**: Variable dt step spikes (cvkg-physics/src/world.rs:290). Clamp `sub_dt` to a sensible maximum or enforce fixed_timestep default.
23. **P3**: Focus candidate tab index bypass (cvkg-layout/src/lib.rs:1779). Filter out candidates with `tab_index < 0` from natural sequential focus traversal.
24. **P3**: Fallback descendant handler bubbling (cvkg-vdom/src/lib.rs:2009). Modify the descendant search fallback to trigger a proper bubble chain.
25. **P3**: Opaque Glass Tint on Custom Color (cvkg-flow/src/node.rs:164). Preserve current tint alpha in `with_tint_oklch`.
26. **P3**: Instantaneous Interaction Color Transition (cvkg-flow/src/edge.rs:252). Introduce interaction transition ticking.
27. **P3**: LRU Stale Ordering Duplicate Entries (cvkg-runic-text/src/global_cache.rs:31). Remove existing key references from ordering list before re-inserting.
28. **P3**: Missing Out-of-Bounds Packed Height Verification (cvkg-runic-text/src/msdf.rs:183). Fail shelf packing if total height exceeds max atlas dimensions.
29. **P3**: Division by Zero on Zero-Radius Spread (cvkg-runic-text/src/msdf.rs:121). Clamp/check spread parameter to be strictly positive.
30. **P3**: UTF-8 Byte Count Character Width Confusion (cvkg-runic-text/src/knuth_plass.rs:351). Use `chars().count()` instead of byte subtraction.
31. **P3**: Monospace measurement bytes overestimation (cvkg-scene/src/test_renderer.rs:230). Count actual characters instead of bytes length.
32. **P3**: Keyboard focus ID sorting stub (cvkg-accessibility/src/focus.rs:179). Implement sorting by visual coordinates.
33. **P3**: Reflect property setting value limits (cvkg-reflect/src/lib.rs:355). Validate and clamp float/int properties to defined schema constraints.

### File Decomposition Priority List
1. `cvkg-render-gpu/src/renderer.rs` (6637 lines) -> Split GPU pipeline, stages command buffers, and subsystems compile targets.
2. `cvkg-core/src/lib.rs` (6611 lines) -> Split into signals, views, color formats, layout structures, and core primitives.
3. `cvkg-render-native/src/lib.rs` (4277 lines) -> Split event loops, audio playback engine, clipboard system, and window managers.
4. `cvkg-svg-filters/src/lib.rs` (4021 lines) -> Split filters parsing and pipeline buffers allocation logic.
5. `cvkg-runic-text/src/lib.rs` (4037 lines) -> Split into `style.rs`, `path.rs`, `span.rs`, `run.rs`, `engine.rs`, `layout_engine.rs`, and `lib.rs`.
6. `cvkg-layout/src/lib.rs` (2811 lines) -> Split into `taffy_engine.rs`, `animation.rs`, `primitives.rs`, `grid.rs`, `spatial_index.rs`, `parallel.rs`, `modality.rs`, `focus.rs`, and `progressive.rs`.
7. `cvkg-vdom/src/lib.rs` (2341 lines) -> Split into `vnode.rs`, `patch.rs`, `diff.rs`, `hit_test.rs`, `events.rs`, `accesskit_bridge.rs`, `renderer.rs`, and `state.rs`.
8. `cvkg-anim/src/physics.rs` (1456 lines) -> Split into `physics/mod.rs`, `physics/rigid.rs`, `physics/soft.rs`, `physics/cloth.rs`, and `physics/fluid.rs`.
9. `cvkg-themes/src/lib.rs` (1309 lines) -> Split into color, material, contrast, theme, builder, and states modules.
10. `cvkg-anim/src/growth.rs` (1231 lines) -> Split into `growth/lsystem.rs`, `growth/voronoi.rs`, and `growth/mod.rs`.
11. `cvkg-physics/src/world.rs` (1172 lines) -> Split into `mod.rs`, `config.rs`, `body_registry.rs`, and `step.rs`.
12. `cvkg-scene/src/lib.rs` (835 lines) -> Split into `node.rs` and `lib.rs`.
13. `cvkg-cli/src/main.rs` (890 lines) -> Split commands into independent subcommand executor files under `src/commands/`.
14. `cvkg-flow/src/canvas.rs` (662 lines) -> Split into `camera.rs` and `canvas.rs`.

### Unwrap/Unsafe Remediation — High/Medium Severity
#### rwlock read lock in `use_state` (cvkg-vdom/src/lib.rs:2319)
```rust
// Proposed replacement:
Some(arc_val) => arc_val.read().unwrap_or_else(|e| e.into_inner()).clone(),
```

#### Unsafe pipeline cache compilation (cvkg-render-gpu/src/renderer.rs:1138)
```rust
// Ensure data is verified via SHA-256 hash before initializing the GPU driver with it:
if verify_cache_checksum(&cache_data, expected_checksum) {
    Some(unsafe { device.create_pipeline_cache(&cache_data) })
} else {
    None
}
```

#### Dangling thread-local pointer in render pass (cvkg-render-native/src/lib.rs:1248)
```rust
// Proposed replacement using a custom RAII scope guard:
struct ThreadLocalGpuGuard;
impl ThreadLocalGpuGuard {
    fn new(raw: *mut cvkg_render_gpu::SurtrRenderer) -> Self {
        GPU_FRAME_PTR.with(|ptr| ptr.set(raw));
        Self
    }
}
impl Drop for ThreadLocalGpuGuard {
    fn drop(&mut self) {
        GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
    }
}

// Inside the locked render block:
let _guard = ThreadLocalGpuGuard::new(gpu as *mut _);
self.view.render(&mut renderer, content_rect);
```
