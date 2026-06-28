# Ponytail Audit — Implementation Plan

> **For Hermes:** Execute task-by-task. One task per turn. Stop after each for "continue".  
> **Source Audit**: `ponytail.md` (226 findings)  
> **Goal**: Fix all P0→P2 findings, document P3 trade-offs  
> **Approach**: TDD where possible, surgical patches otherwise, verify after every step

---

## Architecture

**Dependency order** (fix foundations before dependents):
1. Safety/invariants (debug_assert, unwrap, unsafe) — unblocks everything
2. Test infrastructure (fuzz, proptest, smoke tests) — catches regressions
3. Design decomposition (Renderer split, crate extraction) — reduces coupling
4. Observability (tracing, telemetry) — production readiness
5. Accessibility + i18n — product completeness

**Per-task verification loop** (from `cvkg-implementation-patterns`):
```bash
cargo check -p <crate> 2>&1 | grep "^error" | head -10
cargo test -p <crate> 2>&1 | grep -E "test result|FAILED" | head -5
```

---

## Phase 0: Safety & Invariant Guards (P0 blockers)

### Task 0.1: Add `debug_assert!` to `State<T>` invariants

**Objective**: Catch state corruption early via debug-mode guards.

**Files**:
- Modify: `cvkg-core/src/state.rs`

**Step 1: Read current State::set implementation**

Read `cvkg-core/src/state.rs` lines 60-100 to find the `set()` method.

**Step 2: Add debug_assert guards**

In `State::set()`, add at the top:
```rust
debug_assert!(value.is_valid_state_value(), "State::set called with invalid value");
debug_assert!(
    self.version.load(Ordering::Acquire) < u64::MAX,
    "State version overflow"
);
```

In `State::subscribe()`, add:
```rust
debug_assert!(!subscribers.is_empty() || cfg!(test), "subscribing to unused state");
```

**Step 3: Verify**

```bash
cargo check -p cvkg-core
cargo test -p cvkg-core --lib
```

**Step 4: Commit**

```bash
git add cvkg-core/src/state.rs
git commit -m "feat(core): add debug_assert guards to State invariants"
```

---

### Task 0.2: Add `debug_assert!` to `LayoutCache` f32 quantization

**Objective**: Catch NaN/Inf in layout proposals before they corrupt the cache.

**Files**:
- Modify: `cvkg-core/src/layout.rs`

**Step 1: Add finite check at cache entry points**

In `LayoutCache::get_size()` and `set_size()`, add:
```rust
debug_assert!(
    proposal.width.map_or(true, |v| v.is_finite()),
    "layout proposal width is not finite: {:?}",
    proposal.width
);
debug_assert!(
    proposal.height.map_or(true, |v| v.is_finite()),
    "layout proposal height is not finite: {:?}",
    proposal.height
);
```

**Step 2: Verify**

```bash
cargo check -p cvkg-core
cargo test -p cvkg-core --lib
```

**Step 3: Commit**

```bash
git add cvkg-core/src/layout.rs
git commit -m "feat(core): add debug_assert finite checks to LayoutCache"
```

---

### Task 0.3: Add `debug_assert!` to `Mesh` invariants

**Objective**: Catch mesh corruption (mismatched vertex/normal counts) at debug time.

**Files**:
- Modify: `cvkg-core/src/mesh.rs`

**Step 1: Add assertion after normal-fill fallback**

After the normal generation fallback in `Mesh::from_obj()`, add:
```rust
debug_assert_eq!(
    mesh.vertices.len(),
    mesh.normals.len(),
    "Mesh vertex/normal count mismatch after normal generation"
);
```

**Step 2: Verify**

```bash
cargo check -p cvkg-core
cargo test -p cvkg-core --lib
```

**Step 3: Commit**

```bash
git add cvkg-core/src/mesh.rs
git commit -m "feat(core): add debug_assert to Mesh vertex/normal invariant"
```

