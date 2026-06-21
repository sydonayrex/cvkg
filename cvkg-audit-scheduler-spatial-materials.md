# CVKG Infrastructure Crate Audit Report

**Audited crates:** cvkg-scheduler, cvkg-spatial, cvkg-materials  
**Date:** 2026-06-20  
**Scope:** Bugs, security, theming, unwrap/unsafe analysis  
**Total tests run:** 67/67 pass  

---

## 1. cvkg-scheduler (frame.rs + task.rs — 781 LOC)

### 1.1 Unsafe/Send/Sync Analysis

- **Unsafe blocks: 0** in production code. Zero actual `unsafe` usage.
- **No manual Send/Sync impls.** All auto-derived. `TaskScheduler` and `FrameScheduler` take `&mut self` for all operations, making them naturally single-threaded-bound. This is correct — they are designed for per-frame single-threaded use.
- `Box<dyn FnOnce() + Send>` in `Task` and `PhaseEntry` correctly propagates `Send`. If a `FrameScheduler` were wrapped in `Mutex`, it could be safely sent between threads.
- `KvasirId::new()` uses `Ordering::Relaxed` atomic fetch_add — correct for uniqueness-only semantics.

### 1.2 Unwrap Analysis

- **`unwrap()` calls: 0** in production code.
- **`expect()` calls: 0** in production code.
- All 24 `.unwrap()` hits are exclusively in `#[cfg(test)]` blocks (Mutex::lock().unwrap() in test assertions). **No action needed.**

### 1.3 Race Conditions

- **No channels** used — the task specifically asked about "unwrap on channel operations." There are none.
- All public methods require `&mut self`, preventing concurrent access at compile time.
- **No race windows** in the scheduler logic.
- `flush_current_phase` drains matching PhaseEntry items, routes them through a temporary TaskScheduler, then flushes the inner scheduler. No re-entrancy issue since all takes are `&mut self`.

### 1.4 Panic/Error Handling

- **Documented panic risk:** If a task closure panics inside `run_all()` or `run_priority()`, subsequent tasks in the same batch are dropped without executing. The `run_all()` doc explicitly warns: "If a task panics, subsequent tasks in the drained batch are still lost." This is an accepted design trade-off (callers should use `catch_unwind` if they need resilience).
- **`assert!` in production code:** None in task.rs or frame.rs beyond the module header.

### 1.5 Task Stealing

- **No task stealing implementation.** The scheduler is a straightforward priority-sorted Vec with drain-on-execute. The task specifically asked about "unsafe in task stealing" — there is no work-stealing in this crate.

### 1.6 Bugs

- **None found.** The scheduler is clean, well-commented, and correct for its documented design.

### 1.7 Theming

- **N/A.** Scheduler has no color/theming concerns.

---

## 2. cvkg-spatial (spatial_hash.rs + bvh.rs + quadtree.rs — 829 LOC)

### 2.1 Unsafe/Send/Sync Analysis

- **Unsafe blocks: 0** in production code.
- **No manual Send/Sync impls.** All auto-derived. `SpatialHash`, `Bvh`, and `Quadtree` all use `&mut self` for mutations, with `&self` for queries. This is correct single-threaded design.
- `Query` methods (e.g., `Bvh::query() -> Vec<&T>`) return references tied to `&self` lifetime, so the borrow checker prevents mutation during query. Correct.

### 2.2 Unwrap Analysis

- **`unwrap()` calls in production code: 0**.
- **`SerialHash::new()` returns `Option<Self>`** (returns `None` if `cell_size <= 0.0`). All four test sites use `.unwrap()`, which is fine in tests. Production callers must handle the `None` case.
- **BVH `build_recursive()`:** Contains `assert!(!indices.is_empty())` which panics in debug+release if violated. This is guarded by the recursion logic (never called with empty), but is a runtime panic surface.

### 2.3 Bugs

#### BUG-01: BVH `partial_cmp` on NaN centroids (low severity)

