# Surtr Renderer (`lib.rs`) — Code Audit Report
**File:** `cvkg-render-gpu/src/lib.rs`  
**Lines:** 5,491  
**Auditor:** Static analysis pass  
**Scope:** Coordinate relations, render pipeline order-of-operations, SVG/text/shape drawing correctness, ordinal/index safety, buffer overflow, general coding issues

---

## Executive Summary

The Surtr renderer is a capable and architecturally ambitious wgpu-based GPU renderer. The multi-pass pipeline structure is sound in principle, but several correctness issues, ordering hazards, and latent overflow conditions exist that could cause silent rendering artifacts, GPU validation errors, or hard panics under production load. The most critical issues are the **parallel rayon pass writing to a shared render target without a memory barrier**, **the `stroke_path` DrawCall index calculation using the vertex cursor instead of the index cursor**, and **the `tessellate_node` `.unwrap()` call on tessellation failure**.

---

## Section 1: Render Pipeline Order-of-Operations

### 1.1 🔴 CRITICAL — Parallel Rayon Passes Write to the Same Render Target Without Synchronization

**Lines:** 2947–3139

```rust
let (glass_cb, ui_cb) = rayon::join(
    || { /* writes to ctx_scene_texture */ },
    || { /* also writes to ctx_scene_texture */ },
);
```

**Issue:** Both the Glass pass (Pass 3) and the UI pass (Pass 4) are encoded in parallel via `rayon::join` and both write to `ctx_scene_texture` as their color attachment with `LoadOp::Load`. They are then submitted in sequence as separate command buffers:

```rust
self.staging_command_buffers.push(glass_cb);
self.staging_command_buffers.push(ui_cb);
```

While GPU command buffer *submission* is sequential, the CPU-side encoding happens on two rayon threads simultaneously. The danger here is that both encoders reference `ctx_scene_texture` — a raw pointer captured from the surrounding scope. wgpu command encoders are `!Send` for exactly this reason. This compiles only if wgpu's command encoder is `Send` in the version being used, which varies. If both encoders are recording render passes against the same texture view simultaneously, this is undefined behavior at the GPU API level regardless of submission order. The second command buffer (`ui_cb`) should use `LoadOp::Load` on output that the first (`glass_cb`) hasn't finished writing; without an explicit pipeline barrier or separate intermediate texture, the UI pass may read stale or partially-written glass output.

**Fix:** Do not share a render target between parallel-encoded passes. Either encode sequentially, or give each pass a separate intermediate texture and composite them in a subsequent pass.

---

### 1.2 🔴 CRITICAL — Bloom Extract Pass Overwrites the Backdrop Blur Result

**Lines:** 3174–3199

```rust
// Pass 5: Bloom Extract
p.set_pipeline(&self.bloom_extract_pipeline);
p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
// Renders INTO ctx_blur_texture_a
```

**Issue:** Pass 2 (Backdrop Blur) spends four iterations of horizontal/vertical blur building a high-quality blurred copy of the scene in `blur_texture_a`. Pass 5 (Bloom Extract) then unconditionally clears and overwrites `ctx_blur_texture_a` with the bloom extraction result. This destroys the backdrop blur data that the Glass pass (Pass 3) needs to sample from.

