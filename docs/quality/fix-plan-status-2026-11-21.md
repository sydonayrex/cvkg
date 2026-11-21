# Fix-plan status — First PR batch (2026-11-21)

**Plan:** `plans/fix-plan-2026-11-20.md`
**Scope selected:** Section 5, "Immediate Next Actions (Recommended First PR Batch)", plus the Section 4 documentation deliverables and the parts of P0-1/P0-2 that are low-risk.

## 1. Status by task

| Task | Status in this batch | Evidence |
|---|---|---|
| P0-4 Shader/SPIR-V support clarification | **Done** | `README.md` "Shader and GPU backend support" section; `docs/quality/shader-validation.md` §5. Also corrected README and `docs/onboarding.md` Rust version (1.85 → 1.95) and README license (MPL-2.0 → EPL-2.0) to match `Cargo.toml`/`LICENSE`. |
| P0-3 Composed shader validation | **Done** | New `cvkg-render-gpu/src/shader_composition.rs`; `renderer/init.rs` now composes through it; 5 tests validate all composed variants for native **and** wasm with naga parse + validation. `scripts/validate_shaders.py --check` plus a new CI job. |
| P0-1 Panic hardening | **Partial (top hotspots fixed)** | `material.rs` builtin generation is now fallible + lossy (removed `unwrap`); `renderer/draw.rs` + `renderer/geometry.rs` 22-site `last_call.unwrap()` family removed via `is_none_or`; `renderer/init.rs` 7 `NonZeroUsize::new(..).unwrap()` sites replaced with `nonzero_capacity`; `cvkg-scene` dangling-root/node guards. Full inventory: `docs/quality/hotspot-inventory.md`. |
| P0-2 Unsafe/concurrency safety review | **Inventory only** | `docs/quality/hotspot-inventory.md` §3 lists all 12 unsafe sites and flags 6 missing SAFETY comments. Refactoring deferred (needs targeted review per site). |
| P1-1 Numeric rigor policy + pilot | **Done (pilot)** | New `cvkg-core/src/numeric.rs` (policy + 8 helpers + 6 tests); `cvkg-core/src/animation.rs` migrated with 3 new edge-case tests; `docs/quality/numeric-rigor.md`. |
| P1-3 Web text/ARIA bridge spike | **Spike done** | `demos/adele-web/src/a11y_bridge.rs` (ARIA live region, focusable offscreen IME textarea, node mirror, clipboard hooks, no-panic API, non-wasm stub), minimal wasm-only integration in `src/lib.rs`, README section with manual WebKit/VoiceOver steps and honest limitations. |
| P1-4 WebKit compatibility matrix | **Docs done, runtime detection not implemented** | `docs/platform/webkit-compatibility.md` explicitly labels runtime WebKit detection and fallback UI as gaps. |
| P2-2 Accessibility baseline doc | **Doc done** | `docs/ux/accessibility-baseline.md` with Implemented/Partial/Missing checklist. |
| P2-4 Readiness/adoption doc | **Doc done** | `docs/product/gui-base-readiness.md`; verdict quotes audit §7. |
| P2-5 Initial CI gates | **Partial** | Shader validation job added to `.github/workflows/ci.yml`; wasm build/desktop smoke jobs still absent. |

## 2. Real defects found and fixed by the new validation

The composed-shader tests were not merely additive — they exposed two shipping bugs:

1. **Stale/corrupted wasm shader set.** `common_wasm.wgsl` was an old stripped-down version missing `VertexOutput` fields (`logical`, `clip`, `slice`, `material_id`) and helpers used by `shapes.wgsl`; `material_opaque_wasm.wgsl` contained literal line-number prefixes copied from a tool transcript. All three wasm variants are now deterministically regenerated from their native counterparts by `scripts/validate_shaders.py --sync-wasm` and CI-checked for drift.
2. **Platform-blind generated material WGSL.** `MaterialCompiler` emitted `t_diffuse[<index>]` for every target, which is invalid WGSL against the wasm single-texture binding. `material.rs` now has `*_for(single_texture: bool)` generators, and `shader_composition::generated_materials_for()` picks the correct form from the same `ShaderSet` used by tests.

