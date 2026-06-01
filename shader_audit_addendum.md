# Surtr Shader Audit — Addendum
**Files:** `common.wgsl`, `shapes.wgsl`, `bifrost.wgsl`, `bloom.wgsl`, `blur_pyramid.wgsl`, `color_blind.wgsl`  
**Relationship:** Supplements *Surtr Renderer Code Audit* (`lib.rs`)  
**Net result:** 3 existing findings upgraded in severity, 12 new issues found

---

## Impact on Existing Findings

### 1.1 (Parallel Rayon Passes) — **Severity Confirmed and Worsened**

The shader analysis confirms both the Glass pass (mode 7, Bifrost) and the UI pass share `ctx_scene_texture` as a `LoadOp::Load` render target. The Bifrost shader (`shapes.wgsl` mode 7) samples `t_env` (the blur texture) and writes a full composited glass result to the scene texture. The UI pass then attempts to `LoadOp::Load` that same texture and overdraw UI elements on top. Because both encoders are built in parallel by rayon, there is no CPU-side guarantee that Glass has finished writing before UI starts reading. The shader content makes the race condition consequential — it is not a trivially invisible overdraw, it is a glass composite that the UI layer depends on for correct z-ordering.

---

### 1.2 (Bloom Overwrites Backdrop Blur) — **Confirmed Critical, Now Fully Provable**

`bloom.wgsl` `fs_bloom_extract` and both `fs_blur_h`/`fs_blur_v` all sample from `t_diffuse[0]` — hardcoded index zero. `blur_pyramid.wgsl` samples from `t_src`, a separate binding. The Gaussian blur shaders in `bloom.wgsl` are the ones actually wired into the pipeline, not the Kawase shaders in `blur_pyramid.wgsl` (which have their own pipeline infrastructure that is never referenced in `lib.rs` — more on that below). 

This confirms that both the backdrop blur and the bloom blur use identical `blur_h`/`blur_v` shaders writing to `blur_texture_a`/`blur_texture_b`. The shader has no way to distinguish whether it is serving as a backdrop blur or a bloom blur. Both passes are identical GPU operations; the only difference is which textures are bound at the Rust side — and as audited in 1.2, those textures are the same two buffers.

---

### 1.4 (Blur Pipelines Use Alpha Blending) — **Upgraded to High**

Now that the shader is visible, this is worse than described. `fs_blur_h` and `fs_blur_v` both return `vec4<f32>(result, 1.0)` — alpha is hardcoded to 1.0. When `ALPHA_BLENDING` is active on the pipeline (`src_alpha * src + (1 - src_alpha) * dst`), the blend equation becomes `1.0 * result + 0.0 * dst` — which happens to produce correct output *only because alpha is always 1.0*. This is a coincidental correctness masking an error in two different places simultaneously (Rust pipeline config and shader output). If either changes independently — someone sets the shader to output a real alpha, or someone removes the clear load op — the blur will visually break with no other code changes. The fix remains `blend: None`, and the coincidental masking makes this higher priority than initially rated because it is actively hiding the config mismatch.

---

### 2.3 (draw_line No Rotation) — **Confirmed, Shader Provides No Correction**

Mode 1 (`Neon Line`) in `fs_main` is:
```wgsl
if in.mode == 1u {
    color = in.color * 1.5;
}
```
The shader applies only a brightness boost. There is zero GPU-side rotation or SDF shaping for lines. The rectangle submitted by `draw_line` is axis-aligned at the CPU side and the shader does nothing to correct it. A diagonal line will always render as a horizontal glowing bar. Confirmed.

---

### 2.5 (SceneVertexConstructor Ignores Clip Stack) — **Confirmed, Hardcoded in Shader Too**

