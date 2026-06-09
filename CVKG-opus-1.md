# CVKG Opus v2 — The 2028 Implementation Plan
## Production-Grade Liquid Glass UI for the Cyberpunk Viking Era

---

> *"Most UI frameworks draw rectangles. CVKG simulates matter. Glass refracts. Metal weathers. Ice crystallizes. Fire dances. The screen is not a canvas — it is a physical space."*

---

## Document Status

**Version:** 2.0.0
**Date:** 2026-06-08
**Status:** Draft — Ready for Implementation
**Author:** CVKG Engineering (AI-assisted, human-directed)
**Based on:** CVKG-opus.md v1.0 + full codebase audit 2026-06-08

---

## What Changed from v1.0 and Why

The v1.0 plan was architecturally sound but written for an idealized codebase. This v2.0 plan is written against the **actual** codebase as audited on 2026-06-08. Key differences:

1. **MenuBar exists as a data model** (`cvkg-core/src/lib.rs:6993`) — the gap is rendering, not data structure.
2. **`GlassMaterial` already has `refraction_index`** (`cvkg-themes/src/lib.rs:179`) — the gap is wiring it to the GPU, not defining it.
3. **`AccessibilityPreferences::should_disable_glass()` exists** (`cvkg-core/src/lib.rs:6262`) — the gap is plumbing it into component render paths.
4. **`ButtonVariant` has 5 variants** (`cvkg-components/src/lib.rs:151`) — missing Glass/TintedGlass/Capsule, not the entire enum.
5. **`EffectId` has 6 variants** (`cvkg-render-gpu/src/kvasir/effects.rs:3`) — missing Frosted/LiquidChrome/InkBleed, not the entire enum.
6. **`PassId::PostProcess { pipeline_id: u64 }` exists** — the extensibility mechanism is already there.
7. **`SceneUniforms` already has `berzerker_rage` and `berzerker_mode`** — audio-reactive visuals can hook into existing uniforms.
8. **The `ParticleComputeNode` already exists in the render graph** — EmberDrift needs configuration, not infrastructure.

This plan accounts for every existing wire and every missing wire. No speculation. No idealized APIs. Only what exists and what must be added.

---

## Architectural Principles (Unchanged from v1.0)

**1. The Material Illusion Principle** — Every UI element is matter with physical properties. The renderer simulates light, not draws shapes.

**2. The Surgical Rule** — Touch only what is required. Extend, don't rebuild.

**3. The 2028 Standard** — Will this look exceptional in 2028? Physically-accurate rendering. Semantic adaptivity. Spatial computing readiness.

---

## Current State Assessment (Verified Against Actual Code)

| # | Area | Actual State | File | Gap |
|---|------|-------------|------|-----|
| 1 | Kawase blur pyramid | Production-quality | `cvkg-render-gpu/src/passes/glass.rs` | None — keep as-is |
| 2 | Glass shader | Good, static tint | `cvkg-render-gpu/src/shaders/material_glass.wgsl` | Needs Snell's law, adaptive tint, edge smear |
| 3 | Kvasir render graph | Sophisticated, production | `cvkg-render-gpu/src/kvasir/nodes.rs` | Needs BackdropRegion pass |
| 4 | SleipnirSolver RK4 | Mathematically correct | `cvkg-anim/src/lib.rs` | None — keep as-is |
| 5 | BifrostModifier | 3 fields, no tint mode | `cvkg-core/src/lib.rs:1561` | Needs tint_mode, fresnel_strength |
| 6 | OKLCH theme engine | CPU-only | `cvkg-themes/src/lib.rs:19` | Needs GPU bridge |
| 7 | GlassMaterial struct | Complete with IOR | `cvkg-themes/src/lib.rs:175` | Not wired to ColorTheme |
| 8 | ColorTheme | 13 fields, no berserker | `cvkg-core/src/lib.rs:3149` | Needs berserker(), glass_tint_adapt |
| 9 | MenuBar data model | Complete | `cvkg-core/src/lib.rs:6993` | Needs GPU rendering component |
| 10 | Dock | Missing entirely | — | New component needed |
| 11 | Toolbar/TitleBar | YggdrasilWindow stub | `cvkg-components/src/window.rs` | Needs dedicated component |
| 12 | Segmented Control | Missing entirely | — | New component needed |
| 13 | Button variants | 5 of 8 needed | `cvkg-components/src/lib.rs:151` | Needs Glass, TintedGlass, Capsule |
| 14 | Context menu | RadialMenu only | `cvkg-components/src/radial_menu.rs` | Needs GaldraMenu |
| 15 | Sidebar chrome | Flat surface() | `cvkg-components/src/container.rs:52` | Needs glass wrapper |
| 16 | SearchBar | MimirSpotlight has input | `cvkg-components/src/command_palette.rs:16` | Needs extraction |
| 17 | Berserker preset | Missing | `cvkg-core/src/lib.rs:3200` | Needs berserker() fn |
| 18 | Runic ornament system | Missing | — | New component needed |
| 19 | Ambient particles | ParticleComputeNode exists | `cvkg-render-gpu/src/kvasir/nodes.rs:6` | Needs configuration |
| 20 | Audio-reactive | SceneUniforms has rage | `cvkg-core/src/lib.rs:3228` | Needs audio→rage bridge |
| 21 | Per-element blur | Full-scene only | `cvkg-render-gpu/src/passes/glass.rs:5` | Needs BackdropRegionNode |
| 22 | Edge-smear | Missing | `cvkg-render-gpu/src/shaders/material_glass.wgsl` | New shader code |
| 23 | Parallax depth | Missing | — | New modifier needed |
| 24 | IOR-accurate glass | Ad-hoc math | `cvkg-render-gpu/src/shaders/material_glass.wgsl` | Needs Snell's law |
| 25 | EffectId::Frosted | Missing | `cvkg-render-gpu/src/kvasir/effects.rs:3` | Add variant + dispatch |
| 26 | Accessibility plumbing | should_disable_glass() exists | `cvkg-core/src/lib.rs:6262` | Needs per-component wiring |

---

## Phase 0 — Architectural Debt Clearance
### Timeline: 3 days | Risk: Low | Impact: Unblocks Everything

These are not new features. These are missing wires between pieces that already exist. Every task in this phase is under 100 lines of code.

---

### Task 0.1 — Wire OKLCH GlassMaterial → ColorTheme GPU Uniforms

**Objective:** Connect the existing `GlassMaterial.tint_color` (OKLCH) to the GPU-side `ColorTheme.glass_base` so that theme changes affect glass rendering.

**Files:**
- Modify: `cvkg-themes/src/lib.rs:175-205` (GlassMaterial struct)
- Modify: `cvkg-core/src/lib.rs:3149-3224` (ColorTheme struct)
- Modify: `cvkg-render-gpu/src/api.rs` (set_theme path)

**Step 1: Add conversion function to cvkg-themes**

In `cvkg-themes/src/lib.rs`, add after line 205:

```rust
/// Convert a GlassMaterial's OKLCH tint into RGBA for GPU uniforms.
/// Called by the renderer when the theme changes.
pub fn glass_material_to_gpu_patch(mat: &GlassMaterial) -> [f32; 4] {
    // Convert OKLCH to RGBA. OklchColor already has to_rgba().
    let rgba = mat.tint_color.to_rgba();
    [rgba.r, rgba.g, rgba.b, mat.tint_opacity]
}
```

**Step 2: Add glass_tint_adapt to ColorTheme**

In `cvkg-core/src/lib.rs`, add a field to the `ColorTheme` struct (after `rune_opacity` at line 3161):

```json
{
  "path": "cvkg-core/src/lib.rs",
  "line": 3161,
  "add_after": "    pub rune_opacity: f32,\n    /// Weight of adaptive tint from backdrop [0.0, 1.0].\n    /// 0.0 = static theme tint, 1.0 = fully adaptive.\n    pub glass_tint_adapt: f32,",
}
```

**Step 3: Update all three ColorTheme presets**

In `cvkg-core/src/lib.rs`, add `glass_tint_adapt` to each preset:

- `asgard()` (line ~3178): add `glass_tint_adapt: 0.35,`
- `midgard()` (line ~3196): add `glass_tint_adapt: 0.0,`
- `vibrant_glass()` (line ~3216): add `glass_tint_adapt: 0.65,`

**Step 4: Wire in SurtrRenderer::set_theme**

In `cvkg-render-gpu/src/api.rs`, find the `set_theme` method. After writing the ColorTheme uniform, add:

```rust
// If the theme has a glass material patch, apply it
if let Some(ref glass_mat) = theme.glass_material_override {
    let patch = cvkg_themes::glass_material_to_gpu_patch(glass_mat);
    self.theme_buffer.patch_glass_base(patch);
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-themes -p cvkg-core 2>&1 | tail -5
```
Expected: All tests pass. No warnings about unused fields.

**Step 5: Commit**
```bash
git add cvkg-themes/src/lib.rs cvkg-core/src/lib.rs cvkg-render-gpu/src/api.rs
git commit -m "feat(themes): wire OKLCH GlassMaterial tint to GPU ColorTheme uniforms"
```

---

### Task 0.2 — Wire bcs_frosted into EffectRegistry

**Objective:** Make the existing `bcs_frosted` WGSL function callable through the effect system.

**Files:**
- Modify: `cvkg-render-gpu/src/kvasir/effects.rs:3-11` (EffectId enum)
- Modify: `cvkg-render-gpu/src/kvasir/effects.rs:56-96` (compile_chain)
- Modify: `cvkg-core/src/lib.rs` (Renderer trait — add default no-op method)

**Step 1: Add EffectId variant**

In `cvkg-render-gpu/src/kvasir/effects.rs`, add to the enum after `ColorInvert`:

```rust
    /// Frosted glass approximation: noise-displaced multi-sample scatter.
    /// Parameters: [frost_amount, grain_size, clear_radius, clear_softness]
    Frosted,
```

**Step 2: Add dispatch branch in compile_chain**

In the same file, inside `EffectRegistry::compile_chain()`, add a match arm after the `ColorInvert` branch:

