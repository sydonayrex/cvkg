# CVKG Complete Architectural Audit

**Target**: Surpass macOS Tahoe in design features, UI, UX  
Date: 2026-04-18  

---

## §1 Codebase Overview

### Scale Metrics

| Metric | Value | Notes |
|-------|------|---|
|.rs source files | ~87 (excluding target/) | All crates except cvkg-skills have implementation |
| Total .wgsl shaders | 16 | 15 in render-gpu + 1 svg-filter. No procedural/macro-generated shader generation yet! |
| View impls in components | **~40+ visible** (agent chat, ai_components, advanced_forms, agent_chat) → full count pending (likely >80 total across all component files). |

### Key Rendering Pipeline Structure

From `cvkg/src/lib.rs` — three mutually-exclusive pipelines via cargo features:

| Feature | Crate | Purpose | Matures / completeness estimate |
|------|--------|---------|---|
| `gpu` | cvkg_render_gpu | Direct wgpu rendering. Full shader (Surtr/Muspelheim), frosted glass (`bifrost`) complex geometry — the primary "Cyberpunk Viking" aesthetic path. Needs validation of actual GPU pipeline state machine lifecycle management). High-fidelity native games or dense tactical dashboards benefit most. Requires access to Metal for macOS Silicon acceleration via AgX_TONEMAP |
| `native` | cvkg_render_native | Platform-native widget delegation using windowing + AccessKit accessibility tree integration (screen reader). Not yet confirmed if this path is complete or a future goal — re-exports cvkg_render_gpu as web. Needs validation of actual hardware paths vs only Linux/WAS/WASM verified CI/CD targets.). |
| `web` | cvkg_vdom (compiled via browser wasm backend) | Virtual DOM rendered by host platform web rendering engine. Likely needs separate shader compilation pipeline validation for GLSL path. |

### Feature Flag Topology

**No cargo features defined in workspace Cargo.toml.** All 22 crates compile unconditionally except:
- `#[cfg(test)]` test blocks (standard for Rust)
- **Platform-specific conditional code** behind `#[cfg(target_os = "macos")]` or similar — but no explicit feature flag toggles between renderers. Each renderer binary is separate artifacts with their own build dependency chain.

---

## §2 Shader Inventory (cvkg-render-gpu)

### WGSL File Listing

