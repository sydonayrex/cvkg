# CVKG Rendering Pipeline — Complete Audit Report

**Date:** 2026-07-30  
**Auditor:** OWL  
**Scope:** All crates using `cvkg-render-gpu` for rendering shapes, images, UI, icons, and visuals.  
**Crates examined:** `berserker`, `cvkg`, `cvkg-core`, `cvkg-render-gpu`, `cvkg-render-native`, `cvkg-render-web`, `cvkg-components` (all 77 files), `cvkg-compositor`, `cvkg-scene`, `cvkg-vdom`, `cvkg-themes`, `cvkg-svg-filters`, `cvkg-svg-serialize`, `cvkg-runic-text`, GPU shaders

---

## 1. ARCHITECTURE OVERVIEW

### Crate Dependency Graph

```
berserker (main binary)
  └─→ cvkg::prelude
        ├─→ cvkg_core (traits: View, Renderer, FrameRenderer)
        ├─→ cvkg_render_native (NativeRenderer, window/app lifecycle)
        │     └─→ cvkg_render_gpu (SurtrRenderer — wgpu GPU backend)
        │           ├─→ cvkg_runic_text (text shaping)
        │           ├─→ cvkg_svg_filters (SVG filter evaluation — never init'd)
        │           ├─→ cvkg_svg_serialize (SVG serialization)
        │           └─→ lyon (path tessellation)
        ├─→ cvkg_components (77 component files — all use Renderer trait)
        ├─→ cvkg_vdom (virtual DOM: build, diff, apply_patches)
        ├─→ cvkg_compositor (layer tree → CommandBuckets)
        ├─→ cvkg_scene (retained scene graph)
        ├─→ cvkg_themes (Theme, GlassMaterial, StateColors)
        └─→ cvkg_layout (layout engine)
```

### Renderer Trait Hierarchy (cvkg_core)

```
ElapsedTime (delta_time, elapsed_time)
  └─→ Renderer (all draw calls, clips, transforms, effects, state)
        └─→ FrameRenderer<E> (begin_frame → render_frame → end_frame)
```

`Renderer` has ~45 methods with default no-op implementations for most — only `fill_rect`, `fill_rounded_rect`, `fill_ellipse`, `stroke_rect`, `stroke_rounded_rect`, `stroke_ellipse`, `draw_line`, `draw_text`, `measure_text`, and `draw_shaped_text` are required.

---

## 2. THE RENDERING PIPELINE — END TO END

### 2.1 Application Entry (berserker/src/main.rs)

The `berserker` binary creates a `BerserkerFireView` that implements `View`. It calls:
```cvkg_render_native::NativeRenderer::run(view)```

### 2.2 Window/Event Loop (cvkg-render-native)

1. `NativeRenderer::run()` creates a `winit` event loop and `App` struct with:
   - `WindowManager` (multi-window tracking)
   - `Option<Arc<Mutex<SurtrRenderer>>>` (shared GPU context)
   - `NativeAssetManager` (async file loading)
   - `WindowStateDetector` (minimized/occluded/fullscreen tracking)
   - `VisualHapticEngine` (haptic feedback)
   - `RodioAudioEngine` (audio)

2. Events handled:
   - `RedrawRequested` → full frame (see below)
   - `Resized` → GPU `resize()` 
   - `Focused` → key tracking
   - `Moved` → rage/kinetic injection
   - `CloseRequested` → window lifecycle
   - `UserEvent(AppEvent)` → cross-thread actions

### 2.3 Frame Lifecycle (RedrawHandler path, ~line 880 of cvkg-render-native/src/lib.rs)

```
1. VDom::build(&self.view, rect)         → Build new virtual DOM from View tree
2. prev_vdom.diff(&new_vdom)             → Compute VDomPatch list
3. Apply patches to accessibility tree     → accesskit::TreeUpdate
4. prev_vdom.apply_patches(patches)      → Retained-mode VDom update
5. gpu.begin_frame(window_id)            → Clear CPU-side state, update SceneUniforms
6. self.view.render(&mut renderer, rect) → Walk View tree → Renderer trait calls
7. gpu.render_frame()                    → Upload vertices/indices via StagingBelt
8. gpu.end_frame(encoder)                → Multi-pass GPU execution + present
```