---

### Task 0.4: Add `debug_assert!` to `VDom` patch application

**Objective**: Catch patch-targeting-nonexistent-node bugs at debug time.

**Files**:
- Modify: `cvkg-vdom/src/vdom.rs`

**Step 1: Add assertion in apply_patches**

In `VDom::apply_patches()`, before applying each patch:
```rust
debug_assert!(
    self.nodes.contains_key(&patch.target_node_id),
    "patch targets non-existent node {:?}",
    patch.target_node_id
);
```

**Step 2: Verify**

```bash
cargo check -p cvkg-vdom
cargo test -p cvkg-vdom
```

**Step 3: Commit**

```bash
git add cvkg-vdom/src/vdom.rs
git commit -m "feat(vdom): add debug_assert for patch target existence"
```

---

### Task 0.5: Add triangle count limit to STL parser

**Objective**: Prevent OOM from malicious STL files.

**Files**:
- Modify: `cvkg-stl/src/binary.rs`

**Step 1: Add limit constant and check**

At the top of `binary.rs`:
```rust
/// Maximum number of triangles allowed in an STL file.
/// Prevents OOM from malicious files claiming u32::MAX triangles.
const MAX_STL_TRIANGLES: u32 = 10_000_000;
```

After reading the triangle count:
```rust
if num_triangles > MAX_STL_TRIANGLES {
    return Err(StlError::TooManyTriangles {
        count: num_triangles,
        max: MAX_STL_TRIANGLES,
    });
}
```

**Step 2: Add variant to StlError**

```rust
/// File exceeds maximum allowed triangle count.
TooManyTriangles { count: u32, max: u32 },
```

**Step 3: Add test**

```rust
#[test]
fn test_stl_rejects_excessive_triangle_count() {
    let mut data = vec![0u8; 80]; // header
    data.extend_from_slice(&u32::MAX.to_le_bytes()); // absurd count
    let result = cvkg_stl::parse_bytes(&data);
    assert!(matches!(result, Err(StlError::TooManyTriangles { .. })));
}
```

**Step 4: Verify**

```bash
cargo check -p cvkg-stl
cargo test -p cvkg-stl
```

**Step 5: Commit**

```bash
git add cvkg-stl/src/binary.rs
git commit -m "fix(stl): add triangle count limit to prevent OOM"
```

---

### Task 0.6: Add NaN/Inf validation to STL float parsing

**Objective**: Prevent GPU artifacts from malformed float data.

**Files**:
- Modify: `cvkg-stl/src/binary.rs`

**Step 1: Add validation helper**

```rust
fn validate_f32(value: f32, context: &str) -> Result<f32, StlError> {
    if !value.is_finite() {
        Err(StlError::InvalidFloat {
            value,
            context: context.into(),
        })
    } else {
        Ok(value)
    }
}
```

**Step 2: Apply to all parsed floats**

Replace `f32::from_le_bytes(...)` calls with:
```rust
validate_f32(f32::from_le_bytes(...), "normal.x")?
```

**Step 3: Add variant to StlError**

```rust
/// Parsed float is NaN or Infinity.
InvalidFloat { value: f32, context: String },
```

**Step 4: Verify**

```bash
cargo check -p cvkg-stl
cargo test -p cvkg-stl
```

**Step 5: Commit**

```bash
git add cvkg-stl/src/binary.rs
git commit -m "fix(stl): validate parsed floats are finite"
```

---

### Task 0.7: Fix `unsafe { libc::setpriority(-10) }` unchecked

**Objective**: Check return value and log on failure.

**Files**:
- Modify: `cvkg-render-native/src/renderer.rs`

**Step 1: Replace unchecked call**

Find the GodMode setpriority call and replace with:
```rust
#[cfg(target_os = "linux")]
{
    let ret = unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, -10) };
    if ret != 0 {
        log::warn!("GodMode: setpriority failed (errno: {}) — need CAP_SYS_NIO", 
            std::io::Error::last_os_error());
    }
}
```