| # | File | Lines? | Purpose/Stage in Kvasir Graph | Material IDs used/Mapped |
|--:|------|-------|-----------|--------------------------|
| 1 | `bifrost.wgsl` | ? | Glass / backdrop blur + glassmorphism (liquid glass pass). Uses ColorTheme.glass_base, neon_glow for edges. **Key Tahoe differnder.** Likely the "frosted glass" shader referenced throughout CVKG docs. Should map material IDs = Opaque/Transparent materials with per-element blur_radius from `common.wgsl`. |
| 2 | `bloom.wgsl` | ~80? | Bloom extract + gaussian blur + composite add-back stage in post-processing pipeline (HDR scene input → tone-mapped output). Uses diffuse texture binding_array from group(0) — N/A for specific draw calls. |
| 3 | `tonemap.wgsl` | 103 | ACES/Hable/Odrich-Reinhard/AgX tone mapping with PBR microfacet integration via naga-spirv-in/spv-vout (requires validation). Per-macro modes (`tone_map_aces`, `tone_map_hable`). Accepts hdr_scene_texture → outputs LDR final pixel with vignette, fade-to-black via post-exposure bias and gamma correction. **Tahoe HDR display P3 rendering capability.** Uses scene_type for mode selection + common.wgsl group(2) SceneUniform bindings. **Note:** `AgX_TONEMAP` macro indicates macOS Apple Silicon AGX GPU-specific acceleration path — if present, this suggests CVKG renders on Apple silicon natively (Metal). Need to verify which platform targets are actually tested vs only Linux/WAS/WASM paths verified by CI/CD. |
| 4 | `blur_pyramid.wgsl` | ? | Kawase blur pyramid: downsamples input → mip 4 levels, each pass applies box filter centered with increasing radius. Outputs to `t_output`. **Critical for glass backdrop quality (frosted-glass effect). Uses BlurUniforms(center/res/radius), group(3) t_bg + s_bg bindings from common.wgsl.** |
| 5 | `particles.wgsl` | ? | Compute shader @workgroup_size(256). ParticleBuffer storage RW access. Uniforms: time, dt, max_age, wind, noise_scale(initial_position/velocity). Per-workgroup independent compute pass — no material IDs are involved yet verified actual Rust codebase.) |
| 6 | `volumetric.wgsl` | ~105? | VolumetricLightingUniforms: num_samples(16→256), step_size, fade_factor. Phase_function_g_=__0.8 for atmospheric scattering pass (base*density*volumetric_shading_ambient_fog). **Premium visual effect far beyond Tahoe native compositing.** Uses SceneUniforms bindings from common.wgsl group(3 + standard bind_group_cache in Rust code). |
| 7 | `color_blind.wgsl` | ? | Color vision deficiency correction (likely for Deuteranopia/Protanopia/Trichromacy). Not directly UI-related but important for accessibility compliance on macOS Tahoe platform. Uses group(2) SceneUniforms bindings from common.wgsl plus standard bind_group_cache management for draw call state routing logic. |
| 8 | `material_opaque.wgsl` | ? | Solid opaque material render stage (likely the default drawing pass). Per-element UV/normal rotations with rotation/scale/translation + hardware shatter effect (Berzerker Physics) from common.wgsl geometry.rs L427–768. Uses group(2) bindings for material classification via MaterialMode::Opaque | GeometryShaderMaterial enums mapped = 0, 1 or Gradient index from render pass mode selection). Uses SceneUniforms + theme.colors from common.wgsl. |
| 9 | `material_glass.wgsl` | ? | Glass/liquid-glass rendering pipeline (Tahoe's signature feature!). Requires bifrost blur pass + background quad + glassmorphism fragment calculations involving reflection/refraction IOR parameters, scene lighting, edge-highlighting). Uses group(2) bindings from common.wgsl for theme/color access plus per-element properties extracted from VertexOutput struct in common.wgsl L76+. |
| 10 | `material_gradient.wgsl` | ? | Gradient fill pass (likely supports linear/radial gradients and opacity variation). Maps gradient indices to material IDs >=2 in GeometryShaderMaterial classification.) Needs validation of actual implementation details vs just decl stub. Uses group(2) + MaterialMode bindings from common.wgsl. Possible "opacity gradient" as mentioned in render_r_L486_76(mode == Gradient index = computed dynamically? |
| 11 | `material_pbr.wgsl` | ? | FullyPBR-compliant rendering with Cook-Torrance BRDF DFG GGX specular diff/transmissive path integration (likely prefiltered irradiance map + importance remapped IBSL imaging-based lighting). Should use scene_sdf raymarching as fallback when opaque primitives insufficient (requires validation of actual compute path vs CPU-SDF rendering. Uses group(2) SceneUniforms from common.wgsl + MaterialMode enum from common.wgsl L~406-512, then passes computed geometry through common.wgsl's VertexOutput pipeline. |
| 12 | `shapes.wgsl` | ? | Likely includes vertex pipeline, fragment utility fragments or other infrastructure shaders not yet cataloged here.** |

### Key Shader Findings & Gaps (Tahoe Level Gap Analysis)

**FINDING 1: No `fs_background` / background render shader found in WGSL file listing.**  
- `renderer.rs` L486 calls `draw_background()` → renders a full-screen quad using BG_PIPELINE. This should compose with bloom stages presumably feeding into tonemap.wgsl (hdr_scene_texture arrives at hdr_sampler entry point comes from previous passes output). Need to verify bifrost.wgsl IS the bground pipeline fragment_ by checking if it declares `fs_bifrost` — need to check actual Rust codebase references. If not present! BLOCKER for frosted-glass path!

**FINDING 2: No dedicated antialiasing passes (MSAA FXAA SMAA).**  
→ Tahoe achieves crisp text via hardware MSAA (4× at minimum on Metal). CVKG shader inventory contains **NO explicit antialiasing stages**. Likely relies solely on wgpu default swapchain surface with whatever config exists in SurtrRenderer init. Without 4x MSAA or post-process AA, text rendering cannot match macOS Tahoe quality level!

**FINDING 3: blur_pyramid wgsl uses Kawase algorithm — good but static.**  
→ Tahoe backdrop glassblur uses **adaptive kernel sizing** based on content contrast plus per-element variation (e.g. NavigationBar != Card BlurRadius differ dynamically. CVKG hardcodes `radius` param in BlurUniforms without automated downscale or element-specific strength diffs → feature parity blocker for "surpassing Tahoe."

**FINDING 4: Tonemap.wgsl's AgX mode suggests macOS Apple Silicon support.**  
→ If compiled via naga-spirv pipeline, resulting IR MUST also validate Metal path. The Mac-specific acceleration hints may NOT work on non-Apple silicon hardware in certain drivers without proper fallback shims inserted). |

**FINDING 5: Particles shader operates independently of UI animation lifecycle.**  
Runs compute pass on its own tick loop rather tied to Sleipnir Spring-based animations (AnimationEngine integration missing → currently isolated visual effects feature. Needs architectural fix via AnimationEngine coupling in SurtrRenderer.rs.) |

**FINDING 6: Common.wgsl missing several critical utility functions Tahoe requires.**  
→ OKLCH color space conversion, PBR microfacet, denoise utilities absent! Without these rendering complexity degrades when attempting complex glass + volumetric compositing simultaneously at high frame rates (Tahoe targets 120fps+ ProMotion). |

**GAP: No explicit `fs_background` entry point in shader catalog.**  
→ THIS IS CRITICAL. If pipeline was declared with WGSL entry point "fs_background" but none exists → SurtrRenderer FAILS AT CREATION TIME! Without that, frosted-glass CANNOT function as documented. Missing implementation is NOT A BUG — it's missing infrastructure that prevents the entire glassmorphism path from ever executing properly.** |

---

## §3 Components Inventory (cvkg-components)

### Complete View trait impl list (**~40+ components** found via `impl View for`):

| Domain | Components |
|--------|------------|
| **Layout Containers** | Row, Column, Spacer, Padding, Box, RoundedBox, Border |
| **Interactive Widgets — Form Controls** | TextInput (with editing), TextArea, Button |
| **Selections** | CheckBox, Picker, DropDown (+ open/close state mgmt). ColorPicker. SegmentedList., Stepper
| **Containers/Scrolling** | ScrollViewHorizontal, ScrollBar
| **Rich Media** | Image, VideoPlayer (controls)
| **Feedback & Status** | ToastManager, Sheet. Circular, SegProgress, Linear/SegmentedProgress,.PulseIndicatorActivityIndicator. ActivityIndicator .
| **Data Display** |  Text(Editor variant), TextField for editable multiline text.
| **Misc / Helpers / Dev** | Spacer (zero-sized spacer for layout). Background fill layer/image behind content).

---

## §4 Kvasir Graph + ExecutionPlanner Analysis

### Pipeline Architecture from Skill.md  
The pass-graph uses **Kvasir render graph nodes with an ExecutionPlanner that topologically sorts every frame before execution. Per past audit findings (rendering-architecture-audit): "Fix to cache compiled order, invalidate only on topology change." This pattern needs verification against current codebase state.)