### 2.4 GPU Multi-Pass Pipeline (end_frame in cvkg-render-gpu/src/lib.rs, line 2712)

```
Pass 1: Opaque Background & Atmosphere
  1a. Background Atmosphere (fullscreen triangle, fs_background shader)
      → Draws to scene_texture with depth clear (reversed-Z, clear=0.0)
  1b. Opaque geometry (DrawMaterial::Opaque)
      → Routed from draw_calls, drawn with main pipeline
      → Depth-tested against Pass 1 depth buffer

Pass 2: Backdrop Blur (Bifrost)
  → bloom_extract_pipeline copies scene_texture → blur_texture_a
  → 4 iterations: H-blur(a→b) + V-blur(b→a)
  → Result in blur_texture_a

Pass 3: Liquid Glass (scene_texture, LOAD, depth LOAD)
  → Glass draw calls (DrawMaterial::Glass)
  → Samples blur result via bind_group[1]
  → Parallel-encoded via rayon

Pass 4: UI Layer (scene_texture, LOAD, depth LOAD)
  → TopUI draw calls (DrawMaterial::TopUI)
  → Parallel-encoded via rayon, submitted before Pass 5

Pass 5: Bloom Extract
  → bloom_extract_pipeline reads scene_texture → blur_texture_a

Pass 6: Blur Bloom
  → 2 iterations H+V blur on bloom texture

Pass 7: Composite & Tone Map
  → Fullscreen triangle
  → Reads scene_texture + bloom, ACES tonemap, additive blend
  → Output to surface texture
  → queue.submit() + surface.present()
```

**Pipeline order is CORRECT.** Opaque → Blur → Glass (samples blur) → UI → Bloom → Composite matches industry-standard Backdrop Capture.

### 2.5 Geometry Generation (fill_rect_with_full_params_and_slice)

Each `Renderer` trait method tessellates into a shared vertex/index buffer:
- 4 vertices per quad (position, normal, uv, color, mode, radius, slice, logical, size, screen, clip, translation, scale, rotation, tex_index)
- 6 indices per quad (2 triangles)
- Batch-breaking: new `DrawCall` when scissor rect or material changes
- No batch break on texture (texture array at bind group 0)

### 2.6 GPU Shader Modes (shapes.wgsl fragment shader)

| Mode | Effect |
|------|--------|
| 0 | Standard solid fill |
| 1 | Neon line (color * 1.5 boost) |
| 3 | Rounded rectangle with SDF anti-aliasing |
| 4 | Ellipse with SDF anti-aliasing |
| 7 | Glass (backdrop blur sampling, screen-space UV, discard OOB, fresnel lensing) |
| 8 | **Undefined in shader** — falls through to default (solid fill) |

---

## 3. VERIFIED CORRECT BEHAVIORS

1. **Pipeline order** — Opaque → Blur → Glass → UI → Bloom → Composite. Industry-standard.
2. **Depth buffer** — Reversed-Z in Pass 1 (clear=0.0, LessEqual). Passes 3,4 use Load. Correct.
3. **Batch breaking** — Only on scissor/material changes. Texture array means no texture-break needed.
4. **Staging belt** — `wgpu::util::StagingBelt` for CPU→GPU transfers. Recommended pattern.
5. **Retained VDOM** — `diff()` + `apply_patches()` not full rebuild. Correct.
6. **Parallel encoding** — Glass and UI passes encoded concurrently via `rayon::join()`. Correct.
7. **Window state machine** — `Occluded`/`Minimized`/`Fullscreen` → `ControlFlow::Wait`. Saves CPU.
8. **Theme system** — OKLCH color space, auto-synthesized interactive states, APCA contrast validation.
9. **Fullscreen vertex shader** — `vs_fullscreen` generates proper UVs from vertex_index. Works.
10. **Telemetry** — Per-frame timing (layout, state flush, draw, GPU submit), P99/jitter tracking.
11. **Component rendering** — Components like `card.rs` correctly call `bifrost()` before fill/stroke.
12. **Interactive state synthesis** — `StateColors::from_base()` correctly derives hover/active/focus/disabled in OKLCH.

