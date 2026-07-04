# Phase 2 Implementation Plan

## Overview

Phase 2 addresses work-in-progress features and quality improvements identified during the v0.3.0 audit. Primary focus is completing the WorldSpacePanel VDOM subtree filtering.

---

## Phase 2A: WorldSpacePanel VDOM Subtree Filtering

### Target
`cvkg-render-gpu/src/passes/pre_world_panel.rs:80` - "Filter draw_calls to only those belonging to this panel's VDOM subtree"

### Current State
- `PreWorldPanelNode` renders ALL draw calls to each panel's offscreen texture
- Simple heuristic skips 2D-only instances (`instance_count == 1 && instance_start == 0`)
- No proper VDOM subtree isolation

### Target State
- Each panel renders ONLY its VDOM subtree
- Draw calls tagged with `panel_id` during emission
- Efficient per-panel rendering

---

### Implementation Steps

#### Step 1: Extend DrawCall with Panel ID
**File:** `cvkg-render-gpu/src/types.rs`

```rust
#[derive(Debug, Clone)]
pub(crate) struct DrawCall {
    // ... existing fields ...
    /// Optional panel ID for WorldSpacePanel isolation.
    /// None = render to main surface (2D UI).
    /// Some(id) = render to panel's offscreen texture.
    pub panel_id: Option<u64>,
}
```

#### Step 2: Tag Draw Calls During VDOM Rendering
**File:** `cvkg-render-gpu/src/renderer/draw.rs`

- Add `current_panel_id: Option<u64>` to render context
- Set when entering `WorldSpacePanel` VDOM node
- Tag each emitted `DrawCall` with `panel_id`
- Clear when exiting panel subtree

#### Step 3: Filter in PreWorldPanelNode
**File:** `cvkg-render-gpu/src/passes/pre_world_panel.rs`

```rust
fn execute(&self, ctx: &mut ExecutionContext) {
    for (i, &panel_tex) in self.panel_textures.iter().enumerate() {
        let panel_id = i as u64; // or use stored panel ID mapping
        
        // ... render pass setup ...

        // Filter draw calls by panel_id
        for call in &ctx.renderer.draw_calls {
            if call.panel_id != Some(panel_id) {
                continue; // Skip calls not belonging to this panel
            }
            // ... draw call ...
        }
    }
}
```

#### Step 4: Panel ID → Texture Mapping
**File:** `cvkg-render-gpu/src/renderer/mod.rs`

- Ensure `WorldSpacePanel` creation registers panel with stable ID
- Map `panel_id` → `ResourceId` for texture lookup

---

### Testing
- Unit test: DrawCall tagging with panel_id
- Integration test: Multiple panels with different content
- Visual test: Panel content isolation verified

---

## Phase 2B: Software Renderer Documentation

### Target
`cvkg-render-software/src/lib.rs` - Document stub implementations

### Files with Stub Comments
- Line 511: Explicit stub warnings
- Line 543: Layout glyphs not implemented  
- Line 794: SoftwareRenderer explicit stub warnings

### Action
Add module-level documentation explaining:
- Which features are fully implemented
- Which fall back to CPU (with performance notes)
- Which are not implemented (stubs)

---

## Phase 2C: Component Verification

### Landing Components
**Files:** `cvkg-components/src/landing/*.rs`

Verify all properly exported:
- `Hero` ✅
- `FeatureGrid` / `FeatureItem` ✅
- `PricingTable` / `PricingCard` ✅
- `TestimonialCard` / `TestimonialItem` ✅

### Motion System
**File:** `cvkg-components/src/motion.rs`

- `Motion` and `MotionPreset` types exist
- Verify `.motion()` modifier implementation

### Skeleton Component
**File:** `cvkg-components/src/skeleton.rs`

- `Skeleton` exists
- Verify `Skeleton::new()` works with content

### Game Components
**Files:** `cvkg-components/src/game/*.rs`

- `HealthBar`, `MiniMap`, `DPadControl` ✅ (primitive views)

---

## Phase 2D: Test Coverage

### Missing Test Crates
1. **cvkg-render-3d** - No `tests/` directory
2. **cvkg-render-3d-hierarchy** - No `tests/` directory

### Action
Add basic test files:
- `tests/integration.rs` for each crate
- Test core functionality: frustum culling, transform propagation

### cvkg-gltf Extension
**Feature:** KHR_lights_punctual support
- Map glTF lights to `cvkg-render-3d::Light`
- Update `Scene3D` to include lights

---

## Phase 2E: GPU Broadphase (Optional)

### Target
`cvkg-physics/src/gpu_broadphase.rs` - GPU compute pass

### Current
- CPU fallback works correctly
- WGSL shader stub present

### Action (If Prioritized)
1. Implement spatial hash compute shader
2. Add pair generation compute shader
3. Integrate readback in `GpuBroadphase::execute_gpu`
4. Benchmark vs CPU fallback

---

## Dependencies

| Task | Depends On |
|------|------------|
| 2A Step 1 | types.rs modification |
| 2A Step 2 | draw.rs VDOM rendering changes |
| 2A Step 3 | Panel ID tagging working |
| 2B | None |
| 2C | None |
| 2D | Test infrastructure |

---

## Timeline Estimate

| Phase | Effort | Priority |
|-------|--------|----------|
| 2A - Panel Filtering | 1-2 weeks | High |
| 2B - Software Renderer Docs | 2-3 days | Medium |
| 2C - Component Verification | 1 week | Medium |
| 2D - Test Coverage | 1 week | High |
| 2E - GPU Broadphase | 2-3 weeks | Low (optional) |

---

## Success Criteria

1. **WorldSpacePanel Isolation:** Multiple panels render different content without bleeding
2. **Software Renderer:** Clear documentation of capabilities/limitations
3. **Components:** All landing/game components verified working
4. **Tests:** New crates have basic test coverage
5. **No Regressions:** Desktop/web rendering unchanged

---

## References

- `feedback.md` - Full audit findings
- `cvkg-render-gpu/src/passes/pre_world_panel.rs:80` - Primary TODO
- `cvkg-render-gpu/src/types.rs` - DrawCall definition
- `cvkg-render-gpu/src/renderer/draw.rs` - VDOM rendering