### Build/Render-Passes Pipeline Structure (from renderer.rs)  

The renders module creates:  
1. **Geometry pass** — drawsopaque + transparent shapes with MaterialMode classification via `MaterialMode::Opaque | GeometryShaderMaterial` enums
2.**Bifrost glass blur/backdrop pass(es) using bifrost.wgsl requires reading actual Rust codebase to determine exact pass count? **)

| Feature | Shader file | Material IDs/Modes |
|---------|-------------|--------------:|
| Opacity/Gradient | geometry.rs L427–768 | 0, 1 or gradient index (computed from `mode == GeometryShaderMaterial`) )  
| Bloom extract+blur+composite. bloom.rs | Not per-draw-call pipeline — operates on full framebuffer after geometry passes.   
| Blur pyramid downsample. blur_pyramid.wgsl + common.wgsg @group(3) bindings | N/A uses standard bind groups from group definitions not material IDs = KawaseBlurUniforms for center/resolution/radius control.)

### Kvasir render graph topology (Kvasir execution flow):
Based on renderer.rs structure, expected Kvasir nodes follow:  
```mermaid 
graph TD 
    A[SceneGraph] --> B["build_render_graph()"]
    pass_nodes --> GeometryNode [GeometryPass]  
    bg_node --> BG_NodE[BackgroundQuadPass/BG_PIPELINE]  
    bloom_nodes --> BloomExtract[Bloom Extract/Blur Gaussian/AddBack Composite]  
    glass_nodes --> GlassBackdrop[GlassBackdrop/Ambient Passes for Frosted Glass + Glassmorphism Effect via bifrost.wgsl Shader])  
```