```rust
                EffectId::Frosted => {
                    let amount = node.parameters.get(0).unwrap_or(&0.5);
                    let grain = node.parameters.get(1).unwrap_or(&1.0);
                    let clear_r = node.parameters.get(2).unwrap_or(&0.3);
                    let clear_s = node.parameters.get(3).unwrap_or(&0.15);
                    wgsl.push_str(&format!(
                        "    // Frosted Glass Effect\n    c = bcs_frosted(uv, c, {:.3}, {:.3}, {:.3}, {:.3});\n",
                        amount, grain, clear_r, clear_s
                    ));
                }
```

**Step 3: Add Renderer trait method**

In `cvkg-core/src/lib.rs`, in the `Renderer` trait (around line 2900), add a default no-op method:

```rust
    /// Apply a frosted glass effect to a rectangle.
    /// Default implementation is a no-op; GPU renderers override.
    fn draw_frosted(&mut self, _rect: Rect, _amount: f32, _clear_radius: f32) {}
```

**Step 4: Wire in SurtrRenderer**

In `cvkg-render-gpu/src/renderer.rs`, find the effect dispatch section. Add:

```rust
// Frosted glass: uses the bcs_frosted stitchable function
if let Some(pipeline) = self.effect_pipelines.get("frosted") {
    // Bind and draw fullscreen pass with frosted parameters
    self.draw_effect_pass(pipeline, rect, &[amount, 1.0, clear_radius, 0.15]);
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass. `EffectId::Frosted` is reachable.

**Step 5: Commit**
```bash
git add cvkg-render-gpu/src/kvasir/effects.rs cvkg-core/src/lib.rs cvkg-render-gpu/src/renderer.rs
git commit -m "feat(effects): wire bcs_frosted into EffectRegistry dispatch"
```

---

### Task 0.3 — Add BackdropRegionNode to Kvasir Render Graph

**Objective:** Enable per-element isolated backdrop blur by adding a region-scissored copy pass.

**Files:**
- Create: `cvkg-render-gpu/src/passes/backdrop_region.rs`
- Modify: `cvkg-render-gpu/src/passes/mod.rs` (add module)
- Modify: `cvkg-render-gpu/src/kvasir/nodes.rs:14-29` (add PassId variant)
- Modify: `cvkg-render-gpu/src/kvasir/nodes.rs:60-158` (add to graph builder)

**Step 1: Create BackdropRegionNode**

Create file `cvkg-render-gpu/src/passes/backdrop_region.rs`:

```rust
//! Per-element isolated backdrop blur pass.
//! Copies a scissored region from the scene texture, runs two Kawase
//! downsample passes, and outputs a blur texture the glass pass can sample.

use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::resource::ResourceId;
use crate::kvasir::nodes::RES_SCENE;

/// Copies a rectangular region from the scene texture into a
/// half-resolution blur target, then runs 2 Kawase downsample passes.
/// This gives each glass element its own isolated blurred backdrop.
pub struct BackdropRegionNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    /// Region in logical pixels (from top-left).
    pub region: cvkg_core::Rect,
    /// Output resource ID (allocated by the graph builder).
    pub output_id: ResourceId,
}

impl BackdropRegionNode {
    pub fn new(region: cvkg_core::Rect, output_id: ResourceId) -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![output_id],
            region,
            output_id,
        }
    }
}

impl KvasirNode for BackdropRegionNode {
    fn label(&self) -> &'static str {
        "Backdrop Region"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> crate::kvasir::nodes::PassId {
        // Use PostProcess with a unique pipeline_id for each region
        crate::kvasir::nodes::PassId::PostProcess {
            pipeline_id: self.output_id.0 as u64,
        }
    }
    fn execute(&self, ctx: &mut ExecutionContext) {
        let scene_tex = ctx.registry.get_texture(RES_SCENE).unwrap();
        let blur_tex = ctx.registry.get_texture(self.output_id).unwrap();

        // Scissored copy: only copy the region this glass element occupies
        let mut encoder = ctx.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("backdrop_region_copy") }
        );

        {
            let src_view = scene_tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("backdrop_region_src"),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_view = blur_tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("backdrop_region_dst"),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Backdrop Region Copy"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            // Set scissor to the element's region (in physical pixels)
            let scale = ctx.scale_factor;
            pass.set_scissor_rect(
                (self.region.x * scale) as u32,
                (self.region.y * scale) as u32,
                (self.region.width * scale) as u32,
                (self.region.height * scale) as u32,
            );

            pass.set_pipeline(&ctx.renderer.copy_pipeline);
            // ... bind group setup same as BackdropCopyNode ...
            pass.draw(0..3, 0..1);
        }

        // Run 2 Kawase downsample passes on the region
        // (reuses existing kawase_down_pipeline from BackdropBlurNode)
        ctx.renderer.run_kawase_down(&mut encoder, blur_tex, 2);

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }
}
```

**Step 2: Register the module**

In `cvkg-render-gpu/src/passes/mod.rs`, add:

```rust
pub mod backdrop_region;
pub use backdrop_region::BackdropRegionNode;
```

**Step 3: Add to render graph builder**

In `cvkg-render-gpu/src/kvasir/nodes.rs`, in `build_render_graph()`, after the glass pass setup (after line 107), add a hook for per-element regions:

```rust
    // Per-element backdrop regions are inserted by the compositor
    // when GlassInstanceUniforms::blur_multiplier != 1.0.
    // The graph builder accepts them via add_backdrop_region().
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-render-gpu 2>&1 | tail -10
```
Expected: Compiles clean. No warnings.

**Step 4: Commit**
```bash
git add cvkg-render-gpu/src/passes/backdrop_region.rs cvkg-render-gpu/src/passes/mod.rs cvkg-render-gpu/src/kvasir/nodes.rs
git commit -m "feat(render-graph): add BackdropRegionNode for per-element isolated blur"
```

---

## Phase 1 — Physically Accurate Glass
### Timeline: 5 days | Risk: Medium | Impact: Transformative

This phase rewrites the glass shader from an ad-hoc approximation to a physically-grounded material model. The existing shader in `material_glass.wgsl` is the single most impactful file in the framework.

---

### Task 1.1 — Snell's Law Refraction

**Objective:** Replace the ad-hoc `lens_dir * lens_dist * 0.08` distortion with physically accurate Snell's law refraction.

**Files:**
- Modify: `cvkg-render-gpu/src/shaders/material_glass.wgsl`

**Current code (approximately lines 30-50 of material_glass.wgsl):**
```wgsl
let lens = lens_dir * lens_dist * 0.08 * variation;
```

**Replace with:**

```wgsl
/// Physically accurate refraction using Snell's law.
/// n1 = 1.0 (air), n2 = per-instance IOR from uniforms.
/// Returns UV offset for the refracted sample direction.
fn snell_refraction(normal: vec2<f32>, incident: vec2<f32>, ior: f32) -> vec2<f32> {
    let n_ratio = 1.0 / ior;
    let cos_i = -dot(normal, incident);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);

    // Total internal reflection
    if sin2_t > 1.0 {
        return reflect(incident, normal);
    }

    let cos_t = sqrt(1.0 - sin2_t);
    return n_ratio * incident + (n_ratio * cos_i - cos_t) * normal;
}

// In the main fragment function, replace the lens distortion:
let refracted = snell_refraction(lens_normal, view_dir, uniforms.ior);
let env_sample = textureSampleLevel(t_env, s_env, uv + refracted * 0.04, blur_mip);
```

**Add IOR to the uniform block** (in the same file, at the top):

```wgsl
struct GlassInstanceUniforms {
    tint_override: vec4<f32>,
    ior: f32,
    blur_multiplier: f32,
    frost_override: f32,
    _pad: f32,
};
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass. Shader compiles without WGSL errors.

**Commit:**
```bash
git add cvkg-render-gpu/src/shaders/material_glass.wgsl
git commit -m "feat(glass): replace ad-hoc distortion with Snell's law refraction"
```

---

### Task 1.2 — Adaptive Tint from Backdrop Luminance

**Objective:** Make glass tint respond to the content behind it, not just the theme.

**Files:**
- Modify: `cvkg-render-gpu/src/shaders/material_glass.wgsl`

**Add after the refraction sampling (in the same file):**

```wgsl
// Sample backdrop at 4 coarse mip-6 positions for dominant color
// Mip 6 = ~1/64 resolution, gives us the average color of the region
let s0 = textureSampleLevel(t_env, s_env, uv + vec2(-0.1, -0.1), 6.0).rgb;
let s1 = textureSampleLevel(t_env, s_env, uv + vec2( 0.1, -0.1), 6.0).rgb;
let s2 = textureSampleLevel(t_env, s_env, uv + vec2(-0.1,  0.1), 6.0).rgb;
let s3 = textureSampleLevel(t_env, s_env, uv + vec2( 0.1,  0.1), 6.0).rgb;
let backdrop_dominant = (s0 + s1 + s2 + s3) * 0.25;

// Adaptive tint: mix static theme tint with backdrop-derived tint
// glass_tint_adapt controls the weight (0 = static, 1 = fully adaptive)
let adaptive_tint = mix(theme.glass_base.rgb, backdrop_dominant * 0.3 + 0.7,
                        theme.glass_tint_adapt);

// Per-instance override: if tint_override.w > 0, blend toward it
let final_tint = mix(adaptive_tint, uniforms.tint_override.rgb,
                     uniforms.tint_override.a);
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass.

**Commit:**
```bash
git add cvkg-render-gpu/src/shaders/material_glass.wgsl
git commit -m "feat(glass): add adaptive tint from backdrop luminance sampling"
```

---

### Task 1.3 — Edge Smear Convolution

**Objective:** Add the characteristic Tahoe glass edge where blurred backdrop bleeds slightly beyond the boundary.

**Files:**
- Modify: `cvkg-render-gpu/src/shaders/material_glass.wgsl`

**Add after the main color computation:**

```wgsl
// Edge smear: extend blur slightly beyond the glass edge
// d_sdf is negative inside the glass, positive outside
let smear_dist = clamp(-d_sdf, 0.0, 3.0) / 3.0;
let smear_sample = textureSampleLevel(
    t_env, s_env,
    uv + lens_dir * smear_dist * 0.01,
    blur_mip
).rgb;
let smear_contribution = smear_sample * 0.15;

// Crystalline edge highlight: bright specular at the boundary
let edge_mask = smoothstep(0.5, 0.0, abs(d_sdf));
let crystal_edge = edge_mask * 0.4 * (0.7 + 0.3 * spec);

final_rgb += smear_contribution + crystal_edge;
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass.

