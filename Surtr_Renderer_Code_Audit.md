# Surtr Renderer (`lib.rs`) — Code Audit Report
**File:** `cvkg-render-gpu/src/lib.rs`  
**Lines:** 5,491  
**Auditor:** Static analysis pass  
**Scope:** Coordinate relations, render pipeline order-of-operations, SVG/text/shape drawing correctness, ordinal/index safety, buffer overflow, general coding issues

---

## Executive Summary

The Surtr renderer is a capable and architecturally ambitious wgpu-based GPU renderer. The multi-pass pipeline structure is sound in principle, but several correctness issues, ordering hazards, and latent overflow conditions exist that could cause silent rendering artifacts, GPU validation errors, or hard panics under production load.

**UPDATE:** All CRITICAL issues have been resolved. See Resolution Notes section below for details on the fixes applied.

---

## Section 1: Render Pipeline Order-of-Operations

### 1.1 ✅ RESOLVED — Parallel Rayon Passes Write to the Same Render Target Without Synchronization

**Lines:** 2947–3139

**Status:** **FIXED** - Changed to sequential encoding via helper functions `encode_glass_pass()` and `encode_ui_pass()`.

**Resolution:** Replaced `rayon::join` parallel encoding with sequential encoding to ensure proper GPU API synchronization. The helper functions were extracted to maintain code organization while ensuring passes are encoded on a single thread before submission.

---

### 1.2 ✅ RESOLVED — Bloom Extract Pass Overwrites the Backdrop Blur Result

**Lines:** 3174–3199

**Status:** **FIXED** - Dedicated bloom textures added.

**Resolution:** Added dedicated bloom textures (`bloom_texture_a`, `bloom_texture_b`) to both `SurfaceContext` and `HeadlessContext` structs, with corresponding bind groups (`bloom_bind_group_a`, `bloom_bind_group_b`, `bloom_env_bind_group_a`). Updated Pass 5 (Bloom Extract) and Pass 6 (Bloom Blur) to use these dedicated textures, preventing the conflict.

---

### 1.3 🟡 MEDIUM — Background Pipeline Uses `depth_compare: Always`, Allowing It to Overwrite Scene Geometry

**Lines:** 935–945

**Status:** **VERIFIED SAFE** - Code already uses `depth_write_enabled: false`.

**Resolution:** Upon inspection, the codebase already has `depth_write_enabled: Some(false)` in the background pipeline, which prevents overwriting scene geometry. No change needed.

---

### 1.4 🟡 MEDIUM — Blur Pipelines Use `ALPHA_BLENDING` Instead of No Blend

**Lines:** 1018, 1044

**Status:** **FIXED** - Changed to `blend: None`.

**Resolution:** Changed `blend: Some(wgpu::BlendState::ALPHA_BLENDING)` to `blend: None` for both `blur_h_pipeline` and `blur_v_pipeline`. Blur is a full-screen filter operation that replaces the destination pixel entirely.

---

### 1.5 🟡 MEDIUM — Post-Processing Uses a Shared Bloom Texture as Composite Input While Bloom Is Still Being Written

**Lines:** 3279–3283

**Status:** **RESOLVED** (as part of fix 1.2) - Now uses dedicated bloom textures.

**Resolution:** With the dedicated bloom textures, the composite pass now correctly samples the bloom result without conflict.

---

## Section 2: Coordinate Relations and Ordinal Point Issues

### 2.1 🔴 CRITICAL — `stroke_path` DrawCall Uses Vertex Index as Index Buffer Cursor

**Lines:** 4639–4650

**Status:** **FIXED** - Added separate `base_index_idx` tracking.

**Resolution:** Added `let base_index_idx = self.indices.len() as u32;` and changed `index_start: base_vertex_idx` to `index_start: base_index_idx` in the DrawCall struct initialization.

---

### 2.2 🟡 MEDIUM — SVG Animation Rotation Is Applied in SVG Space, Not Logical Render Space

**Lines:** 5179–5217

**Status:** Pending - Requires parse_svg_animations enhancement.

**Resolution:** TBD - Would require parsing the full `rotate(angle cx cy)` syntax from animateTransform attributes.

---

### 2.3 🟡 MEDIUM — `draw_line` Does Not Apply Rotation to the Rectangle

**Lines:** 3780–3810

**Status:** **FIXED** - Refactored to use Lyon stroke_path.

**Resolution:** Changed from drawing an axis-aligned rectangle to using Lyon's `stroke_path()` which properly tessellates the line and applies rotation through the transform stack.

---

### 2.4 🟡 MEDIUM — `push_transform_3d` Extracts 3D Scale/Rotation Incorrectly into 2D Stack

**Lines:** 4412–4418

**Status:** **FIXED** - Proper matrix decomposition.

**Resolution:** 
- Changed to use `glam::Mat4::to_scale_rotation_translation()` for correct scale/rotation extraction
- Removed the erroneous second `pop()` from `pop_transform_3d()`

---