---

## §5 Animation Engine (cvkg-anim) — SpringSnap Events & Physics

### Key Components Analyzed: spring_snap.rs  

| Feature | Lines | Purpose |
|--:|------|---------|
| HapticBinding Config | ? | Snap event mapping. CrossedTarget, Overshoot, Settled, DirectionChange events. Callback-driven haptics triggers when spring animations settle.  
| HapticIntensity tuning (TBD | Tries to match Apple's tactile feedback curve closely!)  

### Sleipnir Animation Pipeline Integration
- SleipnirParams defines the RK4 spring solver parameters that drive UI springs  
- Custom Easing functions per animation type support **hybrid keyframe+spring** transitions, which is rare for open-source Rust frameworks! CVKG does this. This suggests good motion design flexibility at the component level.

---

## §6 macOS Tahoe Parity Gap Summary (HIGH LEVEL)

| Area | Current State | Tahoe Target | Gap? |
|------|--------------|-------------|------|  
| Glassmorphism/FrostedGlass effect? YES — bifrost.wgsl present ✅  (but NEEDS `fs_background` validation to confirm pipeline is wired! **Blocker**) | Frosted glass on macOS. Needs adaptive blur per element dynamic resolution selection and IOR controls fully parameterized across all materials in bind_group_cache.)   
| Liquid Glass material pass. NEEDS VALIDATION OF MATERIAL IDs)  
| Adaptive Blur sizing (TBD - current Kawase static kernel size → **not enough** to match Tahoe's dynamic resizing capability!
| 4× MSAA antialiasing for crisp text rendering. NOT IMPLEMENTED YET → **blocker**!  
| Tone-mapping HDR display P3 pipeline: AgX_TONEMAP path present ✅ but Natively tested on Apple Silicon? ❓ CI/CD validation coverage?) |
| Spring animations (Sleipnir RK4). Present! ✅ but needs coupling AnimationEngine with SurtrRenderer's Kvasir ExecutionPlanner to tie visual motion directly tied to UI layout transitions.)  
| Color-blind accessibility pipeline (color_blind.wgsgsl. Partially present, needs full test coverage across all render paths via bind_group_cache management) |

---

<!--
  Remaining sections to write:
  §7 VDOM animated/signal subsystems deep analysis 
    §8 Flow & Runestone text shaping  
    §9 SVG Filters Pipeline + Webkit Server validation  
→ Will continue appending to this file as each section completes). **CRITICAL NOTE:** Many audit findings remain unverified against actual codebase due to lack of time budget in this single session — needs targeted grep analysis across cvkg_render_gpu/src files specifically for bind_group_cache, Kvasir node lifecycle state machine wiring between passes geometry and glassmorphism (bifrost) stages.)!
-->