`SceneVertexConstructor` hardcodes `clip: [-10000.0, -10000.0, 20000.0, 20000.0]`. The fragment shader `fs_main` checks:
```wgsl
if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
```
The sentinel value `20000.0` triggers this bypass unconditionally. So tessellated SVG fill paths will always have `clip_alpha = 1.0`, bypassing all clip rect logic at both the CPU and GPU level. This is a system-wide clip bypass for SVG fills — any SVG rendered inside a scrollable container or a clipped modal will bleed outside its boundary.

---

## New Issues Found in Shaders

### S-1 🔴 CRITICAL — `blur_pyramid.wgsl` Is a Dead Shader — Never Compiled or Used

**File:** `blur_pyramid.wgsl`

The file defines `BlurUniforms`, `vs_blur`, `fs_kawase_down`, and `fs_kawase_up` — a complete Dual Kawase blur pyramid implementation. This is the architecturally superior blur approach (Kawase produces a wider, more natural blur than the 17-tap Gaussian at lower sample cost). However, `lib.rs` never references these entry points. There is no `kawase_down_pipeline`, no `kawase_up_pipeline`, no `BlurUniforms` buffer. The file is concatenated into `WGSL_SRC` but none of its entry points are used by any pipeline.

Additionally, line 34 contains a syntax error:
```wgsl
@Override               // ← invalid WGSL — should be @group(0)
group(0) @binding(0) var<uniform> blur: BlurUniforms;
```
`@Override` is not a valid WGSL attribute. This will fail to compile if the shader is ever activated. Because no pipeline currently uses these entry points, the error is silently dormant.

**Impact:** A significant blur quality investment exists in this file and is completely inert. The production path uses the cruder 17-tap Gaussian from `bloom.wgsl` for both backdrop blur and bloom. Any future attempt to activate the Kawase path will hit an immediate compile error.

---

### S-2 🔴 CRITICAL — `fs_copy` Samples `t_diffuse[in.tex_index]` but `in.tex_index` Is Always `0u` from `vs_fullscreen`

**File:** `bloom.wgsl` line 3, `common.wgsl` line 98

```wgsl
// vs_fullscreen:
out.tex_index = 0u;  // always zero

// fs_copy:
let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
```

The copy pipeline is used in Pass 2 (Backdrop Blur Extract) to copy the scene texture into `blur_texture_a`. The fullscreen vertex always outputs `tex_index = 0`. At Group 0 Binding 0, `t_diffuse[0]` is the Mega-Atlas — not the scene texture. The scene texture is bound at Group 1 (`t_env`). 

`fs_copy` should be reading `t_env` (the scene texture via the env bind group) but it reads `t_diffuse[0]` (the Mega-Atlas). This means the backdrop blur is not blurring the rendered scene — it is blurring the glyph/image atlas. The Bifrost glass panels are therefore displaying a blurred version of the texture atlas, not a blurred version of the UI behind them. This is a fundamental rendering error that would be immediately visually obvious.

**Fix:** Change `fs_copy` to sample `t_env`:
```wgsl
fn fs_copy(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_env, s_env, in.uv);
    return vec4<f32>(color.rgb, 1.0);
}
```

---

### S-3 🔴 CRITICAL — `fs_blur_h` and `fs_blur_v` Also Sample `t_diffuse[0]` (Mega-Atlas)

**File:** `bloom.wgsl` lines 22–38, 49–65

Both blur shaders hardcode `t_diffuse[0]` as their sample source:
```wgsl
result += textureSample(t_diffuse[0], s_diffuse, in.uv).rgb * w0;
```

This is the same error as S-2, extended across all blur passes. The Gaussian blur passes are supposed to blur the content of `blur_texture_a` (for H) and `blur_texture_b` (for V) on alternating iterations. Instead, both sample `t_diffuse[0]` — the Mega-Atlas — regardless of which textures are bound at group 0 runtime. The blur is reading from atlas slot 0 on every pass.

