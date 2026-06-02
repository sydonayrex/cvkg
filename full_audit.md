# SURTR RENDERER COMPREHENSIVE AUDIT REPORT

## EXECUTIVE SUMMARY

The Surtr renderer is architecturally ambitious with a capable multi-pass wgpu pipeline, but suffers from **architectural fragmentation** and several **critical correctness defects**. The shader system contains multiple competing architectures (Gaussian Blur vs. Dual Kawase vs. SVG Filter Framework), and the material system uses a monolithic mode-driven approach that should be pipeline-based.

**Immediate action required on 7 CRITICAL issues** that could cause panics, silent rendering corruption, or GPU validation errors.

---

## SEVERITY LEGEND

| Severity | Description |
|----------|-------------|
| 🔴 CRITICAL | Causes crashes, data corruption, or undefined behavior |
| 🟡 HIGH | Causes incorrect rendering or significant performance issues |
| 🟢 MEDIUM | Minor correctness issues or maintainability concerns |
| 🔵 LOW | Code quality, documentation, or optimization opportunities |

---

## SECTION 1: CRITICAL ISSUES (IMMEDIATE ACTION REQUIRED)

### 🔴 CRITICAL-001: Invalid WGSL Syntax in blur_pyramid.wgsl
**Source:** AR WGSL-001 | **File:** blur_pyramid.wgsl | **Severity:** P0

**Issue:** Shader contains invalid syntax that prevents compilation.

```wgsl
// BROKEN:
@Override
group(0) @binding(0)

// CORRECT:
@group(0)
@binding(0)
```

**Impact:** Shader compilation failure. Dual Kawase blur architecture cannot execute.

---

### 🔴 CRITICAL-002: Glass/Blur Architecture Mismatch
**Source:** AR WGSL-006 | **Files:** bifrost.wgsl, bloom.wgsl | **Severity:** P0

**Issue:** Glass shader expects mip-level sampling from a blur pyramid, but renderer generates Gaussian blur ping-pong.

```text
Glass expects:     Renderer generates:
┌─────────────┐    ┌─────────────────────┐
│ Mip Pyramid  │    │ Gaussian Blur H/V   │
│ Sampling     │    │ Ping-Pong           │
└─────────────┘    └─────────────────────┘
```

**Impact:** Glass rendering produces incorrect blur results. This is the largest shader/render-graph mismatch in the project.

---

### 🔴 CRITICAL-003: Parallel Rayon Passes Write to Shared Render Target
**Source:** CA 1.1 | **File:** lib.rs:2947-3139 | **Severity:** CRITICAL

**Issue:** Glass pass and UI pass encoded in parallel via `rayon::join`, both writing to `ctx_scene_texture` with `LoadOp::Load`.

```rust
let (glass_cb, ui_cb) = rayon::join(
    || { /* writes to ctx_scene_texture */ },
    || { /* also writes to ctx_scene_texture */ },
);
```

**Impact:** Undefined behavior at GPU API level. UI pass may read stale or partially-written glass output.

---

### 🔴 CRITICAL-004: Bloom Extract Overwrites Backdrop Blur
**Source:** CA 1.2 | **File:** lib.rs:3174-3199 | **Severity:** CRITICAL

**Issue:** Pass 5 (Bloom Extract) unconditionally overwrites `ctx_blur_texture_a`, destroying backdrop blur data needed by Glass pass.

**Impact:** Shared texture resources between bloom and backdrop blur with no separation. Any change to iteration counts silently breaks the other subsystem.

---

### 🔴 CRITICAL-005: stroke_path DrawCall Index Corruption
**Source:** CA 2.1 | **File:** lib.rs:4639-4650 | **Severity:** CRITICAL

**Issue:** Uses vertex cursor instead of index cursor for `index_start`.

```rust
index_start: base_vertex_idx,  // BUG: vertex cursor, not index cursor
```

