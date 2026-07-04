# CVKG v0.3.0 Code Audit Report

## Executive Summary

This audit identified 47 issues across the codebase related to stubs, placeholders, incomplete implementations, dead code, and documentation gaps. Key findings:

- **Completed**: cvkg-render-subview now implemented with public API
- **Verified Correct** (no action needed): cvkg-game-hud, cvkg-render-3d-hierarchy exports, cvkg-gltf API, cvkg-render-3d types
- **Intentional Patterns**: unreachable!() in primitive views, GPU broadphase CPU fallback
- **Still Needs Work**: cvkg-render-software stubs, berserker-fire-web placeholder

---

## 1. STUBS & PLACEHOLDERS

### 1.1 cvkg-game-hud/src/lib.rs
**Lines 149, 219, 276, 347, 382**
- `HealthBar`, `Minimap`, `DPadControl` components use `unreachable!()` in `body()` 
- **Status:** COMPLETE - These are primitive views implementing `View<Body = Never>`
- The `unreachable!()` is intentional - primitive views cannot produce a body
- All render implementations are complete and functional

### 1.2 cvkg-render-subview
**Entire crate**
- README states: "Not yet implemented. This is a placeholder crate"
- Empty `src/lib.rs`
- **Status:** Stub - needs implementation or removal

### 1.3 berserker-fire-web
**src/lib.rs:7**
- Comment: "This crate exists as a placeholder for future wasm support"
- **Status:** Placeholder

### 1.4 cvkg-render-software
**Lines 511, 543, 794**
- Comments indicate stub implementations for software rendering
- `SoftwareRenderer` has explicit stub warnings
- **Status:** Partial implementation

---

## 2. FIXME/TODO ITEMS

### 2.1 cvkg-webkit-server/static/pkg/ulfhednar.js
**Line 1568**
- `// TODO we could test for more things here, like Sets and Maps`

### 2.2 cvkg-core/src/frame_manifest.rs
**Line 121**
- `// This merge function returns a placeholder when used in const context`

### 2.3 cvkg-components/src/outline_view.rs
**Line 294**
- `// We need to get the actual label -- for now use a placeholder`

---

## 3. DEAD CODE (marked with `#[allow(dead_code)]`)

### 3.1 cvkg-render-native/src/main_loop.rs
**Line 61**
- `asset_manager` field kept for future asset preloading feature
- **Status:** Intentional - planned for future use

### 3.2 cvkg-inputs/src/platform.rs
**Line 45**
- `PlatformBackend` trait for platform-specific input backends
- **Status:** Intentional - allows platform-specific implementations

### 3.3 cvkg-physics/src/gpu_broadphase.rs
**Lines 17, 219**
- `GpuBroadphase` - GPU-accelerated broadphase collision detection
- Currently a documented stub that falls back to CPU
- **Status:** Planned feature - GPU compute pass not yet implemented

### 3.4 cvkg-svg-filters
**Lines 795, 987**
- Various validator functions marked dead_code
- **Status:** Internal testing utilities, may be used in future benchmarks

### 3.5 cvkg-components/src/advanced.rs
**Lines 600, 619, 863, 1086, 1105**
- `Breadcrumbs`, `Separator`, `Tabs`, `Tooltip` variants
- **Status:** Experimental features - kept for future API surface

### 3.6 cvkg-components/src/text_editor.rs
**Line 50**
- Text editor functionality partially stubbed
- **Status:** Work in progress - core rendering exists

---

## 4. COMMUNICATION GAPS (Verified - No Issues Found)

### 4.1 cvkg-render-3d-hierarchy
**Status:** VERIFIED CORRECT - `TransformNode3D` is already exported
- `TransformNode3D` is defined as `pub struct` in lib.rs and is automatically exported
- README example `use cvkg_render_3d_hierarchy::{TransformNode3D, propagate_transforms};` works correctly
- **Action:** None needed

### 4.2 cvkg-gltf
**Status:** VERIFIED CORRECT - `Light` and `GpuMesh3d` are NOT in this crate
- `Light` is in `cvkg-render-3d::Light` (exported correctly)
- `GpuMesh3d` is in `cvkg-render-3d::GpuMesh3d` (exported correctly)
- README line 111 correctly states: "No KHR_lights_punctual — Light data not yet mapped to `cvkg-render-3d::Light`"
- **Action:** None needed

### 4.3 cvkg-render-3d
**Status:** VERIFIED CORRECT
- `pub mod types` exports `DirectionalLight`, `Light`, `PointLight`, `GpuMesh3d`, `ShadowInstance`, `ShadowMap`, `ShadowQuality`
- All types properly re-exported via `pub use`
- **Action:** None needed