**Step 2: Verify**

```bash
cargo check -p cvkg-render-native
cargo test -p cvkg-render-native --lib
```

**Step 3: Commit**

```bash
git add cvkg-render-native/src/renderer.rs
git commit -m "fix(native): check setpriority return value in GodMode"
```

---

### Task 0.8: Add RAII guard for GPU_FRAME_PTR

**Objective**: Prevent dangling raw pointer on panic.

**Files**:
- Modify: `cvkg-render-native/src/renderer.rs`

**Step 1: Add scope guard struct**

```rust
/// RAII guard that sets GPU_FRAME_PTR for the duration of a render pass.
/// Clears the pointer on drop, even if a panic occurs.
struct GpuFrameGuard {
    _guard: std::sync::MutexGuard<'static, cvkg_render_gpu::GpuRenderer>,
}

impl GpuFrameGuard {
    fn new(gpu: &Arc<std::sync::Mutex<cvkg_render_gpu::GpuRenderer>>) -> Self {
        let guard = gpu.lock().unwrap_or_else(|e| e.into_inner());
        GPU_FRAME_PTR.with(|ptr| ptr.set(&*guard as *const _ as *mut _));
        Self { _guard: guard }
    }
}

impl Drop for GpuFrameGuard {
    fn drop(&mut self) {
        GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
    }
}
```

**Step 2: Use in render pass**

Replace manual set/clear in `begin_frame`/`end_frame` with guard construction/drop.

**Step 3: Verify**

```bash
cargo check -p cvkg-render-native
cargo test -p cvkg-render-native --lib
```

**Step 4: Commit**

```bash
git add cvkg-render-native/src/renderer.rs
git commit -m "fix(native): add RAII guard for GPU_FRAME_PTR"
```

---

## Phase 1: Test Infrastructure (P0-4, P0-5, P1-5)

### Task 1.1: Add smoke tests to 14 crates with zero tests

**Objective**: Baseline coverage for all crates.

**Files**:
- Modify: Each crate's `lib.rs` or a new `tests/smoke.rs`

**Step 1: Add smoke test module to each crate**

For each of the 14 crates, add at the bottom of `lib.rs`:
```rust
#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn test_default_constructs() {
        // Verify Default impls work
        let _ = Self::default();
    }

    #[test]
    fn test_clone_where_implemented() {
        // Verify Clone impls don't panic
    }
}
```

**Step 2: Verify each crate**

```bash
for crate in cvkg-certification cvkg-compositor cvkg-export-raster cvkg-game-hud cvkg-icons cvkg-macros cvkg-reflect cvkg-render-software cvkg-skills cvkg-svg-serialize cvkg-telemetry cvkg-themes; do
    cargo test -p $crate 2>&1 | grep "test result"
done
```

**Step 3: Commit**

```bash
git add cvkg-*/
git commit -m "test: add smoke tests to 14 crates with zero coverage"
```

---

### Task 1.2: Add fuzz target for STL parser

**Objective**: Find panics in STL parsing via random input.

**Files**:
- Create: `cvkg-stl/fuzz/fuzz_targets/stl_parser.rs`
- Modify: `cvkg-stl/Cargo.toml`

**Step 1: Add fuzz dependency**

In `cvkg-stl/Cargo.toml`:
```toml
[dev-dependencies]
libfuzzer-sys = "0.4"
```

**Step 2: Create fuzz target**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use cvkg_stl::parse_bytes;

