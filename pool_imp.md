# CVKG Implementation Plan

Generated: 2026-06-20

This document provides a detailed implementation plan for addressing the issues identified in the engineering audit (`pool_audit.md`).

---

## Issue Classification

### High Priority (Immediate Action Required)

These issues pose correctness risks or undefined behavior potential and must be addressed before the next release.

### Medium Priority (Short-term)

These issues may cause panics in edge cases or degrade user experience but have acceptable workarounds.

### Low Priority (Long-term)

These are code quality improvements that do not affect correctness.

---

## Phase 1: Critical Safety Fixes

### Issue 1: Unsafe Arc::from_raw in cvkg-core/src/lib.rs:3678

**Declaration:** This is an issue because `unsafe { Arc::from_raw(...) }` is undefined behavior if the pointer was not originally created by `Arc::into_raw` with the same type, or if the reference count is incorrect. The current code depends on a prior `downcast_ref` check for correctness, but the safety invariant is not explicitly documented, making future maintenance error-prone.

**Disposition:** `cvkg-core/src/lib.rs:3678`

**Example Fix:**
```rust
// BEFORE (problematic - implicit invariant)
let ptr = Arc::into_raw(Arc::new(value));
// ... later ...
let value_ref = unsafe { Arc::from_raw(ptr) };  // If downcast_ref was skipped, UB!

// AFTER (with explicit safety contract)
/// # Safety Invariant
/// The pointer MUST be derived from `Arc::into_raw` and the Arc must still be alive.
/// This is guaranteed by the preceding `downcast_ref` check which validates
/// the underlying type, and by `EVENT_STATE` being a static that outlives the program.
let ptr = Arc::into_raw(Arc::new(value));
// SAFETY: downcast_ref verified the type; EVENT_STATE holds the Arc for 'static lifetime
let value_ref = unsafe { Arc::from_raw(ptr) };
```

**Special Concerns:**
- Any change to the downcast logic must be reviewed for safety implications
- Consider creating a wrapper type `TrackedArc<T>` that encapsulates this pattern
- The static `EVENT_STATE` must remain 'static; refactoring to non-static would break this

---

### Issue 2: Unsafe Send/Sync for SurtrRenderer in cvkg-render-gpu/src/renderer.rs:423-426

**Declaration:** This is an issue because implementing `Send` and `Sync` for a type is unsafe when the type contains thread-unsafe fields. The current implementation is justified for WASM's single-threaded environment, but if the codebase ever adds web worker support, this would become a data race vulnerability. The invariant is implicit and not enforced at compile time.

**Disposition:** `cvkg-render-gpu/src/renderer.rs:423-426`

**Example Fix:**
```rust
// BEFORE
unsafe impl Send for SurtrRenderer {}
unsafe impl Sync for SurtrRenderer {}

// AFTER (compile-time assertion for single-threaded WASM)
#[cfg(target_arch = "wasm32")]
const _: () = {
    // SAFETY: WASM is single-threaded by default; these impls are safe because
    // wgpu types in WASM target don't require additional synchronization.
    // If multi-threading is ever added via web workers, this code will panic
    // at compile time, forcing a review of the Send/Sync safety.
    unsafe impl Send for SurtrRenderer {}
    unsafe impl Sync for SurtrRenderer {}
};

#[cfg(not(target_arch = "wasm32"))]
compile_error!(
    "SurtrRenderer Send/Sync must be verified for multi-threaded targets. \
     See pool_audit.md for details on the safety requirements."
);
```

**Special Concerns:**
- Native multi-threaded support would require re-auditing wgpu state management
- Consider using `wasm-bindgen-rayon` for explicit parallel execution if needed

---

## Phase 2: Panic Prevention (Medium Priority)

### Issue 3: partial_cmp().unwrap() in cvkg-core/src/lib.rs:3703

**Declaration:** This is an issue because `partial_cmp()` returns `None` when comparing floating-point values that are NaN or infinity. Using `.unwrap()` on this result causes a panic. While the input values come from pathological query relevance scores (rare in practice), a malicious or buggy input could trigger this.

**Disposition:** `cvkg-core/src/lib.rs:3703`

**Example Fix:**
```rust
// BEFORE
results.sort_by(|a, b| {
    b.0.partial_cmp(&a.0).unwrap()
});

// AFTER
results.sort_by(|a, b| {
    match b.0.partial_cmp(&a.0) {
        Some(ordering) => ordering,
        None => {
            // Handle NaN/∞: treat them as equal (they'll cluster together)
            std::cmp::Ordering::Equal
        }
    }
});
```