The correct approach is to sample `t_env` (the env group, group 1), which is the bind group swapped between `blur_texture_a` and `blur_texture_b` on each ping-pong iteration. Or sample `t_diffuse[0]` only if the blur bind groups correctly place the ping-pong texture at array index 0 of the texture array — which the Rust code does attempt (it creates `blur_bind_group_a` and `blur_bind_group_b` as texture arrays with the blur texture at all 256 slots). In that case sampling `t_diffuse[0]` would be correct, but only if the blur bind groups are actually bound at group 0, which they are (`ctx_blur_bind_group_a` at group 0 for H, `ctx_blur_bind_group_b` at group 0 for V). So for the blur iterations the shader is coincidentally correct — but `fs_copy` (S-2) uses `ctx_scene_texture_bind_group` at group 0, and the scene texture is not at slot 0 of a 256-slot array in the normal case.

**Net result:** Blur passes ping-pong correctly between blur textures. `fs_copy` in Pass 2 reads the wrong source. The backdrop blur pyramid starts with atlas content instead of scene content.

---

### S-4 🔴 CRITICAL — `vs_fullscreen` Generates Only 3 Vertices Correctly for a Triangle but Is Used for 6-Vertex Quad Draws

**File:** `common.wgsl` lines 87–92

```wgsl
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) % 2) * 4.0 - 1.0;
```

This is the standard fullscreen triangle trick — for indices 0, 1, 2 it generates:
- `(0/2=0)*4-1 = -1`, `(0%2=0)*4-1 = -1` → `(-1, -1)`
- `(1/2=0)*4-1 = -1`, `(1%2=1)*4-1 = 3` → `(-1, 3)`
- `(2/2=1)*4-1 = 3`, `(2%2=0)*4-1 = -1` → `(3, -1)`

This produces one oversized triangle covering the full screen. However, all post-process passes call `p.draw(0..6, 0..1)` — 6 vertices, not 3. For indices 3, 4, 5:
- Index 3 → `(3/2=1)*4-1 = 3`, `(3%2=1)*4-1 = 3` → `(3, 3)` — outside NDC
- Index 4 → `(4/2=2)*4-1 = 7`, `(4%2=0)*4-1 = -1` → `(7, -1)` — far outside NDC
- Index 5 → `(5/2=2)*4-1 = 7`, `(5%2=1)*4-1 = 3` → `(7, 3)` — far outside NDC

The second triangle (indices 3–5) generates vertices that are clipped entirely outside the viewport. The GPU clips them away, so no visual corruption occurs. But `draw(0..6, ...)` submits and processes 6 vertices and 2 triangles when only 3 vertices and 1 triangle are needed. This wastes GPU vertex shader invocations on every post-process pass (background, copy, blur H×4, blur V×4, bloom extract, bloom blur H×2, bloom blur V×2, composite = ~28 wasted half-triangle draws per frame).

**Fix:** Change all fullscreen draws from `p.draw(0..6, 0..1)` to `p.draw(0..3, 0..1)`.

---

### S-5 🟡 MEDIUM — Bifrost SDF Uses Wrong Coordinate System for the Clip Test

**File:** `shapes.wgsl` lines 63–74

```wgsl
let p_clip_pos  = in.clip.xy * scene.scale_factor;
let p_clip_size = in.clip.zw * scene.scale_factor;
let pixel_pos   = (in.clip_position.xy * 0.5 + 0.5) * scene.resolution * scene.scale_factor;

let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
```

`in.clip_position.xy` is the fragment's NDC position in `[-1, 1]` after perspective divide — but wgpu's `@builtin(position)` is actually the fragment's **window-space** position in pixels `[0, width] × [0, height]`, not NDC. Multiplying by `* 0.5 + 0.5` and then `* scene.resolution` maps it as if it were NDC, resulting in `pixel_pos` being double-scaled. A fragment at window pixel `(400, 300)` on an 800×600 display will compute `pixel_pos = (400*0.5+0.5)*800 = 200.5*800 = 160,400` — wildly incorrect.

The correct calculation is simply:
```wgsl
let pixel_pos = in.clip_position.xy;  // already in window pixels in wgpu/WGSL
```