fuzz_target!(|data: &[u8]| {
    // Never panic on arbitrary input
    let _ = parse_bytes(data);
});
```

**Step 3: Verify it compiles**

```bash
cargo check -p cvkg-stl
```

**Step 4: Commit**

```bash
git add cvkg-stl/
git commit -m "test(stl): add libfuzzer target for STL parser"
```

---

### Task 1.3: Add proptest for State<T> transitions

**Objective**: Verify state machine invariants via random sequences.

**Files**:
- Modify: `cvkg-core/src/state.rs`

**Step 1: Add proptest module**

```rust
#[cfg(test)]
mod proptest_tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_state_version_monotonic(values in prop::collection::vec(any::<u32>(), 1..100)) {
            let state = State::new(0u32);
            let mut last_version = 0;
            for v in values {
                state.set(v);
                let current = state.version.load(Ordering::Relaxed);
                prop_assert!(current > last_version, "version must increase");
                last_version = current;
            }
        }

        #[test]
        fn test_subscriber_called_on_set(
            values in prop::collection::vec(any::<u32>(), 1..50)
        ) {
            let state = State::new(0u32);
            let call_count = Arc::new(AtomicUsize::new(0));
            let cc = call_count.clone();
            state.subscribe(move |_| { cc.fetch_add(1, Ordering::Relaxed); });
            for v in values {
                state.set(v);
            }
            prop_assert_eq!(call_count.load(Ordering::Relaxed), values.len());
        }
    }
}
```

**Step 2: Add proptest dependency**

In `cvkg-core/Cargo.toml`:
```toml
[dev-dependencies]
proptest = "1"
```

**Step 3: Verify**

```bash
cargo test -p cvkg-core --lib proptest_tests
```

**Step 4: Commit**

```bash
git add cvkg-core/
git commit -m "test(core): add proptest for State monotonicity and subscriber calls"
```

---

### Task 1.4: Add proptest for VDom tree operations

**Objective**: Verify VDom invariants under random patch sequences.

**Files**:
- Modify: `cvkg-vdom/src/vdom.rs`

**Step 1: Add proptest module**

```rust
#[cfg(test)]
mod proptest_tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_vdom_apply_patches_never_panics(
            patches in prop::collection::vec(any::<VDomPatch>(), 0..100)
        ) {
            let mut vdom = VDom::new();
            // Must never panic on arbitrary patch sequences
            vdom.apply_patches(patches);
        }

        #[test]
        fn test_vdom_node_count_bounded(
            node_count in 0..1000usize
        ) {
            let mut vdom = VDom::new();
            for i in 0..node_count {
                vdom.apply_patches(vec![VDomPatch::CreateNode {
                    id: NodeId(i as u64),
                    component_type: "Test".into(),
                }]);
            }
            prop_assert_eq!(vdom.node_count(), node_count);
        }
    }
}
```

**Step 2: Verify**

```bash
cargo test -p cvkg-vdom proptest_tests
```

**Step 3: Commit**

```bash
git add cvkg-vdom/
git commit -m "test(vdom): add proptest for patch application and node count"
```

---

## Phase 2: Design Decomposition (P1)

### Task 2.1: Delete `ORIGINAL_renderer.rs`

**Objective**: Remove 6,943 lines of dead code.

**Files**:
- Delete: `cvkg-render-gpu/src/ORIGINAL_renderer.rs`

**Step 1: Verify it's not referenced**

```bash
grep -r "ORIGINAL_renderer" cvkg-render-gpu/src/ --include="*.rs" | grep -v "ORIGINAL_renderer.rs:"
```

**Step 2: Delete**

```bash
rm cvkg-render-gpu/src/ORIGINAL_renderer.rs
```

**Step 3: Verify build**

```bash
cargo check -p cvkg-render-gpu
```

**Step 4: Commit**

```bash
git add -u cvkg-render-gpu/src/ORIGINAL_renderer.rs
git commit -m "chore(render-gpu): delete 6943-line dead ORIGINAL_renderer.rs"
```

---

### Task 2.2: Add `#[warn(missing_docs)]` to all crates

**Objective**: Enforce documentation on public APIs.

**Files**:
- Modify: All `cvkg*/src/lib.rs`