**Commit:**
```bash
git add cvkg-render-gpu/src/shaders/material_glass.wgsl
git commit -m "feat(glass): add edge smear convolution and crystalline edge highlight"
```

---

### Task 1.4 — Sub-Surface Scattering Approximation

**Objective:** Make glass appear thicker at edges and thinner at center, simulating light diffusion through semi-translucent material.

**Files:**
- Modify: `cvkg-render-gpu/src/shaders/material_glass.wgsl`

**Replace the alpha computation:**

```wgsl
// Thickness: SDF distance from edge, normalized
// Negative SDF = inside glass. Deeper inside = thinner center.
let thickness = 1.0 - clamp(-d_sdf / (in.size.x * 0.5), 0.0, 1.0);
let sss_tint = mix(vec3(0.92, 0.96, 1.0), vec3(0.7, 0.8, 0.95), thickness);
final_rgb *= sss_tint;

// Alpha model: thicker at edges (more opaque), thinner at center
let sss_alpha = mix(0.06, 0.22, thickness);
let final_alpha = (sss_alpha + fresnel * 0.18) * in.color.a * clip_alpha;
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass.

**Commit:**
```bash
git add cvkg-render-gpu/src/shaders/material_glass.wgsl
git commit -m "feat(glass): add sub-surface scattering approximation for thickness variation"
```

---

### Task 1.5 — Per-Instance Glass Uniforms (Push Constants)

**Objective:** Allow each glass element to override tint, IOR, and blur independently.

**Files:**
- Create: `cvkg-render-gpu/src/types.rs` (add GlassInstanceUniforms)
- Modify: `cvkg-render-gpu/src/renderer.rs` (push constant path)
- Modify: `cvkg-core/src/lib.rs` (BifrostModifier extension)

**Step 1: Add GlassInstanceUniforms to types.rs**

In `cvkg-render-gpu/src/types.rs`, add:

```rust
/// Per-draw-call glass instance parameters.
/// Passed as push constants (fast path, no buffer allocation).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlassInstanceUniforms {
    /// Local tint override: [r, g, b, weight].
    /// weight=0 = use theme tint only, weight=1 = use local tint only.
    pub tint_override: [f32; 4],
    /// Per-instance IOR override. 0.0 = use theme default.
    pub ior_override: f32,
    /// Blur strength multiplier. 1.0 = normal, 2.0 = double blur.
    pub blur_multiplier: f32,
    /// Frost intensity override. 0.0 = theme default.
    pub frost_override: f32,
    pub _pad: f32,
}

impl Default for GlassInstanceUniforms {
    fn default() -> Self {
        Self {
            tint_override: [0.0; 4],
            ior_override: 0.0,
            blur_multiplier: 1.0,
            frost_override: 0.0,
            _pad: 0.0,
        }
    }
}
```

**Step 2: Extend BifrostModifier**

In `cvkg-core/src/lib.rs:1561`, add fields:

```rust
pub struct BifrostModifier {
    pub blur: f32,
    pub saturation: f32,
    pub opacity: f32,
    /// Tint mode: Fixed uses tint_color, Adaptive samples backdrop.
    pub tint_mode: BifrostTintMode,
    /// Fresnel strength multiplier. 0.0 = no fresnel, 1.0 = full.
    pub fresnel_strength: f32,
    /// Radius for backdrop color sampling (in logical pixels).
    pub backdrop_sample_radius: f32,
}

pub enum BifrostTintMode {
    /// Use the theme's glass_base color.
    Fixed,
    /// Sample the backdrop and derive tint adaptively.
    Adaptive,
    /// Use a custom tint color.
    Custom([f32; 4]),
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-render-gpu -p cvkg-core 2>&1 | tail -10
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-render-gpu/src/types.rs cvkg-core/src/lib.rs
git commit -m "feat(glass): add per-instance GlassInstanceUniforms push constants"
```

---

## Phase 2 — Desktop Chrome Components
### Timeline: 5 days | Risk: Low | Impact: Essential

These are the components every macOS-class desktop app requires. They are built on the glass foundation from Phase 1.

---

### Task 2.1 — NornirBar (Menu Bar Component)

**Objective:** Render the existing `MenuBar` data model as a glass menu bar with cascading submenus.

**Files:**
- Create: `cvkg-components/src/chrome/nornir_bar.rs`
- Modify: `cvkg-components/src/chrome/mod.rs` (register module)
- Modify: `cvkg-components/src/lib.rs` (export)

**The MenuBar data model already exists at `cvkg-core/src/lib.rs:6993`.** This task creates the rendering component.

**Create `cvkg-components/src/chrome/nornir_bar.rs`:**

```rust
//! NornirBar — The application menu bar.
//! Named after the Nornir (Urd, Verdandi, Skuld), the three fates.
//!
//! Renders the existing `cvkg_core::MenuBar` data model as a horizontal
//! glass menu bar with cascading submenus.

use cvkg_core::{MenuBar, MenuItem, KeyboardShortcut, Rect, Renderer, View, Never};
use cvkg_core::accessibility::{AriaProperties, AriaRole};

/// The application menu bar. Renders at the top of the window with
/// glass background, horizontal menu items, and cascading submenus.
///
/// CONTRACT: NornirBar owns the top 28pt of the window's content area.
/// It is always present; hiding it causes layout to reflow upward.
pub struct NornirBar {
    /// The menu data model (from cvkg-core).
    pub menu_bar: MenuBar,
    /// Whether the bar floats over content (unified titlebar/toolbar style).
    pub floating: bool,
    /// Currently open menu index (None = all closed).
    open_menu: Option<usize>,
    /// Pointer position for hover tracking.
    pointer_pos: [f32; 2],
}

impl NornirBar {
    /// Create a new NornirBar from a MenuBar data model.
    pub fn new(menu_bar: MenuBar) -> Self {
        Self {
            menu_bar,
            floating: false,
            open_menu: None,
            pointer_pos: [0.0, 0.0],
        }
    }

    /// Set whether the bar floats over content.
    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }

    /// Handle a pointer move event.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) {
        self.pointer_pos = [x, y];
    }

    /// Handle a pointer down event. Returns true if a menu item was activated.
    pub fn on_pointer_down(&mut self) -> bool {
        // Hit-test against menu item rects
        // If a top-level menu header is clicked, toggle it
        // If a submenu item is clicked, fire its action
        false // Stub — full implementation in subagent
    }
}

impl View for NornirBar {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Glass background (full width, 28pt tall)
        renderer.bifrost(rect, 25.0, 1.2, 0.65);

        // 2. Render each top-level menu header
        let mut x = rect.x + 8.0;
        for (i, item) in self.menu_bar.items.iter().enumerate() {
            match item {
                MenuItem::Submenu { label, .. } => {
                    let label_w = renderer.measure_text(label, 13.0).0;
                    let item_rect = Rect {
                        x, y: rect.y,
                        width: label_w + 16.0,
                        height: 28.0,
                    };

                    // Highlight if open or hovered
                    if self.open_menu == Some(i) {
                        renderer.fill_rounded_rect(item_rect, 4.0, [1.0, 1.0, 1.0, 0.12]);
                    }

                    renderer.draw_text(label, x + 8.0, rect.y + 8.0, 13.0, [0.9, 0.9, 0.92, 1.0]);

                    // If open, render submenu as floating glass panel
                    if self.open_menu == Some(i) {
                        render_submenu(renderer, item, item_rect);
                    }

                    x += label_w + 16.0;
                }
                _ => {}
            }
        }
    }

    fn aria_properties(&self) -> AriaProperties {
        AriaProperties::new(AriaRole::Menubar)
    }
}

fn render_submenu(renderer: &mut dyn Renderer, item: &MenuItem, anchor: Rect) {
    // Render a floating glass panel below the menu header
    // with the submenu items. Uses bifrost background + item rows.
    // Full implementation delegated to subagent.
}
```

**Register the module in `cvkg-components/src/chrome/mod.rs`:**

```rust
pub mod nornir_bar;
pub use nornir_bar::NornirBar;
```

**Export from `cvkg-components/src/lib.rs`:**

```rust
pub use chrome::NornirBar;
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -10
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/chrome/nornir_bar.rs cvkg-components/src/chrome/mod.rs cvkg-components/src/lib.rs
git commit -m "feat(components): add NornirBar menu bar component"
```

---

### Task 2.2 — HeimdallDock

**Objective:** Create a macOS-style dock with magnification, auto-hide, and bounce animations.

**Files:**
- Create: `cvkg-components/src/chrome/heimdall_dock.rs`
- Modify: `cvkg-components/src/chrome/mod.rs`

**Create `cvkg-components/src/chrome/heimdall_dock.rs`:**

```rust
//! HeimdallDock — macOS-style dock with magnification and auto-hide.
//! Named after Heimdall, guardian of the Bifrost bridge.

use cvkg_core::{Rect, Renderer, View, Never};
use cvkg_anim::SleipnirSolver;

/// Compute magnified size for a dock item based on pointer proximity.
/// Uses a Gaussian envelope centered on the pointer with σ = 80px.
/// Maximum magnification: 2.0× at zero distance.
pub fn dock_item_magnification(
    item_center: f32,
    pointer_x: f32,
    base_size: f32,
    max_scale: f32,
) -> f32 {
    let sigma = 80.0_f32;
    let dist = (item_center - pointer_x).abs();
    let gaussian = (-dist * dist / (2.0 * sigma * sigma)).exp();
    1.0 + (max_scale - 1.0) * gaussian
}

/// A single item in the dock.
pub struct DockItem {
    pub id: String,
    pub icon: String,
    pub label: String,
    pub badge: Option<u32>,
    pub is_running: bool,
}

/// macOS-style dock with magnification, auto-hide, and bounce animations.
pub struct HeimdallDock {
    pub items: Vec<DockItem>,
    pub position: DockPosition,
    pub auto_hide: bool,
    pub magnification: f32,
    pointer_x: f32,
    hide_solver: Option<SleipnirSolver>,
}

pub enum DockPosition {
    Bottom,
    Left,
    Right,
}

impl HeimdallDock {
    pub fn new(items: Vec<DockItem>) -> Self {
        Self {
            items,
            position: DockPosition::Bottom,
            auto_hide: false,
            magnification: 2.0,
            pointer_x: 0.0,
            hide_solver: None,
        }
    }

    pub fn on_pointer_move(&mut self, x: f32, _y: f32) {
        self.pointer_x = x;
    }
}

