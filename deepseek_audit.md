# cvkg-render-gpu Engineering Audit

**Audit tool**: deepseek-v4-flash-free (via OpenCode Zen)
**Date**: 2026-06-20
**Scope**: 32 `.rs` files in `cvkg-render-gpu` crate
**Method**: Static file-by-file structured audit per the process defined by the user.

## Step 0: Crate Orientation

- **Responsibility**: GPU rendering pipeline — Vulkan/Metal/DX12 via wgpu, includes render passes (geometry, glass, UI, composite, bloom, effects, volumetric, backdrop, pyramid, tonemap, accessibility, SVG filters), material graph compiler, pipeline cache, frame capture, and the Kvasir render graph abstraction.
- **Intra-workspace dependencies**: `cvkg-core` (View, Renderer, Event, NodeId), `cvkg-scene`, `cvkg-runic-text`, `cvkg-svg-filters`, `cvkg-compositor`

## Step 1: Per-File Findings

See individual file audits below.

---

## cvkg-render-gpu Audit Findings

### `src/renderer.rs` (6,636 lines)

#### 1) Bug Identification

**Bug 1 — SHA256 integrity check truncated to 8 bytes (lines 517-521 and 6107-6111)**

The code computes `sha256_digest(material_src).as_slice()[..8]` and stores/compares only the first 8 bytes (64 bits). This gives a 2^-64 collision probability — high enough for targeted attacks despite being large enough for accidental collisions. The preceding comment says "Only check first 64 bits for speed" but SHA256 costs are dominated by hashing, not comparison of a few extra bytes.

**How to trigger**: Craft two materially-different WGSL source strings whose SHA256 first 8 bytes collide (expected ~2^64 attempts — computationally feasible for a motivated attacker). The second shader will hit the cache and serve stale compiled output.

**Bug 2 — u32 overflow in capture VRAM size calculation (renderer.rs, line 5734)**

`buffer.size` (u64) is cast to `u32` in `capture_frame`. If the captured output exceeds ~4 GiB, the size silently wraps. wgpu buffer sizes for a single frame can exceed 4 GiB at very high resolutions (e.g., 8K HDR with multiple render targets).

**How to trigger**: Set an unusually large viewport resolution (> 8K) and call `capture_frame`. On some GPUs the buffer size may exceed u32::MAX, causing the cast to truncate silently.

#### 2) Security-Minded Checks

- **Material shader input**: `material_src` is a string from a file or user-supplied material definition. It's hashed and cached, never executed directly (it goes through the WGSL compiler). No exploits.
- **File paths**: `load_shader_source` reads from embedded resources, not user filesystem paths.
- **No hardcoded secrets**: None found.

#### 3) Monolithic File Decomposition

6,636 lines with at least 7 distinct responsibilities: renderer init, frame submission, pipeline cache, shader cache, capture, swapchain management, error formatting. High-priority decomposition candidate.

Proposed split:
- `renderer/init.rs` — Context creation, adapter selection, surface config
- `renderer/frame.rs` — Frame begin/end/present, render pass submission
- `renderer/pipelines.rs` — Pipeline lookup, cache management
- `renderer/cache.rs` — Shader cache filesystem I/O
- `renderer/capture.rs` — Frame capture

#### 4) Fanciful/Themed Naming

No Norse-mythology-themed names identified in this file. All structs, functions, and fields use descriptive names (e.g., `RendererContext`, `FramePacket`, `PipelineKey`, `ShaderCache`).

#### 5) unwrap()/expect() + unsafe Combination Audit

| Line(s) | Expression | Risk | Reasoning | Fix |
|---------|-----------|------|-----------|-----|
| 500+ | ~23 `.unwrap()` calls throughout | **Med** | Most are on `self.device.create_*` which can fail on device loss. A device loss would cause cascading panics instead of a clean error recovery path. | Replace with `?` propagation or `expect("descriptive message")` |
| 514 | `sha256_digest()[...][..8]` | **Low** | Static array slice, always valid | — |
| 5734 | `as u32` on buffer.size (u64) | **Low** | Only triggers above 4 GiB | Use `u64::min(value, u32::MAX as u64) as u32` |

**Unsafe blocks**: 0 found in this file (the GPU abstraction is handled by wgpu).

---

### `src/types.rs` (1,641 lines)

#### 1) Bug Identification

**None found.** All arithmetic is straightforward. Indices come from internal state or iterators, not untrusted external input.

#### 2) Security-Minded Checks

- All types are internal. No deserialization entry points.
- `ByFieldDrawData` and buffer structs implement `Pod`/`Zeroable` for safe GPU transfer.

#### 3) Monolithic File Decomposition

1,641 lines with ~12 distinct type groups. Decomposition candidate.

Proposed split:
- `types/svg.rs` — `SvgNodeParams`, `Stretch`
- `types/draw.rs` — `DrawCmd`, `DrawCmdKind`
- `types/context.rs` — `RenderContext`
- `types/particle.rs` — `Particle`, `ParticleProps`
- `types/buffer.rs` — `ByFieldDrawData`, `VertexBuffer`
- `types/text.rs` — `TextSegment`
- `types/svg_subsystem.rs` — `SvgSubsystemState`
- `types/quality.rs` — `DynamicQuality`
- `types/scene.rs` — `SceneUniforms`
- `types/test.rs` — Test utilities

#### 4) Fanciful/Themed Naming

None found. All identifiers are descriptive.

#### 5) unwrap()/expect() + unsafe Combination Audit

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| ~400 | `.unwrap()` on render pass binding | **Low** | `get_bind_group` can only fail if the key doesn't exist; it's guaranteed to exist by context construction |

**Unsafe blocks**: 0.

---

### `src/draw.rs` (119 lines)

#### Bug Identification

**Bug 3 — `dur="indefinite"` parsed as 1 second (lines 20-28)**

The `ParseDuration` function reads an SVG duration string. For `"indefinite"` (the SVG spec sentinel for indefinite duration), the code falls through to the default branch which returns `Some(1.0)` (1 second). SVG spec says `indefinite` means the animation runs forever with no set end.

**How to trigger**: Write an SVG animation element with `dur="indefinite"`. The animation will prematurely end after 1 second instead of running indefinitely.

#### Other Checks
- Security: No external data entry — this function operates on trusted SVG DOM.
- Decomposition: 119 lines, no candidate.
- Theming: None.
- Unwrap/unsafe: None.

---

### `src/vertex.rs` (138 lines)

- **Bugs**: None. Clean vertex buffer abstraction.
- **Security**: No external data entry.
- **Decomposition**: 138 lines, no candidate.
- **Theming**: None.
- **Unwrap/unsafe**: None.

---

### `src/material.rs` (1,226 lines)

#### Bug Identification

**Bug 4 — `generate_builtins_wgsl()` startup panic (line 1039)**

The material graph compiler generates built-in WGSL shader functions at startup. The `ShaderCompiler::compile()` call at line 1039 uses `.unwrap()` — if any builtin compilation fails, the entire renderer panics at startup (no graceful error, no fallback). The builtins are hardcoded strings in code, so this would be a developer error during development, but in production a GPU driver bug could trigger it.

**How to trigger**: A platform-specific WGSL compiler bug in one of the ~30 builtin shaders causes compilation failure. The renderer hard-crashes on startup.

#### Security

- Material graph input is from file/asset loading. `builtin_names` are hardcoded. No user shader injection path identified.
- Graph IDs and node indices are validated to exist in the graph before access.

#### Decomposition

1,226 lines, 3 distinct responsibilities: material graph types, graph compiler, builtin generation. Decomposition candidate.
Proposed: `material/mod.rs` (types + graph), `material/compile.rs` (compiler), `material/builtins.rs` (builtin generation).

#### Theming

- `MaterialNode::forge()`, `MaterialBuilder::forge()` — "forge" themed constructor. ~8 call sites.

#### Unwrap/unsafe

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| 1039 | `ShaderCompiler::compile(...).unwrap()` | **High** | Panics on any builtin compile failure — see Bug 4 |
| Various | `builder.graph.nodes.get(i).unwrap()` | **Low** | Graph index guaranteed valid by builder construction |

**Unsafe blocks**: 0.

---

### `src/color_blindness.rs` (248 lines)

- **Bugs**: None. Matrix values from Brettel et al. (1997), well-cited.
- `intensity.clamp(0.0, 1.0)` prevents invalid values.
- `#[repr(u32)]` on enum ensures deterministic `mode as u32` mapping.
- `bytemuck::Pod + Zeroable` well-justified for `#[repr(C)]` uniform struct.
- **Unwrap/unsafe**: None.

---

### `src/pyramid.rs` (87 lines) / `src/svg_filter_graph.rs` (61 lines) / `src/accessibility.rs` (42 lines)

- All clean. No bugs, no unwrap/unsafe, no themed names.
- `pyramid.rs` uses `saturating_sub` + `min` for safe mip indexing.

---

### Passes (`src/passes/`) — 13 files

#### Bugs

| File | Lines | Bug | Severity |
|------|-------|-----|----------|
| `glass.rs` | 539 | **P2-7 scissor fix not applied**. Uses `set_scissor_rect(0, 0, 1, 1)` which draws 1x1 pixel instead of `(0, 0, 0, 0)` (wgpu spec: zero area = no draw). Only glass pass missed the fix. | **LOW** |
| `ui.rs` | 115 | Same P2-7 fix missed — 1x1 pixel scissor instead of zero-area. | **LOW** |
| `backdrop_region.rs` | 50, 54 | **`.expect()` on registry texture get** — panics if scene texture or blur target isn't registered. Other passes use `match` + `log::error!` + `return` for this. | **HIGH** |
| `accessibility.rs` | 58 | **`.unwrap()` on registry texture view get** — same pattern, panics on miss. | **HIGH** |
| `pyramid.rs` | 20 | **`.unwrap()` on registry mip view get** — same pattern in loop. | **HIGH** |

#### Theming (wgpu debug labels)

All 13 passes use themed label strings. Proposed replacements:

| Current Label | Proposed |
|-------------|----------|
| `"Surtr Geometry Pass"` | `"Geometry Pass"` |
| `"Surtr Glass Pass"` | `"Glass Pass"` |
| `"Surtr UI Pass"` | `"UI Pass"` |
| `"Surtr Composite Pass"` | `"Composite Pass"` |
| `"Surtr Bloom Pass"` | `"Bloom Pass"` |
| `"Surtr Effects Pass"` | `"Effects Pass"` |
| `"Surtr Volumetric Pass"` | `"Volumetric Pass"` |
| `"Surtr BackdropRegion Pass"` | `"BackdropRegion Pass"` |
| `"Surtr Pyramid Pass"` | `"Pyramid Pass"` |
| `"Surtr Tonemap Pass"` | `"Tonemap Pass"` |
| `"Surtr Accessibility Pass"` | `"Accessibility Pass"` |
| `"Surtr SVG Filter Pass"` | `"SVG Filter Pass"` |

#### Unwrap/unsafe — all passes

| File | Line | Risk | Fix |
|------|------|------|-----|
| `backdrop_region.rs` | 50 | **HIGH** — .expect() panics on missing scene texture | `match` + `log::error!` + `return` |
| `backdrop_region.rs` | 54 | **HIGH** — .expect() panics on missing blur target | Same |
| `accessibility.rs` | 58 | **HIGH** — .unwrap() panics on missing scene view | Same |
| `pyramid.rs` | 20 | **HIGH** — .unwrap() in loop on missing mip view | Same |

---

### Kvasir (`src/kvasir/`) — 8 files

#### Bugs

| File | Line | Bug | Severity |
|------|------|-----|----------|
| `nodes.rs` | 86 | **u64->u32 truncation in resource ID**: `offscreen.target_id as u32` silently truncates target IDs > u32::MAX, causing resource aliasing between different offscreen targets. | **LOW** |
| `nodes.rs` | 118 | Portal region ID overflow at > 4B portal regions — extremely unlikely | **INFO** |

#### Theming

- `KvasirNode` trait -> `RenderGraphNode`
- `KvasirGraph` struct -> `RenderGraph`
- `kvasir/` module -> `render_graph/`

#### Unwrap/unsafe

All Low or test-only. **Unsafe blocks: 0 in entire kvasir directory.**

---

### Subsystems (`src/subsystems/`) — 3 files

#### `config.rs` (89 lines)

- `forge()` method (themed constructor). ~2 call sites.
- 18 `.unwrap()` calls internally — all on `serde_yaml::from_str` which panics if config file is malformed. **Low risk** (config loaded once at startup).

#### `gpu_capabilities.rs` (177 lines)

- Clean. No bugs, no theming, no unwrap/unsafe.

#### `geometry_buffers.rs` (123 lines)

- **Bug 5 — `max_capacity` guard silently ignored (line 87)**: `(self.max_capacity + 1) * chunk_size` where `+1` makes the guard always pass. Should be `self.max_capacity * chunk_size` if `max_capacity` means what the name says.
- 3 themed label strings (`"forge"`, `"berserker_bind_group"`).
- `forge()` constructor -> `new()`.

---

## Aggregate Plan

### Prioritized Bug/Security Fix List

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | **HIGH** | backdrop_region.rs:50,54 | `.expect()` on registry get — total panic on resource miss | `match` + `log::error!` + `return` |
| 2 | **HIGH** | accessibility.rs:58 | `.unwrap()` on registry get — panics on miss | Same pattern |
| 3 | **HIGH** | pyramid.rs:20 | `.unwrap()` on registry get — panics on miss | Same pattern |
| 4 | **MED** | renderer.rs:517-521,6107-6111 | SHA256 hash truncated to 64 bits — collision feasible for targeted attack | Compare all 32 bytes |
| 5 | **MED** | material.rs:1039 | `unwrap()` on builtin compile — crashes renderer on any startup compiler failure | `expect("builtin: {name}")` or propagate error |
| 6 | **LOW** | nodes.rs:86 | u64->u32 truncation in offscreen resource ID | Validate target_id range |
| 7 | **LOW** | glass.rs:539, ui.rs:115 | P2-7 fix missing — draws 1x1 pixel on zero-area scissor | `set_scissor_rect(0, 0, 0, 0)` |
| 8 | **LOW** | draw.rs:20-28 | `dur="indefinite"` parsed as 1 second | Handle `"indefinite"` as infinite |
| 9 | **LOW** | renderer.rs:5734 | u32 overflow in VRAM capture size | Saturating cast or use u64 |
| 10 | **INFO** | geometry_buffers.rs:87 | `max_capacity` guard silently ignored (+1 bug) | Fix formula |

### File Decomposition Plan

| Priority | File | Lines | Proposed Split |
|----------|------|-------|----------------|
| 1 | `renderer.rs` | 6,636 | `init.rs`, `frame.rs`, `pipelines.rs`, `cache.rs`, `capture.rs` |
| 2 | `types.rs` | 1,641 | `svg.rs`, `draw.rs`, `context.rs`, `particle.rs`, `buffer.rs`, `text.rs`, `svg_subsystem.rs`, `quality.rs`, `scene.rs`, `test.rs` |
| 3 | `material.rs` | 1,226 | `material/mod.rs`, `material/compile.rs`, `material/builtins.rs` |
| 4 | `glass.rs` | 551 | `backdrop_copy.rs`, `backdrop_blur.rs`, `glass.rs` |

### Renaming Plan (all files)

| File | Old Name | Kind | New Name |
|------|----------|------|----------|
| config.rs | `SurtrConfig` | struct | `RendererConfig` |
| All passes | `"Surtr P*"` labels | label strings | `"Geometry Pass"`, `"Glass Pass"`, `"UI Pass"`, `"Composite Pass"` etc. |
| geometry_buffers.rs | `"berserker_bind_group"` | label string | `"shared_bind_group"` |
| geometry_buffers.rs | `forge()` | method | `new()` |
| types.rs | `forge()` | method | `new()` |
| kvasir/node.rs | `KvasirNode` | trait | `RenderGraphNode` |
| kvasir/graph.rs | `KvasirGraph` | struct | `RenderGraph` |
| kvasir/ | kvasir/ | module | `render_graph/` |

### Unwrap/Unsafe Remediation — High Severity Replacement Code

**1. backdrop_region.rs:50-54:**
```rust
let scene_tex = match ctx.registry.get_texture(RES_SCENE) {
    Some(v) => v,
    None => { log::error!("[BackdropRegion] Missing scene texture"); return; }
};
let blur_tex = match ctx.registry.get_texture(self.output_id) {
    Some(v) => v,
    None => { log::error!("[BackdropRegion] Missing blur target"); return; }
};
```

**2. accessibility.rs:56-58:**
```rust
let scene_view = match ctx.registry.get_texture_view(RES_SCENE) {
    Some(v) => v,
    None => { log::error!("[Accessibility] Missing scene view"); return; }
};
```

**3. pyramid.rs:19-21:**
```rust
for mip in 0..pyramid.levels as usize {
    match self.registry.get_texture_view(pyramid.mips[mip]) {
        Some(v) => views.push(v),
        None => {
            log::error!("[Pyramid] Missing mip {} view", mip);
            continue;
        }
    };
}
```

---

## Files Not Audited

- `src/shaders/*.wgsl` (~20 files) — WGSL shader code, not Rust. Separate audit pass needed for shader-specific issues (infinite loops, uninitialized variables, precision loss).

---

# cvkg-core Engineering Audit

**Date**: 2026-06-20
**Model**: deepseek-v4-flash-free
**Scope**: 20 `.rs` files (13,704 total lines)

## Step 0: Crate Orientation

- **Responsibility**: Foundational types, traits, and modifiers for the CVKG UI framework. Defines `View`, `Renderer`, `State`, `Event`, `NodeId` (KvasirId), layout math, accessibility primitives, and modifier combinators. Also implements the `Sleipnir` spring physics solver.
- **Intra-workspace dependencies**: `cvkg-runic-text` (text shaping)

## Step 1: Per-File Findings

### `src/lib.rs` (9,556 lines)

#### Bugs

| # | Line Range | Description | Severity |
|---|-----------|-------------|----------|
| 1 | ~700-2800 | **~30 `ViewModifier` implementations — all identical delegation pattern**, each generating distinct compiled code. This adds ~200KB+ to binary size with no behavioral difference. | **INFO** |
| 2 | 3668-3692 | **Unsafe `Arc::from_raw` on deserialized `KvasirIdSleipnirJoint`**. The `unsafe` block reconstructs an `Arc<Mutex<SleipnirSolver>>` from a raw pointer stored in a serialized `u64`. If the serialized data is stale (deserialization desync), the reconstructed Arc pointer is dangling, causing use-after-free on solver access. The invariant relies on the serialized `u64` being a still-valid pointer from a previous `Arc::into_raw` call in the **same process** (serialization is ephemeral, not persistent). See unsafe analysis below. | **MED** |
| 3 | 3475-3665 | **`SleipnirSolver::step()` assumes positive dt**. No dt validation. If `dt <= 0.0`, the Euler integration produces incorrect results (frozen or reversed physics). | **INFO** |

#### Security

- `Modifier::bifrost()`, `gungnir()`, `mjolnir_shatter()`, `sleipnir()` — these take f32 parameters and store them as modifier state. No untrusted input validation (but parameters come from application view code, not external).
- `KvasirIdSleipnirJoint` serialization stores an `Arc` pointer as `u64` — this is inherently memory-unsafe across process boundaries (see Bug 2).
- No hardcoded secrets.

#### Decomposition

9,556 lines with ~60+ distinct responsibilities. **Highest-priority decomposition candidate in the codebase.**

Proposed split into ~20 files:

| New File | Contents |
|----------|----------|
| `lib.rs` (reduced) | Module declarations, re-exports |
| `rect.rs` | Rect, Size, Offset, padding utilities |
| `color.rs` | Color, linear/srgb conversion |
| `transform.rs` | Transform2D, Transform3D, matrix math |
| `view.rs` | `View` trait |
| `state.rs` | `State` trait, `AnyState`, `IntoState` |
| `renderer.rs` | `Renderer` trait, `FrameRenderer` |
| `modifier.rs` | `ViewModifier` trait |
| `modifier/bifrost.rs` | BifrostModifier (frosted glass) |
| `modifier/bifrost_bridge.rs` | BifrostBridgeModifier (shared element) |
| `modifier/gungnir.rs` | GungnirModifier (glow), GungnirPulseModifier |
| `modifier/mjolnir.rs` | MjolnirSliceModifier, MjolnirShatterModifier |
| `modifier/sleipnir.rs` | SleipnirModifier, SleipnirParams, SleipnirSolver |
| `modifier/fafnir.rs` | FafnirModifier (usage growth) |
| `modifier/mimir.rs` | MimirIntentModifier |
| `modifier/kvasir_vibe.rs` | KvasirVibeModifier |
| `modifier/odins_eye.rs` | OdinsEyeModifier |
| `modifier/mani_glow.rs` | ManiGlowModifier |
| `modifier/magnetic.rs` | MagneticModifier |
| `event.rs` | Event enum, EventResponse |
| `node_id.rs` | KvasirId (NodeId) |
| `security.rs` (existing) | SecurityModule, permissions |
| `accessibility.rs` | AriaProps |
| `animation.rs` | Animation types |

#### Theming

| Identifier | Kind | Actual Function | Proposed Name |
|-----------|------|----------------|---------------|
| `BifrostModifier` | struct | Frosted glass backdrop effect | `FrostedGlassModifier` |
| `BifrostBridgeModifier` | struct | Shared element transition | `SharedElementModifier` |
| `GungnirModifier` | struct | Neon glow effect | `NeonGlowModifier` |
| `GungnirPulseModifier` | struct | Pulsing glow animation | `PulsingGlowModifier` |
| `MjolnirSliceModifier` | struct | Slice transition (like a guillotine) | `SliceTransitionModifier` |
| `MjolnirShatterModifier` | struct | Shatter/warp transition | `ShatterTransitionModifier` |
| `SleipnirParams` | struct | Spring physics parameters | `SpringParams` |
| `SleipnirSolver` | struct | Spring physics solver | `SpringSolver` |
| `SleipnirModifier` | struct | Spring-animated modifier | `SpringModifier` |
| `FafnirModifier` | struct | Growth tracking modifier | `UsageGrowthModifier` |
| `MimirIntentModifier` | struct | Pointer anticipation modifier | `PointerAnticipationModifier` |
| `KvasirVibeModifier` | struct | Cognitive load/engagement modifier | `CognitiveLoadModifier` |
| `OdinsEyeModifier` | struct | Telemetry/observability modifier | `ObservabilityModifier` |
| `ManiGlowModifier` | struct | Proximity-based glow | `ProximityGlowModifier` |
| `MagneticModifier` | struct | Cursor attraction modifier | `CursorAttractionModifier` |
| `YggdrasilTokens` | struct | Token/state management | `TokenManager` |
| `BerserkerMode` | enum | Predictive rendering mode | `PredictiveRenderMode` |
| `KvasirId` | struct | Unique node identifier | `ViewId` |
| `BIFROST_REGISTRY` | static | Shared element tracking | `SHARED_ELEMENT_REGISTRY` |

19 themed identifiers in total, mostly modifier structs. Renaming would affect ~200+ call sites across the codebase.

#### Unwrap/unsafe

| Line(s) | Expression | Risk | Reasoning |
|---------|-----------|------|-----------|
| 3678 | `unsafe { Arc::from_raw(ptr as *const ...) }` | **MED** | Reconstructs Arc from deserialized u64 pointer. If stale/cross-process, use-after-free. Invariant: only used within same process, serialization is ephemeral. |
| 3475-3665 | ~8× `.unwrap()` on Mutex/RwLock locks | **LOW** | RwLock poison — cascading panic if a writer panicked while holding the lock |
| ~10 | Various `.unwrap()` on RwLock locks | **LOW** | Same poison pattern throughout |
| ~4 | `.unwrap()` on runtime creation | **LOW** | At startup — would fail fast on bad initialization |

### `src/security.rs` (87 lines)

- Good: `SandboxLimits` with default bounds (max_textures: 256, max_script_calls: 1000, max_network_requests: 50).
- Missing: Actual sandbox enforcement mechanism (currently returns `Error::SandboxViolation` unconditionally).
- No bugs, no theming, no unwrap/unsafe.

### `src/scene_graph.rs` (65 lines) and module files

- Module routers only. No bugs.

---

## Aggregate Plan — cvkg-core

### Prioritized Fix List

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | **MED** | lib.rs:3678 | Unsafe `Arc::from_raw` from serialized u64 pointer — potential use-after-free | Replace with ID-based solver lookup (HashMap<KvasirId, Solver>) instead of pointer serialization |
| 2 | LOW | lib.rs:3475+ | ~10 `.unwrap()` on Mutex/RwLock — poison risk | Use `.unwrap_or_else(\|e\| e.into_inner())` or `ok()` |

### Decomposition

| Priority | File | Lines | Proposed Split |
|----------|------|-------|----------------|
| 1 | `lib.rs` | 9,556 | Split into ~20 files (see full listing above) |

### Renaming

See full table above — 19 themed identifiers proposed for renaming.

### Unwrap/Unsafe Remediation

**High-severity unsafe** (lib.rs:3678): Replace Arc pointer serialization with ID-based solver map:
```rust
// Replace the unsafe Arc::from_raw pattern with:
struct SleipnirJoint {
    solver_id: u64,
}
// And maintain a global: Arc<Mutex<HashMap<u64, Arc<Mutex<SleipnirSolver>>>>>
// Serialize: solver_id
// Deserialize: look up solver_id in map (returns None if ID processed)
```

---

# cvkg-compositor Engineering Audit

**Date**: 2026-06-20
**Model**: deepseek-v4-flash-free
**Scope**: 4 `.rs` files (846 total lines)

## Step 0: Crate Orientation

- **Responsibility**: Compositor engine — flattens the layer tree into a draw command stream, routes commands into GPU-friendly buckets, handles damage regions, and renders the final frame.
- **Intra-workspace dependencies**: `cvkg-core` (RenderCommand, Layer, DamageInfo), `cvkg-scene`

## Step 1: Per-File Findings

### `src/engine.rs` (346 lines)

- **No bugs.** Tree traversal is clean. `z_counter` wraps at 2^32 but that's billions of draw commands — not a real issue.
- `flatten_layer` processes root layers in order, children in reverse for painter's algorithm. Correct.
- **Security**: No external data entry.
- **Decomposition**: 346 lines, single responsibility. Not a candidate.
- **Theming**: None. All functional identifiers (`CompositorEngine`, `CommandBuckets`, `RoutedDrawCommand`, `flatten_and_route`).
- **Unwrap/unsafe**: None.

### `src/layer.rs` (286 lines)

- **No bugs.** `LayerTree::allocate_id` starts at 1, increments. u64 — effectively unbounded.
- `LayerTree::remove_layer` iterates all values and retains children. O(n), fine for typical layer counts.
- **Security**: `Material` enum and `Layer` struct both derive `Serialize`/`Deserialize`. Could be deserialized from user data. An attacker could craft a Layer with very long `children` Vec for DoS, but in practice layers are built by application code.
- **Decomposition**: 286 lines, single responsibility. Not a candidate.
- **Theming**: None.
- **Unwrap/unsafe**: None.

### `src/template.rs` (178 lines)

