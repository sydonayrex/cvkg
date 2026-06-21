# cvkg-layout Audit Report

Crate: `cvkg-layout` v0.2.13  (not `cvkg-linear` — the task name was slightly wrong)
Path: `/D/rex/projects/cvkg/cvkg-layout/src/lib.rs` (2,810 lines, 1 source file)
Framework: Thin wrapper around **Taffy** v0.6 (flexbox/grid solver)

Audit scope: layout math overflow, f32 NaN propagation, solver convergence, unwrap/unsafe, theming.

---

## 1. CRITICAL BUG: AspectRatio Y-centering broken

**File:** `lib.rs:1428`

```rust
let x = bounds.x + (bounds.width - fit.width) * 0.5;
let y = bounds.y + (bounds.height - fit.height) * 0.0;  // ← BUG: * 0.0 instead of * 0.5
```

The Y-centering multiplier is `0.0`, so `y` always equals `bounds.y`. The child is **never vertically centered**. This is clearly a copy-paste error where `* 0.0` was written instead of `* 0.5`. The X-centering on line 1427 is correct.

**Impact:** All `AspectRatio` views place their child at the top of the container instead of centered vertically.

**Fix:** Change `* 0.0` to `* 0.5`.

---

## 2. f32 NaN Propagation — Zero Sanitization (HIGH)

The entire crate has **zero** `is_nan()`, `is_infinite()`, or `is_finite()` checks. Every `f32` field (spacing, bounds, sizes, aspect ratios, spring velocities, scale factors, timesteps) can propagate NaN without detection.

| Location | Expression | What happens with NaN |
|----------|-----------|----------------------|
| `intrinsic_flex_size` (L275-284) | `f32::sum()` over child sizes | NaN propagates → garbage intrinsic size |
| `AspectRatio::fitted_size` (L1365-1378) | `w / ratio`, `max_h * ratio` | NaN if ratio is NaN (only clamped in constructor, not runtime) |
| `AspectRatio::size_that_fits` (L1405-1413) | `child_size.width / height`, `child_w / intrinsic_ratio` | Division by zero if width=0 → ±inf or NaN |
| Layout animation delta (L449-452) | `(prev.x - target_rect.x).abs()` | NaN → comparison `> epsilon` is always false → transition silently skipped |
| Spring eviction (L179) | `velocity.length_sq() > 0.0001` | NaN → false → transition never evicted (leak) |
| Snap function (L486) | `(v * scale).round() / scale` | NaN propagates through all operations |

**Recommendation:** Add `v.is_finite()` guards on all values entering the solver, and sanitize with `v.max(0.0)` / `v.clamp(min, max)` where negative values don't make sense. At minimum, add a `debug_assert!(v.is_finite())` at key entry points.

---

## 3. u64 Underflow Risk (Potential Debug-Mode Panic)

**File:** `lib.rs:178` (also `cvkg-core/src/lib.rs:4975` — same pattern in two places)

```rust
current_gen - *g < threshold    // both u64
```

If `*g > current_gen`, this **panics in debug mode** (u64 overflow check) or **wraps in release mode** (wrapping_sub), producing a massive `u64` value that makes the comparison always false.

In `cvkg-layout::AnimationEngine::evict_stale_transitions` (L178), `current_gen` is `self.eviction_generation` which increments by 1 each frame. `*g` is a snapshot from `transition_generation`. Under normal operation `current_gen >= *g`, but if a transition persists for 2^64 eviction cycles (~10^19 frames at 60fps), the counter wraps around. In release mode, after 18 quintillion frames, the subtraction wraps and the condition becomes false — causing all transitions to be retained forever. In practice, a process running for decades might hit this.

**Also:** The same pattern in `cvkg-core/src/lib.rs:4975` (`LayoutCache::evict_stale_entries`).

**Recommendation:** Use `current_gen.wrapping_sub(*g) < threshold` or `current_gen.saturating_sub(*g) < threshold` to explicitly handle the wrap case.

---

## 4. Flex::place_subviews — Negative Item Size from Spacing Overflow

**File:** `lib.rs:878-879, 903-905`

```rust
let total_spacing = self.spacing * (n - 1.0);
let item_width = (bounds.width - total_spacing) / n;   // can be negative
```

If `self.spacing > bounds.width` (or even just large enough that `total_spacing > bounds.width`), `item_width` becomes **negative**. This propagates to child rects, potentially producing nonsense layout or downstream NaN/inf from division.

Same issue on L903-905 for vertical orientation.

**Recommendation:** Guard with `.max(0.0)`:
```rust
let item_width = ((bounds.width - total_spacing) / n).max(0.0);
```

---

## 5. Unwrap Panic Vectors — 16 Unchecked Unwrap Calls