impl View for HeimdallDock {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass platter background
        let platter_rect = match self.position {
            DockPosition::Bottom => Rect {
                x: rect.x + 20.0,
                y: rect.y + rect.height - 68.0,
                width: rect.width - 40.0,
                height: 56.0,
            },
            _ => rect, // Left/Right variants
        };

        renderer.bifrost(platter_rect, 25.0, 1.2, 0.7);
        renderer.fill_rounded_rect(platter_rect, 16.0, [0.15, 0.15, 0.18, 0.85]);

        // Render each item with magnification
        let base_size = 48.0;
        let mut x = platter_rect.x + 12.0;
        let center_y = platter_rect.y + platter_rect.height / 2.0;

        for item in &self.items {
            let item_center = x + base_size / 2.0;
            let scale = dock_item_magnification(
                item_center, self.pointer_x, base_size, self.magnification
            );
            let scaled_size = base_size * scale;

            let item_rect = Rect {
                x: x - (scaled_size - base_size) / 2.0,
                y: center_y - scaled_size / 2.0,
                width: scaled_size,
                height: scaled_size,
            };

            // Icon background (rounded rect)
            renderer.fill_rounded_rect(item_rect, 12.0, [0.2, 0.2, 0.25, 0.9]);

            // Running indicator dot
            if item.is_running {
                let dot_rect = Rect {
                    x: item_rect.x + item_rect.width / 2.0 - 2.0,
                    y: item_rect.y + item_rect.height + 4.0,
                    width: 4.0,
                    height: 4.0,
                };
                renderer.fill_ellipse(dot_rect, [0.0, 1.0, 0.95, 1.0]);
            }

            x += base_size + 8.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_magnification_at_zero_distance() {
        let scale = dock_item_magnification(100.0, 100.0, 48.0, 2.0);
        assert!((scale - 2.0).abs() < 0.01, "At zero distance, scale should be 2.0");
    }

    #[test]
    fn test_dock_magnification_at_far_distance() {
        let scale = dock_item_magnification(100.0, 500.0, 48.0, 2.0);
        assert!((scale - 1.0).abs() < 0.05, "At far distance, scale should approach 1.0");
    }

    #[test]
    fn test_dock_magnification_symmetry() {
        let left = dock_item_magnification(100.0, 50.0, 48.0, 2.0);
        let right = dock_item_magnification(100.0, 150.0, 48.0, 2.0);
        assert!((left - right).abs() < 0.001, "Magnification should be symmetric");
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-components --lib -- test_dock 2>&1 | tail -10
```
Expected: 3 tests pass.

**Commit:**
```bash
git add cvkg-components/src/chrome/heimdall_dock.rs cvkg-components/src/chrome/mod.rs
git commit -m "feat(components): add HeimdallDock with magnification and auto-hide"
```

---

### Task 2.3 — ValkyrieToolbar

**Objective:** Create a floating glass toolbar with segmented controls and flexible layout.

**Files:**
- Create: `cvkg-components/src/chrome/valkyrie_toolbar.rs`
- Modify: `cvkg-components/src/chrome/mod.rs`

**Create `cvkg-components/src/chrome/valkyrie_toolbar.rs`:**

```rust
//! ValkyrieToolbar — Floating glass toolbar with flexible layout.
//! Named after the Valkyries, choosers of the slain.

use cvkg_core::{Rect, Renderer, View, Never};

pub struct ValkyrieToolbar {
    pub leading: Vec<ToolbarItem>,
    pub center: Vec<ToolbarItem>,
    pub trailing: Vec<ToolbarItem>,
    pub radius: f32,
}

pub enum ToolbarItem {
    Button { label: String, icon: Option<String> },
    Segmented { options: Vec<String>, selected: usize },
    SearchField { placeholder: String },
    Spacer,
    FlexSpace,
    Separator,
}

impl ValkyrieToolbar {
    pub fn new() -> Self {
        Self {
            leading: Vec::new(),
            center: Vec::new(),
            trailing: Vec::new(),
            radius: 12.0,
        }
    }

    pub fn leading(mut self, items: Vec<ToolbarItem>) -> Self {
        self.leading = items;
        self
    }

    pub fn trailing(mut self, items: Vec<ToolbarItem>) -> Self {
        self.trailing = items;
        self
    }
}

impl View for ValkyrieToolbar {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass platter background
        renderer.fill_rounded_rect(rect, self.radius, [0.12, 0.12, 0.15, 0.88]);
        renderer.bifrost(rect, 20.0, 1.1, 0.6);

        // Render leading items (left-aligned)
        let mut x = rect.x + 8.0;
        let y = rect.y + 6.0;
        for item in &self.leading {
            let w = render_toolbar_item(renderer, item, x, y);
            x += w + 4.0;
        }

        // Render trailing items (right-aligned)
        let mut x = rect.x + rect.width - 8.0;
        for item in self.trailing.iter().rev() {
            let w = render_toolbar_item(renderer, item, x, y);
            x -= w + 4.0;
        }
    }
}

fn render_toolbar_item(renderer: &mut dyn Renderer, item: &ToolbarItem, x: f32, y: f32) -> f32 {
    match item {
        ToolbarItem::Button { label, .. } => {
            let w = renderer.measure_text(label, 12.0).0 + 16.0;
            let h = 28.0;
            renderer.fill_rounded_rect(
                Rect { x, y, width: w, height: h },
                6.0,
                [0.2, 0.2, 0.25, 0.8],
            );
            renderer.draw_text(label, x + 8.0, y + 7.0, 12.0, [0.9, 0.9, 0.92, 1.0]);
            w
        }
        ToolbarItem::Separator => {
            renderer.draw_line(x, y + 4.0, x, y + 24.0, [0.3, 0.3, 0.35, 0.5], 1.0);
            12.0
        }
        ToolbarItem::Spacer => 8.0,
        ToolbarItem::FlexSpace => 0.0, // Handled by layout
        _ => 32.0,
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/chrome/valkyrie_toolbar.rs cvkg-components/src/chrome/mod.rs
git commit -m "feat(components): add ValkyrieToolbar floating glass toolbar"
```

---

### Task 2.4 — HrungnirSegmented Control

**Objective:** Create a glass-segmented control with spring-animated sliding pill indicator.

**Files:**
- Create: `cvkg-components/src/interactive/hrungnir_segment.rs`
- Modify: `cvkg-components/src/interactive/mod.rs`

**Create `cvkg-components/src/interactive/hrungnir_segment.rs`:**

```rust
//! HrungnirSegmented — Glass-segmented control with spring-animated pill.
//! Named after Hrungnir, whose heart was stone with three sharp corners.

use cvkg_core::{Rect, Renderer, View, Never};
use cvkg_anim::{SleipnirSolver, SleipnirParams};

pub struct HrungnirSegmented {
    pub segments: Vec<String>,
    pub selected: usize,
    pub style: SegmentedStyle,
    pill_x: f32,
    pill_width: f32,
    anim: SleipnirSolver,
}

pub enum SegmentedStyle {
    Glass,
    Capsule,
    Iconic,
    Labeled,
}

impl HrungnirSegmented {
    pub fn new(segments: Vec<String>, selected: usize) -> Self {
        Self {
            segments,
            selected,
            style: SegmentedStyle::Glass,
            pill_x: 0.0,
            pill_width: 0.0,
            anim: SleipnirSolver::new(SleipnirParams::snappy()),
        }
    }

    pub fn on_select(&mut self, index: usize) {
        self.selected = index;
        // The anim solver will interpolate pill position on next frame
    }
}

impl View for HrungnirSegmented {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass platter
        renderer.fill_rounded_rect(rect, 8.0, [0.1, 0.1, 0.12, 0.85]);

        // Sliding pill indicator (white tint at low opacity)
        let pill_rect = Rect {
            x: rect.x + self.pill_x,
            y: rect.y + 2.0,
            width: self.pill_width,
            height: rect.height - 4.0,
        };
        renderer.fill_rounded_rect(pill_rect, 6.0, [1.0, 1.0, 1.0, 0.15]);

        // Segment labels
        let mut x = rect.x + 8.0;
        for (i, label) in self.segments.iter().enumerate() {
            let w = renderer.measure_text(label, 12.0).0 + 16.0;
            let color = if i == self.selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.7, 0.7, 0.75, 1.0]
            };
            renderer.draw_text(label, x, rect.y + 8.0, 12.0, color);
            x += w + 4.0;
        }
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/interactive/hrungnir_segment.rs cvkg-components/src/interactive/mod.rs
git commit -m "feat(components): add HrungnirSegmented glass-segmented control"
```

---

### Task 2.5 — Glass Button Variants

**Objective:** Add Glass, TintedGlass, and Capsule variants to the existing `ButtonVariant` enum.

**Files:**
- Modify: `cvkg-components/src/lib.rs:151-162` (ButtonVariant enum)
- Modify: `cvkg-components/src/interactive.rs` (bg_color and border_color match arms)

**Step 1: Add variants to ButtonVariant**

In `cvkg-components/src/lib.rs`, add after `Link`:

```rust
    /// Glass button: frosted background, no border, subtle backdrop.
    Glass,

    /// Tinted glass: glass base with accent color tint.
    TintedGlass,

    /// Capsule button: pill-shaped, solid fill, high contrast.
    Capsule,
```

**Step 2: Add match arms in interactive.rs**

In `cvkg-components/src/interactive.rs`, in the `bg_color` method, add arms after the `Link` branch:

```rust
            ButtonVariant::Glass => {
                if is_pressed {
                    [0.15, 0.15, 0.18, 0.9]
                } else if is_hovered {
                    [0.12, 0.12, 0.15, 0.85]
                } else {
                    [0.08, 0.08, 0.12, 0.7]
                }
            }
            ButtonVariant::TintedGlass => {
                // Accent-tinted glass: use theme accent color at low opacity
                let accent = theme::accent();
                if is_pressed {
                    [accent[0] * 0.3 + 0.1, accent[1] * 0.3 + 0.1, accent[2] * 0.3 + 0.1, 0.9]
                } else if is_hovered {
                    [accent[0] * 0.2 + 0.08, accent[1] * 0.2 + 0.08, accent[2] * 0.2 + 0.08, 0.85]
                } else {
                    [accent[0] * 0.15 + 0.05, accent[1] * 0.15 + 0.05, accent[2] * 0.15 + 0.05, 0.75]
                }
            }
            ButtonVariant::Capsule => {
                let accent = theme::accent();
                if is_pressed {
                    [accent[0] * 0.8, accent[1] * 0.8, accent[2] * 0.8, 1.0]
                } else if is_hovered {
                    [accent[0] * 0.9, accent[1] * 0.9, accent[2] * 0.9, 1.0]
                } else {
                    accent
                }
            }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/lib.rs cvkg-components/src/interactive.rs
git commit -m "feat(components): add Glass, TintedGlass, Capsule button variants"
```

---

## Phase 3 — Panel and Navigation Chrome
### Timeline: 4 days | Risk: Low | Impact: High

---

### Task 3.1 — NiflheimSidebar (Glass Sidebar Chrome)

**Objective:** Add glass chrome styling to the existing `NavigationSplitView` sidebar.

**Files:**
- Create: `cvkg-components/src/chrome/niflheim_sidebar.rs`
- Modify: `cvkg-components/src/chrome/mod.rs`

**Create `cvkg-components/src/chrome/niflheim_sidebar.rs`:**

```rust
//! NiflheimSidebar — Glass chrome wrapper for sidebar panels.
//! Named after Niflheim, the realm of ice and mist.

use cvkg_core::{Rect, Renderer, View, Never};

/// Glass chrome configuration for sidebar panels.
pub struct NiflheimSidebar {
    pub vibrancy: SidebarVibrancy,
    pub source_list_style: bool,
}

pub enum SidebarVibrancy {
    Translucent,
    Standard,
    Heavy,
}

impl NiflheimSidebar {
    pub fn new() -> Self {
        Self {
            vibrancy: SidebarVibrancy::Standard,
            source_list_style: true,
        }
    }

    /// Render the glass background for a sidebar region.
    /// Call this before rendering sidebar content.
    pub fn render_background(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (blur, opacity) = match self.vibrancy {
            SidebarVibrancy::Translucent => (15.0, 0.4),
            SidebarVibrancy::Standard => (25.0, 0.65),
            SidebarVibrancy::Heavy => (35.0, 0.85),
        };

        // Glass background
        renderer.bifrost(rect, blur, 1.2, opacity);

        // Separator line on trailing edge
        let sep_x = rect.x + rect.width - 0.5;
        renderer.draw_line(
            sep_x, rect.y,
            sep_x, rect.y + rect.height,
            [0.3, 0.3, 0.35, 0.6],
            1.0,
        );

        // Inner glow on separator
        renderer.draw_line(
            sep_x - 1.0, rect.y,
            sep_x - 1.0, rect.y + rect.height,
            [0.5, 0.5, 0.6, 0.3],
            1.0,
        );
    }

    /// Render a source-list row with glass highlight.
    pub fn render_row(
        &self,
        renderer: &mut dyn Renderer,
        rect: Rect,
        label: &str,
        is_selected: bool,
        is_hovered: bool,
    ) {
        let bg = if is_selected {
            [1.0, 1.0, 1.0, 0.14]
        } else if is_hovered {
            [1.0, 1.0, 1.0, 0.08]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        if bg[3] > 0.0 {
            renderer.fill_rounded_rect(rect, 6.0, bg);
        }

        renderer.draw_text(label, rect.x + 12.0, rect.y + 6.0, 12.0, [0.9, 0.9, 0.92, 1.0]);
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/chrome/niflheim_sidebar.rs cvkg-components/src/chrome/mod.rs
git commit -m "feat(components): add NiflheimSidebar glass sidebar chrome"
```

---

### Task 3.2 — RuneInspector (Floating Panel)

**Objective:** Create a detachable floating inspector panel with glass background and spring-physics drag.

**Files:**
- Create: `cvkg-components/src/chrome/rune_inspector.rs`
- Modify: `cvkg-components/src/chrome/mod.rs`

**Create `cvkg-components/src/chrome/rune_inspector.rs`:**

```rust
//! RuneInspector — Detachable floating inspector panel.
//! Named after the runic tablets used by Norse scholars.

use cvkg_core::{Rect, Renderer, View, Never};
use cvkg_anim::{SleipnirSolver, SleipnirParams};

pub struct RuneInspector {
    pub title: String,
    pub position: InspectorPosition,
    pub size: (f32, f32),
    pub is_expanded: bool,
    drag_offset: [f32; 2],
    drag_solver: SleipnirSolver,
}

pub enum InspectorPosition {
    TrailingAttached,
    Floating { x: f32, y: f32 },
}

impl RuneInspector {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            position: InspectorPosition::Floating { x: 100.0, y: 100.0 },
            size: (280.0, 400.0),
            is_expanded: true,
            drag_offset: [0.0; 2],
            drag_solver: SleipnirSolver::new(SleipnirParams {
                stiffness: 280.0,
                damping: 28.0,
                mass: 1.0,
            }),
        }
    }
}

impl View for RuneInspector {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background (heavier blur than toolbars)
        renderer.bifrost(rect, 30.0, 1.3, 0.75);
        renderer.fill_rounded_rect(rect, 12.0, [0.06, 0.06, 0.08, 0.9]);

        // Title bar
        let title_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 36.0,
        };
        renderer.fill_rounded_rect(title_rect, 12.0, [0.1, 0.1, 0.12, 0.5]);
        renderer.draw_text(&self.title, rect.x + 12.0, rect.y + 10.0, 13.0, [0.9, 0.9, 0.92, 1.0]);

        // Close button
        let close_rect = Rect {
            x: rect.x + rect.width - 28.0,
            y: rect.y + 8.0,
            width: 20.0,
            height: 20.0,
        };
        renderer.fill_ellipse(close_rect, [0.8, 0.2, 0.2, 0.9]);
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/chrome/rune_inspector.rs cvkg-components/src/chrome/mod.rs
git commit -m "feat(components): add RuneInspector floating panel"
```

---

### Task 3.3 — GaldraMenu (Context Menu)

**Objective:** Create a right-click context menu with glass styling.

**Files:**
- Create: `cvkg-components/src/interactive/galdra_menu.rs`
- Modify: `cvkg-components/src/interactive/mod.rs`

**Create `cvkg-components/src/interactive/galdra_menu.rs`:**

```rust
//! GaldraMenu — Right-click context menu with glass styling.
//! Named after Galdr, the spoken form of Norse magic.

use cvkg_core::{Rect, Renderer, View, Never};
use std::sync::Arc;

pub struct GaldraMenu {
    pub items: Vec<GaldraMenuItem>,
    pub anchor: MenuAnchor,
}

pub enum MenuAnchor {
    Pointer,
    Rect(Rect),
}

pub enum GaldraMenuItem {
    Action {
        label: String,
        shortcut: Option<String>,
        enabled: bool,
        action: Arc<dyn Fn() + Send + Sync>,
    },
    Submenu {
        label: String,
        items: Vec<GaldraMenuItem>,
    },
    Separator,
}

impl GaldraMenu {
    pub fn new(items: Vec<GaldraMenuItem>) -> Self {
        Self {
            items,
            anchor: MenuAnchor::Pointer,
        }
    }

    pub fn render_at(&self, renderer: &mut dyn Renderer, x: f32, y: f32) {
        let item_height = 28.0;
        let menu_width = 200.0;
        let menu_height = self.items.len() as f32 * item_height + 8.0;

        let menu_rect = Rect {
            x, y,
            width: menu_width,
            height: menu_height,
        };

        // Glass background
        renderer.bifrost(menu_rect, 20.0, 1.1, 0.7);
        renderer.fill_rounded_rect(menu_rect, 8.0, [0.08, 0.08, 0.1, 0.92]);

        // Render items
        let mut iy = menu_rect.y + 4.0;
        for item in &self.items {
            match item {
                GaldraMenuItem::Action { label, shortcut, enabled, .. } => {
                    let item_rect = Rect {
                        x: menu_rect.x + 4.0,
                        y: iy,
                        width: menu_rect.width - 8.0,
                        height: item_height,
                    };

                    let color = if *enabled {
                        [0.9, 0.9, 0.92, 1.0]
                    } else {
                        [0.5, 0.5, 0.55, 0.5]
                    };

                    renderer.draw_text(label, item_rect.x + 8.0, item_rect.y + 7.0, 12.0, color);

                    if let Some(shortcut) = shortcut {
                        let sw = renderer.measure_text(shortcut, 11.0).0;
                        renderer.draw_text(
                            shortcut,
                            item_rect.x + item_rect.width - sw - 8.0,
                            item_rect.y + 7.0,
                            11.0,
                            [0.6, 0.6, 0.65, 0.8],
                        );
                    }
                }
                GaldraMenuItem::Separator => {
                    renderer.draw_line(
                        menu_rect.x + 8.0, iy + 4.0,
                        menu_rect.x + menu_rect.width - 8.0, iy + 4.0,
                        [0.2, 0.2, 0.25, 0.5],
                        1.0,
                    );
                }
                _ => {}
            }
            iy += item_height;
        }
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/interactive/galdra_menu.rs cvkg-components/src/interactive/mod.rs
git commit -m "feat(components): add GaldraMenu context menu"
```

---

### Task 3.4 — MimirSearch (Standalone Search Bar)

**Objective:** Extract the search input from `MimirSpotlight` into a standalone component.

**Files:**
- Create: `cvkg-components/src/interactive/mimir_search.rs`
- Modify: `cvkg-components/src/interactive/mod.rs`

**Create `cvkg-components/src/interactive/mimir_search.rs`:**

```rust
//! MimirSearch — Standalone glass search bar.
//! Named after Mimir's well of wisdom.

use cvkg_core::{Rect, Renderer, View, Never};

pub struct MimirSearch {
    pub query: String,
    pub placeholder: String,
    pub style: SearchBarStyle,
    pub searching: bool,
}

pub enum SearchBarStyle {
    Compact,
    Expanded,
}

impl MimirSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            placeholder: "Search...".to_string(),
            style: SearchBarStyle::Compact,
            searching: false,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
}

impl View for MimirSearch {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background (pill shape)
        let radius = match self.style {
            SearchBarStyle::Compact => rect.height / 2.0,
            SearchBarStyle::Expanded => 8.0,
        };

        renderer.bifrost(rect, 15.0, 1.0, 0.5);
        renderer.fill_rounded_rect(rect, radius, [0.1, 0.1, 0.12, 0.85]);

        // Search icon (magnifying glass)
        renderer.draw_text("🔍", rect.x + 10.0, rect.y + 6.0, 14.0, [0.6, 0.6, 0.65, 0.8]);

        // Query text or placeholder
        let text = if self.query.is_empty() {
            &self.placeholder
        } else {
            &self.query
        };
        let color = if self.query.is_empty() {
            [0.5, 0.5, 0.55, 0.6]
        } else {
            [0.9, 0.9, 0.92, 1.0]
        };
        renderer.draw_text(text, rect.x + 32.0, rect.y + 7.0, 13.0, color);

        // Clear button (if query is non-empty)
        if !self.query.is_empty() {
            let clear_x = rect.x + rect.width - 24.0;
            renderer.draw_text("✕", clear_x, rect.y + 6.0, 12.0, [0.6, 0.6, 0.65, 0.8]);
        }
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/interactive/mimir_search.rs cvkg-components/src/interactive/mod.rs
git commit -m "feat(components): add MimirSearch standalone search bar"
```

---

## Phase 4 — Berserker Theme: The Differentiator
### Timeline: 4 days | Risk: Low | Impact: Identity-Defining

---

### Task 4.1 — ColorTheme::berserker()

**Objective:** Add a first-class berserker theme preset to ColorTheme.

**Files:**
- Modify: `cvkg-core/src/lib.rs:3163-3224` (ColorTheme presets)

**Add after the `vibrant_glass()` function:**

```rust
    /// Berserker Mode: Blood-iron neon, aggressive contrast, forge-heated glass.
    /// This is the theme that makes CVKG applications unmistakable.
    pub fn berserker() -> Self {
        Self {
            // Blood-iron neon: warm red with aggressive intensity
            primary_neon: [1.0, 0.08, 0.12, 1.8],
            // Bone-white shatter: cold contrast to the blood red
            shatter_neon: [0.95, 0.92, 0.88, 1.6],
            // Smoked obsidian glass: near-black with iron undertones
            glass_base: [0.03, 0.02, 0.02, 0.88],
            // Forge-edge: hot orange-white at the glass boundary
            glass_edge: [0.8, 0.35, 0.08, 0.7],
            // Elder rune glow: aged amber-gold
            rune_glow: [0.9, 0.72, 0.3, 1.0],
            // Heart of the ember: deep burning orange
            ember_core: [0.98, 0.25, 0.05, 1.0],
            // The void between stars
            background_deep: [0.01, 0.005, 0.005, 1.0],
            // Berserker has no gentle cursor glow — the UI itself blazes
            mani_glow: [0.8, 0.2, 0.05, 0.08],
            // Maximum blur — the glass is thick, ancient, imperfect
            glass_blur_strength: 0.85,
            // Wide shatter edges — violence is visible
            shatter_edge_width: 2.8,
            // Aggressive neon bloom
            neon_bloom_radius: 0.035,
            // Elder runes are always visible in berserker mode
            rune_opacity: 0.85,
            // Low adaptivity — the glass has its own character
            glass_tint_adapt: 0.15,
        }
    }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-core 2>&1 | tail -5
```
Expected: All tests pass.

**Commit:**
```bash
git add cvkg-core/src/lib.rs
git commit -m "feat(themes): add ColorTheme::berserker() preset"
```

---

### Task 4.2 — ÆttiRunes (Runic Ornament System)

**Objective:** Create an ornamental border system using SVG path templates.

**Files:**
- Create: `cvkg-components/src/ornamental/aetti_frame.rs`
- Create: `cvkg-components/src/ornamental/mod.rs`
- Modify: `cvkg-components/src/lib.rs`

**Create `cvkg-components/src/ornamental/aetti_frame.rs`:**

```rust
//! ÆttiFrame — Runic ornamental border system.
//! Named after the Ættir, the three groups of eight runes in the Elder Futhark.

use cvkg_core::{Rect, Renderer, View, Never};

pub struct ÆttiFrame {
    pub style: RunicStyle,
    pub intensity: f32,
    pub animate: bool,
}

pub enum RunicStyle {
    /// Elder Futhark characters carved into stone.
    CarvedStone,
    /// Interlocking knotwork pattern.
    Knotwork,
    /// Hammered metal with rivets at corners.
    HammeredMetal,
    /// Dragon-scale tessellation.
    DragonScale,
    /// Ice crystal formations.
    IceCrystal,
}

impl ÆttiFrame {
    pub fn new(style: RunicStyle) -> Self {
        Self {
            style,
            intensity: 0.8,
            animate: true,
        }
    }

    /// Render the ornamental border for a given rect.
    pub fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Border width
        let bw = 4.0;

        // Render each edge with the selected style
        match self.style {
            RunicStyle::CarvedStone => {
                // Top edge: rune sequence
                self.render_rune_border(renderer, rect, bw);
            }
            RunicStyle::Knotwork => {
                self.render_knotwork_border(renderer, rect, bw);
            }
            RunicStyle::HammeredMetal => {
                self.render_metal_border(renderer, rect, bw);
            }
            RunicStyle::DragonScale => {
                self.render_scale_border(renderer, rect, bw);
            }
            RunicStyle::IceCrystal => {
                self.render_ice_border(renderer, rect, bw);
            }
        }
    }

    fn render_rune_border(&self, renderer: &mut dyn Renderer, rect: Rect, bw: f32) {
        // Simplified: draw a border line with rune-colored glow
        let rune_color = [0.9, 0.72, 0.3, self.intensity * 0.6];

        // Top edge
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, rune_color, bw);
        // Bottom edge
        renderer.draw_line(rect.x, rect.y + rect.height, rect.x + rect.width, rect.y + rect.height, rune_color, bw);
        // Left edge
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.height, rune_color, bw);
        // Right edge
        renderer.draw_line(rect.x + rect.width, rect.y, rect.x + rect.width, rect.y + rect.height, rune_color, bw);
    }

    fn render_knotwork_border(&self, _renderer: &mut dyn Renderer, _rect: Rect, _bw: f32) {
        // Full implementation in subagent
    }

    fn render_metal_border(&self, _renderer: &mut dyn Renderer, _rect: Rect, _bw: f32) {
        // Full implementation in subagent
    }

    fn render_scale_border(&self, _renderer: &mut dyn Renderer, _rect: Rect, _bw: f32) {
        // Full implementation in subagent
    }

    fn render_ice_border(&self, _renderer: &mut dyn Renderer, _rect: Rect, _bw: f32) {
        // Full implementation in subagent
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/ornamental/aetti_frame.rs cvkg-components/src/ornamental/mod.rs cvkg-components/src/lib.rs
git commit -m "feat(components): add ÆttiFrame runic ornament border system"
```

---

### Task 4.3 — ÞrymrSurface (Material Wear Shaders)

**Objective:** Add battle-worn surface damage shaders for berserker mode.

**Files:**
- Modify: `cvkg-render-gpu/src/shaders/material_opaque.wgsl`

**Add at the end of the file (before the final closing brace):**

```wgsl
/// Apply battle-worn surface damage: scratches, cracks, burn marks.
/// damage_level: [0.0, 1.0] — 0 = pristine, 1 = heavily damaged.
/// damage_seed: per-component random seed for variation.
fn worn_surface(
    uv: vec2<f32>,
    base_color: vec4<f32>,
    damage_level: f32,
    damage_seed: f32,
) -> vec4<f32> {
    var color = base_color;

    // Scratches: high-frequency noise along a directional gradient
    let scratch_dir = normalize(vec2(0.7, 0.3) + vec2(damage_seed * 0.2, damage_seed * 0.15));
    let scratch_uv = vec2(dot(uv, scratch_dir), dot(uv, vec2(-scratch_dir.y, scratch_dir.x)));
    let scratch = fbm(scratch_uv * 80.0 + damage_seed * 10.0);
    let scratch_mask = smoothstep(0.72, 0.78, scratch) * damage_level;

    // Cracks: larger, branching fractures
    let crack_n = fbm(uv * 12.0 + damage_seed * 7.0);
    let crack_mask = smoothstep(0.68, 0.73, crack_n) * damage_level * 0.6;

    // Burn marks: radial dark patches
    let burn_center = vec2(fract(damage_seed * 3.7), fract(damage_seed * 5.3));
    let burn_dist = distance(uv, burn_center);
    let burn_mask = smoothstep(0.3, 0.0, burn_dist) * damage_level * vnoise(uv * 5.0) * 0.7;

    // Apply: scratches lighten (exposed metal), cracks and burns darken
    color.rgb += scratch_mask * 0.25;
    color.rgb -= crack_mask * 0.4;
    color.rgb -= burn_mask * 0.5;

    return color;
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo test -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: All tests pass.

**Commit:**
```bash
git add cvkg-render-gpu/src/shaders/material_opaque.wgsl
git commit -m "feat(shaders): add ÞrymrSurface worn/damaged material shader"
```

---

### Task 4.4 — EmberDrift (Ambient Particle System)

**Objective:** Configure the existing `ParticleComputeNode` for ambient UI particles.

**Files:**
- Modify: `cvkg-render-gpu/src/passes/compute.rs` (ParticleComputeNode configuration)
- Modify: `cvkg-render-gpu/src/kvasir/nodes.rs` (graph builder — enable particles by scene type)

**Step 1: Add particle configuration to the graph builder**

In `cvkg-render-gpu/src/kvasir/nodes.rs`, modify the `build_render_graph` function to accept a `ParticleConfig` parameter and conditionally enable the particle pass:

```rust
pub struct ParticleConfig {
    pub max_particles: u32,
    pub emit_rate: f32,
    pub particle_type: ParticleType,
    pub gravity: f32,
    pub turbulence: f32,
    pub color: [f32; 4],
    pub lifetime: f32,
}

pub enum ParticleType {
    Ember,      // Berserker mode: orange-red, upward drift
    Snow,       // Niflheim: white, gentle fall
    Spark,      // Asgard: cyan, upward
    RuneFragment, // Yggdrasil: gold/green leaf fragments
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            max_particles: 512,
            emit_rate: 32.0,
            particle_type: ParticleType::Ember,
            gravity: -20.0, // Negative = upward
            turbulence: 0.3,
            color: [0.98, 0.25, 0.05, 0.8],
            lifetime: 3.0,
        }
    }
}
```

**Step 2: Gate the particle pass on scene type**

In the graph builder, the particle pass is already wired. Add a condition:

```rust
    // Particles: only when scene type is ambient-capable
    let scene_type = self.scene_type;
    if matches!(scene_type, SCENE_AURORA | SCENE_NEBULA | SCENE_YGGDRASIL)
        || berzerker_rage > 0.1
    {
        let particles = builder.add_node(Box::new(ParticleComputeNode::new(
            particle_config,
        )));
        builder.connect(last_scene_node, RES_SCENE, particles);
        last_scene_node = particles;
    }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-render-gpu 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-render-gpu/src/kvasir/nodes.rs cvkg-render-gpu/src/passes/compute.rs
git commit -m "feat(particles): configure EmberDrift ambient particle system"
```

---

### Task 4.5 — Seiðr Holographic Effect Application

**Objective:** Apply the existing Seiðr holographic scanline effect to sidebar and panel glass surfaces.

**Files:**
- Modify: `cvkg-components/src/chrome/niflheim_sidebar.rs`
- Modify: `cvkg-render-gpu/src/kvasir/effects.rs` (add Seiðr to EffectId)

**Step 1: Add Seiðr to EffectId**

In `cvkg-render-gpu/src/kvasir/effects.rs`, add variant:

```rust
    /// Holographic scanline effect with flicker.
    /// Parameters: [intensity, scan_speed]
    Seiðr,
```

**Step 2: Add dispatch in compile_chain**

```rust
                EffectId::Seiðr => {
                    let intensity = node.parameters.get(0).unwrap_or(&0.3);
                    let speed = node.parameters.get(1).unwrap_or(&0.8);
                    wgsl.push_str(&format!(
                        "    // Seiðr Holographic Scanlines\n    let scan = sin(u.y * 200.0 - time * {:.3}) * {:.3};\n    c.rgb += vec3<f32>(scan, scan, scan) * {:.3};\n",
                        speed, intensity, intensity
                    ));
                }
```

**Step 3: Apply in NiflheimSidebar**

In `niflheim_sidebar.rs`, after rendering the glass background:

```rust
        // Apply Seiðr holographic effect in Asgard/Berserker mode
        if self.realm == Realm::Asgard || self.realm == Realm::Berserker {
            renderer.draw_frosted(rect, 0.15, 0.0); // Subtle frost as base
        }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-render-gpu -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-render-gpu/src/kvasir/effects.rs cvkg-components/src/chrome/niflheim_sidebar.rs
git commit -m "feat(effects): apply Seiðr holographic scanlines to sidebar glass"
```

---

### Task 4.6 — MjolnirFrame Variants

**Objective:** Add RuneStone, HammeredMetal, DragonScale, IceCrystal, and VoidRift frame styles.

**Files:**
- Create: `cvkg-components/src/ornamental/mjolnir_frame_ext.rs`
- Modify: `cvkg-components/src/ornamental/mod.rs`

**Create `cvkg-components/src/ornamental/mjolnir_frame_ext.rs`:**

```rust
//! Extended MjolnirFrame variants for berserker theming.

use cvkg_core::{Rect, Renderer};

pub enum MjolnirFrameStyle {
    /// Standard geometric frame (existing).
    Standard,
    /// Carved runestone: weathered edges with embedded runes.
    RuneStone { runes: Vec<RuneGlyph> },
    /// Hammered metal: irregular forged surface with rivets.
    HammeredMetal { oxidation: f32 },
    /// Dragon scale: interlocking scale tessellation.
    DragonScale { scale_size: f32 },
    /// Ice crystal: fractal ice growth from corners.
    IceCrystal { growth_progress: f32 },
    /// Void rift: dark energy tearing at boundaries.
    VoidRift { rift_intensity: f32 },
}

pub struct RuneGlyph {
    pub character: char,
    pub position: f32, // 0.0 to 1.0 along the edge
    pub glow_intensity: f32,
}

/// Render a frame with the selected style.
pub fn render_mjolnir_frame(
    renderer: &mut dyn Renderer,
    rect: Rect,
    style: &MjolnirFrameStyle,
    color: [f32; 4],
) {
    match style {
        MjolnirFrameStyle::Standard => {
            renderer.stroke_rect(rect, color, 2.0);
        }
        MjolnirFrameStyle::RuneStone { runes } => {
            // Weathered border
            renderer.stroke_rect(rect, [0.4, 0.35, 0.3, 0.8], 3.0);
            // Rune glyphs along the top edge
            for rune in runes {
                let x = rect.x + rect.width * rune.position;
                let y = rect.y - 8.0;
                renderer.draw_text(
                    &rune.character.to_string(),
                    x, y,
                    10.0,
                    [0.9, 0.72, 0.3, rune.glow_intensity],
                );
            }
        }
        MjolnirFrameStyle::HammeredMetal { oxidation } => {
            let base = [0.5 - oxidation * 0.2, 0.45 - oxidation * 0.15, 0.4, 0.9];
            renderer.stroke_rect(rect, base, 4.0);
            // Rivets at corners
            for corner in &[
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x, rect.y + rect.height),
                (rect.x + rect.width, rect.y + rect.height),
            ] {
                renderer.fill_ellipse(
                    cvkg_core::Rect {
                        x: corner.0 - 3.0, y: corner.1 - 3.0,
                        width: 6.0, height: 6.0,
                    },
                    [0.6, 0.55, 0.5, 1.0],
                );
            }
        }
        MjolnirFrameStyle::DragonScale { scale_size } => {
            // Simplified: overlapping rounded rects along edges
            let count = (rect.width / scale_size) as usize;
            for i in 0..count {
                let x = rect.x + i as f32 * scale_size;
                renderer.fill_rounded_rect(
                    cvkg_core::Rect { x, y: rect.y - 2.0, width: scale_size * 0.8, height: 6.0 },
                    3.0,
                    [0.2, 0.5, 0.3, 0.7],
                );
            }
        }
        MjolnirFrameStyle::IceCrystal { growth_progress } => {
            // Fractal-like ice growth from corners
            let corners = [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x, rect.y + rect.height),
                (rect.x + rect.width, rect.y + rect.height),
            ];
            for (cx, cy) in &corners {
                let size = 8.0 + growth_progress * 12.0;
                renderer.draw_line(*cx, *cy, *cx + size, *cy - size, [0.7, 0.85, 1.0, 0.6], 1.5);
                renderer.draw_line(*cx, *cy, *cx - size, *cy - size, [0.7, 0.85, 1.0, 0.6], 1.5);
            }
        }
        MjolnirFrameStyle::VoidRift { rift_intensity } => {
            // Dark energy tearing: jagged dark lines
            let jitter = rift_intensity * 4.0;
            renderer.stroke_rect(
                cvkg_core::Rect {
                    x: rect.x - jitter,
                    y: rect.y - jitter,
                    width: rect.width + jitter * 2.0,
                    height: rect.height + jitter * 2.0,
                },
                [0.1, 0.0, 0.15, 0.9],
                2.0 + jitter,
            );
        }
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/ornamental/mjolnir_frame_ext.rs cvkg-components/src/ornamental/mod.rs
git commit -m "feat(components): add MjolnirFrame variants (RuneStone, HammeredMetal, DragonScale, IceCrystal, VoidRift)"
```

---

## Phase 5 — Motion and Physics Refinement
### Timeline: 3 days | Risk: Low | Impact: Polish

---

### Task 5.1 — Parallax Depth System

**Objective:** Add depth-based parallax offset for scroll and window drag.

**Files:**
- Create: `cvkg-core/src/parallax.rs`
- Modify: `cvkg-core/src/lib.rs` (export)

**Create `cvkg-core/src/parallax.rs`:**

```rust
//! Parallax depth system — elements shift based on scroll/drag velocity.

use crate::{Rect, Renderer, View, ViewModifier, ModifiedView};

/// Modifier that applies parallax depth offset during scroll or window drag.
pub struct ParallaxModifier {
    /// Depth in the UI stack. 0.0 = background, 1.0 = foreground.
    pub depth: f32,
    /// Maximum parallax offset in logical pixels.
    pub max_offset: f32,
}

impl ParallaxModifier {
    pub fn new(depth: f32) -> Self {
        Self {
            depth: depth.clamp(0.0, 1.0),
            max_offset: 4.0,
        }
    }

    pub fn with_max_offset(mut self, max: f32) -> Self {
        self.max_offset = max;
        self
    }
}

impl ViewModifier for ParallaxModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Get scroll velocity from the renderer's scene state
        // Apply offset proportional to depth
        let offset_x = self.depth * self.max_offset;
        let offset_y = self.depth * self.max_offset;

        renderer.push_transform(
            [offset_x, offset_y],
            [1.0, 1.0],
            0.0,
        );
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-core 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-core/src/parallax.rs cvkg-core/src/lib.rs
git commit -m "feat(core): add ParallaxModifier for depth-based scroll offset"
```

---

### Task 5.2 — Audio-Reactive Visuals Bridge

**Objective:** Connect the existing `AudioEngine` trait to `SceneUniforms::berzerker_rage` so the UI pulses with audio.

**Files:**
- Modify: `cvkg-core/src/lib.rs` (SceneUniforms — add audio fields)
- Modify: `cvkg-render-gpu/src/renderer.rs` (audio analysis to rage bridge)

**Step 1: Add audio analysis fields to SceneUniforms**

In `cvkg-core/src/lib.rs`, in the `SceneUniforms` struct (after `berzerker_mode`), add:

```rust
    /// Audio analysis: bass energy [0,1]
    pub audio_bass: f32,
    /// Audio analysis: mid energy [0,1]
    pub audio_mid: f32,
    /// Audio analysis: treble energy [0,1]
    pub audio_treble: f32,
    /// Beat detected this frame (1 = true, 0 = false)
    pub audio_beat: u32,
```

**Step 2: Bridge audio to berserker rage in the renderer**

In `cvkg-render-gpu/src/renderer.rs`, in the frame update path:

```rust
// Audio-reactive berserker rage boost
if let Some(ref audio) = self.audio_engine {
    let analysis = audio.analyze();
    let beat_boost = if analysis.beat { analysis.amplitude * 1.5 } else { 0.0 };
    self.scene_uniforms.audio_bass = analysis.bass;
    self.scene_uniforms.audio_mid = analysis.mid;
    self.scene_uniforms.audio_treble = analysis.treble;
    self.scene_uniforms.audio_beat = if analysis.beat { 1 } else { 0 };
    self.scene_uniforms.berzerker_rage =
        (self.scene_uniforms.berzerker_rage + beat_boost).clamp(0.0, 1.0);
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-render-gpu -p cvkg-core 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-core/src/lib.rs cvkg-render-gpu/src/renderer.rs
git commit -m "feat(audio): bridge AudioEngine analysis to berzerker rage uniforms"
```

---

### Task 5.3 — Declarative Animation Primitives

**Objective:** Add a declarative animation API on top of the existing `SleipnirSolver`.

**Files:**
- Modify: `cvkg-core/src/lib.rs`

**Add after the modifier implementations (around line 1700):**

```rust
/// Animation target property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTarget {
    Opacity, Scale, OffsetX, OffsetY, Rotation,
}

/// Declarative animation modifier wrapping SleipnirSolver.
pub struct AnimatedModifier {
    pub target: AnimationTarget,
    pub target_value: f32,
    pub params: cvkg_anim::SleipnirParams,
}

/// Trait for views that support declarative animation.
pub trait AnimatableView: View + Sized {
    fn animated(
        self,
        target: AnimationTarget,
        value: f32,
        params: cvkg_anim::SleipnirParams,
    ) -> ModifiedView<Self, AnimatedModifier> {
        ModifiedView::new(self, AnimatedModifier { target, target_value: value, params })
    }

    fn spring_opacity(self, opacity: f32) -> ModifiedView<Self, AnimatedModifier> {
        self.animated(AnimationTarget::Opacity, opacity, cvkg_anim::SleipnirParams::default())
    }

    fn spring_scale(self, scale: f32) -> ModifiedView<Self, AnimatedModifier> {
        self.animated(AnimationTarget::Scale, scale, cvkg_anim::SleipnirParams::snappy())
    }
}

impl<V: View> AnimatableView for V {}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-core 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-core/src/lib.rs
git commit -m "feat(core): add declarative AnimatableView trait with spring animations"
```

---

## Phase 6 — Production Hardening and 2028-Readiness
### Timeline: 3 days | Risk: Low | Impact: Longevity

---

### Task 6.1 — Accessibility: ShieldWall Completion

**Objective:** Wire `AccessibilityPreferences::should_disable_glass()` into every glass component's render path.

**Files:**
- Modify: All files in `cvkg-components/src/chrome/` and `cvkg-components/src/interactive/`

**Pattern for each component — add at the start of `render()`:**

```rust
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let prefs = cvkg_core::accessibility_preferences();
        if prefs.should_disable_glass() {
            return self.render_opaque(renderer, rect);
        }
        // ... existing glass rendering ...
    }

    fn render_opaque(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, crate::theme::surface());
        // ... render content without glass ...
    }
```

**Add ARIA properties to each component:**

```rust
    fn aria_properties(&self) -> cvkg_core::AriaProperties {
        cvkg_core::AriaProperties::new(cvkg_core::AriaRole::Menubar) // or appropriate role
    }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-components 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-components/src/chrome/ cvkg-components/src/interactive/
git commit -m "feat(a11y): wire accessibility preferences into all glass components"
```

---

### Task 6.2 — Performance Contracts

**Objective:** Add `PerformanceContract` type for principled quality degradation.

**Files:**
- Create: `cvkg-core/src/performance.rs`
- Modify: `cvkg-core/src/lib.rs` (export)

**Create `cvkg-core/src/performance.rs`:**

```rust
//! Performance contracts — declare quality degradation behavior per component.

pub struct PerformanceContract {
    pub max_render_us: u32,
    pub uses_glass: bool,
    pub continuous_animation: bool,
    pub min_tier: RenderTier,
    pub tier3_fallback: Tier3Fallback,
}

pub enum RenderTier { Tier3, Tier2, Tier1 }

pub enum Tier3Fallback { FlatOpaque, NoEffects, Hidden }

impl PerformanceContract {
    pub fn chrome_standard() -> Self {
        Self {
            max_render_us: 300,
            uses_glass: true,
            continuous_animation: false,
            min_tier: RenderTier::Tier2,
            tier3_fallback: Tier3Fallback::FlatOpaque,
        }
    }
}
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-core 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-core/src/performance.rs cvkg-core/src/lib.rs
git commit -m "feat(core): add PerformanceContract for quality degradation"
```

---

### Task 6.3 — Display Environment Enum (2028 Readiness)

**Objective:** Add `DisplayEnvironment` enum for spatial computing readiness.

**Files:**
- Modify: `cvkg-core/src/lib.rs`

**Add near the Renderer trait:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayEnvironment {
    #[default]
    Flat,
    Spatial,
    HeadsUp,
}

// Add to Renderer trait:
fn display_environment(&self) -> DisplayEnvironment { DisplayEnvironment::Flat }
```

**Verification:**
```bash
cd /D/rex/projects/cvkg && cargo check -p cvkg-core 2>&1 | tail -5
```
Expected: Compiles clean.

**Commit:**
```bash
git add cvkg-core/src/lib.rs
git commit -m "feat(core): add DisplayEnvironment enum for 2028 spatial computing readiness"
```

---

## Implementation Order Matrix

| Task | Dependency | Parallel With | Est. Lines | Risk |
|------|-----------|---------------|------------|------|
| 0.1 OKLCH to GPU | None | 0.2, 0.3 | ~60 | Low |
| 0.2 bcs_frosted wire | None | 0.1, 0.3 | ~30 | Low |
| 0.3 BackdropRegion | None | 0.1, 0.2 | ~150 | Medium |
| 1.1 Snell's law | 0.3 | 1.2-1.5 | ~80 | Medium |
| 1.2 Adaptive tint | 0.1 | 1.1, 1.3-1.5 | ~50 | Low |
| 1.3 Edge smear | 1.1 | 1.2, 1.4-1.5 | ~60 | Low |
| 1.4 SSS approx | 1.1 | 1.2, 1.3, 1.5 | ~40 | Low |
| 1.5 Per-instance uniforms | 0.1 | 1.1-1.4 | ~100 | Low |
| 2.1 NornirBar | 1.5 | 2.2-2.5 | ~250 | Low |
| 2.2 HeimdallDock | 1.5 | 2.1, 2.3-2.5 | ~200 | Low |
| 2.3 ValkyrieToolbar | 2.4 | 2.1, 2.2 | ~200 | Low |
| 2.4 HrungnirSegmented | 1.5 | 2.1-2.3, 2.5 | ~150 | Low |
| 2.5 Glass buttons | 1.5 | 2.1-2.4 | ~80 | Low |
| 3.1 NiflheimSidebar | 1.5 | 3.2-3.4 | ~150 | Low |
| 3.2 RuneInspector | 1.5 | 3.1, 3.3-3.4 | ~150 | Low |
| 3.3 GaldraMenu | 1.5 | 3.1, 3.2, 3.4 | ~180 | Low |
| 3.4 MimirSearch | None | 3.1-3.3 | ~120 | Low |
| 4.1 Berserker preset | 0.1 | 4.2-4.6 | ~40 | Low |
| 4.2 AEttiRunes | 4.1 | 4.3-4.6 | ~250 | Low |
| 4.3 ThrymrSurface | 4.1 | 4.2, 4.4-4.6 | ~80 | Low |
| 4.4 EmberDrift | 4.1 | 4.2, 4.3, 4.5-4.6 | ~100 | Low |
| 4.5 Seiðr application | None | 4.1-4.4, 4.6 | ~40 | Low |
| 4.6 MjolnirFrame ext. | 4.1 | 4.2-4.5 | ~200 | Low |
| 5.1 Parallax | 1.5 | 5.2, 5.3 | ~80 | Low |
| 5.2 Audio-reactive | None | 5.1, 5.3 | ~60 | Low |
| 5.3 Declarative anim | None | 5.1, 5.2 | ~100 | Low |
| 6.1 A11y completion | 2.1-3.4 | 6.2-6.5 | ~200 | Low |
| 6.2 Performance contracts | All | 6.3 | ~120 | Low |
| 6.3 DisplayEnvironment | 5.1 | 6.1, 6.2 | ~40 | Low |

**Total estimated new code: ~3,200 lines**
**Total calendar time: ~24 working days (5 weeks)**

---

## The Non-Negotiable Design Standards

1. **No hardcoded color values in component code.** All colors from `use_theme()` or `GlassInstanceUniforms`.
2. **Every `pub fn` has a doc comment** (WHY and WHAT, not HOW). CVKG Guideline #6.
3. **Every animation uses `SleipnirSolver`.** No `lerp()` in animation code. Springs only.
4. **Every component degrades gracefully on Tier3.** Test the `tier3_fallback` path.
5. **Every glass element declares `depth: f32`** for the parallax system.
6. **The Norse naming convention is sacred.** No exceptions.
7. **Every phase compiles and all tests pass before moving to the next.** No exceptions.

---

## Verification Protocol (After Each Phase)

```bash
# 1. Build check (must be clean)
cd /D/rex/projects/cvkg && cargo check --workspace 2>&1 | grep -E "^error" | head -20

# 2. All tests pass
cargo test --workspace 2>&1 | grep -E "test result|FAILED"

# 3. Format check
cargo fmt -- --check 2>&1 | head -5

# 4. Commit
git add -A
git commit -m "feat(phase-N): <description>"
git push origin HEAD:main
```

---

## What Success Looks Like

When this plan is complete, a CVKG application will:

- Open with a `HeimdallDock` that magnifies under the pointer with a Gaussian falloff, items bouncing on launch with `SleipnirSolver` physics
- Display a `NornirBar` menu whose glass background adapts to window content via backdrop luminance sampling
- Show floating `RuneInspector` panels that drag with physical weight (stiffness 280, damping 28), snapping to edges with springs
- In berserker mode, pulse with `EmberDrift` embers, `AEttiRunes` borders breathing amber, chromatic aberration on bass beats, and `ThrymrSurface` battle damage
- On accessibility-enabled systems, render as clean, high-contrast, fully accessible with zero glass
- On 2028 hardware, activate spatial computing mode with real environment-mapped glass

No other native UI framework in any language will produce applications that look or feel like this.

---

*Valhöll or nothing.*

**Document version:** 2.0.0 | **Architecture level:** Magnum Opus | **Task count:** 29 atomic tasks