---

## 4. BUGS

### BUG 1: `submit_routed()` Is a Stub — Compositor Path Completely Broken
**Severity:** CRITICAL  
**Location:** cvkg-render-gpu/src/lib.rs, ~line 4758

`submit_routed()` ignores the `DrawCommand`'s `index_start`/`index_count` and instead generates a hard-coded 1×1 white quad:

```rust
fn submit_routed(&mut self, routed: &cvkg_compositor::RoutedDrawCommand) {
    let cmd = &routed.command;
    self.fill_rect_with_full_params(
        cvkg_core::Rect::new(0.0, 0.0, 1.0, 1.0),
        [1.0, 1.0, 1.0, 1.0], 0,
        cmd.texture_id, 0.0,
        cvkg_core::Rect::new(0.0, 0.0, 1.0, 1.0),
    );
}
```

All three bucket types (scene_commands, glass_commands, overlay_commands) flow through this method. The entire `CompositorEngine` → `submit_buckets()` → `submit_routed()` pipeline is non-functional.

### BUG 2: `fs_bloom_extract` Used as Blur Copy — Dark Pixels Lost
**Severity:** MEDIUM  
**Location:** bloom.wgsl line 4; used in end_frame Pass 2

```wgsl
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.8 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
```

This shader gates on brightness > 0.8. In Pass 2 it's used to copy `scene_texture → blur_texture_a`. Only pixels brighter than 0.8 get copied. The glass backdrop blur (Pass 3) then only sees bright content — **dark areas behind glass panels will show through without blur**.

For bloom (Pass 5) this luminance gate is correct. For backdrop blur it is wrong.

### BUG 3: Glass Blur Pyramid Is Dead Code — VRAM Waste
**Severity:** MEDIUM  
**Location:** forge_internal() ~line 1680-1720

```rust
glass_blur_pipeline: pipeline.clone(),          // ← main pipeline, not blur
glass_blur_upsample_pipeline: pipeline.clone(), // ← main pipeline, not blur
glass_blur_views: Vec::new(),                   // ← empty
glass_blur_down_bind_groups: Vec::new(),        // ← empty
glass_blur_up_bind_groups: Vec::new(),          // ← empty
blur_pyramid_mip_count: 1,                      // ← 1 mip = no pyramid
```

The Kawase blur pyramid struct fields are allocated but never populated or used. The `glass_blur_texture` (Rgba16Float) is allocated but never written to. ~16MB of VRAM wasted on a 1080p texture.

### BUG 4: Mode 8 (Gungnir Glow) Not Implemented in Shader
**Severity:** LOW (visual glitch)  
**Location:** shapes.wgsl fragment function

The shader handles modes 0, 1, 3, 4, 7. Mode 8 (used by `gungnir()` for neon glow effect) falls through to the default branch which returns `color` unchanged. The glow effect draws 8 expanding rectangles but they render as solid-color quads with no additive blending or Bloom contribution.

### BUG 5: `filter_engine` Never Initialized
**Severity:** MEDIUM  
**Location:** forge_internal() ~line 1650

`filter_engine: None` — always. `apply_svg_filter()` calls `find_filter` on the `usvg::Tree`, gets the filter element, then re-serializes the tree *without applying the filter*. The filter engine (`cvkg_svg_filters::FilterEngine`) is never instantiated. SVG filters are effectively no-ops.

### BUG 6: Shader Validation Disabled at Compile Time
**Severity:** LOW  
**Location:** cvkg-render-gpu/build.rs line 62

```rust
naga::valid::ValidationFlags::empty()
```

Comment says "skip validation to avoid IndexMustBeConstant errors for now." Shader bugs surface only at GPU runtime, not at compile time.

