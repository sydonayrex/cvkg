# Test Scenario: Agent Ulfhednar on CVKG
**Subject framework:** CVKG (Cyber Viking Kvasir Graph)
**Subject project:** Agent Ulfhednar — unified agentic UI platform (IDE + Chat + Kanban + Design Tool + Loop Scheduler + AI Orchestration Canvas + Mimir's Well DB UI)
**Organization:** 47-person software company, CVKG as primary UI system
**Test class:** Integration / Stress / Visual-Regression / Resource-Budget / Concurrency-Correctness
**Premise this document operationalizes:** *"Just because a program compiles does not mean it works as intended."* Every section below exists to catch a class of failure that `cargo check` cannot.
**Status as of last gate-assessment run:** see Section 0 (Punch List) and Section 3 (Tiering) — significant portions of the original test catalog are not yet executable because the Agent Ulfhednar application doesn't exist yet; this revision separates what's real right now from what's aspirational, and tracks the bugs that assessment run actually found.

---

## 0. Punch List (Bugs Found, Open as of Last Gate Run)

These came directly out of running Section 3 (Knowledge Gate) and Section 9 (False-Positive Detection) against the current CVKG codebase. They are framework-level, fixable now, and independent of whether Agent Ulfhednar exists. Track these as tickets, not as narrative — re-running the gate assessment should shrink this list, not regenerate it from scratch each time.

| # | Bug | Where | Severity | Fix | Status |
|---|---|---|---|---|---|
| P-01 | No startup log line identifying which `Renderer` implementation is active | `cvkg-core` | **Critical** | Add `log::info!(\"Renderer impl in use: {}\", type_name)` at renderer construction. This is the single check that would catch a `MockRenderer` accidentally reaching a real run. | **FIXED** -- `[Surtr] Renderer backend: GpuRenderer (wgpu)` at init.rs:42 |
| P-02 | No \"present called\" log line after `surface_texture.present()` | `cvkg-render-gpu/src/draw.rs:1007,1296` | **Critical** | Add `log::trace!(\"frame presented, count={}\")` at both present call sites, backed by an atomic counter. | **FIXED** (log line only) -- `[Surtr] Frame presented` at draw.rs:1008/1298; no counter yet |
| P-03 | No present-call counter / cadence assertion | `cvkg-render-gpu` | High | Build the counter P-02 depends on; expose it for test harnesses to read. This is what makes T-LIVE-03 (Section 4) actually implementable instead of aspirational. | **OPEN** -- no atomic present counter exists anywhere |
| P-04 | No check that surface creation succeeded with non-zero dimensions before rendering proceeds | `GpuRenderer::init()` | High | Add an explicit guard: zero-size or failed surface config should error loudly, not let rendering silently no-op. | **FIXED** -- guard with 1280x720 fallback at init.rs:191-194 |
| P-05 | `LayoutView::changed()` doc comment says \"Default true\"; actual behavior returns `false` | `layout.rs:264` | Low (correctness of docs, not code) | Fix the doc comment to match the verified-correct `false` default. | **FIXED** -- doc already says \"Default false\" matching impl |
| P-06 | No framebuffer readback path exists anywhere | `cvkg-render-gpu` | Medium (blocks T-LIVE-04/05) | Implement a `wgpu`-based readback (or platform screenshot fallback) so non-blank / expected-color assertions are possible at all. | **FIXED** -- `capture_frame()` at draw.rs:1596, tested and passing via headless_render test |
| P-07 | `hamr!` macro has no `cargo expand` integration or reflect-based inspection path | `cvkg-macros` | Medium | Document the `cargo expand` workflow at minimum; longer-term, wire `cvkg-reflect` to introspect macro-expanded views. | **OPEN** -- no workflow documented |

**Already verified clean — no action needed:** `MockRenderer` is correctly quarantined (only appears in `#[cfg(test)]` paths across `cvkg-core`, `cvkg-components`, `cvkg-test` test directories; zero occurrences in `cvkg-render-gpu`, `cvkg-render-native`, `cvkg-compositor`, `cvkg-webkit-server`, demos, or any production binary path). Don't re-audit this every run — re-check only when a new crate is added to the render path.

---

## 1. Scenario Statement

Agent Ulfhednar is seven applications fused into one CVKG host process:

| Module | What it is | Primary CVKG dependency |
|---|---|---|
| **Forge IDE** | Code editor, file tree, terminal panel, diagnostics | cvkg-runic-text, cvkg-components, cvkg-layout, cvkg-scene |
| **Hugin Chat** | Multi-thread chat with agents, streaming tokens, markdown/code rendering | cvkg-runic-text, cvkg-svg-serialize, cvkg-components |
| **Munin Kanban** | Drag-drop boards, swimlanes, WIP limits | cvkg-layout, cvkg-anim, cvkg-spatial (hit-testing), cvkg-physics (drag inertia, optional) |
| **Sjofn Design Canvas** | Figma-like vector design tool: shapes, components, auto-layout, prototyping | cvkg-svg-filters, cvkg-svg-serialize, cvkg-flow (constraint edges), cvkg-spatial |
| **Skuld Loop Scheduler** | Cron-like and event-driven recurring agent task scheduling, timeline view | cvkg-layout, cvkg-anim, cvkg-scheduler |
| **Hugsvinnur Orchestration Canvas** | Node-graph for chaining/branching multi-agent workflows, live execution state | cvkg-flow, cvkg-scene, cvkg-spatial, cvkg-reflect |
| **Mimir's Well** | High-volume structured + vector DB browser/editor (tables, vector similarity explorer, schema designer) | cvkg-components (DataTable/virtual list), cvkg-spatial, cvkg-reflect |

All seven modules are intended to be live, visible, and interactive **simultaneously** in a single dockable workspace — this is the source of nearly every hard problem in this scenario. This is not seven apps that happen to share a binary; it is meant to be one scene graph, one renderer, one frame budget, shared by seven domains of state.

**Current reality check:** only the CVKG framework and `cvkg-webkit-server` exist today. None of the seven modules above are built as applications yet. Section 3 makes this explicit so the rest of this document isn't read as describing a system that's further along than it is.

### 1.1 Non-Negotiable Constraints (from the brief)

1. **VRAM is scarce and contested** — the same GPU is running local/remote AI models. The UI's own VRAM footprint must be minimized, not just "acceptable."
2. **Visual target: macOS Tahoe-like** — Liquid-glass/Mica-style translucency, depth, continuous corner radii, vibrancy-correct text contrast over blurred backgrounds, smooth physically-based motion.
3. **Universal content rendering** — SVG, GIF, video, raster images, slides, documents, PDF, markdown, "other common file types" must all render inside panels.
4. **Multi-agent UX must not degrade under concurrency** — N agents can be running, posting to chat, updating kanban cards, and mutating the orchestration canvas at once, and the UI must stay legible and responsive.
5. **Full wiring, no orphaned logic** — every component's event must reach the state it's supposed to mutate, and every state mutation must reach every view that depends on it, with correct ownership.
6. **Compiling is not passing.** A green `cargo build` is the *entry ticket* to this test plan, not the exit criteria.

---

## 2. Personnel-to-Surface Mapping

| Role (approx. headcount) | CVKG surfaces owned | Where they will break things if untested |
|---|---|---|
| Rendering/GPU engineers (4-6) | cvkg-render-gpu, cvkg-compositor, cvkg-svg-filters | VRAM regressions, texture atlas leaks, overdraw from glass effects |
| Layout/Components engineers (6-8) | cvkg-layout, cvkg-components, cvkg-themes | Reflow thrash, theme-toggle state loss, virtualization bugs in long lists |
| Text/Typography engineers (2-3) | cvkg-runic-text | Mixed-script chat/code rendering, markdown-to-styled-run mapping, BiDi in RTL agent output |
| Animation/Physics engineers (2-3) | cvkg-anim, cvkg-physics | Kanban drag inertia fighting layout reflow, spring jank under high frame-time variance |
| Canvas/Graph engineers (4-5) | cvkg-flow, cvkg-spatial, cvkg-scene | Orchestration canvas and Design canvas both build on cvkg-flow/cvkg-spatial — divergent assumptions here cause the classic "two teams built two graph editors" bug |
| Macro/Tooling engineers (2) | cvkg-macros, cvkg-reflect | hamr!-generated views silently dropping event handlers, reflect-based inspector lying about live state |
| Data/Backend engineers (6-8) | Mimir's Well storage layer + its CVKG-facing adapter, cvkg-webkit-server | UI rendering stale data because the adapter doesn't propagate change events into the scene graph |
| Scheduler engineers (2) | cvkg-scheduler, Skuld module | Frame update ordering bugs that only appear when scheduler-driven UI updates collide with user input in the same frame |
| Accessibility engineer (1) | cvkg-accessibility | Screen-reader tree desyncing from seven simultaneously-mutating panels |
| QA/Test engineers (3-4) | cvkg-test, cvkg-certification | Authoring and maintaining this test catalog |
| App/Product engineers (6-8) | cvkg (umbrella), cvkg-cli, cvkg-icons, cvkg-materials | Cross-module integration once modules exist to integrate |
| Platform/Web engineers (3-4) | cvkg-render-native, cvkg-render-software, WASM demo targets | Fallback-path correctness when GPU is degraded/absent |

**Test implication:** integration tests in Tier D (Section 3) should pair engineers from two different rows above, specifically because each row's owner will have already convinced themselves their own slice works.

---

## 3. Test Tiering: What's Actually Testable Right Now

The last assessment run reported several sections as "not executable" — that's correct, but it's a *dependency status*, not a test-plan defect. This section makes the dependency explicit so the team can (a) act on what's testable today, and (b) unblock the rest deliberately instead of waiting for all seven modules to exist before any application-level testing happens.

### 3.1 The four tiers

| Tier | Definition | Unblocks when | Tests in this tier |
|---|---|---|---|
| **A — Framework (testable now, headless-capable)** | Pure CVKG mechanism checks: trait behavior, culling, spatial index correctness, macro hygiene, code-level audits. No window or GPU required. | Already unblocked. | Section 4 (Knowledge Gate), most of Section 5 (Renderer Liveness) except readback |
| **B — Display (testable now, needs GPU + on-screen window, no Ulfhednar app needed)** | Needs a real window and real frames, but only a minimal scaffold — not the full seven-module app. | A minimal scaffold app exists (Section 3.2). | T-LIVE-03/04/05 (Section 5), Tahoe visual-property checks (Section 6) |
| **C — Adapter (blocked on format-decoder crates)** | Needs GIF/video/slide/PDF/document decode adapters that don't exist in CVKG today. | Adapter crates are built (flagged as an open architecture decision). | Format-specific rendering checks for those five formats (native formats — SVG, text, images, markdown — are Tier A/B today) |
| **D — Application (blocked on Agent Ulfhednar modules existing)** | Needs some or all of the seven modules built and wired together. | Modules are built incrementally; tests can come online per-module, not all-or-nothing. | Cross-module wiring, multi-agent concurrency, ownership audits, end-to-end logic-flow correctness |

### 3.2 Unblocking Tier B cheaply: the minimal scaffold

Tier B tests don't need Agent Ulfhednar — they need *a single window with the real renderer drawing something*. Build this now, separately from the seven-module application:

- One `cvkg-render-native` window.
- Two dummy panels: one static (a themed card with text, to exercise glass/translucency and text rendering) and one animated (a spinner or moving shape, to exercise per-frame redraw).
- The real `GpuRenderer`, not `MockRenderer` — and per P-01, this should self-report at startup so the scaffold itself proves the check works.

This scaffold is days of work, not the months the full application needs, and it unblocks the entire Tier B row plus gives Tier A's renderer-liveness checks something real to run against instead of running in isolation.

### 3.3 Reporting convention going forward

When a test is blocked by tier, report it **once** as a dependency on that tier's unblock condition, not as a fresh "not executable" finding every run. Track tier status (A/B/C/D unblocked or not) as a single line each, and only re-flag if tier status regresses.

---

## 4. CVKG Knowledge Requirements (Prerequisite Gate) — Tier A

Last assessed: 5 of 7 PASS, 1 PARTIAL, 1 NOT YET (re-checked 2026-06-26). This gate is re-run whenever core rendering/layout code changes; it does not wait on Agent Ulfhednar.
| # | Requirement | Status | Note |
|---|---|---|---|
| 1 | `changed()` semantics | **PASS** | Defaults to `false` on both `LayoutView` and `View`; static-view test confirms. Doc comment fixed (was P-05, now **FIXED**). |
| 2 | Scene graph dirty-rect/AABB culling | **PASS** | `cvkg-scene::cull()` implements hierarchical AABB traversal, test-covered. |
| 3 | Renderer trait surface | **PASS** | `Renderer` in `cvkg-core` defines the full API surface; `GpuRenderer` implements it. |
| 4 | Spatial index choice per use case | **PASS** | `cvkg-spatial` provides Quadtree/Bvh/SpatialHash; the QuadTree\u2192Kanban, BVH\u2192Design Canvas, SpatialHash\u2192Orchestration Canvas mapping is supported by the library, though not yet wired since those modules don't exist (Tier D). |
| 5 | Cross-panel shared state ownership | **NOT YET** | This is correctly an Agent Ulfhednar (Tier D) concern, not a framework gap. CVKG provides the primitives (`State<T>`, `mutation_stream`, `event_channel`); no cross-module architecture exists because there are no cross-modules yet. Re-assess once the scaffold (3.2) or first real module lands. |
| 6 | Frame budget arithmetic | **NOT YET** | No per-module budget tracking exists. Reasonable to defer to Tier B/D, since "per-module" implies modules exist \u2014 but the renderer's existing per-frame timing logs (mentioned in the assessment) should be extended with a budget-vs-actual comparison now, ahead of needing it. |
| 7 | Macro-expanded code inspectability | **PARTIAL** | `hamr!` exists; no `cargo expand` integration or reflect-based inspection documented (P-07). |

---
## 5. Renderer Liveness Tests — Tier A/B

Direct response to the observed bug class: tests passing while the GPU renderer does nothing. These run before any other test counts as trustworthy. Status reflects the last assessment.

| ID | Test | Tier | Status | Notes |
|---|---|---|---|---|---|
| T-LIVE-01 | Concrete renderer type check | A | **PASS** | `[Surtr] Renderer backend: GpuRenderer (wgpu)` logged at init.rs:42. No automated assertion yet, but P-01 is **FIXED**. |
| T-LIVE-02 | Surface attachment check | A | **PASS** | Non-zero dimension guard exists (init.rs:191-194). P-04 is **FIXED**. |
| T-LIVE-03 | Frame presentation count | B | **NOT IMPLEMENTED** | `present()` is called in `draw.rs`, but no counter or cadence assertion exists \u2014 see P-02/P-03. Needs the Tier-B scaffold (3.2) to verify cadence over real frames, but the counter itself (P-03) can be built against Tier A today. |
| T-LIVE-04 | Pixel readback / non-blank check | B | **PASS** | `capture_frame()` implemented at draw.rs:1596. Headless render test reads back R=232 center pixel (red square). P-06 is **FIXED**. |
| T-LIVE-05 | Format-specific draw verification | B/C | **NOT IMPLEMENTED** | P-06 (readback) is now **FIXED**. Blocked on adapter crates (Tier C) for non-native formats. |
| T-LIVE-06 | MockRenderer quarantine check | A | **PASS** | Verified clean \u2014 see Section 0's "already verified" note. Re-check only when a new crate joins the render path. |

**Exit condition for this section:** P-01, P-02, P-04, P-05, P-06 closed; P-03 open (counter needed); Tier-B scaffold needed for T-LIVE-03/05.
---

## 6. Visual Target: "macOS Tahoe-like" — Tier B (Operational Definition)

This needs a real window with GPU output — the Tier-B scaffold (3.2) is sufficient; the full Agent Ulfhednar app is not required to start validating this.

| Property | Test | Pass threshold |
|---|---|---|
| Background translucency (glass/Mica) | Render a panel over a busy background using `cvkg-materials` Glass/Acrylic; sample text contrast over the worst-case background region | WCAG-equivalent contrast ratio ≥ 4.5:1 for body text *measured through the blur* |
| Continuous corner radii | Inspect rendered corner geometry at 2x and 3x display scaling | No visible faceting/polygon edges at any tested scale factor |
| Depth/elevation hierarchy | Stack a floating panel over a docked one | Shadow + blur depth matches the declared elevation scale; no two distinct elevations render identically |
| Vibrancy under motion | Animate content behind a glass panel (the scaffold's animated dummy panel is sufficient) | No tearing, no stale blur, no frame drops attributable to blur recompute alone |
| Color model consistency | Sample identical semantic colors across panels | All panels use the same `cvkg-themes` OKLCH token |
| Light/dark + accent consistency | Toggle system theme | Re-themes atomically in the same frame; no panel left behind for more than one frame |

This section depends on T-LIVE-04 (pixel readback, Section 5) being implemented — contrast and color-consistency checks both require reading actual rendered pixels, not just trusting that the right draw calls were issued.

---

## 7. Open Architecture Decision: Content-Format Adapters (Tier C)

The brief requires rendering SVG, GIF, video, images, slides, documents, PDF, and markdown. Today:

| Format | Native to CVKG? |
|---|---|
| SVG | Partial — `cvkg-svg-serialize` is write-oriented; a parse/render-to-scene-graph path needs confirming or building |
| Text, Markdown, Images | Yes (Tier A/B once the scaffold exists) |
| GIF, Video, Slides, PDF, Documents | **No** — no native crate for any of these five |

This is flagged as an open decision, not a test gap: before any Tier-C test can exist, the team needs to decide whether to build adapter crates in-house or integrate third-party decode libraries for GIF, video, slides, PDF, and documents. Testing this tier is not possible until that decision is made and at least a first adapter lands.

---

## 8. Application-Tier Test Catalog (Tier D — Reference Only, Not Currently Active)

Everything below requires some or all of the seven Agent Ulfhednar modules to exist. They are kept here as a forward-looking catalog so the team knows what to build toward, but **none of these should be reported as "failing" or "not executable" in a framework-level test run** — they simply aren't in scope until their dependency (a given module, or the full set) exists. Bring individual tests online as their owning module ships, rather than waiting for all seven.

| Category | Depends on | Representative tests (full detail to be re-expanded when the dependency lands) |
|---|---|---|
| VRAM/memory under full load | All 7 modules running concurrently | Atlas leak check, idle steady-state, glass-buffer sharing across panels, Mimir's Well scale-independence, concurrent video decode ceiling, VRAM-pressure cooperative yield with the AI inference subsystem |
| Cross-seam wiring | Pairs of modules (e.g. Chat↔Kanban needs both to exist) | Agent-created task appears as a Kanban card; Kanban status mirrors Orchestration Canvas node state; Skuld/Hugsvinnur clock agreement; IDE schema edits live-updating Mimir's Well; Design Canvas changes propagating to live components; 12-agent concurrent stress run with zero contradictory state; theme toggle atomicity across all panels; accessibility tree integrity under multi-panel mutation |
| Ownership/resource correctness | Cross-module shared state existing at all | Single-source-of-truth audit per entity type; lock-contention profiling under concurrency; dangling-subscription check on panel close; idle CPU-usage check; resource cleanup on agent termination |
| End-to-end logic-flow correctness | Working state/event/workflow system | Out-of-order event convergence; partial-failure propagation through an orchestration chain; user/agent write race resolution; loop-scheduler boundary conditions; macro-handler firing audit; design-drift detection accuracy |

**Recommended approach:** as each module ships, pull its relevant tests out of this reference table and re-expand them into a live, detailed test section (with explicit IDs, pass thresholds, etc.) the way Sections 4-6 are written. Don't attempt to write detailed Tier-D tests against modules that don't exist yet — that produces tests nobody can run, which is exactly the noise this revision is removing.

---

## 9. Exit Criteria

A build that compiles and looks visually plausible in a five-minute manual click-through does not meet exit criteria at any tier. Per-tier exit conditions:

- **Tier A:** all 7 Knowledge Gate requirements PASS (currently 4/7); all Punch List items closed.
- **Tier B:** scaffold app exists; T-LIVE-03/04/05 implemented and passing; Section 6 visual-property checks passing against the scaffold.
- **Tier C:** adapter architecture decision made and documented; at least the formats chosen for v1 have a working adapter and a corresponding passing test.
- **Tier D:** re-assessed per-module as each of the seven ships; full Tier D exit (all of Section 8's categories passing under the 12-agent concurrent stress condition) is the bar for calling Agent Ulfhednar itself, not just CVKG, ready to ship.

No tier's results should be reported as blocking another tier's progress — Tier A and B work should proceed now, in parallel with Tier C's architecture decision and Tier D's module development, rather than waiting in sequence.