## 3. Verification actually run

```text
cargo check -p cvkg-render-gpu                         PASS (1 pre-existing warning)
cargo check -p cvkg-render-gpu -p cvkg-scene           PASS
cargo check --workspace                                 PASS (pre-existing warnings only)
cargo test  -p cvkg-render-gpu --lib shader_composition PASS (5 tests, native + wasm)
cargo test  -p cvkg-core --lib numeric                  PASS (6 tests)
cargo test  -p cvkg-core --lib animation                PASS (9 tests)
cargo test  -p cvkg-scene --lib                         PASS (15 tests)
python3 scripts/validate_shaders.py --check             PASS (7 unreferenced-shader warnings)
```

Not run: full workspace test suite, clippy, wasm-target build of the whole demo,
browser/WebKit manual matrix, GPU-device shader compilation. The wasm bridge
module itself was verified by its author with `cargo check` on host and
`wasm32-unknown-unknown` in isolation; the full demo's wasm build still fails on
pre-existing `cvkg-render-gpu` `Send + Sync` constraints, which is a follow-up.

## 4. Deviations and notes

- The inventory suggested `debug_assert!(false, ...)` for the scene recovery
  paths. That would still panic in test/debug builds, so the implemented guards
  recover silently (no logging dependency exists in `cvkg-scene`).
- `cvkg-gallery`'s 67 panic/lock sites are demo-only and were deliberately left
  unchanged; the inventory recommends a `dev-demo` feature boundary rather than
  hardening showcase code.
- Pre-existing uncommitted changes in `.gitignore`, `cvkg-components/...niflheim_sidebar.rs`, `cvkg-components/...hrungnirsegmented.rs`, `cvkg-gallery/src/app.rs` and `cvkg-gallery/src/canvas.rs` were **not touched**.
- 7 shader files (`filter_*.wgsl`, `gradient_multi.wgsl`, `material_gradient.wgsl`, `taa.wgsl`) are not referenced from Rust source; flagged as warnings, not failures.

## 5. Batch 2 (continued work, same day)

| Task | Status | Evidence |
|---|---|---|
| P0-2 Unsafe/concurrency safety | **Comments closed + 3 hardening changes** | All 12 sites inventoried; all 6 comment gaps now carry `// SAFETY (P0-2)` rationale. wasm `Send`/`Sync` impls gated on `not(target_feature = "atomics")` so enabling threads fails at compile time instead of UB. `GpuFramePtrGuard::set` now takes the source mutex and debug-asserts it is locked. `RodioAudioEngine` records its creating `ThreadId` and warns on foreign-thread use. Full safe refactor of the raw-pointer fast path would require threading `&mut GpuRenderer` through the `Renderer` trait — deferred. |
| P0-5 SVG component transfer TODO | **Done (explicit gating)** | `cvkg-svg-filters/src/types.rs` gains `FilterError::Unsupported`; `pipeline.rs` compares the four transfer functions and returns that error when G/B/A differ from R instead of silently rendering with R's function. 10 new tests; `cargo test -p cvkg-svg-filters` → 38 passed. True per-channel support remains out of scope and is documented as such in the crate README. |
| P1-1 numeric rigor (2nd pilot) | **Done** | `cvkg-core/src/geometry_modifiers.rs`: `FrameModifier::bounded()` replaces `f32::clamp` (which panics on reversed bounds); explicit sizes sanitized with `finite_or`; 4 new tests. `cargo test -p cvkg-core --lib` → 152 passed. |
| P1-2 Shader numeric hardening | **Partial (PBR pilot)** | Added `is_finite_f32` / `safe_div_f32` / `safe_normalize_vec3` to `common.wgsl` (and regenerated wasm variants). `material_pbr.wgsl` hotspots guarded: Schlick-GGX denominator, irradiance spacing, shadow homogeneous divide, texel size, all normal/light/view/half normalizations. All composed variants still pass naga validation. Glass/raymarch/effects remain outstanding. |
| P1-4 Runtime WebKit detection + fallback UX | **Implemented (message-only)** | `demos/adele-web/src/capability_probe.rs`: wasm probe for `navigator.gpu`, throwaway-canvas WebGL2 probe, labelled UA heuristic, idempotent visible `<div role="alert">` notice, non-wasm stub. Wired additively into `src/lib.rs`. Probe verified on host + wasm32 in isolation; full demo wasm build still blocked by pre-existing `cvkg-render-gpu` `Send + Sync` errors. No browser/VoiceOver matrix performed. |
| P2-5 Initial CI gates | **Done** | `.github/workflows/ci.yml` now has 4 jobs: `ci`, `shader-validation`, `wasm-build` (25 verified-green crates, full-workspace wasm explicitly excluded with rationale), and `desktop-smoke` (builds `cvkg-gallery`, runs headless `trace_gallery` test). YAML validated locally. |
| P0-1 completion (low-risk remainder) | **Done** | `cvkg-anim`: missing solver now skipped; `growth.rs` stochastic pick returns `Option` instead of indexing/unwrapping empty slices. `cvkg-render-native`: translation-stack guard uses `copied().unwrap_or(ZERO)`; missing window entry returns cleanly. Fixed a pre-existing failing test fixture (`native_pointer_capture_falls_back_to_rebuilt_target`): the rebuilt node now receives re-registered handlers, matching a real VDOM rebuild. `cargo test -p cvkg-render-native --lib` → 23 passed. |