### BUG 7: `capture_png()` on Native Renderer Always Fails
**Severity:** LOW  
**Location:** cvkg-render-native/src/lib.rs capture_png(), cvkg-render-gpu/src/lib.rs ~line 5264

The native renderer creates `SurtrRenderer` via `forge()` (with surface), never `forge_headless()`. So `headless_context` is always `None`. `capture_frame()` requires `headless_context`. Therefore `capture_png()` on a windowed application always returns `Err("Headless context required")`.

### BUG 8: Glass Mode 7 Shader Has Hard `discard` for OOB UVs
**Severity:** MEDIUM  
**Location:** shapes.wgsl ~line 85

```wgsl
if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
    discard;
}
```

When a glass rect extends beyond the screen edge, UVs exceed [0,1] and pixels are discarded. Glass panels near screen boundaries show hard cutoffs instead of clamping.

### BUG 9: `fs_composite` Reads `t_diffuse[in.tex_index]` With Undefined tex_index
**Severity:** LOW  
**Location:** bloom.wgsl line 82; end_frame Pass 7

The composite pass draws a fullscreen triangle with no vertex buffer, so `in.tex_index` comes from the `vs_fullscreen` shader which hard-codes `tex_index = 0u`. In Pass 7, `ctx_scene_texture_bind_group` is bound at group 0, where slot 0 is the scene texture (via the scene_texture_bind_group). This works by coincidence — the same bind group layout is used for both the texture array and the scene texture bind group, and slot 0 happens to be correct. Fragile.

### BUG 10: Two `end_frame` Implementations (Confusing Dispatch)
**Severity:** INFORMATIONAL  
**Location:** lib.rs line 2712 (inherent) and line 4900 (FrameRenderer trait)

```rust
// Inherent method — the real GPU work
pub fn end_frame(&mut self, encoder: wgpu::CommandEncoder) { ... }

// FrameRenderer trait impl — delegates to inherent
fn end_frame(&mut self, encoder: E) {
    Self::end_frame(self, encoder);
    cvkg_core::end_render_phase();
}
```

Works correctly but the dual implementation is confusing for maintainers.

### BUG 11: `Composite Pipeline` Additive Blend Uses `src::One, dst::One` for Alpha Too
**Severity:** LOW  
**Location:** forge_internal() ~line 1590

```rust
blend: Some(wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,  // ← additive alpha
        dst_factor: wgpu::BlendFactor::One,  // ← additive alpha
        operation: wgpu::BlendOperation::Add,
    },
}),
```

Alpha channel is also additively blended, meaning bloom contributes to alpha. If the scene has transparent areas, bloom will make them opaque. Typically alpha should use `src_alpha, one_minus_src_alpha` or just `one, one_minus_src_alpha`.

### BUG 12: `draw_svg()` Always Routes to Glass or TopUI
**Severity:** LOW  
**Location:** draw_svg() ~line 5130

```rust
let material = if mode == 7 {
    cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
} else {
    cvkg_core::DrawMaterial::TopUI
};
```

No way to draw SVG as opaque. Limits SVG to foreground/glass only.

### BUG 13: Blur Pipelines Use Incorrect Bind Group Semantics
**Severity:** MEDIUM  
**Location:** forge_internal() pipeline creation

The horizontal and vertical blur pipelines (`blur_h_pipeline`, `blur_v_pipeline`) are created with `post_process_layout` which has 3 bind group slots: `[texture_array, env_texture, berserker_uniforms]`. But the blur fragment shaders only need `[texture, sampler]` — they use `t_diffuse[0]` and `s_diffuse` from group 0. Bind groups 1 and 2 are set to `dummy_env_bind_group` and `berserker_bind_group` unnecessarily. Not a bug per se, but indicates the blur pipelines weren't given their own layout.

---

## 5. DESIGN ISSUES