| # | Location | Call | Risk |
|---|----------|------|------|
| 1-2 | `TaffyLayoutEngine::get_or_insert_engine` (L124-126) | `cache.engine.as_mut().unwrap().downcast_mut::<TaffyLayoutEngine>().unwrap()` | Wrong type in `LayoutCache::engine` → panic |
| 3-4 | `AnimationEngine::get_or_insert_engine` (L164-166) | Same double-unwrap pattern | Wrong type in `LayoutCache::animators` → panic |
| 5-12 | Flex/Grid tree ops (L359,364,403,411,417,421,1062,1067,1096,1104,1110,1114) | `new_leaf().unwrap()`, `new_with_children().unwrap()`, `compute_layout().unwrap()`, `layout().unwrap()` | Taffy returns errors for invalid node IDs, tree corruption, or unsolvable constraints |

`get_or_insert_engine` is called frequently (once per flex/grid layout pass). A bug in `LayoutCache` initialization (e.g., setting engine to the wrong downcast type, or engine being unexpectedly `None`) causes a **hard panic** with no recovery path.

**Recommendation:** Use `.unwrap_or_else(|| panic!(...))` with informative messages, or better, return `Result` from the public API. At minimum add `expect()` with context.

---

## 6. Solver Convergence — No Batch Fallback in Taffy

**File:** `lib.rs:416-417, 1109-1110`

```rust
engine.tree.compute_layout(root_node, taffy::Size::MAX_CONTENT).unwrap();
```

Taffy's solver is called with `Size::MAX_CONTENT` as the available space. If the tree contains extreme values (very large/small flex weights, negative sizes, NaN dimensions), Taffy may:
- Return `Err` with `TaffyError` → **panic** (unwrapped)
- Produce degenerate layouts that, while not erroring, contain NaN or inf positions

There is **no convergence timeout** or iteration budget — the Taffy solver runs to completion or error. The `cache.is_over_budget()` check (L298) correctly short-circuits the solver when the time budget is exceeded, but if the solver has already been invoked when the budget expires, it still runs to completion.

**Recommendation:** Apply NaN/Inf clamping on the inputs before passing to Taffy. Catch errors with `unwrap_or_else` and return fallback rects.

---

## 7. Spring Animation — No Timestep Clamping

**File:** `lib.rs:481`

```rust
spring.step(delta);
```

`delta` comes from `cache.delta_time`. If the application is paused, backgrounded, or frame interleaving causes a large gap, `delta` could be large (seconds). Without clamping, the spring integrator can overshoot significantly, producing visible jitter on resume.

**Recommendation:** Clamp `delta` to a maximum step size:
```rust
let dt = delta.min(1.0 / 30.0);  // max 30 FPS step
spring.step(dt);
```

---

## 8. int→i16/u16 Cast Overflow in Grid

**File:** `lib.rs:1038-1043`

```rust
start: taffy::prelude::line((p.column + 1) as i16),
end:   taffy::prelude::span(p.column_span as u16),
```

`GridPlacement::column` is `i32` (not `i16`). If `p.column + 1 > i16::MAX` (32767), the `as i16` cast silently truncates. Similarly `p.column_span` is `u32`, truncating to `u16` (max 65535 → wraps). These values come from user code; a malicious or buggy caller could trigger this.

**Recommendation:** Add `i16::try_from().expect(…)` or clamp to valid range.

---

## 9. Progressive Layout — Div-by-Zero in Fallback

**File:** `lib.rs:2081-2084`

```rust
let cols = (remaining.len() as f32).sqrt().ceil() as usize;
let rows = (remaining.len() + cols - 1) / cols;
let cell_w = self.bounds.width / cols as f32;   // cols > 0 guaranteed by sqrt.ceil()
let cell_h = self.bounds.height / rows as f32;  // rows > 0 guaranteed by ceiling division
```

If `remaining.len() == 0`, the early return on L2076 prevents this. If `self.bounds.width` or `self.bounds.height` is 0 or negative, `cell_w`/`cell_h` will be 0 or negative, producing zero-area or negative-area fallback rects.

**Recommendation:** Already guarded for empty case, but add `max(0.0)` on computed cell dimensions as defense-in-depth.

---

## 10. Unsafe Blocks: **None**

The entire crate uses **zero** `unsafe` blocks. The only mention of "unsafe" is in the doc license header (comment).

---

## 11. Theming: **None**

This crate is purely layout computation. No theme colors, semantic colors, or style constants.

---

## Summary

| Category | Count | Critical | High | Medium | Low |
|----------|-------|----------|------|--------|-----|
| Math overflow / negative values | 5 | 1 | 0 | 2 | 2 |
| NaN propagation | 6 | 0 | 1 | 4 | 1 |
| Solver convergence | 3 | 0 | 0 | 3 | 0 |
| Unwrap panic risk | 16 | 0 | 0 | 14 | 2 |
| Unsafe | 0 | — | — | — | — |
| Theming | 0 | — | — | — | — |

**Top priority fix:** `AspectRatio::place_subviews` Y-centering bug (`* 0.0` → `* 0.5`).
**Widespread concern:** Complete absence of NaN/Inf sanitization on f32 inputs.
**Most panic-prone:** Double-unwrap `get_or_insert_engine` and all Taffy tree operations.
**Most subtle:** u64 subtraction underflow in eviction logic (debug panic, release logic error).

All 23 existing unit tests pass. No test covers: NaN inputs, negative spacing, or the Y-centering bug.