### Batch 2 verification

```text
cargo test -p cvkg-core --lib              152 passed
cargo test -p cvkg-render-gpu --lib        179 passed
cargo test -p cvkg-scene --lib              15 passed
cargo test -p cvkg-svg-filters --lib        38 passed
cargo test -p cvkg-anim --lib              106 passed
cargo test -p cvkg-render-native --lib      23 passed
cargo check -p cvkg-render-native -p cvkg-render-gpu   PASS
cargo check --workspace                     PASS (batch 1; re-run before final commit)
python3 scripts/validate_shaders.py --check PASS
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"  PASS
```

## 5e. Hardware verification (2026-11-21)

Environment: RTX 4070 (driver 620.73.04), Wayland/X display, Chromium 158,
wasm32 target, wasm-pack. `agent-browser` CLI was not installed, so a small
Chrome DevTools Protocol driver was used instead (outside the repo, in
`/tmp/opencode`).

- **Full wasm build unblocked (38/43 members):** removed the unused
  `wasm-bindgen-rayon` dependency (it force-enabled atomics, which disabled the
  safety-gated `Send`/`Sync` impls and surfaced all the `PassNode: Send + Sync`
  errors); added `getrandom` `js` feature for `cvkg-flow` and gated `tokio`
  `"full"`/`rt-multi-thread` off wasm in `cvkg`/`cvkg-test`. CI's `wasm-build`
  job now checks the workspace with five native-only members excluded, and
  `desktop-smoke` now also runs `cvkg-render-native --lib`. `.github/workflows/ci.yml`
  was force-added to tracking — it was previously ignored by `.gitignore:1`
  (`.*`) and had never been committed, so the manual-only trigger had not
  actually reached the remote.
- **Wasm demo now renders in Chromium (verified):** `wasm-pack build --target
  web` succeeds and `src/web_render.rs` paints a real frame via
  `cvkg-render-software` + `ImageData`. Browser result: zero console errors,
  `[web-render] frame blitted to canvas`, non-transparent pixels. The software
  renderer's `Instant::now()` panicked on wasm, so its clock now uses
  `js_sys::Date` on that target.
- **WebGPU runtime bugs found by hardware verification** (naga/native tests
  could not catch these): out-of-range `f32` literal, `BINDING_INDEXING` /
  TextureViewArray bindings, derivative/`textureSample`/`textureSampleCompare`
  calls in non-uniform branches, a background font thread on wasm, renderer
  `Instant::now()`/`temp_dir()` panics, and depth textures with filtering
  samplers. **All fixed.** The shipped demo now renders via WebGPU in Chromium
  with zero errors and falls back to the software renderer if WebGPU fails.
  Details in `docs/platform/webkit-compatibility.md` §7.