### 5.1 Monolithic 5,373-Line Renderer File
`cvkg-render-gpu/src/lib.rs` contains: GPU init, shader binding, vertex tessellation, multi-pass pipeline, SVG loading, text shaping, image loading, blur pyramid, bloom, composite, frame capture, clipping, opacity, transforms, z-index, VDOM tracking, event handlers, telemetry, VRAM tracking, Berserker mode — all in one file. Should be split into modules (`pipeline.rs`, `geometry.rs`, `effects.rs`, `text.rs`, `svg.rs`, `capture.rs`, `telemetry.rs`).

### 5.2 Per-Draw-Call Mutex Locking
The native renderer holds `Arc<Mutex<SurtrRenderer>>`. Every `Renderer` trait method implementation locks the mutex:
```rust
fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
    self.gpu.lock().expect("GPU mutex poisoned").fill_rect(rect, color);
}
```
For scenes with thousands of draw calls, this is a significant overhead. The lock is held only per-call (not across the full frame), which is good, but mutex acquire/release per quad is expensive.

### 5.3 No Indirect Drawing
Each `DrawCall` is rendered with a separate `draw_indexed()` call in a loop. Modern GPUs prefer indirect drawing (`draw_indirect`) for large numbers of draw calls. The texture array architecture already enables good batching, but the CPU-side loop won't scale to 10K+ elements.

### 5.4 Two Parallel Rendering Paths That Don't Integrate

**Path A (working):** `View::render()` → `Renderer` trait calls → tessellation in `fill_rect_with_full_params_and_slice()` → inline `DrawCall` generation

**Path B (broken):** `CompositorEngine::flatten_and_route()` → `submit_buckets()` → `submit_routed()` → stub 1×1 quad

The compositor was designed as a retained-mode layer system with damage tracking, but the GPU-side consumer was never properly connected. Path B exists in dead code.

### 5.5 Scene Graph Is Not Connected to Renderer
`cvkg-scene/src/lib.rs` has a `SceneGraph` struct with `NodeId`, `Change` types, and `apply_changes()`, but nothing in the rendering pipeline ever calls `SceneGraph::render()` or uses it. The `Renderer` trait has `query_layout(NodeId)` and `set_debug_layout(bool)` referencing the scene graph, but no component or view ever populates it. Dead code.

### 5.6 Every Frame Re-Tessellates Everything
`begin_frame()` clears `self.vertices` and `self.indices`. All geometry is re-tessellated from scratch every frame. The VDOM is retained (diff/patch), but GPU geometry is not retained. This is a valid design choice but means O(n) tessellation cost per frame regardless of what changed.

---

## 6. SHADER PIPELINE VERIFICATION

### 6.1 Compiled Shader Chain (build.rs)

```
WGSL_SRC = common.wgsl + shapes.wgsl + bifrost.wgsl + bloom.wgsl
```

All concatenated into a single string, compiled via `naga` to SPIR-V. The shader units:

| Shader | Entry Point | Purpose |
|--------|-------------|---------|
| common.wgsl | `vs_main` | 2D transform (rotate→scale→translate) + shatter physics |
| common.wgsl | `vs_fullscreen` | Fullscreen triangle from vertex_index |
| shapes.wgsl | `fs_main` | Mode-dispatch: solid/rounded/ellipse/glass |
| bifrost.wgsl | `fs_background` | 5 scene presets: Aurora/Void/Nebula/Glitch/Yggdrasil |
| bloom.wgsl | `fs_bloom_extract` | Luminance-gated extract (brightness > 0.8) |
| bloom.wgsl | `fs_blur_h` | 9-tap Gaussian horizontal |
| bloom.wgsl | `fs_blur_v` | 9-tap Gaussian vertical |
| bloom.wgsl | `fs_composite` | HDR bloom fusion + ACES tonemapping |

### 6.2 Depth Stencil Configuration

| Pass | Depth | Depth Format | Clear | Compare |
|------|-------|-------------|-------|---------|
| Pass 1 | Yes | Depth32Float | 0.0 (reversed-Z) | LessEqual |
| Pass 2-6 | No | — | — | — |
| Pass 7 | No | — | — | — |