### 2.5 🟡 MEDIUM — `SceneVertexConstructor` for Fill Paths Uses Hardcoded Infinite Clip Rect

**Lines:** 4808–4809

**Status:** Pending - Requires architectural change.

**Resolution:** TBD - Would require passing clip state through SceneVertexConstructor.

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

**Status:** **FIXED** - Graceful error handling.

**Resolution:** Changed `.unwrap()` to `if let Err(e)` with `log::warn!` and early return. Also fixed `load_svg` to use graceful error handling instead of `.expect()`.

---

### 3.2 🟡 MEDIUM — SVG Stroke Paths Are Not Tessellated at Load Time

**Lines:** 5101–5138

**Status:** **FIXED** - Stroke tessellation added.

**Resolution:** Added separate `StrokeTessellator` instance and branch to tessellate stroke paths alongside fill paths. The fix properly handles `path.stroke().is_some()` and tessellates strokes with correct width extraction.

---

### 3.3 🟡 MEDIUM — SVG Paint Gradients and Patterns Fall Back to White Silently

**Lines:** 5104–5112

**Status:** **FIXED** - Added warning log.

**Resolution:** Added `log::warn!` when gradient/pattern fills encounter unsupported `usvg::Paint` types, providing diagnostic output for incorrect SVG rendering.

---

### 3.4 🟡 MEDIUM — `draw_svg` Computes `_scale_x`/`_scale_y` but Never Uses Them

**Lines:** 5160–5161

**Status:** Partial - usvg API limitation.

**Resolution:** Attempted to read viewBox from `tree.view_box()` but the usvg Tree API doesn't expose this method. The `view_box` remains hardcoded to `(0, 0)` origin. This may require upstream usvg changes or alternative viewBox extraction approach.

---

## Section 4: Text Glyph Rendering Issues

### 4.1 🔴 CRITICAL — Glyph Atlas Upload After `reclaim_vram()` Uses Stale Pack Position `(0, 0)` as Fallback

**Lines:** 3845–3852, 3977–3983

**Status:** **FIXED** - Skip glyph with error logging.

**Resolution:** Changed `unwrap_or((0, 0))` to proper match expressions in both `draw_text` and `draw_shaped_text` that log errors and `continue` to skip the glyph rather than corrupting the atlas origin.

---

### 4.2 🟡 MEDIUM — Glyph Rasterization Assumes Alpha-Only Output but Assumes No Subpixel Color

**Lines:** 3854–3860

**Status:** Pending - Requires format detection.

**Resolution:** TBD - Would require checking the rasterizer's output format before expanding RGB channels.

---

### 4.3 🟡 MEDIUM — `draw_text` Calls `shape_text_with_stack` at Physical Scale but `measure_text` Does Not

**Lines:** 3829–3832 vs 3914–3917

**Status:** **FIXED** - HiDPI scale factor applied.

**Resolution:** Added scale factor multiplication before shaping and division after for `measure_text`, ensuring consistent logical units on HiDPI displays.

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