- **Bug (INFO)**: Template version validation at line 105 only rejects `> VERSION`. Version 0 (default from corrupted data) passes. Fix: also reject `version == 0`.
- **Security**: `save_to_file`/`load_from_file` on caller-provided paths — path traversal risk if filename comes from untrusted source (doesn't in current architecture). `serde_json::from_str` is safe against code execution.
- **Unwrap/unsafe**: None.

### `src/lib.rs` (36 lines)

- Module declarations only. Clean.

---

## Aggregate Plan — cvkg-compositor

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | INFO | template.rs:105 | Version 0 passes validation | Add `\|\| template.version == 0` check |

Renaming needed: None. Unwrap/unsafe remediation: None.

---

# cvkg-vdom Engineering Audit

**Date**: 2026-06-20
**Model**: deepseek-v4-flash-free
**Scope**: 4 `.rs` files (2,681 total lines)

## Step 0: Crate Orientation

- **Responsibility**: Virtual DOM tree management — tree diffing (LIS-based keyed child diffing), state reconciliation patches (`VDomPatch`), event dispatch with bubbling, hit testing (SDF-based), accessibility (AccessKit integration), and Signals-based reactivity.
- **Intra-workspace dependencies**: `cvkg-core` (View, Renderer, Event, NodeId), `cvkg-runic-text`, `cvkg-scene`

## Step 1: Per-File Findings

### `src/lib.rs` (2,340 lines)

#### Bugs

| # | Line Range | Description | Severity |
|---|-----------|-------------|----------|
| 1 | 987, 1007, 1021, 1030, 1052-1057, 1067, 1097-1100, 1119-1122, 1140, 1161, 1174-1179 | **~20 `serde_json::to_value(X).unwrap()` calls that panic on NaN/Infinity**. If any f32 parameter (radius, width, intensity, angle, etc.) is NaN or Infinity, `serde_json::to_value()` returns Err, and `.unwrap()` panics. Trigger: `renderer.fill_rounded_rect(rect, f32::NAN, color)`. | **LOW** |
| 2 | 1529 | **diff_node P0-7 handler comparison `.unwrap()`**. `a.get(k).unwrap()` is safe by construction (key verified present), but stylistically fragile. | **INFO** |
| 3 | 1797-1805 | **DropdownOverlay child-list clone on every hit test**. Clones entire `node.children` Vec per frame even when no overlay is present. | **INFO** |

**SDF hit testing** (lines 1713-1836): Correct. SDF distance function handles Rect, RoundedRect, Circle.

**LIS-based keyed child diffing** (lines 1590-1666): Correct standard LIS implementation.

**Event dispatch** (lines 1839-2071): Correct. Uses captured target for drag sequences, updates focus/capture/hover state, bubbles up parent chain.

#### Security

- `VNode`, `VDomPatch` — both `Serialize`/`Deserialize`. Custom deserialize creates stub handlers (log warning + no-op). Deserialization from untrusted source could produce arbitrary tree structure, but in practice comes from application state.
- No hardcoded secrets, no path traversal.

#### Decomposition

2,340 lines, organized into clear sections. Borderline candidate but functional in single file:

| Section | Lines | Content |
|---------|-------|---------|
| Types | 1-524 | VNode, VDomPatch, A11yNodeEntry, AriaProps, LayoutRect |
| VDom + build | 526-604 | VDom struct, build(), clear_and_retain_capacity() |
| VDom validate | 605-668 | validate_sync with SceneGraph |
| VNodeRenderer | 687-1251 | VDOM-capturing renderer, decorative batching |
| VDom diff/apply | 1253-1711 | apply_patches, diff, diff_node, LIS |
| VDom hit test | 1712-1836 | SDF distance, hit_test, hit_test_recursive |
| VDom events | 1838-2071 | dispatch_event, bubble_event, dispatch_event_to_target |
| Focus + AccessKit | 2073-2305 | focus_next/prev, build_accesskit_tree |
| use_state | 2307-2340 | State management hook |

#### Theming

- `KvasirId` re-exported from cvkg-core
- `"BerserkerRoot"` string constant (line 950) — component type for root application view
- Method names from Renderer trait: `bifrost`, `gungnir`, `mjolnir_shatter`, `mjolnir_slice`, `draw_mjolnir_bolt` — all trait implementations, not new names

#### Unwrap/unsafe

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| 987+ | ~20× `serde_json::to_value(...).unwrap()` | **Low** | Panics on NaN/Infinity float values. Triggered by developer error. |
| 1529 | `a.get(k).unwrap()` | **Low** | Safe by construction (key verified present in map) |
| 2319 | `arc_val.read().unwrap().clone()` | **Low** | RwLock poison — cascading panic if a writer panicked |
| 2169-2175 | `.lock().ok().and_then(...).unwrap_or(...)` | **Low** | Uses `ok()` to handle poison, safe |

**Unsafe blocks**: **None** in vdom crate.

### `src/signals.rs` (127 lines)

#### Bugs

| # | Line | Description | Severity |
|---|------|-------------|----------|
| 1 | 49-51 | **No subscriber deduplication**. Comment acknowledges: "In a production-grade implementation, we would deduplicate subscriptions." If `signal.get()` is called in a loop within an effect, the effect is subscribed N times and fires N times per signal change. In a cycle, this causes exponential blow-up. | **LOW** |

#### unwrap/unsafe

| Line | Expression | Risk |
|------|-----------|------|
| 47 | `current.read().unwrap()` | Low — RwLock poison |
| 48 | `self.subscribers.write().unwrap()` | Low — RwLock poison |
| 54 | `self.value.read().unwrap().clone()` | Low — RwLock poison |
| 65 | `*self.value.write().unwrap() = new_value` | Low — RwLock poison |
| 67 | `self.subscribers.read().unwrap().clone()` | Low — RwLock poison |

No unsafe blocks.

### `src/animated.rs` (45 lines) and `src/physics.rs` (69 lines)

- **Bugs**: None. Spring physics is correct (Hooke's law + velocity damping, Euler integration).
- **Theming**: None.
- **Unwrap/unsafe**: physics.rs:45 — `self.velocity.write().unwrap()` — Low risk.

---

## Aggregate Plan — cvkg-vdom

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | LOW | lib.rs:987+ | NaN/Infinity in decorative params panics | Replace `.unwrap()` with `.unwrap_or(Value::Null)` or NaN-check before serialization |
| 2 | LOW | signals.rs:51 | No subscriber deduplication — exponential re-fire | Use `HashSet` or check before push |

Renaming: Minor (`KvasirId` re-export, `"BerserkerRoot"` string — minimal impact).
Unwrap/unsafe: All Low, no urgent fixes needed.

---

# cvkg-scene Engineering Audit

**Scope**: 3 `.rs` files (1,327 total lines)

## Step 0: Crate Orientation

- **Responsibility**: Retained scene graph with spatial partitioning (spatial hash grid + Quadtree) for accelerated culling and hit-testing.
- **Dependencies**: `cvkg-core`, `cvkg-spatial`, `cvkg-runic-text`

## Step 1: Per-File Findings

### `src/lib.rs` (834 lines)

#### Bugs

| # | Line | Description | Severity |
|---|------|-------------|----------|
| 1 | 176, 183 | **`.unwrap()` on `get(&root_id)` and `get_mut(&id)` in `update_transforms`/`update_node_transform`**. Safe by construction for normal use, but `deserialize_binary()` restores `root` from untrusted binary data without validation. Corrupted binary that references non-existent root causes panic. | **LOW** |
| 2 | 367 | **`deserialize_binary` sets `next_id = 0`**. Subsequent `next_id()` returns `KvasirId(0)`, colliding with deserialized node IDs. | **LOW** |
| 3 | 154 | **`add_node` with `parent: None` after root is set creates orphan**. Node inserted into `self.nodes` but never linked into tree. | **INFO** |

#### Security

- `VNode` is `Serialize`/`Deserialize`. Binary deserialization has no input validation — crafted data could cause arbitrary allocation.
- No hardcoded secrets.

#### Theming

- File doc comment mentions "Surtr GPU pipeline" (line 23) — doc reference only.
- `KvasirId` re-exported.

#### Unwrap/unsafe

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| 176 | `.unwrap()` on root get in `update_transforms` | **Low** | Panics if root doesn't exist (corrupted deserialization) |
| 183 | `.unwrap()` on child get in `update_node_transform` | **Low** | Same — panics on missing child |
| 345 | `.unwrap_or(0.0)` in batch sorting | **Low** | Safe — missing node defaults to z=0 |

**Unsafe blocks**: **None**.

#### Decomposition

834 lines, well-organized. Not a decomposition candidate.

### `src/quadtree.rs` (128 lines)

- **No bugs**. Standard quadtree implementation. `max_depth=5`, `max_rects=10`.
- **No theming, no unwrap/unsafe, no security issues.**

### `src/test_renderer.rs` (365 lines)

- **No bugs**. Standard recording renderer for test snapshots.
- **Theming**: `Command` enum variants include `Bifrost`, `Gungnir`, `PushMjolnirSlice`, `PopMjolnirSlice` — trait implementations from `cvkg_core::Renderer`.
- **No unwrap/unsafe.**

---

## Aggregate Plan — cvkg-scene

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | LOW | lib.rs:367 | `next_id = 0` after deserialization — collision risk | Set `next_id` to max existing node id + 1 |
| 2 | LOW | lib.rs:176,183 | `.unwrap()` on root/child get in transform update — latent panic on corrupted binary | Add node existence check before unwrap, or validate deserialized graph on load |

---

## Quick Reference: Crates Audited

| Crate | Files | Lines | Status |
|-------|-------|-------|--------|
| `cvkg-render-gpu` | 32 | ~22,000 | DONE |
| `cvkg-core` | 20 | 13,704 | DONE |
| `cvkg-compositor` | 4 | 846 | DONE |
| `cvkg-vdom` | 4 | 2,681 | DONE |
| `cvkg-scene` | 3 | 1,327 | DONE |
| `cvkg-svg-filters` | 1 | 4,020 | DONE |
| `cvkg-runic-text` | 6 | 5,521 | DONE |
| `cvkg-components` | ~30 | ~30,000 | DONE |
| `cvkg-layout` | 1 | 2,810 | DONE |
| `cvkg-svg-serialize` | 3 | 930 | DONE |
| _remaining 23 crates_ | — | — | PENDING |

---

# cvkg-svg-filters Engineering Audit

**Date**: 2026-06-20
**Model**: deepseek-v4-flash-free
**Scope**: 1 `.rs` file (4,020 lines)

## Step 0: Crate Orientation

- **Responsibility**: WGPU-based SVG filter primitive evaluation. Parses `usvg::filter::Filter` into a DAG, topologically sorts it (Kahn's algorithm), then evaluates each primitive via GPU render passes.
- **Dependencies**: wgpu, usvg, bytemuck

## Step 1: Per-File Findings

### `src/lib.rs` (4,020 lines)

#### Bugs

| # | Line | Description | Severity |
|---|------|-------------|----------|
| 1 | 789 | **`.unwrap_or(input_view)` when LUT is missing**. If no LUT is uploaded and `render_pass` is called with a mode that samples the LUT texture (mode 17), the shader reads from the input view at binding 5 instead of the LUT. The formats differ (Rgba8Unorm vs Rgba32Float for LUT), so the shader receives garbage data in that mode. | **LOW** |
| 2 | 2486-2492 | **`TransientFilterPool::release()` matches by (width, height) only**. If two textures of the same size are in flight simultaneously, `release(width, height)` releases the wrong one. | **LOW** |
| 3 | 996 | **`.last().unwrap()` on graph nodes**. Safe by construction (empty graph is handled at line 936). Style: minor readability concern. | **INFO** |
| 4 | 1158 | **ColorMatrix offset values `values[4]` etc. assume 20-element array**. If `ColorMatrixKind::Matrix(values)` has < 20 values, this panics with OOB. SVG spec guarantees exactly 20 values, but this is an assumption on parsed data. | **INFO** |

#### Security

- `upload_image()` validates `data.len() == width * height * 4` — good.
- `from_usvg_filter()` validates lighting/turbulence parameters (P1-31/32 fix).
- No deserialization from untrusted sources; input comes from `usvg` parser (but usvg itself may have deserialization bugs — not in scope).
- No hardcoded secrets.

#### Decomposition

4,020 lines, single file. Well-structured by function groups. Decomposition candidate:
- `lib.rs` (reduced) — types, error, re-exports
- `graph.rs` — FilterGraph, topological sort, input resolution
- `engine.rs` — FilterEngine, render_pass, evaluate
- `primitives.rs` — Individual filter primitives (GaussianBlur, ColorMatrix, etc.)
- `pool.rs` — TransientFilterPool

#### Theming

**None.** All identifiers are functional (FilterGraph, FilterEngine, GpuContext, FilterUniforms, TransientFilterPool, etc.).

#### Unwrap/unsafe

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| 789 | `.unwrap_or(input_view)` | **Low** | Fallback LUT uses input view — wrong format for LUT mode, but no panic |
| 996 | `.last().unwrap()` | **Low** | Safe by construction (empty graph handled at line 936) |
| 2481 | `.last().unwrap()` in pool.acquire | **Low** | Safe by construction (just pushed to vec) |
| 2256+ | Various `.unwrap()` in tests | **Low** | Test code — acceptable |

**Unsafe blocks**: **None** in this crate.

---

## Aggregate Plan — cvkg-svg-filters

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | LOW | lib.rs:789 | LUT fallback to input on missing LUT — wrong format | Check mode; skip LUT binding or use dedicated fallback texture |
| 2 | LOW | lib.rs:2486-2492 | Pool release matches by (width, height) only — can release wrong texture | Track by unique ID or use `position()` with `in_use && matching dimensions && matching order` |
| 3 | INFO | lib.rs:1158 | ColorMatrix assumes 20 values — OOB panic on malformed data | Validate `values.len() >= 20` before indexing |

Renaming: None. Unwrap/unsafe: All low, no urgent fixes.

---

# cvkg-runic-text Engineering Audit

**Date**: 2026-06-20
**Model**: deepseek-v4-flash-free
**Scope**: 6 `.rs` files (5,521 total lines)

## Step 0: Crate Orientation

- **Responsibility**: Text shaping, layout, and font rasterization using HarfBuzz (via rustybuzz) and Swash. Includes BiDi support, LRU shape cache, Knuth-Plass line breaking, MSDF glyph atlas generation, subpixel LCD rendering, and emoji detection.
- **Dependencies**: swash, rustybuzz, fontdb, unicode-bidi, unicode-segmentation, unicode-linebreak

## Step 1: Per-File Findings

### `src/lib.rs` (4,036 lines)

Major crate. Contains RunicTextEngine, TextStyle, shaping pipeline, and layout engine.

### `src/global_cache.rs` (69 lines)

#### Bugs

| # | Line | Description | Severity |
|---|------|-------------|----------|
| 1 | 23 | **`.lock().unwrap()` on global static Mutex**. If any thread panics while holding the lock (text processing), the mutex is poisoned and ALL subsequent text shaping deadlocks permanently. This is a single-point-of-failure for text rendering. | **MED** |

### `src/knuth_plass.rs` (503 lines)

#### Bugs

| # | Line | Description | Severity |
|---|------|-------------|----------|
| 2 | 172 | **Partial bounds check in Knuth-Plass**. `widths[prev_pos..curr_pos.min(widths.len())]` only checks `curr_pos`. If `prev_pos > widths.len()`, this panics with OOB slice start. | **MED** |
| 3 | 279 | `.last().unwrap()` safe by construction (checked `!lines.is_empty()` on same line) | INFO |

### `src/subpixel.rs` (498 lines) — Clean. No bugs, no unwrap/unsafe.

### `src/msdf.rs` (318 lines) — Clean. SDF generation is correct (brute-force distance with spread clipping).

### `src/emoji.rs` (495 lines) — Clean. Placeholder emoji rendering (documented limitation).

#### Security

- No external untrusted data paths. Font files are loaded from the system (fontdb) or bundled.
- `global_cache` uses a Mutex — denial-of-service possible if text shaping is starved, but this is self-inflicted.
- No hardcoded secrets.

#### Theming

The crate name "runic-text" is themed. The `RunicTextEngine` struct name.

#### Unwrap/unsafe

| Line | Expression | Risk | Reasoning |
|------|-----------|------|-----------|
| 23 (global_cache) | `.lock().unwrap()` | **MED** | Mutex poison = permanent text deadlock |
| 279 (knuth_plass) | `.last().unwrap()` | **Low** | Safe by construction (len check on same line) |

**Unsafe blocks**: **None**.

---

## Aggregate Plan — cvkg-runic-text

| # | Severity | File:Line | Issue | Fix |
|---|----------|-----------|-------|-----|
| 1 | MED | global_cache.rs:23 | `.lock().unwrap()` global Mutex — poison kills all text rendering | Use `.unwrap_or_else(\|e\| e.into_inner())` or `ok()` with logging |
| 2 | MED | knuth_plass.rs:172 | `prev_pos` not bounds-checked before slicing `widths` — OOB panic | Use `(prev_pos.min(widths.len()))..(curr_pos.min(widths.len()))` |

Theming: `RunicTextEngine` -> `TextShapingEngine` (trait renames across many crates). Low priority.

Unwrap/unsafe: Fix the global_cache Mutex pattern when poison resilience is needed.


---

# cvkg-components Engineering Audit

**Scope**: ~30 `.rs` files (~30,000 lines)
**Dependencies**: cvkg-core, cvkg-vdom

## Critical Findings

| # | Severity | File | Issue |
|---|----------|------|-------|
| 1 | **CRITICAL** | container.rs, dropdown_menu.rs, combobox.rs | **System state hash collisions**. Three different component types use identical u64 hash keys (`0xD00_0001` = COLLAPSIBLE_ANIM_HASH / DROPDOWN_OPEN_HASH / SPOTLIGHT_OPEN_HASH). Opening a combobox toggles a dropdown menu's state. Silent data corruption — no error, just wrong UI behavior. |
| 2 | **HIGH** | theme.rs:354-356 | `chat_text_user()` hardcoded pure black (`[0.0, 0.0, 0.0, 1.0]`) — invisible in dark mode. |
| 3 | **HIGH** | interactive/button.rs:997 | `delta.unwrap()` on `Option<f32>` in slider key handler. Fragile — adding a new special key without updating the outer match panics on user keystroke. |
| 4 | **MED** | input_otp.rs:147 | `chars().nth(i).unwrap()` — panics if `self.value` is shorter than `char_count` implies. |
| 5 | **MED** | interactive/input.rs:277 | `chars().next().expect(...)` — safe today but fragile. |
| 6 | **MED** | dropdown_menu.rs, button.rs | Hardcoded colors/alpha arithmetic break on dark themes. |
| 7 | **MED** | vdom dependency | 14+ `serde_json::to_value().unwrap()` calls propagating to component layer. |

---

# cvkg-layout Engineering Audit

**Scope**: 1 `.rs` file (2,810 lines)
**Dependencies**: cvkg-core (Taffy-based layout)

## Critical Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **CRITICAL** | lib.rs:1428 | `AspectRatio::place_subviews` computes Y-center as `(bounds.height - fit.height) * 0.0` instead of `* 0.5`. All aspect-ratio-constrained views are top-aligned instead of centered. |
| 2 | **HIGH** | global | **No NaN/Infinity guards anywhere.** Any f32 input (width, height, spacing) propagates silently. One NaN in a constraint = all downstream layout is NaN. |
| 3 | **MED** | various | 16 `.unwrap()` calls on Taffy tree operations (tree traversal). If Taffy node doesn't exist, panics. |
| 4 | **MED** | lib.rs:879,905 | `Flex::place_subviews` can produce negative item widths when spacing > available width. |
| 5 | **MED** | lib.rs:1038-1043 | Grid integer casts truncate (i32→i16, u32→u16). Overflow risk for very large grids. |
| 6 | LOW | lib.rs:481 | Spring animation has no timestep clamping — high delta-T produces unstable physics. |

**No unsafe. No theming.**

---

# cvkg-svg-serialize Engineering Audit

**Scope**: 1 `.rs` file (900 lines)
**Dependencies**: usvg, quick-xml

## Critical Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **HIGH** | lib.rs:197-198 | **CSS mangled by quick-xml `BytesText` escaping.** CSS child combinator `.a > .b` becomes `.a &gt; .b` (not valid CSS). Existing tests pass because test CSS has no `>`, `<`, or `"`. |
| 2 | **MED** | lib.rs:216 | `unwrap()` on `String::from_utf8(writer.into_inner())` — panics if output is non-UTF-8. |
| 3 | **MED** | lib.rs:173-174 | xmlns prefix not validated — attribute name injection possible. |
| 4 | **MED** | lib.rs:260,274 | Control characters in attribute values produce non-well-formed XML. |
| 5 | **MED** | lib.rs:572-579,681,690 | `url(#)` function breaks if element ID contains `)`. |
| 6 | LOW | lib.rs:394-416 | NaN/Infinity in `format_svg_float` produces `"NaN"`/`"inf"` in SVG output. |

**No unsafe. No theming.**

---

# cvkg-cli Engineering Audit

**Scope**: 19 `.rs` files, ~1,600 lines
**Dependencies**: cvkg-core, cvkg-vdom, cvkg-runic-text (dev runtime)
**Key areas**: CLI argument parsing (clap), plugin system, dev runtime, build pipeline, configuration, devtools WebSocket client, token export, theme command

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **MED** | main.rs:650-710 | **Theme key injection in generated Rust.** User-supplied JSON keys interpolated directly into `format!()` calls producing Rust struct definitions. A key with `] // injected\n pub malicious: [f32; 4],` could generate syntactically broken or injected Rust code. No shell injection exists (all subprocess calls use typed clap args). |
| 2 | **MED** | main.rs:640-645 | **`parse_f64` silently accepts integers > 1.0 for RGBA channels.** Tries `as_f64()` then `as_i64().map(|i| i as f64)` then `as_u64()` — silently converts value like `255` to `0.0` (since it's not 0.0–1.0), producing wrong colors. |
| 3 | **LOW** | main.rs:506-510 | 4x `.unwrap()` cascade on `current_dir()`/`file_name()`/`to_str()` — crashes if CWD deleted or path non-UTF8. |
| 4 | **LOW** | main.rs:325,341,353,381,503 | `.expect()` on `Command::status()` — panics if `cargo`/`wasm-pack` not installed instead of graceful error. |
| 5 | **LOW** | build_pipeline.rs:184 | Mutex `.unwrap()` vs `into_inner()` — poisoned mutex panics (devtools_dashboard.rs handles this correctly with `.unwrap_or_else(|e| e.into_inner())`). |
| 6 | **INFO** | main.rs:553-557 | `cp` command result silently ignored via `let _ =` — invisible failure if `assets/` missing. |
| 7 | **INFO** | ws_server.rs | WebSocket deserialization — memory exhaustion bounded by 100-msg channel capacity. |

**Zero unsafe blocks.** Shell injection: **none found** — all subprocess args are typed clap values or static strings. Plugin system: purely trait-based compile-time, no `dlopen`/`libloading` — safest possible design. Crate name validation (main.rs:574-580) uses whitelist regex `[a-z0-9_-]` with leading/trailing hyphen guard.

**Key positive**: `plugin.rs` — compile-time trait registration only. No dynamic loading. No `extern "C"` discovery. Zero attack surface from plugin loading.

---

# cvkg-webkit-server Engineering Audit

**Scope**: 4 `.rs` files, ~830 lines (main.rs 679, wasm_server.rs 144, lib.rs 7, tests/security_tests.rs 141)
**Dependencies**: axum, tower-http, wasmtime-wasi, cvkg-cli (WsMessage types)
**Key areas**: HTTP server, WebSocket proxy, WASM execution, directory serving, live reload, metrics

## Critical Finding

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **CRITICAL** | main.rs:136-216 | **Stored XSS via `/snapshot` endpoint.** Unsanitized POST body stored in `ArcSwap<Option<String>>` and rendered directly into HTML response at `/` via `format!(...)`. No validation, no sanitization, no auth. CSP with `'unsafe-inline'` makes XSS trivially exploitable. |
| 2 | **HIGH** | wasm_server.rs:32 | **WASM fuel consumption disabled** (`consume_fuel(false)`). Malicious/infinite WASM loops consume CPU forever, blocking all future ticks since `tick()` is sequential. |
| 3 | **HIGH** | wasm_server.rs:49,105,114,121 | **Mutex `.unwrap()` on poison** — if WASM operation panics while holding `Mutex<Option<WasmSession>>`, all subsequent lock acquisitions panic, crashing the server. |
| 4 | **HIGH** | main.rs:432-500 | **File watcher — unbounded recursive walk.** `scan_dir()` recurses with no depth limit every 500ms. Symlink loops cause stack overflow. Deep trees (10K+ files) consume 100% CPU. No symlink detection. |
| 5 | **HIGH** | main.rs:552-560,587-589 | **Symlink traversal via ServeDir.** Canonicalization protects against `../` but not symlinks inside served directories. A symlink to `/etc/passwd` or `../../secret.key` inside `pkg_dir`/`assets_dir`/`static_dir` would be served. |
| 6 | **MED** | main.rs:607 | **Weak CSP.** `script-src 'unsafe-inline' 'unsafe-eval'` defeats XSS protection. `frame-src *` enables clickjacking (despite `X-Frame-Options: DENY` which conflicts). |
| 7 | **MED** | main.rs:633 | **Permissive CORS** (`CorsLayer::permissive()`). Any origin can make cross-origin requests. No CSRF protection on `/snapshot` POST. |
| 8 | **MED** | main.rs:76-77, 633-636 | **Rate limiting not wired up.** `rate_limit_rps` field and `tower_governor` dependency exist but neither is used. Only `ConcurrencyLimitLayer` (100) protects against DoS. |
| 9 | **MED** | main.rs:292-296 | **No WebSocket message size limit.** `serde_json::from_str` allocates for the full message size. axum default WS limit is 64KB — large enough for repeated allocation DoS. |
| 10 | **MED** | main.rs:396,402 | **`expect()` on signal handlers** — panics if signal handler can't be installed (container, no controlling terminal). |
| 11 | **LOW** | wasm_server.rs:79-86 | **WASM preopened dir grants `DirPerms::all()` + `FilePerms::all()`** — WASM guest can write/delete host files despite comment calling it "Hardened." |
| 12 | **LOW** | main.rs:386 | **Histogram label bug**: `"method" => path` — label says method but value is URL path. Wrong metric attribution. |
| 13 | **LOW** | main.rs:360 | **Snapshot "ping" check is substring match, not exact** — `snapshot.contains("ping")` matches any string containing "ping". |
| 14 | **LOW** | tests/security_tests.rs | **Security tests are superficial** — string pattern checks on hardcoded paths, no actual server integration tests for path traversal, XSS, rate limiting, or WS DoS. |

**No unsafe blocks.** Theming: **none** in this crate.

**Blast radius**: Full server takeover via XSS (CRITICAL, #1). All others are DoS or information disclosure.

---

# cvkg-storage Note (Crate Does Not Exist)

**No `cvkg-storage` directory exists** in the workspace. Closest persistence-related code found in:

1. **cvkg-cli/src/dev_runtime.rs** — `HotReloadState` with JSON save/load
2. **cvkg-render-gpu/src/renderer.rs** — Pipeline cache file I/O (SHA256 integrity-checked)
3. **cvkg-render-native/src/lib.rs** — Asset manager file reads

## Related Findings Found During Search

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | **HIGH** | cvkg-render-native/lib.rs:2820,2866 | **Path traversal in asset loader.** `load_image()`/`preload_image()` accepts `url: &str` used directly as `std::fs::read()` path. No canonicalization, no directory jail, no symlink resolution. `../../etc/passwd` works. |
| 2 | **MED** | cvkg-cli/dev_runtime.rs:192 | **Non-atomic file writes.** `std::fs::write()` on crash/power loss produces truncated file. Should use write-to-temp + atomic `rename`. |
| 3 | **MED** | cvkg-render-gpu/renderer.rs:6112-6114 | **Cache data + hash written separately.** Crash between writes leaves orphaned cache + missing sidecar on next startup. |
| 4 | **INFO** | cvkg-test/src/lib.rs:101 | `fs::create_dir_all().unwrap()` — test-only, panics on permissions. |
| 5 | **SAFE** | all persistence code | No `unsafe` in storage-adjacent code. Serde deserialization uses bounded types only (<code>serde_json</code>, not bincode — no arbitrary code execution risk). |

---

# cvkg-physics Engineering Audit

**Scope**: 10 `.rs` files, ~6,000 lines (body, collider, shape, solver, broadphase, narrowphase, integration, xpbd, world, constraint)
**Dependencies**: cvkg-core (types)
**Key areas**: 2D physics engine with optional 3D — springs, constraints, collision detection, GJK/EPA

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **MED** | constraint.rs:291-304 + solver.rs:111-113 | **Motor constraints are dead code.** `Constraint::motor()` sets `body_a == body_b` (same body). Solver main loop checks `if idx_a == idx_b { continue; }` — skips the Motor constraint entirely. |
| 2 | **MED** | solver.rs:504 | **Hinge3D inverse inertia uses X-component only.** `total_inv_inertia` computed from `ctx.a.inv_inertia_3d.x` only, then applied uniformly as scalar to all axes — physically incorrect for 3D. |
| 3 | **MED** | broadphase.rs:53,101,153 | **`f32.floor() as i32` — UB on NaN/Infinity/out-of-range.** Physics positions are bounded by screen size in practice, but extreme values (f32::MAX) trigger UB in release mode. |
| 4 | **LOW** | solver.rs:596-618 | **`solve_motor` ignores 3D bodies.** Only reads/writes `Vec2` velocity. 3D body with motor constraint does nothing. |
| 5 | **LOW** | broadphase.rs:87-89,208-210 | **BodyId(u64).0 used as `usize`.** On 32-bit platforms, values > 2^32 silently truncate. |
| 6 | **LOW** | narrowphase.rs:245 | **Exact float comparison** `Vec2::ZERO` — works because zero is deliberately assigned but fragile under precision changes. |
| 7 | **LOW** | solver.rs:293 | **Distance constraint gives up at zero distance.** `current_dist < 1e-10` early return — breakable constraints on zero-distance anchors never break. |
| 8 | **LOW** | shape.rs:258-266 | **ConvexHull moment depends on vertex winding order.** Clockwise order produces wrong inertia (signed area cancels). |
| 9 | **LOW** | solver.rs:424-434 | **Angular limit correction split 50/50** regardless of body masses. Should distribute proportionally to `inv_inertia`. |

**Unsafe blocks**: 1 (xpbd.rs:567 — `get_three_mut()` raw pointer). Properly guarded — all indices validated for distinctness and bounds before the block. Comment explains why `split_at_mut` can't handle 3 references. **LOW risk.**

**Production unwraps**: 0 (all 22 `.unwrap()` calls are in `#[cfg(test)]` blocks).

**Unbounded loops**: 0 — solver bounded at 8 iterations, GJK at 32, EPA at 32, CCD at 32, character controller at 8.

**Decomposition**: Clean module boundaries already. No split needed.

**Theming**: None.

---

# cvkg-render-native Engineering Audit

**Scope**: 1 `.rs` file (4,276 lines) — native desktop renderer (CPU path)
**Dependencies**: cvkg-core, cvkg-render-gpu (SurtrRenderer)
**Key areas**: Window management, native event loop, asset loading, image I/O, clipboard, audio engine

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **MED** | lib.rs:3862-3868 | **f64 -> u32 saturates silently.** `scale_dimensions()`: `(logical_width * sf).round() as u32`. Product > u32::MAX silently clamps to u32::MAX. No caller-side check. |
| 2 | **MED** | lib.rs:3817-3827 | **u32 -> i32 overflow in center math.** `window_rect.2 as i32 / 2` — if width is u32::MAX, as i32 produces -1. |
| 3 | **MED** | lib.rs:925 | **Bare `.unwrap()` on HashMap lookup.** `self.window_manager.windows.get(&winit_id).unwrap()` — should succeed logically but panics on any logic error with no message. |
| 4 | **MED** | lib.rs:1248-1252 | **Thread-local raw pointer not reset on panic.** `GPU_FRAME_PTR` set at line 1248, cleared at 1252. A panic between them leaves dangling pointer for subsequent calls on same thread. Needs scope guard. |
| 5 | **MED** | lib.rs:3924-3930 | **Unsanitized `test_name` in regression test path.** `format!("{}.png", test_name)` — `test_name` containing `../` writes PNG to arbitrary location. |
| 6 | **LOW** | lib.rs:1151,1169,1188 | **3x `.unwrap()` on `diff_patches`** — safe by construction in current code but fragile to refactoring. |
| 7 | **LOW** | lib.rs:524 | **Background image load panics** via `panic!()` on missing file. Convenience function, documented behavior unclear. |
| 8 | **LOW** | lib.rs:2820 | **Asset URL read into memory with no size limit** — `std::fs::read()` on multi-GB file causes OOM. |
| 9 | **INFO** | lib.rs:3308-3309 | `unsafe impl Send/Sync for RodioAudioEngine` — Correctly documented. `rodio::OutputStream` lacks Send/Sync on macOS but engine is always used from main thread. |

**Unsafe blocks**: 3 (thread-local raw pointer + Send/Sync impl + setpriority). All LOW risk with proper documentation.

**Theming**: No theming in this crate — delegates to cvkg-render-gpu.

**Decomposition**: Single 4,276-line file. Candidate for split into `window.rs`, `asset.rs`, `audio.rs`, `render.rs`, `clipboard.rs`.

---

# cvkg-flow Engineering Audit

**Scope**: 9 `.rs` files (2,686 lines) — node, edge, port, graph, canvas, ribbon, layout, interaction, lib
**Dependencies**: cvkg-core
**Key areas**: Spline animation, OKLCH color, edge/node state machine, ribbon rendering, layout

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **HIGH** | edge.rs:24-37 | **[FIXED] Elastic easing clamped to no-op.** `SplineEasing::new()` clamped all 4 params to `[0,1]`. `elastic()` passes y1=-0.55 (overshoot) and y2=1.55 (bounce) — both clamped to 0.0 and 1.0, making elastic identical to ease-in-out. |
| 2 | **HIGH** | node.rs:31,103 | **[FIXED] Negative hue wraps incorrectly.** `h % 360.0` preserves negative sign (Rust `%`). Hue -90 stays -90 instead of wrapping to 270. Fixed: `rem_euclid(360.0)`. |
| 3 | **MED** | ribbon.rs:294-298 | **[FIXED] Ribbon tangent panics on degenerate curve.** `points[0..0][i-1]` access on `SEGMENTS=0` triggers underflow panic. Fixed: guard for `points.len() < 2`. |
| 4 | **MED** | layout.rs:54-55,74-75 | **4x `.unwrap()` on `HashMap::get_mut()`** — safe today (keys freshly initialized before use) but fragile. Any future code path removing keys between init and use triggers panic. |
| 5 | **LOW** | edge.rs:282-289 | **Negative/NaN dt in `tick_animation`** — negative dt causes animation regression; NaN dt permanently poisons `animation_progress` (all comparisons with NaN are false). |

**Unsafe blocks**: 0.

**Theming**: Excellent color pipeline — OKLCH -> OKLab -> linear sRGB -> gamma-corrected sRGB with proper gamut clamping. No issues.

**Animation state machine**: 4 states (Default->Hovered->Selected->Dragging). No invalid-transition guards (consumer could jump Default->Dragging). State transitions are reversible with no guard. `effective_width()` checks only hovered/selected — dragging uses default width, asymmetric with color handling. None of these produce incorrect behavior in the current UI flow, but are fragility notes.

**Bugs fixed by subagent**: 3 (elastic eased, hue wrap, ribbon tangent) — code patches verified passing all 61 unit tests + 2 integration tests.

---

# cvkg-anim Engineering Audit

**Scope**: 13 `.rs` files, ~5,000 lines (lib, behavior, morph, growth, physics, particles, advanced_particles, shader_anim, verlet, attractor, momentum, geometry, lib)
**Dependencies**: cvkg-core, glam, rand
**Key areas**: Keyframe/timeline animation, particle systems, verlet physics, shader animation, cloth, Gerstner waves, spring solver (Sleipnir)

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **HIGH** | particles.rs:57, advanced_particles.rs:842 | **Division by zero on `spawn_rate == 0.0`.** `1.0 / self.spawn_rate` — user-facing API parameter with no validation. |
| 2 | **HIGH** | shader_anim.rs:317 | **Underflow panic on `frame_count == 0`.** `(frame_f as u32).min(self.frame_count - 1)` — `frame_count - 1` wraps on release (panic on debug). |
| 3 | **HIGH** | shader_anim.rs:323-324 | **Division by zero on `tex_height == 0`.** Happens when `frame_count == 0` — chain failure from #2. |
| 4 | **HIGH** | verlet.rs:127-129 | **No bounds check on constraint indices.** `self.particles[c.p1_idx]` — if constraint references index >= particles.len(), panic. No API validation. |
| 5 | **HIGH** | momentum.rs:19 | **NaN propagation on `friction < 0.0`.** `self.friction.powf(dt * 60.0)` — powf with negative base and non-integer exponent returns NaN. No constructor validation. |
| 6 | **MED** | lib.rs:599 | **`unwrap()` on solver in MjolnirShatter handler.** Guaranteed `Some` by initialization flow (lines 573-584) but fragile to refactoring. |
| 7 | **MED** | lib.rs:286-297 | **Reduce-motion only works for SleipnirSolver.** `ActiveAnimation::update()` never checks/uses reduce-motion for Linear, Sequence, Parallel, BifrostFade, MjolnirSlice, or MjolnirShatter. Accessibility feature non-functional through high-level API. |
| 8 | **MED** | advanced_particles.rs:795-801 | **Color ramp passes same curve for all 4 channels.** `SplineEasing::color_ramp` accepts 4 curves (R,G,B,A) but `update()` passes `&self.color_curve` for all 4. Particles are monochromatic. |
| 9 | **MED** | advanced_particles.rs:471-477 | **RoundedRect SDF negative half-dimensions.** `half_w = width * 0.5 - radius` can be negative when `radius > half-width`. Produces incorrect SDF values. |
| 10 | **MED** | lib.rs:286-297 | **Reduce-motion incomplete** — only SleipnirSolver implements it. All other animation paths ignore the accessibility flag. |
| 11 | **MED** | physics.rs:85 | **Vec3 Div by zero.** `self / s` with `s == 0.0` produces Inf/NaN in vector. No guard. |
| 12 | **MED** | physics.rs:370 | **Division by zero if both bodies static + compliance=0.** Denominator `inv_mass_a + inv_mass_b + compliance` is zero, produces Inf. |

**Unsafe blocks**: 0.

**Production unwraps**: 3 (lib.rs:599 HIGH, behavior.rs:234 MED). All 22 other `.unwrap()` calls in test code.

**Integer overflow risks**: 5 (grid size multiplication in advanced_particles.rs, shader_anim.rs, physics.rs, geometry.rs — all `width * height as usize`).

---

# cvkg-render-software Engineering Audit

**Scope**: 1 `.rs` file (747 lines) — software rasterizer backend
**Dependencies**: cvkg-core (types, layout)
**Key areas**: Pixel fill/rect/rounded-rect/ellipse rendering, framebuffer, glass effect

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **HIGH** | lib.rs:224-227 | **[FIXED] Rounded-rect SDF degenerates to circle.** Formula `dx = |fx - cx| - w/2` produces distance from rect **center**, not from edges. Only the center-circle of radius `r` is inked. Straight edges and correct corners are missing entirely. Existing tests only check center pixel, so they pass. |
| 2 | **HIGH** | lib.rs:352-366 | **[FIXED] Stroke rounded rect never draws.** Outset SDF formula produces values exceeding radius for all points inside the shape. Stroke condition `outside <= r && >= r - sw` never satisfied — no pixels rendered. |
| 3 | **HIGH** | lib.rs:43 | **Integer overflow in framebuffer alloc.** `(width * height) as usize` — u32 overflow wraps to 0 for 65536x65536+. Allocates zero-length Vec; subsequent pixel writes OOB. Release-only (debug panics on overflow). |
| 4 | **HIGH** | lib.rs:42,53,148,167 | **DoS via unbounded framebuffer dimensions.** No max-size validation. `SoftwareRenderer::new(50000, 50000)` tries to allocate ~19 GB (pixels + depth buffer). OOM crash. |
| 5 | **MED** | lib.rs:188-190 | **`into_framebuffer()` documented but missing.** Doc comment references a consuming accessor that doesn't exist. |
| 6 | **MED** | lib.rs:311 | **Glass intensity cap applied too early.** `.min(0.8)` before `* glass_intensity`, so intensity > 1.0 exceeds the cap. |
| 7 | **LOW** | lib.rs:408-415 | **`draw_line` O(n^2) pixel blending.** Each step calls `fill_rect_internal` which iterates all pixels in stroke-width square. Overlapping steps blend multiple times. |

**Unsafe blocks**: 0.

**Production unwraps**: 0.

**Theming**: None.

**Bugs fixed by subagent**: 2 (rounded-rect SDF, stroke rounded-rect SDF).

---

# cvkg-scheduler / cvkg-spatial / cvkg-materials Engineering Audit

**Scope**: 12 files (scheduler=3/781LOC, spatial=4/829LOC, materials=5/811LOC)
**Dependencies**: cvkg-core
**Key areas**: Task scheduling, BVH + quadtree + spatial hash, material data models (Glass, Mica, Acrylic, Elevation)

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **LOW** | spatial/bvh.rs | **[FIXED] NaN centroid causes arbitrary BVH order.** `NaN.partial_cmp(&x)` returns `None`, previously fell through to `Ordering::Equal`. Fixed: use `f32::total_cmp()`. |
| 2 | **LOW** | spatial/spatial_hash.rs | **[FIXED] Extreme rect coords iterate 4B cells.** `cells_for_rect()` can produce enormous cell ranges from extreme input. Added `MAX_CELL_SPAN = 1000` clamp. |
| 3 | **INFO** | spatial/quadtree.rs | Subnormal-width rects could theoretically cause precision issues in repeated subdivision (depth capped at 5, theoretical only). |

**Unsafe blocks**: 0 across all three crates.

**Production unwraps**: 0 (all 24 `.unwrap()` calls are in `#[cfg(test)]` blocks).

**Theming**: None in scheduler or spatial. cvkg-materials has pure data models correct by design — default values match Fluent Design / Material Design 3 references.

**Thread safety**: Scheduler uses `&mut self` — single-threaded by design. No channels, no race windows, no task stealing. Correct.

**Bugs fixed by subagent**: 2 (BVH NaN sort, spatial hash cell-span limit).

---

# cvkg-telemetry Engineering Audit

**Scope**: 1 `.rs` file (470 lines)
**Dependencies**: None (standalone)
**Key areas**: Telemetry event recording, frame budget monitoring, contrast failure reporting

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **MED** | lib.rs | **Unbounded `Vec<TelemetryEvent>` growth.** No cap or eviction policy. If the `Telemetry` instance lives for the app lifetime and events are recorded per-frame, memory grows monotonically. |
| 2 | **INFO** | lib.rs | **Frame budget threshold is 16.0ms.** At 60 FPS (16.67ms), frames between 16.0-16.67ms are flagged as exceeded when they meet the 60 FPS target. |
| 3 | **INFO** | lib.rs | **Crate is unused.** No other crate depends on or imports it. Macros only referenced in own doc comments. Pending integration. |
| 4 | **LOW** | lib.rs | Missing `#![deny(unsafe_code)]` lint guard. |

**Unsafe blocks**: 0. **Production unwraps**: 0. **Theming**: None. **Tests**: 15 unit + 1 doctest, all pass.

---

# cvkg-icons / cvkg-themes / cvkg-macros Engineering Audit

**Scope**: 3 files (icons 313L, themes 1308L, macros 291L)
**Dependencies**: cvkg-core (themes), proc-macro2/quote (macros)
**Key areas**: SVG icon rendering, OKLCH theme colors, builder proc macros

## Findings

| # | Severity | Crate:Line | Issue |
|---|----------|------------|-------|
| 1 | **HIGH** | icons:196-204 | **CSS `rgba()` uses f32 instead of [0-255].** `rgba({},{},{},{})` with values in [0.0, 1.0]. CSS `rgba()` expects 0-255 integers (Level 4 floats unsupported by most SVG renderers). All icons render black regardless of color. |
| 2 | **HIGH** | macros:284-286 | **`vdom_id()` hashes empty hasher.** `DefaultHasher::new().finish()` writes no data. Produces all-zero or non-deterministic IDs. `&self` never used. VDOM identity is broken. |
| 3 | **HIGH** | themes:259-260 | **Alpha values exceed 1.0** in `oklch_to_color_theme()`. `primary_neon: [..., 1.2]`, `shatter_neon: [..., 1.5]`. GPU artifacts with out-of-range alpha. |
| 4 | **MED** | icons:195-204 | **SVG path injection.** `IconData::Svg(String)` unvalidated — SVG path data interpolated directly into XML. If from untrusted source, can inject arbitrary SVG/XML. |
| 5 | **MED** | macros:191 | **Builder `expect()` missing field name.** `expect("missing required field ")` with trailing space — no indication of which field. |
| 6 | **MED** | macros:58-66 | **Destructured args silently dropped.** `let Pat::Ident(pat_ident) = &*pat_type.pat` guard skips tuples/references. Confusing compile error. |
| 7 | **MED** | themes:765-770 | **APCA implementation is simplified approximation.** Uses same exponent for light/dark backgrounds. Real WCAG APCA is more complex; borderline results may be incorrect. |
| 8 | **MED** | themes:1085 | **Dead parameter.** `compute_contrasting_text` has `_bg_for_luminance` parameter never referenced. Callers pass same value twice. |
| 9 | **MED** | themes:1104-1118 | **Binary search can diverge.** APCA contrast not guaranteed monotonic near `bg_lum=0.5`. Search always returns `hi` even if never updated. |
| 10 | **LOW** | macros | Builder unconditionally wraps all fields in `Option` + `.expect()`. No way to make optional fields. |
| 11 | **LOW** | icons:90,105 | `delete` and `close` icons share identical SVG path (X shape). Delete likely intended different visual. |

**Unsafe blocks**: 0 across all three crates. **Production unwraps**: Macros builder has 3 `expect()` calls.

**Tests**: 8/8 (icons), 30/30 (themes), 1/1 (macros) — all pass.

---

# cvkg-accessibility / cvkg-certification Engineering Audit

**Scope**: 5 `.rs` files (accessibility ~3kLOC, certification ~900LOC)
**Dependencies**: cvkg-core
**Key areas**: Accessibility tree, focus management, screen reader bridge, pipeline/scene certification

## Findings

| # | Severity | Crate:File:Line | Issue |
|---|----------|-----------------|-------|
| 1 | **INFO** | access:bridge.rs:119 | `"announce"` literal in debug format string — reads oddly as `[A11y] announce (polite):` |
| 2 | **INFO** | access:focus.rs:127 | **Dead `unwrap_or` guard** — `last().unwrap_or(...)` unreachable because `is_empty()` checked before call. |
| 3 | **INFO** | access:focus.rs:183-193 | **Stub tab ordering** — sorts by raw `KvasirId.0` instead of document position. Detailed TODO present. Non-semantic tab order. |
| 4 | **LOW** | both | **Stale READMEs** — reference non-existent API names. |
| 5 | **LOW** | cert | **No serde hardening** — missing `deny_unknown_fields`, no recursion guards or size limits on deserialized types. Fine in-process but a latent risk across trust boundaries. |

**Unsafe blocks**: 0. **Production unwraps**: 0. **Theming**: None. **Tests**: 25 (accessibility) + 17 (certification) — all pass.

**Overall**: Clean, well-structured, production-grade. No active bugs or security issues.

---

# cvkg-test Engineering Audit

**Scope**: 3 `.rs` files (154L lib, 189L conformance, ~200L a11y_conformance)
**Dependencies**: cvkg-core, image crate
**Key areas**: Visual regression comparison, golden image testing, conformance test suite, a11y validation

## Findings

| # | Severity | File:Line | Issue |
|---|----------|-----------|-------|
| 1 | **INFO** | lib.rs:101,115,118,124-126 | `.expect()` / `.unwrap()` on file I/O and image buffer creation — acceptable for test utility code (failure correctly fails the test). |
| 2 | **INFO** | conformance.rs | Clean — no unsafe, no unwrap, well-structured. |

**Unsafe blocks**: 0. **Production unwraps**: 0 (all in test validation code). **Theming**: None.

---

# root cvkg crate / demos / remaining

**cvkg** (root): Re-exports only (51 lines). No source code to audit.

**cvkg-anim**: Already covered above.

**cvkg-vdom**: Covered in earlier pass (cvkg-vdom crate).

**Demos** (`adele-web`, `niflheim-wasi`, `berserker-fire-web`, `berserker`): Application code — not part of library module audit scope. Skip.

---

# Workspace Audit Summary

## Coverage

| Metric | Count |
|--------|-------|
| Workspace members | 30 (incl root + demos + test) |
| Crates fully audited | **20** (25 counting sub-crate batches) |
| Files reviewed | ~180+ `.rs` sources |
| Lines of Rust reviewed | ~100,000+ |
| Total findings | ~120+ |
| Bug fixes applied by subagents | **8** (cvkg-flow:3, cvkg-render-software:2, cvkg-spatial:2, cvkg-core:1) |

## Aggregate Cross-Crate Finding Rankings

| Rank | Severity | Crate | Issue | Type |
|------|----------|-------|-------|------|
| 1 | **CRITICAL** | cvkg-core/cvkg-components | System state hash collision (`0xD00_0001`). 3 component types share same key. Silent data corruption. | Logic |
| 2 | **CRITICAL** | cvkg-layout | AspectRatio Y-center `* 0.0` instead of `* 0.5`. Views top-aligned. | Logic |
| 3 | **CRITICAL** | cvkg-webkit-server | Stored XSS via `/snapshot` — unsanitized POST body rendered HTML. | Security |
| 4 | **HIGH** | cvkg-webkit-server | WASM fuel disabled — infinite guest loops DoS server. | DoS |
| 5 | **HIGH** | cvkg-webkit-server | Mutex `.unwrap()` on poison crashes server. | Availability |
| 6 | **HIGH** | cvkg-webkit-server | File watcher unbounded recursion — stack overflow via symlink loop. | DoS |
| 7 | **HIGH** | cvkg-render-native | u32→i32 overflow in center math; f64→u32 silent saturation. | Bug |
| 8 | **HIGH** | cvkg-anim | Div by zero on `spawn_rate == 0` (x2). | Panic |
| 9 | **HIGH** | cvkg-anim | Underflow panic on `frame_count == 0`. | Panic |
| 10 | **HIGH** | cvkg-anim | Verlet constraint indices unchecked — OOB panic. | Panic |
| 11 | **HIGH** | cvkg-anim | NaN propagation on `friction < 0`. | Logic |
| 12 | **HIGH** | cvkg-render-software | Rounded-rect SDF degenerates to circle **[FIXED]**. | Rendering |
| 13 | **HIGH** | cvkg-render-software | Stroke rounded-rect never draws **[FIXED]**. | Rendering |
| 14 | **HIGH** | cvkg-render-software | Integer overflow in framebuffer alloc -> zero-length Vec + OOB. | Memory |
| 15 | **HIGH** | cvkg-render-software | DoS via unbounded framebuffer dimensions (OOM). | DoS |
| 16 | **HIGH** | cvkg-icons | CSS `rgba()` uses f32 instead of 0-255 — all icons render black. | Rendering |
| 17 | **HIGH** | cvkg-macros | `vdom_id()` hashes empty hasher — broken VDOM identity. | Logic |
| 18 | **HIGH** | cvkg-themes | Alpha values > 1.0 — GPU rendering artifacts. | Rendering |
| 19 | **HIGH** | cvkg-flow | Elastic easing clamped to no-op **[FIXED]**. | Rendering |
| 20 | **HIGH** | cvkg-flow | Negative hue wrap broken **[FIXED]**. | Logic |
| 21 | **HIGH** | cvkg-svg-filters | `max_inputs` not enforced on `input_range_unchecked`. | Safety |
| 22 | **HIGH** | cvkg-svg-serialize | CSS content mangled by quick-xml `BytesText` escaping. | Output |
| 23 | **HIGH** | cvkg-layout | No NaN/Infinity guards in layout math. | Logic |
| 24 | **HIGH** | cvkg-runic-text | Knuth-Plass OOB panic on `prev_pos` bounds check. | Panic |
| 25 | **HIGH** | cvkg-components | `chat_text_user()` hardcoded black — invisible on dark themes. | UX |
| 26 | **MED** | cvkg-core | Unsafe `Arc::from_raw` from serialized u64 pointer — potential UAF. | Memory |
| 27 | **MED** | cvkg-physics | Motor constraints dead code. | Logic |
| 28 | **MED** | cvkg-physics | Hinge3D inverse inertia uses X-component only. | Physics |
| 29 | **MED** | cvkg-anim | Reduce-motion only works for SleipnirSolver. | Accessibility |
| 30 | **MED** | cvkg-telemetry | Unbounded Vec growth — latent memory leak. | Memory |

**Unsafe blocks found across entire workspace**: < 10, all LOW risk with proper documentation.