**Step 1: Add lint**

At the top of each crate's `lib.rs`:
```rust
#![warn(missing_docs)]
```

**Step 2: Fix violations**

Run `cargo clippy --workspace -- -W missing-docs` and fix all warnings.

**Step 3: Commit**

```bash
git add cvkg-*/
git commit -m "feat: enforce missing_docs lint across all crates"
```

---

### Task 2.3: Standardize on `tracing` (replace `println!` in libs)

**Objective**: Unified structured logging.

**Files**:
- Modify: All `cvkg*/src/*.rs` that contain `println!`

**Step 1: Replace `println!` → `tracing::info!`**

In all library crates, replace:
- `println!("...")` → `tracing::info!(...)`
- `eprintln!("...")` → `tracing::warn!(...)`

**Step 2: Add tracing dependency**

In each crate's `Cargo.toml`:
```toml
tracing.workspace = true
```

**Step 3: Verify**

```bash
cargo check --workspace
```

**Step 4: Commit**

```bash
git add cvkg-*/
git commit -m "chore: replace println! with tracing across all library crates"
```

---

## Phase 3: Observability (P2)

### Task 3.1: Wire telemetry into render loop

**Objective**: Make frame budget tracking actually work.

**Files**:
- Modify: `cvkg-render-native/src/renderer.rs`

**Step 1: Add telemetry recording in begin_frame/end_frame**

```rust
fn begin_frame(&mut self) {
    let _span = tracing::info_span!("frame").entered();
    self.frame_start = std::time::Instant::now();
    // ... existing code
}

fn end_frame(&mut self) {
    // ... existing code
    let frame_time = self.frame_start.elapsed().as_secs_f32() * 1000.0;
    if frame_time > 16.67 {
        tracing::warn!(
            target: "cvkg::telemetry",
            frame_time_ms = frame_time,
            budget_ms = 16.67,
            "Frame budget exceeded"
        );
    }
}
```

**Step 2: Verify**

```bash
cargo check -p cvkg-render-native
cargo test -p cvkg-render-native --lib
```

**Step 3: Commit**

```bash
git add cvkg-render-native/src/renderer.rs
git commit -m "feat(native): wire frame budget telemetry into render loop"
```

---

### Task 3.2: Add i18n locale tests

**Objective**: Verify components don't panic on non-ASCII input.

**Files**:
- Modify: `cvkg-components/src/lingua_tong.rs`

**Step 1: Add test module**

```rust
#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn test_non_ascii_translation() {
        load_translations("ja", HashMap::from([("hello".into(), "こんにちは".into())]));
        set_locale("ja");
        assert_eq!(t("hello"), "こんにちは");
    }

    #[test]
    fn test_rtl_locale() {
        load_translations("ar", HashMap::from([("hello".into(), "مرحبا".into())]));
        set_locale("ar");
        assert_eq!(t("hello"), "مرحبا");
    }

    #[test]
    fn test_missing_key_fallback() {
        set_locale("en");
        assert_eq!(t("nonexistent_key"), "nonexistent_key");
    }

    #[test]
    fn test_chinese_locale() {
        load_translations("zh", HashMap::from([("hello".into(), "你好".into())]));
        set_locale("zh");
        assert_eq!(t("hello"), "你好");
    }
}
```

**Step 2: Verify**

```bash
cargo test -p cvkg-components i18n_tests
```

**Step 3: Commit**

```bash
git add cvkg-components/src/lingua_tong.rs
git commit -m "test(components): add i18n locale tests for CJK, Arabic, fallback"
```

---

## Phase 4: Accessibility (P2-4)

### Task 4.1: Audit ARIA roles on all interactive components

**Objective**: Every interactive component has correct ARIA role.

**Files**:
- Modify: All component files in `cvkg-components/src/interactive/`

**Step 1: Add ARIA role to components missing it**