- **Journey reflow test de-flaked:** `cvkg-scene/tests/journey_tests.rs` looked up
  any node whose `component_type.contains("Text")`, which nondeterministically
  matched either the 500×500 `Text` VNode or the shaped `Primitive::Text` child.
  Narrowed to `component_type == "Primitive::Text"`; 3/3 pass.
- **P1-5 native call site:** `convert_keyboard_event` now routes winit physical
  codes through `cvkg_core::normalize_key_name`, so `KeyA` → `a`, `Digit1` → `1`
  and text-input keydown handlers receive the same logical names as the web
  path. Extracted a pure `normalize_key_code` helper with unit tests; native
  renderer lib suite → 24 passed. Web key-event call sites are still absent
  (the web demo has no keyboard handling yet).

- **Frame/translation contract fixes:** `FrameModifier::render` was using the
  child's raw intrinsic size, leaving `transform_size` dead code so
  `.frame(w, h)` had no effect (zero-size buttons). It now applies the
  configured constraints. `GpuRenderer` now owns the cumulative translation
  stack for `push_vnode`/`push_translation` and sets the thread-local offset;
  `NativeRenderer` delegates to it instead of maintaining a second stack.
- **Known remaining red test:** `cvkg-test --test visual_regression::test_generate_niflheim_screenshot`
  reports 100% diff against its golden. Verified on a clean `HEAD` worktree with
  identical output, and pixel-compared the clean-HEAD render with the current
  render — **byte-identical**, so this work did not cause it. The current render
  is a blank background with only tiny top-left text while the golden shows the
  full Niflheim demo; this is a genuine pre-existing rendering regression
  (likely the nodal-coordinate migration). The golden was restored rather than
  overwritten with the broken output. The two layout fixes above improve the
  render (buttons now have their configured 160×30 size) but the golden also
  predates the tone-mapping/color changes and deeper Grid/VStack placement
  differences. Completing this needs a layout pass, not a golden update.
- **Scene consistency fixtures fixed:** `cvkg-scene/tests/consistency_tests.rs`
  still expected `Text::render` to emit exactly one command; it now emits
  `PushVNode("Text")`, `DrawText`, `PopVNode` (companion attachment). Updated the
  assertion to check the three-command contract and accepted the regenerated
  insta snapshot. These failures were previously masked because Cargo stops
  after the first failing test binary; they surfaced once the GPU integration
  suite went green. Verified 2/2.
- **GPU integration tests fixed:** the two long-standing failures in
  `cvkg-render-gpu/tests/integrated_ui_scenarios.rs` were a fixture problem, not
  a hardware bug. `forge_headless` already selects the NVIDIA RTX 4070; the
  unconditional ACES tone map in `bloom.wgsl::fs_composite` compresses channel
  separation before sRGB8 readback, so the old raw-sRGB thresholds failed.
  Recalibrated the two predicates to tone-mapped output without weakening hue
  assertions. Verified with and without
  `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json WGPU_BACKEND=vulkan`:
  **20/20 passed** on real hardware.
- **P1-4 capability probe, real browser:** compiled the shipped
  `capability_probe.rs` into a standalone wasm harness and ran it in Chromium.
  With WebGPU/WebGL2 enabled: `webgpu=true`, `webgl2=true`,
  `webkit_like=true`, `supported=true`, `tier=WebGpu`. With
  `--disable-webgl --disable-3d-apis`: `webgl2=false` but `navigator.gpu`
  still present — confirming the documented "API presence only" limitation.
- **Fallback UI:** `show_unsupported_notice()` inserted a visible
  `role="alert"` element with the documented text (DOM query + screenshot).
- Recorded in `docs/platform/webkit-compatibility.md` §7 and
  `docs/ux/accessibility-baseline.md`. **No Safari/WebKit/iOS run** was possible;
  all WebKit matrix cells remain Untested.

## 5d. Component/token parity docs

- **P2-1:** `docs/quality/component-parity.md` — 8-dimension rubric, two
  scorecard tables covering all seven priority families with file:line
  evidence, prioritized gaps and fix order. Top gaps: `View::on_key_event` has
  no call site; ARIA state (`invalid`, `selected`, `expanded`) is shallow;
  menus/tables publish no semantic role.