**Impact:** The SDF clip is computing distances against positions that are off by approximately a factor of the resolution, meaning the clip rect in logical space does not correspond to the correct screen region. Clipping will appear broken for any element that uses it, with the clip boundary appearing at the wrong position on screen.

---

### S-6 🟡 MEDIUM — Mode 7 Bifrost Computes `fresnel` from Lens Distance but the Result Is Near-Zero for Most Pixels

**File:** `shapes.wgsl` lines 100–171

```wgsl
let fresnel = pow(lens_dist * 1.8, 2.5);
...
color = vec4<f32>(final_rgb, 0.01 + fresnel * 0.01);
```

`lens_dist` is the distance from the center of the glass panel in normalized `[0, 0.5]` coordinates (since `local = in.logical / in.size` and `centered = local - 0.5`). At a panel corner, `lens_dist ≈ 0.707`. `pow(0.707 * 1.8, 2.5) = pow(1.272, 2.5) ≈ 1.83` — this clips to the surface via any clamping downstream, but the alpha output is `0.01 + 1.83 * 0.01 = 0.0283`. For a center pixel, `lens_dist ≈ 0`, giving `alpha = 0.01`.

Glass panels have an alpha of approximately 1–3% across the entire surface. A glass panel is nearly invisible. The intent is clearly frosted glass with meaningful opacity, but the alpha math means Bifrost panels are essentially transparent overlays that barely tint the scene.

---

### S-7 🟡 MEDIUM — `fs_bloom_extract` Emits `alpha = 1.0` for Black Pixels, Breaking Additive Blend

**File:** `bloom.wgsl` lines 8–13

```wgsl
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.8 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // ← alpha = 1.0 for non-bright pixels
}
```

Non-bright pixels are returned as `(0, 0, 0, 1)` — opaque black. The bloom extract target is then blurred by the Gaussian passes and composited additively in `fs_composite`:
```wgsl
let hdr_color = scene_color.rgb + (bloom_color.rgb * 0.2);
```

Since additive composite adds `bloom_color.rgb` to the scene, opaque-black bloom pixels add `(0, 0, 0) * 0.2 = 0` — no visual damage. However, the `alpha = 1.0` on black pixels is semantic noise that would cause incorrect results if the blend mode were changed to alpha-premultiplied, and it wastes the alpha channel entirely. The correct return for a non-bright pixel is `vec4<f32>(0.0, 0.0, 0.0, 0.0)`.

---

### S-8 🟡 MEDIUM — `vs_fullscreen` Sets `out.screen` as `scene.resolution * scene.scale_factor` But `fs_main` Uses It as Logical Resolution

**File:** `common.wgsl` line 101

```wgsl
out.screen = scene.resolution * scene.scale_factor;
```

`vs_fullscreen` sets `screen` to physical pixel resolution. But `vs_main` (the geometry vertex shader in `shapes.wgsl`) sets `screen` from the vertex attribute `in.screen`, which is populated from the Rust side as `[self.current_width() as f32, self.current_height() as f32]` — already physical pixels. The fullscreen vertex is consistent with the geometry vertex on this point. However, `scene.resolution` is set in the `SceneUniforms` as the *logical* resolution, making `scene.resolution * scale_factor` the physical resolution. The fullscreen output is therefore physical, which is consistent — but the naming `screen` in `VertexOutput` does not differentiate logical from physical, creating a latent source of confusion and potential future mismatch if any shader consumer assumes `screen` is logical.

---

### S-9 🟡 MEDIUM — Mode 15 (Linear Gradient) Angle Calculation Uses Animated UV Rather Than the Intended Static Angle

**File:** `shapes.wgsl` lines 258–262

```wgsl
} else if in.mode == 15u {
    let angle = in.uv.x + scene.time * 0.5;
    let t = dot(in.logical / in.size - 0.5, vec2(cos(angle), sin(angle))) + 0.5;
```