**Impact:** GPU reads index data from wrong position. Causes incorrect geometry, garbage rendering, or out-of-bounds GPU access.

---

### 🔴 CRITICAL-006: tessellate_node Panics on Malformed SVG
**Source:** CA 3.1 | **File:** lib.rs:5118-5132 | **Severity:** CRITICAL

**Issue:** `.unwrap()` on tessellation result causes process panic on degenerate SVG paths.

```rust
tessellator.tessellate_path(...).unwrap();  // PANICS on malformed SVG
```

**Impact:** Any user-supplied SVG with degenerate paths crashes the application.

---

### 🔴 CRITICAL-007: Glyph Atlas Fallback Writes to Origin (0,0)
**Source:** CA 4.1 | **File:** lib.rs:3845-3852, 3977-3983 | **Severity:** CRITICAL

**Issue:** When atlas is full after reclaim, fallback `unwrap_or((0, 0))` silently overwrites atlas origin.

```rust
self.atlas_packer.pack(gw, gh).unwrap_or((0, 0))  // Overwrites origin!
```

**Impact:** Glyphs at atlas origin render incorrectly. Silent corruption with no error logging.

---

## SECTION 2: HIGH SEVERITY ISSUES

### 🟡 HIGH-001: Dead Blur Architecture
**Source:** AR WGSL-002 | **Files:** bloom.wgsl, blur_pyramid.wgsl | **Severity:** P1

**Issue:** blur_pyramid.wgsl implements Dual Kawase blur but is compiled and never executed. Evidence of unfinished architecture migration.

---

### 🟡 HIGH-002: Monolithic Material Shader
**Source:** AR WGSL-003 | **File:** Fragment shader | **Severity:** P1

**Issue:** Single fragment shader handles 10+ material types via mode dispatch:
- Rectangle, Rounded Rectangle, Ellipse
- Gradient, Glass, PBR, Raymarch
- Stroke, Shadow, Lightning

**Impact:** Branch divergence, register pressure, instruction cache pressure, compilation complexity.

---

### 🟡 HIGH-003: Legacy Bloom Architecture
**Source:** AR WGSL-008, WGSL-009 | **File:** bloom.wgsl | **Severity:** P1

**Issue:** Full-resolution blur with no downsampling pyramid.

```text
Current:     Extract → Blur H → Blur V → Blur H → Blur V → Composite
Expected:    Extract → 1/2 → 1/4 → 1/8 → 1/16 → Upsample → Composite
```

**Impact:** Poor scaling, unnecessary GPU cost.

---

### 🟡 HIGH-004: SVG Alpha Compositing Defects
**Source:** AR WGSL-012 | **File:** svg_filters.wgsl | **Severity:** P1

**Issue:** Blend modes operate on RGBA instead of premultiplied alpha.

**Impact:** Dark edges, transparency halos, incorrect blending.

---

### 🟡 HIGH-005: SVG Stroke Paths Not Tessellated
**Source:** CA 3.2 | **File:** lib.rs:5101-5138 | **Severity:** MEDIUM

**Issue:** Only fill paths tessellated. Stroke-only SVGs render invisible.

---

### 🟡 HIGH-006: SVG Gradients Fall Back to White
**Source:** CA 3.3 | **File:** lib.rs:5104-5112 | **Severity:** MEDIUM

**Issue:** Gradient/pattern fills silently become solid white.

---

### 🟡 HIGH-007: Vertex Format Too Large (~140+ bytes)
**Source:** AR VP-003 | **File:** Vertex structure | **Severity:** P1

**Issue:** Monolithic vertex format carries unused attributes for all geometry types.

---

### 🟡 HIGH-008: Transform Ordering Ambiguity
**Source:** AR VP-004 | **File:** Vertex shader | **Severity:** P1

**Issue:** Transform components stored separately; shader must reconstruct. Order (SRT vs RST vs TSR) produces different results.

---