| ID | Severity | Category | Description | Status |
|---|---|---|---|---|
| 1.1 | 🔴 CRITICAL | Pipeline Order | Parallel rayon passes write to shared render target | ✅ **FIXED** - Sequential encoding |
| 1.2 | 🔴 CRITICAL | Pipeline Order | Bloom extract overwrites backdrop blur texture | ✅ **FIXED** - Dedicated bloom textures |
| 2.1 | 🔴 CRITICAL | Ordinal/Index | `stroke_path` DrawCall uses vertex cursor as index cursor | ✅ **FIXED** - Use index cursor |
| 3.1 | 🔴 CRITICAL | SVG | Tessellation `.unwrap()` panics on malformed SVG | ✅ **FIXED** - Graceful error handling |
| 4.1 | 🔴 CRITICAL | Text | Glyph atlas fallback writes to origin `(0,0)` silently | ✅ **FIXED** - Skip glyph with error log |
| 1.3 | 🟡 MEDIUM | Pipeline Order | Background depth compare `Always` is fragile | ✅ **VERIFIED SAFE** - Already `depth_write_enabled: false` |
| 1.4 | 🟡 MEDIUM | Pipeline Order | Blur pipelines use alpha blending instead of replace | ✅ **FIXED** - `blend: None` |
| 1.5 | 🟡 MEDIUM | Pipeline Order | Composite/bloom share layout without type safety | 🔄 Pending |
| 2.2 | 🟡 MEDIUM | Coordinates | SVG rotation center ignored from `from`/`to` attribute | 🔄 Pending |
| 2.3 | 🟡 MEDIUM | Coordinates | `draw_line` does not rotate the rectangle to match the line angle | ✅ **FIXED** - Uses Lyon stroke_path |
| 2.4 | 🟡 MEDIUM | Coordinates | `push_transform_3d` extracts scale incorrectly; `pop_transform_3d` double-pops | ✅ **FIXED** - Proper decomposition |
| 2.5 | 🟡 MEDIUM | Coordinates | Fill tessellator ignores active clip rect | 🔄 Pending |
| 3.2 | 🟡 MEDIUM | SVG | SVG stroke paths silently skipped during tessellation | ✅ **FIXED** - Stroke tessellation added |
| 3.3 | 🟡 MEDIUM | SVG | SVG gradient/pattern fills silently become white | ✅ **FIXED** - Added warn! log |
| 3.4 | 🟡 MEDIUM | SVG | `viewBox` origin hardcoded to `(0,0)` | ⚠️ Partial - API limitation in usvg |
| 4.2 | 🟡 MEDIUM | Text | Subpixel glyph data incorrectly expanded as alpha-only | 🔄 Pending |
| 4.3 | 🟡 MEDIUM | Text | `measure_text` does not apply HiDPI scale factor | ✅ **FIXED** - Scale factor applied |
| 5.1 | 🟡 MEDIUM | Buffer Safety | Texture array index write unchecked against bounds | 🔄 Pending |
| 5.2 | 🟡 MEDIUM | Buffer Safety | `upload_data_texture` grows `texture_bind_groups` without cap | 🔄 Pending |
| 5.3 | 🟡 MEDIUM | Buffer Safety | Buffer truncation does not invalidate referencing DrawCalls | 🔄 Pending |
| 6.1 | 🟡 MEDIUM | General | `memoize` is a no-op | ✅ **FIXED** - Implemented caching |
| 6.2 | 🟡 MEDIUM | General | Three pipeline layouts are identical — wasteful and misleading | 🔄 Pending |
| 2.6 | 🟢 LOW | Coordinates | `bifrost` screen-space UVs not clamped to `[0,1]` | 🔄 Pending |
| 4.4 | 🟢 LOW | Text | Duplicate contradictory doc comment on `load_image` | 🔄 Pending |
| 5.4 | 🟢 LOW | Buffer Safety | `YggdrasilPacker` skyline loop brittle on exact-match consumption | 🔄 Pending |
| 6.3 | 🟢 LOW | General | `find_filter` is dead code | 🔄 Pending |
| 6.4 | 🟢 LOW | General | `parse_svg_animations` silently accepts non-UTF-8 data | 🔄 Pending |
| 6.5 | 🟢 LOW | General | File exceeds Tier 1 modularization threshold (5,491 LOC) | 🔄 Pending |

---

## Resolution Notes

### CRITICAL Issues (All Resolved)

1. **1.1 Parallel Rayon Passes** - Replaced `rayon::join` with sequential encoding via `encode_glass_pass()` and `encode_ui_pass()` helper functions to ensure proper GPU API synchronization.

2. **1.2 Bloom Overwrite** - Added dedicated bloom textures (`bloom_texture_a`, `bloom_texture_b`) to both `SurfaceContext` and `HeadlessContext` structs, with corresponding bind groups. Updated Pass 5 (Bloom Extract) and Pass 6 (Bloom Blur) to use these dedicated textures.

3. **2.1 Index Cursor Bug** - Added `base_index_idx` variable and used it for `index_start` in DrawCall instead of `base_vertex_idx`.

4. **3.1 Tessellation Panic** - Changed `.unwrap()` to `if let Err(e)` with `log::warn!` and early return for graceful handling.

5. **4.1 Glyph Atlas Origin Bug** - Changed `unwrap_or((0, 0))` to proper match expressions that log errors and `continue` to skip glyphs.

### MEDIUM Issues (Partially Resolved)

6. **1.3 Background Depth** - Verified codebase already uses `depth_write_enabled: false` with `CompareFunction::Always`, which is correct.

7. **1.4 Blur Blending** - Changed `blend: Some(wgpu::BlendState::ALPHA_BLENDING)` to `blend: None` for both blur pipelines.

8. **2.3 draw_line Rotation** - Refactored to use Lyon path tessellation via `stroke_path()` which properly handles rotation through the transform stack.

9. **2.4 Transform 3D** - Fixed matrix decomposition using `glam::Mat4::to_scale_rotation_translation()` and removed erroneous second pop from `pop_transform_3d`.

10. **3.2 SVG Stroke Tessellation** - Added separate `StrokeTessellator` and branch to tessellate stroke paths alongside fill paths.

11. **3.3 SVG Gradient Warning** - Added `log::warn!` when gradient/pattern fills encounter unsupported paint types.

12. **3.4 viewBox Origin** - Attempted fix but usvg's `Tree` API doesn't expose `view_box()` method; may require upstream API or alternative approach.

13. **4.3 measure_text HiDPI** - Added scale factor multiplication/division to convert between logical and physical units correctly.

14. **6.1 memoize Implementation** - Implemented frame-level cache with `memo_cache: HashMap<u64, u64>` that tracks (id, data_hash) pairs and skips rendering when unchanged.

---

*Audit generated from static analysis of `lib.rs` — no runtime profiling or GPU validation layer output was used. All findings should be verified with `wgpu` validation layer enabled (`WGPU_BACKEND=vulkan RUST_LOG=wgpu=debug`) and `cargo test --workspace`.*