In `bvh.rs:162`, centroid floats are compared with `partial_cmp` and fall back to `Ordering::Equal` on NaN:
```rust
ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
```
If any `Rect` contains NaN x/y/width/height (not prevented by `Rect`'s constructor), the centroid computation produces NaN, `partial_cmp` returns `None`, and the sort uses `Ordering::Equal` for all NaNs. This won't crash, but will produce an arbitrary tree structure. Since `Rect` doesn't validate inputs, this is a latent robustness hole.

**Fix:** Add `f32::is_nan()` check or use `total_cmp` on the floats instead:
```rust
ca.total_cmp(&cb)
```
`f32::total_cmp` handles NaN deterministically (NaN sorts above all other values).

#### BUG-02: SpatialHash `cells_for_rect` unbounded cell range (low severity / DoS surface)

In `spatial_hash.rs:132-146`, the cell range computation clamps to `i32::MIN..i32::MAX` but doesn't limit the *span*. With pathological rect values (e.g., `x = -1e9`, `width = 2e9`), the loop in `insert()` and `query()` iterates over ~4 billion cells, causing CPU exhaustion.

**Fix:** Add a max-cell-span limit (e.g., `1000` cells in each dimension) and document the constraint:
```rust
const MAX_CELL_SPAN: i32 = 1000;
let min_cx = /* existing */;
let max_cx = /* existing */;
let max_cx = max_cx.min(min_cx + MAX_CELL_SPAN);
```
Same for y. Or at minimum document the unbounded behavior and let callers pre-clamp their rects.

#### BUG-03: Quadtree `subdivide` floating point precision with very small widths (low severity)

In `quadtree.rs:112-113`, half-extents `hw = bounds.width / 2.0` and `hh = bounds.height / 2.0`. For extremely small widths (e.g., subnormal f32), repeated subdivision could cause the quadrants to become zero-width, leading to infinite loops or panics in `insert()`. However, `max_depth = 5` bounds this to at most 2^5 = 32 subdivisions, making this theoretical. Not a practical issue.

### 2.4 API Foot-Guns

- **`Quadtree::retrieve()` appends to the output vec without clearing it.** Callers must clear before calling or they get accumulated results. This is documented but easy to misuse.
- **`SpatialHash::query()` returns duplicates (items overlapping multiple cells appear multiple times).** Documented, but callers must deduplicate.

### 2.5 Theming

- **N/A.** Spatial indexing has no theming concerns.

---

## 3. cvkg-materials (glass.rs + mica.rs + acrylic.rs + elevation.rs — 811 LOC)

### 3.1 Unsafe/Send/Sync Analysis

- **Unsafe blocks: 0** in production code.
- All types are `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]` — pure data structs with no thread-safety concerns.
- No Send/Sync manual impls needed or present.
- **Cleanest crate of the three.**

### 3.2 Unwrap Analysis

- **`unwrap()` calls in production code: 0**.
- **`expect()` calls in production code: 0**.
- Four `.unwrap()` calls in `elevation.rs` tests (for `shadow()` on non-Level0 levels) — safe because the test arrays exclude Level0.

### 3.3 Bugs

#### BUG-04: Elevation shadow test uses `.unwrap()` on non-zero levels (cosmetic)

In `elevation.rs:185,208,266,281`, `.unwrap()` is called on `ElevationLevel::shadow()`. Since the test code only iterates Level1-Level5 (which always return `Some`), this is safe. But if someone copies this pattern into production code with Level0, it would panic.

**Fix:** Not a bug in the crate per se, but a documentation note: callers should check `shadow()` for `None` on `Level0`.

### 3.4 Bounds Checking

- `GlassMaterial`, `AcrylicMaterial`, `MicaMaterial` use `.clamp(0.0, 1.0)` or `.max(0.0)` for all builder setters — good.
- `MicaMaterial::with_luminosity()` only clamps at 0.0 (no upper bound) — intentional to allow HDR values >1.0. Documented.
- `GlassMaterial::with_tint()` does NOT clamp — intentional for HDR values. Documented.

### 3.5 Theming

- These types are **the data models that a theming system would populate**. They don't contain theming logic themselves, which is the correct separation of concerns.
- `ElevationLevel` maps discrete levels (0-5) to well-defined shadow parameters derived from Material Design 3 / Fluent Design — good design.
- `GlassMaterial`, `AcrylicMaterial`, `MicaMaterial` each have sensible defaults that match their respective design language references.

### 3.6 Default Values Consistency

- `GlassMaterial` default: `tint = [1.0, 1.0, 1.0, 0.0]` (fully transparent). The doc says this is "fully transparent tint by default" to match `GlassNodeMaterial` from `cvkg-flow`. Transparent white by default might surprise callers expecting a visible tint, but it's consistent with the design goal of providing raw material parameters without OKLCH color.
- `MicaMaterial` default: `tint = [1.0, 1.0, 1.0, 1.0]` (opaque white overlay at 50% opacity). Reasonable.
- `AcrylicMaterial` default: `tint = [1.0, 1.0, 1.0, 0.6]` (semi-transparent white). Reasonable.

---

## Summary

| Category | cvkg-scheduler | cvkg-spatial | cvkg-materials |
|---|---|---|---|
| **Unsafe blocks** | 0 | 0 | 0 |
| **unwrap() in prod** | 0 | 0 | 0 |
| **expect() in prod** | 0 | 0 | 0 |
| **Manual Send/Sync** | 0 | 0 | 0 |
| **Bugs found** | 0 | 3 (low sev.) | 0 (cosmetic) |
| **Tests passing** | 14/14 | 12/12 | 41/41 |

### Bug Fix Recommendations (Priority Order)

1. **BUG-02 (Medium):** Add max-cell-span limit in `SpatialHash::cells_for_rect` to prevent CPU-exhaustion with extreme rect values.
2. **BUG-01 (Low):** Replace `partial_cmp().unwrap_or(Equal)` with `total_cmp()` in `Bvh::build_recursive` to handle NaN centroids deterministically.
3. **BUG-03 (Low):** Consider a minimum-width guard in `Quadtree::subdivide` for subnormal float edge cases (theoretical only given max_depth=5).
4. **BUG-04 (Informational):** Document that `ElevationLevel::Level0.shadow()` returns `None` for all callers — the test pattern is fine.