## SECTION 3: MEDIUM SEVERITY ISSUES

### 🟢 MEDIUM-001: Background Depth Compare Always
**Source:** CA 1.3 | **File:** lib.rs:935-945

**Issue:** Background pipeline uses `depth_compare: Always`, allowing it to overwrite scene geometry depth.

---

### 🟢 MEDIUM-002: Blur Pipelines Use Alpha Blending
**Source:** CA 1.4 | **File:** lib.rs:1018, 1044

**Issue:** Gaussian blur uses `ALPHA_BLENDING` instead of `blend: None`. Semantically wrong and fragile.

---

### 🟢 MEDIUM-003: SVG Animation Rotation Center Ignored
**Source:** CA 2.2 | **File:** lib.rs:5179-5217

**Issue:** `rotate(angle cx cy)` center coordinates silently dropped.

---

### 🟢 MEDIUM-004: draw_line Does Not Rotate Rectangle
**Source:** CA 2.3 | **File:** lib.rs:3780-3810

**Issue:** Lines drawn as axis-aligned rectangles, never rotated to match line direction.

---

### 🟢 MEDIUM-005: 3D Transform Decomposition Incorrect
**Source:** CA 2.4 | **File:** lib.rs:4412-4418

**Issue:** Scale extraction from 4x4 matrix uses diagonal only, ignoring rotation. `pop_transform_3d` double-pops stack.

---

### 🟢 MEDIUM-006: Fill Tessellator Ignores Clip Rect
**Source:** CA 2.5 | **File:** lib.rs:4808-4809

**Issue:** Hardcoded "infinite" clip rect bypasses active clip state.

---

### 🟢 MEDIUM-007: SVG viewBox Origin Hardcoded
**Source:** CA 3.4 | **File:** lib.rs:5160-5161

**Issue:** viewBox x,y always set to (0,0), ignoring actual SVG attribute.

---

### 🟢 MEDIUM-008: measure_text HiDPI Scale Mismatch
**Source:** CA 4.3 | **File:** lib.rs:3829-3832, 3914-3917

**Issue:** `draw_text` scales by scale_factor; `measure_text` does not. Returns physical pixels on HiDPI.

---

### 🟢 MEDIUM-009: Texture Array Index Unchecked
**Source:** CA 5.1 | **File:** lib.rs:4111-4123

**Issue:** No bounds check on `texture_views[index as usize]`.

---

### 🟢 MEDIUM-010: memoize Is No-Op
**Source:** CA 6.1 | **File:** lib.rs:4154-4156

**Issue:** Accepts hash and ID but ignores both, unconditionally calling render function.

---

## SECTION 4: LOW SEVERITY ISSUES

### 🔵 LOW-001: bifrost UV Coordinates Not Clamped
**Source:** CA 2.6 | **File:** lib.rs:3499-3507

**Issue:** Screen-space UVs can exceed [0,1] for off-screen panels.

---

### 🔵 LOW-002: Duplicate Doc Comment
**Source:** CA 4.4 | **File:** lib.rs:4060-4062

**Issue:** Conflicting doc comments on `load_image`.

---

### 🔵 LOW-003: find_filter Dead Code
**Source:** CA 6.3 | **File:** lib.rs:5483-5489

**Issue:** Function defined but never called.

---

### 🔵 LOW-004: File Exceeds Modularization Threshold
**Source:** CA 6.5 | **File:** lib.rs

**Issue:** 5,491 lines exceeds Tier 1 critical threshold (4,500 LOC).

---

## SECTION 5: ARCHITECTURAL RECOMMENDATIONS

### 5.1 Adopt SVG Filter Framework as Post-Processing Foundation

The SVG filter framework is the most advanced shader system in the renderer:
- Render graph node processing architecture
- Supports 14+ filter primitives
- Already structured for composable effects

**Recommendation:** Extend svg_filters.wgsl to unify bloom, glass, and accessibility under a single post-processing architecture.