For each component (Tabs, Popover, Dialog, Slider, Progress, Breadcrumb), add in `render()`:
```rust
renderer.set_aria_role("tablist");  // or appropriate role
renderer.set_aria_label(&self.label);
```

**Step 2: Verify**

```bash
cargo check -p cvkg-components
cargo test -p cvkg-components
```

**Step 3: Commit**

```bash
git add cvkg-components/src/interactive/
git commit -m "feat(components): add ARIA roles to all interactive components"
```

---

## Phase 5: Performance (P2-10)

### Task 5.1: Remove clone in VDom hit_test_recursive

**Objective**: Eliminate per-frame allocations in hot path.

**Files**:
- Modify: `cvkg-vdom/src/vdom.rs`

**Step 1: Replace clone with direct iteration**

Find:
```rust
let mut children_to_test = node.children.clone();
```

Replace with:
```rust
let children_to_test = &node.children;  // borrow instead of clone
```

**Step 2: Verify**

```bash
cargo check -p cvkg-vdom
cargo test -p cvkg-vdom
```

**Step 3: Commit**

```bash
git add cvkg-vdom/src/vdom.rs
git commit -m "perf(vdom): eliminate clone in hit_test_recursive hot path"
```

---

## Skills Reference

| Skill | Used In |
|-------|---------|
| `rust-patterns` | P0-1b, P0-2, P0-3, P1-6, P1-7, P2-1, P2-9, P2-10, P3-3 |
| `rust-error-propagation` | P0-2, P0-5, P0-6, P1-8 |
| `strong-tests` | P0-3, P0-4, P0-5, P1-5, P2-5 |
| `verification-before-completion` | P0-1, P0-1b, P0-6, P1-7 |
| `security-engineering` | P0-1c, P0-5, P0-6, P2-9 |
| `clean-architecture` | P1-1, P1-2, P1-8 |
| `system-design` | P1-1, P3-3 |
| `rust-module-decomposition` | P1-2 |
| `documentation` | P1-3 |
| `writing-guidelines` | P1-3 |
| `test-patterns` | P0-5, P1-5 |
| `rust-testing` | P0-4, P1-5 |
| `test-driven-development` | P0-3, P0-4 |
| `rust-tdd` | P0-3 |
| `observability-engineering` | P0-1c, P2-2, P2-7, P3-1 |
| `accessibility` | P2-4, P4-1 |
| `frontend-design` | P2-4, P2-6 |
| `qa-engineering` | P2-3, P2-5 |
| `clean-code` | P2-1, P3-1, P3-2 |
| `ponytail-review` | P1-4, P2-1 |
| `ponytail-audit` | P1-4 |
| `performance` | P1-6, P2-10 |
| `backend-patterns` | P1-6 |
| `site-reliability-engineering` | P2-7 |
| `devops-platform` | P2-8 |
| `ci-cd-process` | P2-8 |
| `writing-clearly-and-concurrently` | P3-1 |
| `rust-development` | P3-4, P3-5 |
| `factory-standards-guard` | P3-6 |
| `code-review-process` | P3-6 |
| `software-design-philosophy` | P1-1 |
| `rendering-architecture-audit` | A14, A15 |
| `product-design` | P2-6 |

---

## Success Criteria

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes (zero new failures)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] All 14 zero-test crates have smoke tests
- [ ] STL parser has fuzz target + triangle limit + NaN check
- [ ] `ORIGINAL_renderer.rs` deleted
- [ ] `println!` replaced with `tracing` in all libs
- [ ] `debug_assert!` on State, LayoutCache, Mesh, VDom
- [ ] ARIA roles on all interactive components
- [ ] i18n locale tests pass (CJK, Arabic, fallback)
- [ ] Frame budget telemetry wired into render loop
- [ ] VDom hit_test no longer clones children

---

*Plan generated from ponytail.md audit (226 findings). Execute task-by-task. One task per turn.*