**Alternative (more explicit):**
```rust
results.sort_by(|a, b| {
    b.0.partial_cmp(&a.0)
        .unwrap_or_else(|| {
            // Log the NaN occurrence for debugging
            eprintln!("Warning: NaN/∞ relevance score encountered in query sort");
            std::cmp::Ordering::Equal
        })
});
```

---

### Issue 4: partial_cmp().unwrap() in cvkg-runic-text/src/lib.rs:381

**Declaration:** Same issue as Issue 3. If polygon intersection calculations produce NaN values (possible with degenerate geometry), the sort will panic.

**Disposition:** `cvkg-runic-text/src/lib.rs:381`

**Example Fix:**
```rust
// BEFORE
intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

// AFTER
intersections.sort_by(|a, b| {
    match a.partial_cmp(b) {
        Some(ordering) => ordering,
        None => {
            // NaN values: preserve original order using their index
            std::cmp::Ordering::Equal
        }
    }
});
```

---

### Issue 5: Mutex Poison Handling in cvkg-core/src/lib.rs:1210, 1228, etc.

**Declaration:** This is an issue because `MutexGuard::lock().unwrap()` panics when the mutex is poisoned, which happens when another thread panicked while holding the lock. In a UI framework, this could crash the entire application on a recoverable error.

**Disposition:** `cvkg-core/src/lib.rs:1210, 1228, 3482, 3529, 3565, 3582, 3602, 3623`

**Example Fix:**
```rust
// BEFORE
let value = stored.read().unwrap();

// AFTER
let value = stored.read().ok()?;  // Gracefully degrade to None if poisoned
// OR, if the operation must continue:
match stored.read() {
    Ok(guard) => { /* use guard */ },
    Err(poisoned) => {
        log::warn!("State mutex poisoned, recovering with stale data");
        // Continue with the poisoned guard's inner value (may be inconsistent but won't crash)
        let guard = poisoned.into_inner();
        // use guard
    }
}
```

---

### Issue 6: Off-by-one in berserker_fire_demo.rs:74-78

**Declaration:** This is an issue because array indexing without bounds checking can cause a panic. The `pts[segment_idx + 1]` access at line 78 can index `pts[4]` when the array only has 4 elements (indices 0-3), if `t` equals exactly 3.0.

**Disposition:** `cvkg/examples/berserker_fire_demo.rs:74-78`

**Example Fix:**
```rust
// BEFORE (unsafe)
fn get_triangle_point(pts: &[[f32; 2]; 4], mut t: f32) -> [f32; 2] {
    let total_len = 3.0f32;
    while t < 0.0 { t += total_len; }
    t = t % total_len;
    let segment_idx = t.floor() as usize;
    let p_start = pts[segment_idx];
    let p_end = pts[segment_idx + 1];  // PANICS if segment_idx == 3
    // ...
}

// AFTER (safe)
fn get_triangle_point(pts: &[[f32; 2]; 4], mut t: f32) -> [f32; 2] {
    let total_len = 3.0f32;
    t = t % total_len;
    if t < 0.0 { t += total_len; }
    
    // Clamp segment index to valid range [0, 3)
    let segment_idx = ((t / total_len) * 3.0).floor() as usize % 3;
    let p_start = pts[segment_idx];
    let p_end = pts[(segment_idx + 1) % pts.len()];  // Safe modulo
    // ...
}
```

---

### Issue 7: vdom_id hash always returns 0 in cvkg-macros/src/lib.rs:285

**Declaration:** This is an issue because `DefaultHasher::new().finish()` with no data written returns 0, making all instances of `cvkg_model!` share the same vdom_id. This breaks VDOM diffing which relies on unique IDs for efficient updates.

**Disposition:** `cvkg-macros/src/lib.rs:285`

**Example Fix:**
```rust
// BEFORE (always returns 0)
let mut hasher = std::collections::hash_map::DefaultHasher::new();
format!("{}_{}", stringify!(#name), hasher.finish())

// AFTER (hashes struct fields)
impl #name {
    fn vdom_id(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut s);  // Hash the struct's fields
        format!("{}_{}", stringify!(#name), s.finish())
    }
}
```