---

### 5.2 Pipeline-Based Material System

**Current:**
```
One Pipeline → Many Modes (10+)
```

**Recommended:**
```
UI Pipeline
Glass Pipeline
Gradient Pipeline
SVG Pipeline
PBR Pipeline
```

**Benefits:** Lower divergence, smaller binaries, better compiler optimization, cleaner debugging.

---

### 5.3 Separate Bloom and Backdrop Blur Textures

Allocate dedicated textures:
- `bloom_tex_a`, `bloom_tex_b` (lower resolution)
- `backdrop_tex_a`, `backdrop_tex_b` (full resolution)

---

### 5.4 Reduce Vertex Format Size

Consider instancing for per-quad transform data:
```rust
// Current: 4x duplication per quad
translation, scale, rotation → per vertex

// Recommended: Instance buffers
translation, scale, rotation → per instance
```

---

## SECTION 6: ISSUE SUMMARY TABLE

| ID | Severity | Category | Source | Description |
|----|----------|----------|--------|-------------|
| CRITICAL-001 | 🔴 P0 | WGSL | AR WGSL-001 | Invalid syntax in blur_pyramid.wgsl |
| CRITICAL-002 | 🔴 P0 | Architecture | AR WGSL-006 | Glass expects blur pyramid, gets Gaussian |
| CRITICAL-003 | 🔴 CRITICAL | Pipeline | CA 1.1 | Parallel rayon passes share render target |
| CRITICAL-004 | 🔴 CRITICAL | Pipeline | CA 1.2 | Bloom overwrites backdrop blur texture |
| CRITICAL-005 | 🔴 CRITICAL | Index | CA 2.1 | stroke_path uses vertex cursor as index cursor |
| CRITICAL-006 | 🔴 CRITICAL | SVG | CA 3.1 | tessellate_node .unwrap() panics on malformed SVG |
| CRITICAL-007 | 🔴 CRITICAL | Text | CA 4.1 | Glyph atlas fallback writes to origin (0,0) |
| HIGH-001 | 🟡 P1 | Architecture | AR WGSL-002 | Dead blur architecture (compiled, not executed) |
| HIGH-002 | 🟡 P1 | Architecture | AR WGSL-003 | Monolithic material shader with 10+ modes |
| HIGH-003 | 🟡 P1 | Performance | AR WGSL-008 | Legacy bloom at full resolution |
| HIGH-004 | 🟡 P1 | SVG | AR WGSL-012 | Incorrect alpha model for blend modes |
| HIGH-005 | 🟡 MEDIUM | SVG | CA 3.2 | Stroke paths not tessellated |
| HIGH-006 | 🟡 MEDIUM | SVG | CA 3.3 | Gradients fall back to white |
| HIGH-007 | 🟡 P1 | Vertex | AR VP-003 | Vertex format ~140+ bytes |
| HIGH-008 | 🟡 P1 | Vertex | AR VP-004 | Transform ordering ambiguity |

---

## SECTION 7: VERIFICATION CHECKLIST

### Before Deployment:
- [ ] Fix all CRITICAL issues (CRITICAL-001 through CRITICAL-007)
- [ ] Run with `WGPU_BACKEND=vulkan RUST_LOG=wgpu=debug` validation layer
- [ ] Test with malformed SVG inputs
- [ ] Test with HiDPI displays (scale_factor > 1.0)
- [ ] Test with stroke-only SVGs
- [ ] Test with gradient-filled SVGs
- [ ] Test glass panels at screen edges
- [ ] Verify text measurement on Retina displays
- [ ] Run `cargo test --workspace` with all features

---

**Report Generated:** 2026-06-01  
**Sources:** Surtr_Renderer_Code_Audit.md, Surtr-Arch-Review.md, shader_audit_addendum.md  
**Auditor:** Static analysis + architectural review synthesis