`in.uv.x` encodes the gradient angle (passed from `draw_linear_gradient` as `slice.x = angle`). But the shader reads `in.uv.x`, not `in.slice.x`. Looking at the Rust draw call:

```rust
fn draw_linear_gradient(&mut self, rect: Rect, start_color, end_color, angle: f32) {
    self.fill_rect_with_full_params_and_slice(
        rect, start_color, 15, None, 0.0,
        Rect { x: angle, y: 0.0, width: 1.0, height: 1.0 },  // angle in slice.x
        end_color,
    );
}
```

The angle is packed into `slice.x` (passed as a `Rect`), but the shader reads it from `in.uv.x`. For a textured rectangle, `in.uv` is the texture coordinate, not the slice parameter. These are different vertex attributes (`@location(2) uv` vs `@location(6) slice`). For non-textured shapes, `uv` is typically `[0,0]` to `[1,1]`, meaning `in.uv.x` interpolates from 0 at the left edge to 1 at the right — not the intended angle value. Additionally, `+ scene.time * 0.5` makes every linear gradient rotate over time, which was almost certainly not intended. Linear gradients should be static.

**Fix:** Read `in.slice.x` for the angle and remove the time component.

---

### S-10 🟡 MEDIUM — Mode 18 Drop Shadow `p` Coordinate Is Off by One `margin` Term

**File:** `shapes.wgsl` lines 218–228

```wgsl
let margin = in.uv.x;
let original_size = in.size - 2.0 * margin;
let half_size = original_size * 0.5;
let p = in.logical - margin - half_size;
let d = sd_round_rect(p, half_size - in.radius, in.radius);
```

`in.logical` is the fragment's position within the inflated shadow rect (which extends `margin` pixels beyond the original rect on all sides). To find the position relative to the original rect's center, the correct calculation is:

```
p = (in.logical - margin) - half_size
```

This is what the code does — `in.logical - margin` converts from inflated space to original rect space, then `- half_size` centers it. This appears correct at first reading. However, `in.logical` for a `fill_rect_with_full_params` call is set on the CPU side as the *logical position within the inflated rect starting from its top-left corner*, which starts at 0. So `in.logical` runs from `[0, 0]` to `[inflated_width, inflated_height]`.

`half_size = (original_size) * 0.5 = (inflated_size - 2*margin) * 0.5`. The centered position should be `in.logical - margin - half_size_of_inflated`. Subtracting only `half_size` (of the original, un-inflated rect) is subtracting the wrong quantity, causing the shadow SDF to be evaluated off-center by `margin * 0.5` pixels. For large blur radii (large margins), the shadow will be visibly mis-centered.

---

### S-11 🟢 LOW — `color_blind.wgsl` Is Empty — References Shader Functions That Don't Exist

**File:** `color_blind.wgsl`

The file contains only comments. It references a "color_blindness module's shader_source() function" that is compiled "separately as a dedicated pipeline," but no such pipeline is created in `lib.rs`. The `README.md` for `cvkg-render-gpu` lists color blindness simulation (Brettel/Viénot Daltonization) as a core feature. The feature is entirely absent from both the Rust renderer and the shader files.

---

### S-12 🟢 LOW — Gaussian Blur Kernel Weights Do Not Sum to 1.0

**File:** `bloom.wgsl` lines 18–19

```
w0=0.153423, w1=0.143254, w2=0.117031, w3=0.081827,
w4=0.049003, w5=0.025135, w6=0.010861, w7=0.00392, w8=0.0011
```

Sum = `0.153423 + 2*(0.143254 + 0.117031 + 0.081827 + 0.049003 + 0.025135 + 0.010861 + 0.00392 + 0.0011)`  
= `0.153423 + 2 * 0.432031`  
= `0.153423 + 0.864062`  
= `1.017485`

