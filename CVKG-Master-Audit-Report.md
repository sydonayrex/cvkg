# CVKG Workspace — Reconciled Master Audit Report

**Compiled:** 2026-06-21
**Source audits reconciled (4):**
| Source | File | Date | Approx. coverage |
|---|---|---|---|
| DeepSeek | `deepseek_audit.md` | 2026-06-20 | Per-crate, file-by-file, ~25 crates, ~100k LOC claimed |
| Pool | `pool_audit.md` | 2026-06-20 | Per-file checklist, ~28 crates surveyed |
| Google Flash | `Google_audit.md` | 2026-06-21 | Per-file checklist, broad pass across ~45 files |
| Owl | `owl_audit.md` | 2026-06-21 | Per-file + dedicated scheduler/spatial/materials deep-dive, cvkg-core 9556-line file in heavy depth |
| *(Specialist)* | `cvkg-audit-scheduler-spatial-materials.md` | 2026-06-20 | Deep, narrow audit of cvkg-scheduler, cvkg-spatial, cvkg-materials only |

**Severity normalization applied throughout this document:**
`Critical → P0`, `High → P1`, `Medium → P2`, `Low → P3`, `Info/cosmetic → P4`

Where two audits disagree on severity for the same finding, the **higher** severity is kept and the disagreement is noted.

---

## 1. How to read this report

Four independent LLM-driven audits (plus one specialist deep-dive) covered overlapping but not identical sets of files. They frequently:
- **Agree** on the same bug at the same line (high confidence — flagged ✅✅).
- **Disagree on severity** for the same bug (e.g., SHA256 truncation rated P2 by DeepSeek/Owl).
- **Contradict each other outright** — e.g., Pool and Google audit the same file and reach opposite conclusions (see §5, Conflicts).
- **Claim a bug is already fixed** — DeepSeek's later workspace-summary pass marks several bugs `[FIXED]` that its own earlier per-file pass had reported as open, implying a subagent applied patches mid-audit. These are marked **[FIXED-PER-SOURCE]** below — unverified by this reconciliation, since no test run or diff was inspected.

---

## 2. P0 (Critical) — Workspace-wide