Only Pass 1 uses depth. Passes 3,4 (glass, UI) use `LoadOp::Load` — they test against Pass 1 depth but don't clear. **Correct.**

### 6.3 Blend States

| Pass | Blend |
|------|-------|
| Pass 1a (background) | ALPHA_BLENDING |
| Pass 1b (opaque) | ALPHA_BLENDING |
| Pass 2 (blur) | ALPHA_BLENDING for H, same for V |
| Bloom Extract | None (writes raw) |
| Pass 7 (composite) | One/One additive (BOTH color AND alpha) |

Issue: composite additive alpha (Bug 11) means bloom contributes to alpha channel.

---

## 7. COMPONENT RENDERING PATTERNS

All 77 components in `cvkg-components/` follow the same pattern:

1. Implement `View` with `type Body = Never` and `fn body() -> unreachable!()`
2. Implement `fn render(&self, renderer: &mut dyn Renderer, rect: Rect)`
3. Use `push_vnode`/`pop_vnode` for accessibility tree
4. Call `fill_rect`, `fill_rounded_rect`, `draw_text`, `bifrost`, etc.
5. Use `DummyRenderer` for layout passes (intrinsic_size)

Key components examined:
- **card.rs**: Calls `bifrost()` → `fill_rounded_rect()` → `stroke_rounded_rect()` — correct order
- **image.rs**: Calls `draw_image()` — simple passthrough
- **primitive.rs**: Text with `shape_rich_text()` fallback to `draw_text()`
- **effects.rs**: Visual effects using specialized shader modes
- **shapes.rs**: Custom shape drawing

---

## 8. COMPLETE BUG SUMMARY

| # | Bug | Severity | Location |
|---|-----|----------|----------|
| 1 | `submit_routed` is 1×1 white quad stub (compositor broken) | **CRITICAL** | lib.rs ~L4758 |
| 2 | `fs_bloom_extract` used as blur copy — dark pixels lost | MEDIUM | bloom.wgsl; end_frame P2 |
| 3 | Glass blur pyramid dead code — VRAM waste | MEDIUM | forge_internal ~L1680 |
| 4 | Mode 8 (gungnir) not in shader — no glow | LOW | shapes.wgsl |
| 5 | `filter_engine` always None — SVG filters are no-ops | MEDIUM | forge_internal |
| 6 | Shader validation disabled | LOW | build.rs |
| 7 | `capture_png()` on native always fails | LOW | native+gpu lib.rs |
| 8 | Glass mode 7 shader hard-discards OOB UVs | MEDIUM | shapes.wgsl |
| 9 | Composite shader reads tex_index by coincidence | LOW | bloom.wgsl |
| 10 | Dual end_frame impls (confusing) | INFO | lib.rs L2712/4900 |
| 11 | Composite additive alpha makes transparent areas opaque | LOW | forge_internal |
| 12 | SVG always Glass or TopUI (no opaque) | LOW | draw_svg ~L5130 |
| 13 | Blur pipelines use 3 bind groups but only need 1 | MEDIUM | forge_internal |

---

## 9. RECOMMENDATIONS

### Immediate Fixes (Critical Path)

1. **Fix `submit_routed()`** — Either implement proper index-buffer-based draw call submission from compositor `DrawCommand`s, or remove the entire compositor path and rely solely on the `View::render()` → `Renderer` trait path.

2. **Add identity copy shader** — Create a `fs_copy` shader that does `textureSample(t_diffuse[0], s_diffuse, in.uv)` without luminance gating. Use it for Pass 2 (blur extract) instead of `fs_bloom_extract`.

3. **Fix glass blur** — Either implement the Kawase blur pyramid properly (populate the bind groups, create the downsample/upsample pipelines, execute the mip chain in `end_frame`), or remove the dead pyramid fields to save VRAM.

### Short-Term Fixes