**Special Concerns:**
- The `Hash` derive must be added to all structs using `cvkg_model!`
- Or use a manual hash implementation that includes field values

---

## Phase 3: File Decomposition

### Issue 8: cvkg-core/src/lib.rs (9557 lines)

**Declaration:** This is an issue because the file mixes multiple unrelated responsibilities (views, modifiers, state, animation) making it hard to navigate, review, and maintain. Large files are a maintenance burden.

**Disposition:** Split into logical modules:

| New File | Content | Responsibility |
|----------|---------|----------------|
| `cvkg-core/src/view/mod.rs` | View trait, ViewModifier trait, ModifiedView | Core view abstraction |
| `cvkg-core/src/modifiers/fx.rs` | BifrostModifier, GungnirModifier, MjolnirSliceModifier, etc. | Visual effect modifiers |
| `cvkg-core/src/renderer/renderer_trait.rs` | Renderer trait and sub-traits | Drawing interface |
| `cvkg-core/src/state/knowledge.rs` | KnowledgeState, KnowledgeFragment, TemporalNode | Agentic memory system |
| `cvkg-core/src/state/bindings.rs` | State<T>, Binding<T> | Reactive state management |
| `cvkg-core/src/anim/spring.rs` | SleipnirParams, SleipnirSolver, SpringConfig | Physics animation system |

**Special Concerns:**
- Static `SYSTEM_STATE` requires careful refactoring to avoid breaking module initialization order
- Public API must remain unchanged; use `pub use` re-exports in lib.rs

---

### Issue 9: cvkg-render-gpu/src/renderer.rs (6637 lines)

**Declaration:** Same concern as Issue 8. The SurtrRenderer mixes pipeline setup, frame management, draw submission, and buffer management.

**Disposition:** Split into:

| New File | Content | Responsibility |
|----------|---------|----------------|
| `renderer/mod.rs` | SurtrRenderer struct, public API | Entry point |
| `renderer/pipelines.rs` | Pipeline creation and management | GPU pipeline setup |
| `renderer/frame.rs` | begin_frame, end_frame, render_frame | Frame lifecycle |
| `renderer/draw_calls.rs` | DrawCall, vertices, indices collection | Draw submission |
| `renderer/particles.rs` | Particle buffer management | Particle system |
| `renderer/buffers.rs` | GPU buffer creation and updates | Buffer management |

---

## Phase 4: Naming Improvements

These are low-priority but improve code maintainability and onboarding.

| Old Name | New Name | Declaration |
|----------|----------|-------------|
| Muspelheim | `shape_pipeline` | Pipeline label |
| Surtr | `gpu_renderer` | Renderer name |
| BerserkerMode | `RageState` | Enum |
| BifrostModifier | `GlassModifier` | Frosted glass effect |
| GungnirModifier | `GlowModifier` | Neon glow effect |
| MjolnirSliceModifier | `SliceModifier` | Geometric slice effect |
| SleipnirSolver | `SpringSolver` | Spring physics solver |
| Tyr | `PhysicsEngine` | Physics engine name |

---

## Resolution Prioritization

| Priority | Phase | Issues | Timeline |
|----------|-------|--------|----------|
| 1 (Immediate) | Phase 1 | Issues 1, 2 | Before next release |
| 2 (Short-term) | Phase 2 | Issues 3, 4, 5, 6, 7 | Within 2 weeks |
| 3 (Medium-term) | Phase 3 | Issues 8, 9 | Within 1 month |
| 4 (Long-term) | Phase 4 | Issues 10+ (naming) | As convenient |

---

## Special Concerns Summary

### Undefined Behavior Risks
- **Issue 1**: The `Arc::from_raw` safety depends on global static state lifetime
- **Issue 2**: WASM single-threading assumption could be violated by future features

### Error Recovery
- **Issue 5**: Mutex poisoning handling requires distinguishing between "acceptable" and "critical" state corruption
- Consider adding a `StateRecovery` trait for customizable recovery behavior

### Breaking Changes to Avoid
- File decomposition must preserve public API via `pub use` re-exports
- Naming changes should be done via `#[deprecated]` attributes first, then rename in next major version

### Testing Requirements
- Add unit tests for `get_triangle_point` with edge cases (t = 0.0, 3.0, -3.0, NaN)
- Add tests for query sorting with NaN/infinity values
- Add tests for partial_cmp fallback behavior in text layout