| # | Crate / File | Issue | Source(s) | Status |
|---|---|---|---|---|
| P0-1 | `cvkg-core` / `cvkg-components` | System-state hash collision (`0xD00_0001`) — 3 component types share the same key, causing silent data corruption | DeepSeek (aggregate ranking) | Open |
| P0-2 | `cvkg-layout` | `AspectRatio` Y-center computed with `* 0.0` instead of `* 0.5` — views render top-aligned instead of centered | DeepSeek (aggregate ranking) | Open |
| P0-3 | `cvkg-webkit-server` | Stored XSS via `/snapshot` endpoint — unsanitized POST body rendered as HTML | DeepSeek (aggregate ranking) | Open — **not corroborated** by Google/Owl/Pool, who instead flag *directory traversal* and *fuel/stdin* issues in this crate (see §5) |
| P0-4 | `cvkg-render-native` (`src/lib.rs:1248`) | Dangling thread-local GPU pointer (`GPU_FRAME_PTR`) if `view.render()` panics during a locked render pass — leaves UB-prone dangling pointer for subsequent frames | Google ✅, DeepSeek (as MED unsafe finding), Owl (notes file as monolith, doesn't flag this specific bug) | Open. Google supplies a RAII drop-guard fix (§6) |
| P0-5 | `cvkg-render-gpu/src/renderer.rs:1138` | Unverified binary pipeline-cache bytes passed straight into `unsafe { device.create_pipeline_cache(&cache_data) }` — tampered/corrupted cache files can corrupt the GPU driver | Google ✅ | Open. Treated as P0 by Google; related SHA-truncation issue at a *different* line is rated only P2 by DeepSeek/Owl (see P2-1) — these are **two distinct findings about pipeline-cache integrity**, not duplicates |

> **Note on P0-3:** Only DeepSeek's single workspace-summary table asserts a stored-XSS vulnerability in cvkg-webkit-server. None of Owl, Google, or Pool — all of which separately examined `cvkg-webkit-server/src/main.rs` or `wasm_server.rs` — mention it. Given 3-of-4 audits are silent on this specific claim, it should be **independently re-verified before treating as confirmed**, even though it's listed at Critical/P0 by its source.

---

## 3. P1 (High)

| # | Crate / File | Issue | Source(s) | Status |
|---|---|---|---|---|
| P1-1 | `cvkg-render-gpu/passes/backdrop_region.rs:50,54` | `.expect()` on registry texture/blur-target lookup — panics on resource miss instead of graceful `match`+`log::error!`+`return` (the pattern every other pass uses) | DeepSeek ✅, Owl ✅ | Open — high-confidence, 2 independent audits, exact line match |
| P1-2 | `cvkg-render-gpu/passes/accessibility.rs:58` | `.unwrap()` on registry texture-view lookup — same panic pattern | DeepSeek ✅, Owl ✅ | Open |
| P1-3 | `cvkg-render-gpu/passes/pyramid.rs:20` | `.unwrap()` on registry mip-view lookup inside a loop — same panic pattern | DeepSeek ✅, Owl ✅ | Open |
| P1-4 | `cvkg-webkit-server/src/wasm_server.rs` | WASI preopened directory grants WASM guest full read/write (`DirPerms::all()`/`FilePerms::all()`) — privilege escalation if untrusted WASM is loaded | Owl ✅ (rated MED→normalized here to P1 given severity of full FS R/W to untrusted code), Google notes generic "directory traversal" risk in same crate | Open |
| P1-5 | `cvkg-webkit-server/src/wasm_server.rs:112-124` | `tick()` takes ownership of the WASM session, and if `execute_tick()` panics the session is dropped/lost — subsequent calls permanently fail | Owl ✅ | Open |
| P1-6 | `cvkg-core/src/lib.rs:3678` (or `~3677-3678` depending on version) | `unsafe { Arc::from_raw(...) }` reconstructing an `Arc<Mutex<SleipnirSolver>>` from a `u64` stored during serialization — sound only if same-process/non-stale; cross-process or stale deserialization is a UAF | DeepSeek ✅ (rated MED), Pool ✅ (rated HIGH), Owl (reviewed same code, judged it **sound-by-construction** via `downcast_ref` guard, only wants a safety comment) | **Disputed severity** — see §5. Treated as P1 here as the conservative middle ground |
| P1-7 | `cvkg-runic-text/src/subpixel.rs:290` | Subpixel LCD phase computed as `(local_x * 3.0) as i32 % 3` on an already-integer `local_x` — always evaluates to 0, permanently disabling G/B channel coverage and causing color fringing | Google ✅ | Open. Not flagged by DeepSeek/Owl/Pool, who reviewed adjacent files but possibly not this exact bug — treat as credible single-source P1 given concrete repro logic |
| P1-8 | `cvkg-render-gpu/src/color_blindness.rs:122` | `fs_main_vs` hardcodes both components to `-1.0` for vertex index 2 — produces a degenerate line instead of a full-screen triangle, making colorblindness simulation invisible | Google ✅ | Open |
| P1-9 | `cvkg-anim` — division-by-zero cluster | `particles.rs:57`, `advanced_particles.rs:842` (spawn_rate==0), `shader_anim.rs:317/323-324` (frame_count==0 underflow + div/0), `momentum.rs:19` (NaN via `powf` on negative friction), `verlet.rs:127-129` (OOB constraint indices) | DeepSeek ✅ (detailed, rated HIGH each) | Open — none of the other 3 audits went this deep into cvkg-anim's particle/verlet code, but the line-level detail here is credible and actionable |
| P1-10 | `cvkg-render-software` rounded-rect SDF | `(fx - rect.x).max(...).max(0.0) - rect.width*0.5` collapses to `|fx-center|`, drawing a small circle instead of a rounded rect; companion stroke-rect bug never draws at all | DeepSeek ✅ ("**[FIXED]**" per DeepSeek's own later pass), Google ✅ (still reports as open, same line `224`) | **Conflicting fix-status** — DeepSeek claims fixed by a subagent; Google's independent pass (one day later) still finds the same defective math at the same line. Treat as **still open** until verified by re-reading current source |
| P1-11 | `cvkg-icons` CSS `rgba()` | Uses float `[0.0,1.0]` values inside CSS `rgba()`, which expects 0–255 integers — renders all icons black | DeepSeek ✅ | Open. Not independently corroborated (Owl/Google/Pool list cvkg-icons as "clean, no bugs found") — **conflict**, see §5 |
| P1-12 | `cvkg-macros` `vdom_id()` | `DefaultHasher::new().finish()` is called without writing any data into the hasher — produces a constant/non-deterministic ID for every model node, breaking VDOM identity | DeepSeek ✅, Google ✅ | Open — corroborated by 2 sources, high confidence |
| P1-13 | `cvkg-themes` | Alpha values exceeding 1.0 in `oklch_to_color_theme()` (`primary_neon: [...,1.2]`, `shatter_neon: [...,1.5]`) causing GPU rendering artifacts | DeepSeek ✅ | Open |
| P1-14 | `cvkg-flow/src/edge.rs`, `node.rs` | Elastic easing clamped to a no-op; negative hue `%` doesn't wrap | DeepSeek (marked **[FIXED]** by subagent, with passing-test claim) | **[FIXED-PER-SOURCE]** — only DeepSeek covered this file in depth; no independent corroboration of either the bug or the fix |
| P1-15 | `cvkg-render-gpu/src/material.rs:1039` | `ShaderCompiler::compile(...).unwrap()` on built-in WGSL generation — any built-in shader compile failure (e.g. driver bug) hard-crashes the renderer at startup with no fallback | DeepSeek ✅, Owl (aggregate list) ✅ | Open |

---

## 4. P2 (Medium)

| # | Crate / File | Issue | Source(s) | Status |
|---|---|---|---|---|
| P2-1 | `cvkg-render-gpu/src/renderer.rs:517-521,6107-6111` | Shader cache integrity check (SHA256) truncated to first 8 bytes (64 bits) — collision space reduced from 2^256 to 2^64, feasible for a targeted attacker to poison the shader cache | DeepSeek ✅, Owl ✅ | Open. *Distinct* from the unverified-pipeline-cache-bytes finding at P0-5 — same renderer.rs file, two different integrity gaps |
| P2-2 | `cvkg-core/src/lib.rs` mutex-poison cluster | `update_system_state` (`STATE_WRITE_MUTEX`), `enqueue_batch_task` (`BATCH_QUEUE`), `ENVIRONMENT`, `State::subscribe` — all use `.lock().unwrap()`; one panicking holder poisons the lock for the rest of the app's life | Owl ✅ (most detailed), Pool ✅ (same lines `1210,1228`), DeepSeek ✅ (generic "~10 unwrap on Mutex/RwLock") | Open — 3-source agreement, high confidence. Fix: `.lock().unwrap_or_else(\|p\| p.into_inner())` |
| P2-3 | `cvkg-core` `DependencyGraph.register` | Doesn't deduplicate reverse-map entries; calling `register()` twice with the same pair and then `unregister()` once leaves a stale dependency reference | Owl ✅ | Open |
| P2-4 | `cvkg-vdom/src/lib.rs` | `VDomPatch::Update` handler serialization round-trip bug — handlers can never be correctly deserialized | Owl ✅ | Open |
| P2-5 | `cvkg-scene/src/lib.rs:367 (or 361)` | `deserialize_binary` resets `next_id` to 0 — collides with IDs already present in the deserialized graph instead of resuming from `max_id + 1` | DeepSeek ✅, Google ✅, Owl ✅ | Open — 3-source agreement |
| P2-6 | `cvkg-scene/src/lib.rs:176,183` | `.unwrap()` on node lookup during transform propagation — panics on a corrupted/malformed deserialized graph with dangling parent references | DeepSeek ✅, Google ✅, Owl ✅ | Open |
| P2-7 | `cvkg-scene/src/lib.rs:390` | `merge_dirty_regions` rebuilds a `Quadtree` from scratch on every inner-loop iteration — O(N² log N) performance bottleneck under high dirty-rect counts | Google ✅, Owl ✅ | Open |
| P2-8 | `cvkg-layout/src/lib.rs` | Thread-local layout-cycle guard (`ACTIVE_LAYOUT_NODES`) is never cleared if the layout closure panics — false "cycle detected" warnings persist for the rest of the thread's life | Google ✅ | Open |
| P2-9 | `cvkg-layout/src/lib.rs:1779` | Focus-order partitioning treats any `tab_index <= 0` (including the semantically distinct `-1`, meaning "focusable but not tab-traversable") as part of natural tab order | Google ✅ | Open |
| P2-10 | `cvkg-layout` (Taffy) | Multiple `.unwrap()` on Taffy layout operations — panics on invalid layout input | Owl ✅ (rated as fix priority #6) | Open |
| P2-11 | `cvkg-themes/src/lib.rs:35` | `from_rgb` accepts negative/unbounded inputs; `to_linear`'s `powf(2.4)` on a negative base yields `NaN` | Google ✅ | Open |
| P2-12 | `cvkg-themes/src/lib.rs:~1085-1118` | APCA contrast binary search isn't guaranteed monotonic near `bg_lum=0.5` and can diverge or fail to return the best extreme when both black/white fall below threshold | DeepSeek ✅, Google ✅ (different specific framing, same area) | Open |
| P2-13 | `cvkg-runic-text/src/lib.rs:1980` | `char_at_cluster` calls `.chars().nth(glyph.cluster as usize)` but `cluster` is a **byte** offset, not a char index — breaks line-wrapping for non-ASCII text | Google ✅ | Open |
| P2-14 | `cvkg-runic-text/src/knuth_plass.rs:351` | Line-break length computed as a byte count, then used directly as a character-width multiplier — non-ASCII lines wrap prematurely | Google ✅ | Open |
| P2-15 | `cvkg-runic-text/src/global_cache.rs:31` | LRU `cache_order` push without dedup/removal of stale keys on update — list grows unbounded with stale pointers | Google ✅ | Open |
| P2-16 | `cvkg-runic-text/src/msdf.rs` | Missing validation that accumulated atlas height stays under `max_size` (line ~183); division-by-zero on zero/negative SDF spread (line ~121) | Google ✅ | Open |
| P2-17 | `cvkg-anim/src/lib.rs:286-297` | Reduce-motion accessibility flag only honored by `SleipnirSolver`; `Linear`, `Sequence`, `Parallel`, and shatter/slice animations ignore it entirely | DeepSeek ✅ | Open |
| P2-18 | `cvkg-anim/src/lib.rs:386` | `ProgressDriver::Scalar(t)` passes an *absolute* timeline value into a function expecting a *delta* — can cause physics-driven animations to explode when combined with scroll-based scalar drivers | Google ✅ | Open |
| P2-19 | `cvkg-anim/src/advanced_particles.rs` | Color ramp uses the same curve for all 4 RGBA channels — particles render monochromatic; rounded-rect SDF half-dimensions can go negative when `radius > half-width` | DeepSeek ✅ | Open |
| P2-20 | `cvkg-physics/src/world.rs:275-290` | Variable-timestep mode passes an unclamped `dt` straight into substep division — a renderer lag spike causes `sub_dt` to explode, destabilizing the XPBD solver | Google ✅ | Open |
| P2-21 | `cvkg-flow/src/canvas.rs:314` | `node_at_screen` uses `HashMap::find()` (iteration order) instead of sorting by `z_index` — hit-testing picks an arbitrary node under overlapping nodes, not the topmost | Google ✅ | Open |
| P2-22 | `cvkg-flow/src/node.rs:164` | `with_tint_oklch` overwrites tint alpha to `1.0`, destroying the default `0.15` translucency — glass node material becomes fully opaque | Google ✅ | Open |
| P2-23 | `cvkg-compositor/src/engine.rs:248` | `flatten_layer` recurses (DFS) without a visited-set — a cyclic `LayerTree` reference causes an immediate stack overflow | Google ✅ | Open |
| P2-24 | `cvkg-macros/src/lib.rs:191` | Generated `build()` calls `.expect("missing required field ")` (note trailing space, no field name) on **every** struct field, making all fields mandatory with an unhelpful panic message | DeepSeek ✅, Google ✅ | Open |
| P2-25 | `cvkg-reflect/src/lib.rs:355` | `ColorStop::set_field` performs no range validation — `position` can be set to negative/NaN/huge values, violating the documented `[0,1]` constraint | Google ✅ | Open |
| P2-26 | `cvkg-render-native/src/lib.rs:2581` | `libc::setpriority(-10, ...)` privilege-elevation call has its failure silently ignored | Google ✅, DeepSeek (mentions same line area) | Open |
| P2-27 | `cvkg-spatial` (cell-span / NaN sort) | See §4.1 below — superseded findings, retained here as P2 historically | Specialist audit, DeepSeek (later pass marks **[FIXED]**) | **[FIXED-PER-SOURCE]** — see §4.1 |

### 4.1 cvkg-scheduler / cvkg-spatial / cvkg-materials — reconciled in detail

This trio was covered by **all five** sources, with the specialist audit (`cvkg-audit-scheduler-spatial-materials.md`) being by far the most thorough (line-level, with fix snippets). The other four audits (DeepSeek, Pool, Google, Owl) each independently characterize these three crates as **clean / no bugs found**, which seemingly contradicts the specialist's findings — but on inspection this is **not a real conflict**: the specialist found only *latent robustness* issues (no crashes triggered under any test), which a lighter-touch review would reasonably pass over.

| # | Issue | Severity (normalized) | Status |
|---|---|---|---|
| 1 | **BVH NaN-centroid sort** — `bvh.rs:162`, `partial_cmp().unwrap_or(Equal)` on possibly-NaN centroid floats produces arbitrary tree ordering (no panic, but non-deterministic structure) | P3 | Specialist: open, with fix (`total_cmp()`). DeepSeek's later workspace summary marks this **[FIXED]** as bug-fix #1 in the scheduler/spatial/materials section — **unverified**, treat as fixed-pending-confirmation |
| 2 | **SpatialHash unbounded cell span / DoS** — `spatial_hash.rs:132-146`, pathological rect (`x=-1e9, width=2e9`) causes the insert/query loop to iterate ~4 billion cells, exhausting CPU | P2 (DoS surface) | Specialist: open, with fix (`MAX_CELL_SPAN` clamp). DeepSeek's later pass marks **[FIXED]** — **unverified** |
| 3 | **Quadtree subdivide precision** — subnormal-width rects could theoretically zero out half-extents during repeated subdivision; bounded by `max_depth=5` so only 32 subdivisions possible | P4 (theoretical only) | Open, low priority |
| 4 | **Elevation shadow test `.unwrap()` pattern** — test-only code calls `.unwrap()` on `ElevationLevel::shadow()` for Level1-5 (always `Some`); documented risk if the pattern is copied into production code including Level0 | P4 (informational/test-only) | Open — documentation fix only |

**Reconciliation note:** cvkg-scheduler itself (frame.rs/task.rs) has **zero findings across all five audits** — unanimous agreement this crate is clean: 0 unsafe, 0 production unwrap/expect, no race windows (everything is `&mut self`-gated), no task-stealing implementation to misuse. cvkg-materials is likewise unanimous: 0 unsafe, 0 production unwrap, pure data structs with sensible bounds-checking (`.clamp()`) and intentional unclamped HDR fields, documented.

---

## 5. Conflicts and Discrepancies Between Source Audits

These are cases where reconciliation could **not** produce a single confident answer, and a human/engineering follow-up is recommended before acting:

1. **cvkg-icons "all icons render black" (P1-11).** DeepSeek reports a concrete float-vs-0–255 `rgba()` bug. Owl, Google, and Pool all separately reviewed `cvkg-icons` and reported **no bugs found**. Given the bug as described would be immediately visually obvious (every icon black) and yet three reviewers missed it, this needs direct visual/code verification — it is either a real regression that's easy to miss in static-only review, or a misread by DeepSeek.

2. **cvkg-render-software rounded-rect SDF (P1-10).** DeepSeek's final workspace table marks this `[FIXED]`. Google's audit, run a day later, still finds the exact same defective formula at the same line number. Either DeepSeek's claimed fix was never actually applied/merged, or Google audited a stale/different checkout. **Action: verify current source directly rather than trusting either claim.**

3. **cvkg-webkit-server severity and nature of issue.** DeepSeek's aggregate ranking calls out a P0 stored-XSS via `/snapshot`. Owl's dedicated pass over the same crate finds no XSS but does find a P1 WASI permission/fuel-metering issue and a session-panic bug. Google finds a generic "directory traversal" risk via `ServeDir`. Pool doesn't mention this crate's web-facing surface at all. These are four different threat models for the same small crate — likely each audit looked at a different file within it (`main.rs` vs `wasm_server.rs`) rather than a true contradiction, but the XSS claim specifically has zero corroboration.

4. **`Arc::from_raw` unsafe cast severity (P1-6).** DeepSeek: MED. Pool: HIGH. Owl: reviewed the *same* invariant and concluded it's **sound by construction** (guarded by `downcast_ref`), recommending only a safety-comment addition rather than a structural fix. This is a genuine difference in risk assessment, not a missed location — all three cite `lib.rs:~3677-3678`. Recommendation: keep the structural fix (ID-based lookup instead of pointer serialization, as DeepSeek proposes) as defense-in-depth, but Owl's assessment that today's invariant already holds is plausible and lowers urgency.

5. **cvkg-flow elastic-easing / hue-wrap bugs "[FIXED]".** Only DeepSeek touched these files in depth and only DeepSeek claims a subagent fix with passing tests. No independent confirmation exists in any other source.

6. **Bug counts / "crates audited" claims disagree sharply.** DeepSeek's own summary claims ~180 files / ~100,000 LOC / ~120 findings across "20 fully audited crates." Owl claims ~80 files / ~35,000 LOC / 26 bugs. Pool claims all 28 workspace crates "audited or surveyed" but many entries are one-line "Bugs: None found" without per-line detail, suggesting a shallow pass. These scope claims are **not mutually reconcilable** — they represent different actual depths of review labeled with similar-sounding "complete" language. Treat coverage claims skeptically; the per-finding cross-referencing in §2–4 above is the more reliable signal of what was actually inspected by more than one reviewer.

---

## 6. Cross-Cutting Themes (all sources agree on direction, differ on detail)

### 6.1 Monolithic files needing decomposition
Unanimous across DeepSeek/Pool/Google/Owl that the following are oversized and mix unrelated responsibilities (line counts vary slightly by audit date/checkout):

| File | Approx. lines | Priority |
|---|---|---|
| `cvkg-core/src/lib.rs` | ~9,556–9,557 | P1 — universally called the #1 decomposition target (35+ proposed submodules per Owl) |
| `cvkg-render-gpu/src/renderer.rs` | ~6,611–6,637 | P1 |
| `cvkg-render-native/src/lib.rs` | ~4,276–4,277 | P2 |
| `cvkg-svg-filters/src/lib.rs` | ~4,020–4,021 | P2 |
| `cvkg-runic-text/src/lib.rs` | ~4,036–4,037 | P2 |
| `cvkg-layout/src/lib.rs` | ~2,810–2,811 | P1 |
| `cvkg-vdom/src/lib.rs` | ~2,340–2,341 | P2 |
| `cvkg-anim/src/physics.rs` | ~1,456 | P3 |
| `cvkg-themes/src/lib.rs` | ~1,308–1,309 | P2 |
| `cvkg-anim/src/growth.rs` | ~1,231 | P3 |
| `cvkg-physics/src/world.rs` | ~1,172 | P3 |
| `cvkg-render-gpu/src/material.rs` | ~1,226 | P3 |
| `cvkg-render-gpu/src/types.rs` | ~1,641 | P3 |
| `cvkg-cli/src/main.rs` | ~890 | P4 |
| `cvkg-scene/src/lib.rs` | ~834–835 | P4 |
| `cvkg-flow/src/canvas.rs` | ~662 | P4 |

### 6.2 Norse/fantasy theming → functional naming
All four broad audits independently flagged extensive Norse-mythology theming throughout `cvkg-core`, `cvkg-render-gpu`, `cvkg-themes`, `cvkg-components`, and demo crates (`Bifrost*`, `Gungnir*`, `Mjolnir*`, `Sleipnir*`, `Surtr`, `Berserker*`, `Yggdrasil*`, `Kvasir*`, `Odin*`, `Mani*`, `Tyr`, `Heimdall*`, `Valkyrie*`, etc. — 100+ identifiers per Owl's count). Owl produced the most exhaustive rename table; Pool and DeepSeek produced overlapping but smaller tables for the files they covered. There is strong cross-source agreement that this is purely a naming/discoverability issue, not a functional bug, and that a coordinated single-pass rename (not file-by-file) is the correct remediation given 200+ call sites are affected in `cvkg-core` alone.

### 6.3 `partial_cmp().unwrap()` NaN-panic pattern
Found independently and repeatedly across **at least 4 different crates** by **3 different audits**:
- `cvkg-core/src/lib.rs:3703` (Pool, Google, Owl-adjacent)
- `cvkg-runic-text/src/lib.rs:381` (Pool, Google)
- `cvkg-spatial/src/bvh.rs:162` (Specialist)
- `cvkg-vdom` LIS comparisons (mentioned generically by DeepSeek)

**Recommendation (P2, workspace-wide):** a single sweep replacing `.partial_cmp(...).unwrap()` / `.unwrap_or(Ordering::Equal)` with `f32::total_cmp()` wherever a deterministic total order is sufficient, rather than fixing each site individually.

### 6.4 Mutex/RwLock `.unwrap()` poison-panic pattern
Found across `cvkg-core` (multiple statics), `cvkg-vdom` (`Signal`), `cvkg-scene`, `cvkg-compositor`, `cvkg-webkit-server` (`wasm_server.rs`), and `cvkg-undo` test code. Same remediation pattern recommended everywhere: `.lock().unwrap_or_else(|p| p.into_inner())` / `.read().unwrap_or_else(|e| e.into_inner())`.

---

## 7. Consolidated Prioritized Fix List

### P0 — Fix immediately
1. System-state hash collision, `cvkg-core`/`cvkg-components` (§2, P0-1)
2. AspectRatio Y-center math bug, `cvkg-layout` (§2, P0-2)
3. *(unverified, re-check first)* Stored XSS via `/snapshot`, `cvkg-webkit-server` (§2, P0-3)
4. Dangling thread-local GPU pointer on panic, `cvkg-render-native/src/lib.rs:1248` (§2, P0-4) — RAII guard fix below
5. Unverified pipeline-cache bytes into `unsafe device.create_pipeline_cache`, `cvkg-render-gpu/src/renderer.rs:1138` (§2, P0-5) — checksum-before-load fix below

### P1 — Fix this sprint
6–18. See §3 in full (panic-prone resource lookups in GPU passes, WASI privilege/fuel issues, anim div-by-zero cluster, subpixel rendering bug, colorblind-sim bug, macro `vdom_id` collision, themes alpha overflow, shader-compile startup panic, `Arc::from_raw` hardening).

### P2 — Fix next 1–2 sprints
19–46. See §4 in full (SHA256 truncation, mutex-poison sweep, dependency-graph dedup, VDOM patch round-trip, scene ID collisions, layout cycle-guard leak, theme NaN/APCA edges, runic-text UTF-8 byte/char confusion cluster, anim reduce-motion gap, physics dt clamp, flow hit-test z-order, compositor cycle guard, macro builder panic message, reflect bounds validation).

### P3/P4 — Backlog / hygiene
- BVH NaN-sort and SpatialHash cell-span DoS guard (status: claimed fixed, unverified — confirm before closing)
- Quadtree subnormal-width theoretical edge case
- Elevation shadow `.unwrap()` documentation note
- Full monolithic-file decomposition program (§6.1)
- Full Norse-theming rename program (§6.2)

---

## 8. Suggested Remediation Snippets (carried forward from source audits, unverified against current source)

**Thread-local GPU pointer RAII guard** (`cvkg-render-native`, P0-4):
```rust
struct ThreadLocalGpuGuard;
impl ThreadLocalGpuGuard {
    fn new(raw: *mut cvkg_render_gpu::SurtrRenderer) -> Self {
        GPU_FRAME_PTR.with(|ptr| ptr.set(raw));
        Self
    }
}
impl Drop for ThreadLocalGpuGuard {
    fn drop(&mut self) {
        GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
    }
}
// Inside the locked render block:
let _guard = ThreadLocalGpuGuard::new(gpu as *mut _);
self.view.render(&mut renderer, content_rect);
```

**Pipeline-cache checksum gate** (`cvkg-render-gpu`, P0-5):
```rust
if verify_cache_checksum(&cache_data, expected_checksum) {
    Some(unsafe { device.create_pipeline_cache(&cache_data) })
} else {
    None
}
```

**Graceful resource-registry lookups** (`backdrop_region.rs` / `accessibility.rs` / `pyramid.rs`, P1-1..3):
```rust
let scene_tex = match ctx.registry.get_texture(RES_SCENE) {
    Some(v) => v,
    None => { log::error!("[BackdropRegion] Missing scene texture"); return; }
};
```

**Mutex poison recovery** (workspace-wide pattern, §6.4):
```rust
let guard = STATE_WRITE_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
```

**NaN-safe float sort** (workspace-wide pattern, §6.3):
```rust
values.sort_by(|a, b| a.total_cmp(b));
```

**BVH/SpatialHash fixes** (specialist audit, §4.1):
```rust
// bvh.rs — replace partial_cmp().unwrap_or(Equal) with:
ca.total_cmp(&cb)

// spatial_hash.rs — add a span clamp:
const MAX_CELL_SPAN: i32 = 1000;
let max_cx = max_cx.min(min_cx + MAX_CELL_SPAN);
```

---

*End of reconciled report. Severity labels are normalized per the mapping in the header; where source audits used non-standard labels (e.g. "INFO"), they were mapped to P4. This document does not itself re-run tests or re-read the current repository state — several "[FIXED-PER-SOURCE]" claims and the single-source P0/P1 items flagged above should be independently re-verified before being closed or escalated.*