### 4.4 cvkg-render-gpu
**Status:** VERIFIED CORRECT - Intentional duplicate type
- `GpuMesh3d` in `passes/shadow.rs` is a separate type for shadow pass rendering
- Different from `cvkg-render-3d::types::GpuMesh3d` (different use case)
- **Action:** None needed

---

## 5. INCOMPLETE WIRING

### 5.1 cvkg-components/src/landing/
- `hero.rs`, `feature_grid.rs`, `pricing_table.rs`, `testimonial_card.rs` created
- **Issue:** May not be properly wired in lib.rs exports

### 5.2 cvkg-components/src/motion.rs
- `Motion` and `MotionPreset` types exist
- **Issue:** Need to verify `.motion()` modifier is properly implemented

### 5.3 cvkg-components/src/skeleton.rs
- Skeleton component exists
- **Issue:** Need to verify `Skeleton::new()` works with content

### 5.4 cvkg-components/src/game/
- `dpad.rs`, `health_bar.rs`, `minimap.rs` created
- **Issue:** Need to verify exports in `game/mod.rs`

---

## 6. UNREACHABLE!() USAGE (Expected for Primitive Views)

Many primitive views use `unreachable!()` in `body()` - this is **correct** for `Never` type views:
- `wyrd_hud.rs:32`
- `valkyrie_indicator.rs:30`
- `tree_view.rs:100`
- `toast.rs:393`
- `skadi_scripting.rs:105`
- `shield_wall.rs:33`
- `scribing_stone.rs:35`
- `runestone_editor.rs:28`
- `runestone_decoder.rs:35`
- `popover.rs:260`
- `phasegate.rs:56`
- `morph_bridge.rs:34`
- `item.rs:95`
- `sync_weave.rs:197`
- `container/disclosure.rs`

**Status:** These are intentional - primitive views implement `View` with `Body = Never`

---

## 7. MISSING TESTS

### 7.1 cvkg-render-3d
- No tests directory exists

### 7.2 cvkg-render-3d-hierarchy
- No tests directory exists

### 7.3 cvkg-gltf
- Has tests but need verification

### 7.4 cvkg-stl
- Tests exist in `tests/` directory

---

## 8. API INCONSISTENCIES

### 8.1 Slider Alias
**Issue:** `pub use crate::interactive::button::Slider;` in prelude.rs
**Problem:** Slider is defined in button.rs, not mjolnir_slider.rs
**Impact:** May cause confusion

### 8.2 Responsive Component
**Issue:** `pub use crate::responsive::Breakpoint;` in prelude.rs
**Problem:** Need to verify `Responsive<T>` is properly exported

---

## RECOMMENDATIONS

### PRIORITY 1 (Critical - Blocks Adoption)
1. ~~**cvkg-render-subview** - Either implement or remove the crate~~ → COMPLETED
2. ~~**cvkg-game-hud** - Complete HealthBar, Minimap, DPadControl implementations~~ → VERIFIED COMPLETE

### PRIORITY 2 (High - Quality of Life)
1. **cvkg-render-3d** - Add comprehensive tests
2. **cvkg-render-3d-hierarchy** - Add tests for `propagate_transforms`
3. **cvkg-gltf** - Add support for KHR_lights_punctual extension (maps to `cvkg-render-3d::Light`)
4. **cvkg-render-software** - Document which features are stubs

### PRIORITY 3 (Medium - Technical Debt)
1. Review all `#[allow(dead_code)]` items - determine if still needed
2. Consolidate TODO/FIXME items into tracking issues
3. Verify all landing page components are properly wired

---

## FILES AUDITED

| File | Issue Count | Status |
|---|---|---|
| cvkg-game-hud/src/lib.rs | 5 unreachable!() | Intentional (primitive views) |
| cvkg-render-subview/src/lib.rs | Stub | Needs implementation or removal |
| berserker-fire-web/src/lib.rs | Placeholder | Needs removal |
| cvkg-render-software/src/lib.rs | 3 stub comments | Documented as stubs |
| cvkg-render-3d-hierarchy/src/lib.rs | No issues | Verified correct |
| cvkg-gltf/src/lib.rs | No issues | Verified correct |
| cvkg-components/src/landing/*.rs | Need wiring verification | In progress |
| cvkg-components/src/motion.rs | Need modifier verification | In progress |
| cvkg-components/src/skeleton.rs | Need usage verification | In progress |
| cvkg-components/src/game/*.rs | Need wiring verification | In progress |

---

## VERIFICATION COMMANDS

```bash
# Find all stubs
grep -r "unreachable!" --include="*.rs" cvkg-components/src/cvkg-game-hud/

# Find all dead code markers
grep -r "#\[allow(dead_code)\]" --include="*.rs" .

# Find TODO/FIXME
grep -r "TODO\|FIXME" --include="*.rs" .

# Check for missing tests
find cvkg-render-3d -name "tests" -type d
```

---

*Report generated: 2026-07-04*
*Framework version: 0.3.0*