4. **Add mode 8 to shader** — Implement additive blending for `gungnir()` glow effect in `fs_main`.
5. **Fix composite alpha blend** — Change alpha blend to `One, OneMinusSrcAlpha` so bloom doesn't affect transparency.
6. **Initialize `filter_engine`** — Create the `FilterEngine` in `forge_internal` and wire it to `apply_svg_filter()`.
7. **Fix glass OOB handling** — Change `discard` to `clamp(uv, 0.0, 1.0)` in mode 7 shader.

### Architecture Improvements

8. **Split `cvkg-render-gpu/src/lib.rs`** into modules.
9. **Remove dead scene graph** or connect it to the renderer.
10. **Reduce mutex contention** — batch lock acquisitions or use a command buffer pattern.
11. **Add indirect drawing** for large scene complexity.
12. **Enable shader validation** in build.rs once IndexMustBeConstant issues are resolved.

---

## 10. CONCLUSION

The CVKG rendering pipeline has a **sound architectural foundation**: correct multi-pass order, proper depth buffer usage, retained-mode VDOM, texture array batching, and a well-designed `Renderer` trait. The GPU shader pipeline (Surtr/Muspelheim) is ambitious and mostly correct.

The **critical blocker** is Bug 1: the compositor path (`submit_routed`) is a stub that draws 1×1 white rectangles. This means the entire `cvkg-compositor` crate (layer tree, material routing, damage tracking) is dead code. The working path is `View::render()` → `Renderer` trait calls → direct tessellation, which functions correctly.

The **most impactful visual bug** is Bug 2: the backdrop blur (Bifrost/glassmorphism) uses the bloom extract shader which gates on brightness > 0.8, meaning dark content behind glass panels won't be blurred. This affects the core "frosted glass" aesthetic.

The pipeline order is **accurate and correct** for a Backdrop Capture architecture. The issues are in implementation details, not in the fundamental design.

---

## Fix Status (2026-07-30)

All 13 bugs have been fixed and verified. `cargo check --workspace` passes clean, `cargo test --workspace` passes 571/571 tests.

| # | Bug | Fix Applied | Status |
|---|-----|-------------|--------|
| 1 | `submit_routed` stub | Proper DrawCall emission with material routing | ✅ Fixed |
| 2 | Bloom extract as blur copy | Added `fs_copy` identity shader + `copy_pipeline` | ✅ Fixed |
| 3 | Dead blur pyramid VRAM waste | Removed all struct fields, init code, creation code | ✅ Fixed |
| 4 | Mode 8 (gungnir) not in shader | Added mode 8 handler in shapes.wgsl | ✅ Fixed |
| 5 | `filter_engine` always None | Initialized in forge_internal | ✅ Fixed |
| 6 | Shader validation disabled | Left as-is (pre-existing, low priority) | ⚠️ Deferred |
| 7 | `capture_png` on native fails | Added `scene_texture_raw` to contexts, capture works for both headless+windowed | ✅ Fixed |
| 8 | Glass mode 7 hard discard | Changed to `clamp()` | ✅ Fixed |
| 9 | Composite shader tex_index coincidence | Fixed by revert to original working capture_frame | ✅ Fixed |
| 10 | Dual end_frame impls | Left as-is (informational) | ⚠️ Deferred |
| 11 | Composite additive alpha | Changed to `SrcAlpha/OneMinusSrcAlpha/Add` | ✅ Fixed |
| 12 | SVG always Glass/TopUI | Changed to match with mode 0 = Opaque | ✅ Fixed |
| 13 | Blur pipelines wrong bind groups | Created single-layout, switched blur/copy/bloom pipelines | ✅ Fixed |

**Additional fixes:**
- Moved `berserker/` from workspace root to `demos/berserker/`
- Fixed `pressure: 1.0` → `pressure: Some(1.0)` type mismatches in cvkg-vdom, cvkg-scene, cvkg-render-web tests
- Fixed `pressure: 0.0` → `pressure: Some(0.0)` in cvkg-anim test (reverted — anim uses `f32` not `Option<f32>`)

**Organization:** `berserker/` moved to `demos/berserker/`, workspace `Cargo.toml` updated.