The Glass pass runs *before* the post-process encoder is built (it's encoded in the parallel rayon step), so it reads `blur_env_bind_group_a` which correctly points to the pre-bloom `blur_texture_a`. However, if anything in Pass 5 or 6 needs the backdrop blur texture (e.g. a second glass composite), it is gone. More importantly, the bloom blur passes (Pass 6) then ping-pong between `blur_texture_a` and `blur_texture_b`, which were already used as the backdrop blur ping-pong buffers. The two subsystems share the same two textures with no separation, meaning any change to one pass's iteration count or texture assignments will silently break the other.

**Fix:** Allocate dedicated textures for bloom (`bloom_tex_a`, `bloom_tex_b`) separate from the backdrop blur textures (`backdrop_tex_a`, `backdrop_tex_b`). Their resolutions can differ (bloom is typically lower resolution).

---

### 1.3 🟡 MEDIUM — Background Pipeline Uses `depth_compare: Always`, Allowing It to Overwrite Scene Geometry

**Lines:** 935–945

```rust
depth_stencil: Some(wgpu::DepthStencilState {
    depth_compare: Some(wgpu::CompareFunction::Always),
    depth_write_enabled: Some(true),
    ...
})
```

**Issue:** The background pipeline writes depth with `CompareFunction::Always`, meaning it will overwrite any depth value already in the depth buffer unconditionally. If any geometry is rendered before the background pass and writes depth, the background will clobber it. In the current pipeline order (background is Pass 1a), this is safe because the depth buffer is freshly cleared. However, if the pass ordering is ever changed or a pre-pass is added, the background will silently overwrite depth written by earlier passes, causing all subsequent `LessEqual` depth tests to fail against the background's depth rather than the actual scene geometry.

**Fix:** Use `CompareFunction::Always` with `depth_write_enabled: false` for a true background pass, or use a dedicated background depth value (e.g. write `1.0` explicitly in the vertex shader) with `LessEqual` compare.

---

### 1.4 🟡 MEDIUM — Blur Pipelines Use `ALPHA_BLENDING` Instead of No Blend

**Lines:** 1018, 1044

```rust
blend: Some(wgpu::BlendState::ALPHA_BLENDING),
```

**Issue:** Both `blur_h_pipeline` and `blur_v_pipeline` use `ALPHA_BLENDING` (`src_alpha * src + (1 - src_alpha) * dst`). A Gaussian blur pass is a full-screen filter operation — it should replace the destination pixel entirely. Using alpha blending here means the blur result is blended with whatever was previously in the target texture. Since the target is cleared before each pass, this is coincidentally harmless in the current code, but it is semantically wrong and fragile. Any change that removes the `LoadOp::Clear` will cause visual artifacts.

**Fix:** Use `blend: None` for both blur pipelines. They are replacement operations, not compositing operations.

---

### 1.5 🟡 MEDIUM — Post-Processing Uses a Shared Bloom Texture as Composite Input While Bloom Is Still Being Written

**Lines:** 3279–3283

```rust
// Pass 7: Composite
p.set_bind_group(0, ctx_scene_texture_bind_group, &[]); // scene
p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]);   // blur (also bloom result)
```

**Issue:** Pass 7 composites the scene (`bind_group(0)`) with `ctx_blur_env_bind_group_a` as the bloom/blur input (`bind_group(1)`). After Pass 6, `blur_texture_a` holds the final blurred bloom result, which is correct. However `ctx_blur_env_bind_group_a` is the `env_bind_group` for `blur_texture_a`, not the `texture_bind_group`. The composite pipeline (`composite_layout`) was created with the same layout as `post_process_layout` — all three bind group layouts are identical (`texture`, `env`, `berserker`). This means group 1 is an `env`-type binding (single texture + sampler) while the shader may expect a texture array at group 0 and a single texture at group 1. This alignment is coincidentally correct only because both layouts happen to match. Any future layout differentiation will silently break the composite pass without a compile error.

**Fix:** Create distinct, descriptively named bind group layouts for each pipeline stage to make mismatches a compile-time or validation-time error rather than a silent rendering artifact.

---

## Section 2: Coordinate Relations and Ordinal Point Issues

### 2.1 🔴 CRITICAL — `stroke_path` DrawCall Uses Vertex Index as Index Buffer Cursor

**Lines:** 4639–4650

```rust
self.draw_calls.push(DrawCall {
    texture_id: tid,
    scissor_rect: ...,
    index_start: base_vertex_idx,  // ← BUG: this is the VERTEX cursor, not the INDEX cursor
    index_count: buffers.indices.len() as u32,
    material,
});
```

**Issue:** `base_vertex_idx` is captured as `self.vertices.len() as u32` before tessellation. This is the starting *vertex* offset, but `index_start` in `DrawCall` must be a *index buffer* offset — the position within `self.indices` where this draw call's indices begin. The vertex count and index count are different quantities and will diverge as geometry accumulates. This will cause the GPU to read index data from the wrong position in the index buffer, resulting in incorrect geometry, rendering garbage, or an out-of-bounds GPU access.

**Fix:**
```rust
let base_index = self.indices.len() as u32; // capture BEFORE extending indices
// ... tessellate and extend ...
self.draw_calls.push(DrawCall {
    index_start: base_index,  // correct
    ...
});
```

---

### 2.2 🟡 MEDIUM — SVG Animation Rotation Is Applied in SVG Space, Not Logical Render Space

**Lines:** 5179–5217

```rust
if anim.attribute_name == "transform" {
    // Computes AABB center of the vertex range in SVG coordinates
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    // Rotates vertices in-place in SVG coordinate space
    local_vertices[i].position[0] = cx + dx * c - dy * s;
    local_vertices[i].position[1] = cy + dx * s + dy * c;
}
```

**Issue:** SVG animations are evaluated by rotating vertices in tessellated SVG coordinate space before the `draw_svg` function maps them into logical screen coordinates. This works for simple icons but breaks for any SVG where the coordinate system origin is not at (0,0) or where the animation's `from`/`to` rotation center is specified via a `rotate(angle cx cy)` attribute. The parser (`parse_svg_animations`) reads only the scalar `from` and `to` values, discarding any rotation center specified in the attribute. For `<animateTransform type="rotate" from="0 50 50" to="360 50 50" />`, the `50 50` center is silently dropped and the rotation pivots around the vertex bounding box center instead, producing incorrect animation.

**Fix:** Parse the full `from`/`to` rotate syntax to extract `(angle, cx, cy)` and use the specified center, not the computed AABB center.

---

### 2.3 🟡 MEDIUM — `draw_line` Does Not Apply Rotation to the Rectangle

**Lines:** 3780–3810

```rust
fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, ...) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    self.fill_rect_with_mode(
        Rect {
            x: (x1 + x2) / 2.0 - len / 2.0,
            y: (y1 + y2) / 2.0 - stroke_width / 2.0,
            width: len,
            height: stroke_width,
        },
        ...
    );
}
```

**Issue:** This draws a line as an axis-aligned rectangle centered at the midpoint of the two points. The rectangle has the correct length, but it is never rotated to align with the actual direction of the line. A line from `(0,0)` to `(100,100)` will render as a horizontal bar at 45° off-center rather than a diagonal line. The angle `atan2(dy, dx)` is computed implicitly via `len` but never passed to `fill_rect_with_full_params` as a rotation parameter.

**Fix:** Use the existing `stroke_path` pipeline to build a proper line path with Lyon, or pass the computed angle to `fill_rect_with_full_params` so the GPU shader can apply the rotation via the transform stack.

---

### 2.4 🟡 MEDIUM — `push_transform_3d` Extracts 3D Scale/Rotation Incorrectly into 2D Stack

**Lines:** 4412–4418

```rust
fn push_transform_3d(&mut self, transform: &cvkg_core::Transform3D) {
    let m = transform.to_matrix();
    let translation = [m.w_axis.x, m.w_axis.y];
    let scale = [m.x_axis.x, m.y_axis.y];         // ← diagonal-only extraction, ignores shear
    let rotation = m.x_axis.y.atan2(m.x_axis.x);  // ← atan2 of mat[0][1] / mat[0][0]
    self.push_transform(translation, scale, rotation);
}
```

**Issue:** Extracting scale from a 4x4 matrix using only the diagonal (`m.x_axis.x`, `m.y_axis.y`) is incorrect when the matrix includes rotation. The diagonal elements of a rotated matrix encode `cos(θ) * scale`, not `scale`. This means 3D transforms with any non-zero rotation will be decomposed into incorrect scale values, causing 3D meshes to render at the wrong size when pushed through the 2D transform stack.

Additionally, `pop_transform_3d` calls `self.pop_transform()` followed by a second `self.transform_stack.pop()` if the stack is non-empty, removing two entries from the stack when only one was pushed. This will corrupt the transform stack after the first 3D pop.

**Lines:** 4421–4426
```rust
fn pop_transform_3d(&mut self) {
    self.pop_transform();                          // pops 1
    if !self.transform_stack.is_empty() {
        self.transform_stack.pop();                // pops a SECOND — BUG
    }
}
```

**Fix:** Use `glam::Mat4::to_scale_rotation_translation()` for correct decomposition. Remove the second `pop()` from `pop_transform_3d`.

---

### 2.5 🟡 MEDIUM — `SceneVertexConstructor` for Fill Paths Uses Hardcoded Infinite Clip Rect

**Lines:** 4808–4809

```rust
clip: [-10000.0, -10000.0, 20000.0, 20000.0],
```

**Issue:** Fill tessellated shapes (used for SVG paths and the fill tessellator path) always use a hardcoded "infinite" clip rect, bypassing any active `push_clip_rect` state. This means SVG shapes rendered inside a clipped container will bleed outside the clip boundary. The `CustomStrokeVertexConstructor` correctly reads `self.clip_stack`, but `SceneVertexConstructor` does not have access to the renderer state and cannot do so in the current architecture.

**Fix:** Pass the active clip rect as a field on `SceneVertexConstructor` at construction time, the same way `CustomStrokeVertexConstructor` receives `clip: [f32; 4]`.

---

### 2.6 🟢 LOW — `bifrost` Screen-Space UV Coordinates Can Exceed `[0, 1]` If Rect Extends Beyond Viewport

**Lines:** 3499–3507

```rust
let screen_uv = Rect {
    x: rect.x / self.current_width() as f32,
    y: rect.y / self.current_height() as f32,
    width: rect.width / self.current_width() as f32,
    height: rect.height / self.current_height() as f32,
};
```

**Issue:** If a glass panel is partially off-screen (e.g. a tooltip near the edge), `screen_uv` values will exceed `[0.0, 1.0]`. The blur sampler uses `ClampToEdge`, so the visual artifact would be edge-color smearing rather than a hard error. However the shader receives UVs outside the valid range, which may produce unexpected results depending on the shader implementation.

**Fix:** Clamp the computed UV rect to `[0.0, 1.0]` before passing to the shader.

---

## Section 3: SVG Drawing Issues

### 3.1 🔴 CRITICAL — `tessellate_node` Uses `.unwrap()` on Tessellation Failure

**Lines:** 5118–5132

```rust
tessellator
    .tessellate_path(
        &lyon_path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(...),
    )
    .unwrap();  // ← panics on any malformed SVG path
```

**Issue:** `tessellate_path` returns a `Result`. Calling `.unwrap()` on a malformed or geometrically degenerate SVG path (self-intersecting paths, NaN coordinates, zero-area paths) will **panic the entire renderer process**. Any user-supplied or network-fetched SVG that contains a degenerate path will crash the application. The surrounding `load_svg` function itself also uses `.expect()`:

```rust
let tree = usvg::Tree::from_data(data, &opt).expect("Failed to parse SVG");
```

**Fix:** Convert both to `Result`-returning functions. Log and skip degenerate paths rather than unwrapping.

---

### 3.2 🟡 MEDIUM — SVG Stroke Paths Are Not Tessellated at Load Time

**Lines:** 5101–5138

```rust
} else if let usvg::Node::Path(ref path) = *node
    && let Some(fill) = path.fill()
{
    // Only fill is tessellated. Stroke is silently ignored.
}
```

**Issue:** `tessellate_node` only tessellates filled paths. SVG stroke paths (those with `stroke` paint but no `fill`, or with `fill="none"`) are silently skipped. This means any SVG icon that uses strokes for its visual content will render as completely invisible geometry. No warning is logged.

**Fix:** Add a stroke tessellation branch using `StrokeTessellator` for paths that have `path.stroke().is_some()`. Log a debug message when a path with no fill and no stroke is encountered.

---

### 3.3 🟡 MEDIUM — SVG Paint Gradients and Patterns Fall Back to White Silently

**Lines:** 5104–5112

```rust
let color = match fill.paint() {
    usvg::Paint::Color(c) => [...],
    _ => [1.0, 1.0, 1.0, 1.0],  // gradient, pattern, context — all become white
};
```

**Issue:** Any SVG fill that uses a gradient (`linearGradient`, `radialGradient`) or pattern will render as solid white. There is no log message. Complex SVG icons that rely on gradients (very common in modern icon sets) will render incorrectly with no diagnostic output.

**Fix:** At minimum, log a `warn!` when a non-`Color` paint is encountered. Implement gradient sampling as a future enhancement.

---

### 3.4 🟡 MEDIUM — `draw_svg` Computes `_scale_x`/`_scale_y` but Never Uses Them

**Lines:** 5160–5161

```rust
let _scale_x = rect.width / model.view_box.width;
let _scale_y = rect.height / model.view_box.height;
```

**Issue:** These scale factors are computed and immediately prefixed with `_`, indicating they are unused. The actual mapping from SVG space to logical screen space is done further down via `rel_x * rect.width` / `rel_y * rect.height`, which is equivalent but the unused variables represent dead code and a missed opportunity to use named intermediates. More importantly, if the SVG `viewBox` has a non-zero `x` or `y` origin, the `rel_x` calculation:

```rust
let rel_x = (v.position[0] - model.view_box.x) / model.view_box.width;
```

is correct only if `view_box.x` is correctly parsed. The `view_box` is constructed as:

```rust
let view_box = Rect { x: 0.0, y: 0.0, width: tree.size().width(), height: tree.size().height() };
```

This hardcodes `x: 0.0, y: 0.0` regardless of the SVG `viewBox` attribute, so SVGs with a non-zero viewBox origin will render incorrectly.

**Fix:** Read the actual `viewBox` from `tree.view_box()` rather than hardcoding origin to `(0, 0)`.

---

## Section 4: Text Glyph Rendering Issues

### 4.1 🔴 CRITICAL — Glyph Atlas Upload After `reclaim_vram()` Uses Stale Pack Position `(0, 0)` as Fallback

**Lines:** 3845–3852, 3977–3983

```rust
let (nx, ny) = if let Some(pos) = pack_res {
    pos
} else {
    self.reclaim_vram();
    self.atlas_packer.pack(gw, gh).unwrap_or((0, 0))  // ← fallback writes to atlas origin
};
```

**Issue:** If `reclaim_vram()` is called and the atlas is still full after compaction (i.e. a single glyph is larger than the available space, or the compaction didn't free enough), the fallback `unwrap_or((0, 0))` silently places the glyph at atlas origin `(0, 0)`. This overwrites whatever is stored at the atlas origin (likely a different glyph or image) with the current glyph's pixel data. The original occupant at `(0, 0)` will now render the wrong glyph. No error is logged. This bug exists in both `draw_text` (line 3851) and `draw_shaped_text` (line 3982).

**Fix:** Return early and skip rendering if the glyph cannot be packed even after reclaim. Log an error indicating the atlas is critically full.

---

### 4.2 🟡 MEDIUM — Glyph Rasterization Assumes Alpha-Only Output but Assumes No Subpixel Color

**Lines:** 3854–3860

```rust
let mut rgba_data = Vec::with_capacity((gw * gh * 4) as usize);
for alpha in &image.data {
    rgba_data.push(255);   // R
    rgba_data.push(255);   // G
    rgba_data.push(255);   // B
    rgba_data.push(*alpha);
}
```

**Issue:** The rasterizer output is treated as a single-channel alpha mask expanded to RGBA by setting RGB to `255`. This is correct for grayscale antialiasing but incorrect for subpixel (LCD) rendering, which produces RGB coverage values per channel. If `swash`'s rasterizer returns subpixel data, this code discards the per-channel information and treats the R channel as the alpha for all three channels, producing incorrect glyph rendering on LCD-subpixel displays.

**Fix:** Check the rasterizer's output format before expanding. For subpixel output, use the three channels directly; for grayscale output, use the single channel as alpha.

---

### 4.3 🟡 MEDIUM — `draw_text` Calls `shape_text_with_stack` at Physical Scale but `measure_text` Does Not

**Lines:** 3829–3832 vs 3914–3917

```rust
// draw_text: scales by scale factor before shaping
let scaled_size = size * self.current_scale_factor();
let shaped = self.shape_text_with_stack(text, scaled_size);

// measure_text: does NOT scale — uses raw logical size
fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
    let shaped = self.shape_text_with_stack(text, size);
    (shaped.width, shaped.height)
}
```

**Issue:** `draw_text` shapes at physical pixel size (logical × scale_factor), then divides the resulting glyph positions back by scale_factor for placement. `measure_text` shapes at logical size without scaling. The returned dimensions from `measure_text` are therefore in physical pixels on HiDPI displays, while callers expect logical units. This will cause text layout measurements to be `scale_factor` times too large on Retina/HiDPI displays, producing incorrect wrapping, alignment, and overflow calculations.

**Fix:** `measure_text` should also scale the input size and divide the output dimensions by `current_scale_factor()`, or alternatively shape at logical size in both functions (and only scale the rasterization step).

---

### 4.4 🟢 LOW — Duplicate Doc Comment on `load_image`

**Lines:** 4060–4062

```rust
/// load_image — Proactively pushes a raw asset into the Mega-Atlas.
/// load_image — Proactively pushes a raw asset into the Texture Array.
fn load_image(...)
```

**Issue:** The function has two conflicting doc comment lines. One says "Mega-Atlas", the other says "Texture Array". `load_image` actually loads into the texture *array* (it calls `self.texture_views[index as usize] = view`), not the atlas. The first line is incorrect. The `load_image_to_atlas` function is the one that writes to the Mega-Atlas.

**Fix:** Remove the duplicate and incorrect first line. Keep only "Proactively pushes a raw asset into the Texture Array."

---

## Section 5: Buffer Overflow and Index Safety

### 5.1 🟡 MEDIUM — `load_image` Texture Array Index Can Exceed `texture_views` Bounds

**Lines:** 4111–4123

```rust
let index = if self.texture_registry.len() < 255 {
    (self.texture_registry.len() + 1) as u32
} else {
    // LRU eviction
    if let Some((old_name, old_index)) = self.texture_registry.pop_lru() {
        ...
        old_index
    } else {
        1 // Fallback
    }
};
self.texture_views[index as usize] = view;  // ← unchecked index write
```

**Issue:** `texture_views` is a `Vec` initialized with 256 entries (indices 0–255). Index 0 is reserved for the dummy/atlas texture, so valid image indices are 1–255. The condition `self.texture_registry.len() < 255` gates the growth path, but after the LRU eviction path, `old_index` is a value that was previously stored in the registry — it could theoretically be `0` if the registry was populated incorrectly. There is no bounds check on `self.texture_views[index as usize]`. If `index` is 256 or higher for any reason, this panics.

**Fix:** Add an explicit bounds assertion: `assert!(index < 256, "Texture array index overflow: {}", index)`, or better, use `self.texture_views.get_mut(index as usize)` and handle the `None` case with an error log and early return.

---

### 5.2 🟡 MEDIUM — `upload_data_texture` Pushes to `texture_bind_groups` Without Bound Check

**Lines:** 4310–4312

```rust
self.texture_bind_groups.push(bind_group);
let tid = (self.texture_bind_groups.len() - 1) as u32;
self.texture_registry.put(id.to_string(), tid);
```

**Issue:** `texture_bind_groups` grows unboundedly. `tid` is used as an index into `texture_bind_groups` in draw calls via `self.texture_bind_groups.get(id as usize)`. If `texture_bind_groups` exceeds `u32::MAX` entries (theoretical but worth noting), `tid` wraps. More practically, there is no cap on how many data textures can be uploaded, meaning repeated calls to `upload_data_texture` from a misbehaving caller will grow GPU memory until OOM.

**Fix:** Add a maximum capacity check (matching the 256-entry texture array limit) with LRU eviction, consistent with how `load_image` handles the texture array.

---

### 5.3 🟡 MEDIUM — Dynamic Buffer Growth Truncates Geometry Silently

**Lines:** 4925–4930

```rust
if req_v_size > max_v_size {
    log::error!("Exceeded dynamic vertex buffer max capacity! Capping geometry.");
    self.vertices
        .truncate((max_v_size / std::mem::size_of::<Vertex>() as u64) as usize);
    cur_v_size = max_v_size;
}
```

**Issue:** When the vertex buffer exceeds its maximum capacity (4× `MAX_VERTICES`), the CPU-side vertex vector is truncated. However, the corresponding `self.indices` vector is *not* truncated, and existing `DrawCall` entries referencing the truncated vertices are not updated. This means draw calls that reference vertex indices beyond the new truncation point will cause GPU validation errors (out-of-bounds vertex buffer reads) or silent rendering of garbage geometry.

**Fix:** When truncating vertices, also truncate or invalidate any `DrawCall` entries whose `index_start + index_count` would reference truncated vertices. The same logic applies to the index buffer truncation path.

---

### 5.4 🟢 LOW — `YggdrasilPacker::pack` Skyline Removal Loop Does Not Restore Partially Consumed Segments

**Lines:** 120–130

```rust
while remaining > 0 {
    if self.skyline[insert_idx].w <= remaining {
        remaining -= self.skyline[insert_idx].w;
        self.skyline.remove(insert_idx);  // removes in-place, shifts all subsequent
    } else {
        self.skyline[insert_idx].x += remaining;
        self.skyline[insert_idx].w -= remaining;
        remaining = 0;
    }
}
```

**Issue:** When a segment is partially consumed (the `else` branch), the segment's `x` is advanced by `remaining` but the segment's `x` should be `original_x + original_w - new_w`, not `original_x + remaining`. For the first partial segment this is equivalent (since `remaining` == the consumed width), but the logic is brittle and relies on the loop exiting immediately after the partial consumption. If the loop structure is ever modified, this will produce incorrect packer state. Additionally, this loop does not handle the case where `remaining` exactly matches the last segment width, in which case `self.skyline.remove(insert_idx)` may remove the sentinel segment.

**Fix:** Rewrite the consumption loop using a cleaner split-and-consume approach with explicit segment range tracking.

---

## Section 6: General Coding Issues

### 6.1 🟡 MEDIUM — `memoize` Is a No-Op

**Lines:** 4154–4156

```rust
fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
    render_fn(self);
}
```

**Issue:** `memoize` accepts a content hash and an ID but ignores both, unconditionally calling `render_fn`. This means the memoization contract is broken — callers that rely on memoization to skip redundant rendering work are instead re-rendering every frame. This is a correctness issue for performance-sensitive UI paths, not just a quality issue.

**Fix:** Implement a frame-level cache keyed by `(id, data_hash)` that skips `render_fn` when the hash has not changed since the last frame.

---

### 6.2 🟡 MEDIUM — `post_process_layout` and `composite_layout` Are Identical to `pipeline_layout`

**Lines:** 862–881

```rust
let post_process_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    bind_group_layouts: &[
        Some(&texture_bind_group_layout),
        Some(&env_bind_group_layout),
        Some(&berserker_bind_group_layout),
    ],
    ...
});

let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    bind_group_layouts: &[
        Some(&texture_bind_group_layout),
        Some(&env_bind_group_layout),
        Some(&berserker_bind_group_layout),
    ],
    ...
});
```

**Issue:** All three pipeline layouts (`pipeline_layout`, `post_process_layout`, `composite_layout`) are created with identical bind group layouts. This wastes GPU resources (three layout objects that are logically the same) and defeats the purpose of having separate layouts, which would be to enforce that post-process pipelines only use the bind groups they need. The comment on line 861 says "only need Group 0 + Globals" but the layout actually includes all three groups.

**Fix:** Either use `pipeline_layout` for all pipelines, or actually restrict `post_process_layout` to only the bind groups post-process passes need.

---

### 6.3 🟢 LOW — `find_filter` Is Defined but Never Called

**Lines:** 5483–5489

```rust
fn find_filter<'a>(tree: &'a usvg::Tree, filter_id: &str) -> Option<&'a usvg::filter::Filter> {
    tree.filters().iter().find(|f| f.id() == filter_id).map(|arc| arc.as_ref())
}
```

**Issue:** This function exists but is never called anywhere in the file. SVG filters are not applied during tessellation. This is dead code.

**Fix:** Remove the function or wire it into `tessellate_node` if SVG filter support is planned.

---

### 6.4 🟢 LOW — `parse_svg_animations` Uses `unwrap_or("")` on UTF-8 Decode Failure

**Lines:** 4656

```rust
if let Ok(xml_doc) = roxmltree::Document::parse(std::str::from_utf8(data).unwrap_or("")) {
```

**Issue:** If `data` is not valid UTF-8, `std::str::from_utf8` returns `Err` and the fallback `""` causes `roxmltree` to parse an empty document and return no animations. There is no log entry indicating that the SVG data was not valid UTF-8. This will silently produce an SVG with no animations.

**Fix:** Return early with a `log::warn!` if UTF-8 decode fails.

---

### 6.5 🟢 LOW — File Exceeds 4,500 LOC Threshold for Mandatory Modularization

**Line count:** 5,491

Per the project's own `Code_Modularization_Plan.docx` (v1.0), this file is a **Tier 1 Critical** monolith and requires immediate modularization action. Suggested module boundaries:

| Proposed Module | Contents |
|---|---|
| `packer.rs` | `YggdrasilPacker`, `SkylineSegment` |
| `svg.rs` | `load_svg`, `draw_svg`, `tessellate_node`, `parse_svg_animations`, `usvg_to_lyon`, `SvgAnimation`, `SvgModel` |
| `text.rs` | `draw_text`, `draw_shaped_text`, `shape_text_with_stack`, `shape_rich_text`, `measure_text` |
| `passes.rs` | All render pass encoding (background, blur, glass, ui, bloom, composite) |
| `atlas.rs` | `load_image_to_atlas`, `load_image`, `reclaim_vram`, `rebuild_texture_array_bind_group` |
| `primitives.rs` | `fill_rect*`, `stroke_rect`, `stroke_path`, `draw_line`, `gungnir`, `bifrost`, `mani_glow`, etc. |
| `lib.rs` | `SurtrRenderer` struct definition, `forge`, `forge_headless`, `begin_frame`, `end_frame`, `capture_frame` |

---

## Issue Summary Table

| ID | Severity | Category | Description |
|---|---|---|---|
| 1.1 | 🔴 CRITICAL | Pipeline Order | Parallel rayon passes write to shared render target |
| 1.2 | 🔴 CRITICAL | Pipeline Order | Bloom extract overwrites backdrop blur texture |
| 2.1 | 🔴 CRITICAL | Ordinal/Index | `stroke_path` DrawCall uses vertex cursor as index cursor |
| 3.1 | 🔴 CRITICAL | SVG | Tessellation `.unwrap()` panics on malformed SVG |
| 4.1 | 🔴 CRITICAL | Text | Glyph atlas fallback writes to origin `(0,0)` silently |
| 1.3 | 🟡 MEDIUM | Pipeline Order | Background depth compare `Always` is fragile |
| 1.4 | 🟡 MEDIUM | Pipeline Order | Blur pipelines use alpha blending instead of replace |
| 1.5 | 🟡 MEDIUM | Pipeline Order | Composite/bloom share layout without type safety |
| 2.2 | 🟡 MEDIUM | Coordinates | SVG rotation center ignored from `from`/`to` attribute |
| 2.3 | 🟡 MEDIUM | Coordinates | `draw_line` does not rotate the rectangle to match the line angle |
| 2.4 | 🟡 MEDIUM | Coordinates | `push_transform_3d` extracts scale incorrectly; `pop_transform_3d` double-pops |
| 2.5 | 🟡 MEDIUM | Coordinates | Fill tessellator ignores active clip rect |
| 3.2 | 🟡 MEDIUM | SVG | SVG stroke paths silently skipped during tessellation |
| 3.3 | 🟡 MEDIUM | SVG | SVG gradient/pattern fills silently become white |
| 3.4 | 🟡 MEDIUM | SVG | `viewBox` origin hardcoded to `(0,0)` |
| 4.2 | 🟡 MEDIUM | Text | Subpixel glyph data incorrectly expanded as alpha-only |
| 4.3 | 🟡 MEDIUM | Text | `measure_text` does not apply HiDPI scale factor |
| 5.1 | 🟡 MEDIUM | Buffer Safety | Texture array index write unchecked against bounds |
| 5.2 | 🟡 MEDIUM | Buffer Safety | `upload_data_texture` grows `texture_bind_groups` without cap |
| 5.3 | 🟡 MEDIUM | Buffer Safety | Buffer truncation does not invalidate referencing DrawCalls |
| 6.1 | 🟡 MEDIUM | General | `memoize` is a no-op |
| 6.2 | 🟡 MEDIUM | General | Three pipeline layouts are identical — wasteful and misleading |
| 2.6 | 🟢 LOW | Coordinates | `bifrost` screen-space UVs not clamped to `[0,1]` |
| 4.4 | 🟢 LOW | Text | Duplicate contradictory doc comment on `load_image` |
| 5.4 | 🟢 LOW | Buffer Safety | `YggdrasilPacker` skyline loop brittle on exact-match consumption |
| 6.3 | 🟢 LOW | General | `find_filter` is dead code |
| 6.4 | 🟢 LOW | General | `parse_svg_animations` silently accepts non-UTF-8 data |
| 6.5 | 🟢 LOW | General | File exceeds Tier 1 modularization threshold (5,491 LOC) |

---

*Audit generated from static analysis of `lib.rs` — no runtime profiling or GPU validation layer output was used. All findings should be verified with `wgpu` validation layer enabled (`WGPU_BACKEND=vulkan RUST_LOG=wgpu=debug`) and `cargo test --workspace`.*