- **P2-3:** `docs/quality/token-consolidation.md` — token source map, 10
  duplication/drift findings, a labelled single-semantic-token proposal and an
  API-preserving four-phase migration. Flags disagreement on
  `body=16/17`, `space md=12/16`, `radius md=6/8`.
- Both are static audits; no runtime/browser verification.

## 5c. Continuation after first push

- **P1-5 (focused slice):** `cvkg-core/src/keyboard.rs` gains a pure, additive
  shortcut foundation (`ShortcutPlatform`, `normalize_key_name`,
  `KeyModifiers::{is_empty,count,primary,from_pressed_keys}`,
  `KeyShortcut::{parse,matches,matches_on}` + `FromStr`) so native physical
  codes and web logical keys can share one representation. Registered a new
  integration test target in `cvkg-core/Cargo.toml`
  (`tests/input_parity.rs`, 4 tests). `cargo test -p cvkg-core --lib` → 162
  passed; `--test input_parity` → 4 passed. Call sites in native/web are not
  migrated yet; clipboard/text-input/focus parity remains open.

## 5b. Final adjustments (user request)

- **P1-2 completed** in a follow-up pass: hardened the remaining `common.wgsl`
  divisions (SDF extents, squircle exponent, segment parameter, smooth-min,
  bloom), raymarch step monotonicity and gradient normalization, plus the
  PBR reflection homogeneous divide. A structural test pins the numeric guard
  helpers in every composed variant. `cargo test -p cvkg-render-gpu --lib` →
  180 passed.
- `.github/workflows/ci.yml` triggers switched to **manual only** (`workflow_dispatch`);
  `push`/`pull_request` triggers removed. `docs/ci-plan.md` updated to match.
- Two pre-existing test-compile failures that made `cargo test --workspace` red
  were fixed so manual runs can go green:
  - `cvkg-render-3d/src/flattener.rs`: the `gltf`-dependent test module now uses
    `#[cfg(all(test, feature = "gltf"))]` (default features no longer fail).
  - `cvkg-macros/src/lib.rs`: removed a duplicate `mod smoke_tests` (E0428).
- Shader numeric hardening extended to `material_glass.wgsl` (Snell IOR divide,
  sqrt radicand clamp, GGX NDF/Smith denominators, screen-space normalizations).
  All composed native + wasm variants still pass naga validation.
- Full `cargo test --workspace --tests --lib --bins` now compiles and passes for
  all lib/bin targets. Two integration tests in
  `cvkg-render-gpu/tests/integrated_ui_scenarios.rs`
  (`test_standard_ui_photo_editor`, `test_standard_ui_svg_editor`) still fail on
  pixel assertions; both were confirmed to fail identically on a clean HEAD
  worktree, so they are pre-existing and unrelated to this work. A strict
  `all_builtin_materials_compile` test was added to prove no builtin is silently
  dropped by the lossy generator.

## 6. Recommended next batch (after batch 2)

1. **P1-2 remainder:** migrate `material_glass.wgsl`, raymarch/SDF loops and
   `common.wgsl` division hotspots to the shared WGSL helpers; add visual/numeric
   snapshots where practical.
2. **P1-5 / P2-2:** unify text/clipboard/focus semantics across native and web;
   run the actual Safari/VoiceOver matrix against the P1-3/P1-4 prototypes.
3. **P2-1 / P2-3:** implement the component parity rubric and token
   consolidation, starting with buttons/inputs/selects/dialogs.
4. **Full-workspace wasm build:** fix the pre-existing `Send + Sync` conflicts in
   `cvkg-render-gpu`, `cvkg-render-3d`, `cvkg-export-raster`, `cvkg-inputs`
   (`js_sys::Object`) and `cvkg-flow` (getrandom `js` feature), then replace the
   curated wasm crate list in CI with `--workspace`.
5. **P0-1 completion:** gate `cvkg-gallery` behind a `dev-demo` feature so core
   crate lint rules can exclude demo-only panics, then finish the remaining
   low-priority unwrap sites in `cvkg-anim` and `cvkg-render-native`.