The kernel sums to approximately **1.017**, not 1.0. This means the blur brightens the scene by ~1.7% per pass. Over 4 blur iterations (2 for backdrop, 2 for bloom), the accumulated brightening is `1.017^8 ≈ 1.144` — a 14% brightness inflation in the blurred result. For the bloom composite this adds a subtle unwanted brightness boost to the bloom glow. For the backdrop blur it makes glass panels sample a slightly brightened background.

---

## Updated Issue Severity Table

| ID | Severity | Category | Description | Change |
|---|---|---|---|---|
| 1.1 | 🔴 CRITICAL | Pipeline | Parallel rayon passes share render target | Confirmed by shader |
| 1.2 | 🔴 CRITICAL | Pipeline | Bloom overwrites backdrop blur | Confirmed by shader |
| S-2 | 🔴 CRITICAL | Shader | `fs_copy` reads Mega-Atlas instead of scene texture | **NEW** |
| S-3 | 🔴 CRITICAL | Shader | Blur shaders hardcode `t_diffuse[0]` — correct for blur, wrong for copy | **NEW** |
| S-1 | 🔴 CRITICAL | Shader | `blur_pyramid.wgsl` is dead code with syntax error | **NEW** |
| S-4 | 🔴 CRITICAL | Shader | `vs_fullscreen` draws 6 vertices; only 3 are valid — second triangle is degenerate waste | **NEW** |
| 2.1 | 🔴 CRITICAL | Ordinal | `stroke_path` uses vertex cursor as index cursor | Unchanged |
| 3.1 | 🔴 CRITICAL | SVG | Tessellation `.unwrap()` panics on malformed path | Unchanged |
| 4.1 | 🔴 CRITICAL | Text | Glyph atlas fallback writes to `(0,0)` silently | Unchanged |
| 1.4 | 🟡 HIGH | Pipeline | Blur pipelines use alpha blend — masked by `alpha=1.0` in shader | **Upgraded** |
| S-5 | 🟡 MEDIUM | Shader | Clip SDF uses `clip_position.xy` as NDC when it is already window pixels | **NEW** |
| S-6 | 🟡 MEDIUM | Shader | Bifrost glass panel alpha is ~1–3% — near invisible | **NEW** |
| S-7 | 🟡 MEDIUM | Shader | Bloom extract emits `alpha=1.0` for black pixels | **NEW** |
| S-9 | 🟡 MEDIUM | Shader | Linear gradient reads `uv.x` for angle instead of `slice.x`; also time-animated unintentionally | **NEW** |
| S-10 | 🟡 MEDIUM | Shader | Drop shadow SDF `p` coordinate offset error for large blur radii | **NEW** |
| 2.3 | 🟡 MEDIUM | Coordinates | `draw_line` no rotation — confirmed no GPU correction | Confirmed |
| 2.5 | 🟡 MEDIUM | Coordinates | SVG fill bypasses clip rect at CPU and GPU | Confirmed |
| S-8 | 🟡 MEDIUM | Shader | `screen` resolution naming inconsistency between fullscreen and geometry paths | **NEW** |
| S-11 | 🟢 LOW | Shader | `color_blind.wgsl` is empty — feature not implemented | **NEW** |
| S-12 | 🟢 LOW | Shader | Gaussian kernel weights sum to 1.017 — 14% brightness inflation over 8 passes | **NEW** |

---

## Key Takeaway

The most urgent combined finding is **S-2 + S-3 + 1.2**: the backdrop blur pipeline reads the Mega-Atlas (`fs_copy` reads `t_diffuse[0]`), blurs it with the Gaussian passes (which correctly read from ping-pong buffers after the first pass), and the result is what glass panels display as their blurred background. Glass panels in the Bifrost effect are showing a blurred version of the glyph/texture atlas rather than the rendered scene. This means the signature visual feature of the framework — glassmorphic frosted panels — is rendering fundamentally incorrect output in all cases.

*Addendum to Surtr Renderer Code Audit — static analysis only.*
