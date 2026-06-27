# Test Scenario: Agent Ulfhednar on CVKG
**Subject framework:** CVKG (Cyber Viking Kvasir Graph)
**Subject project:** Agent Ulfhednar — unified agentic UI platform (IDE + Chat + Kanban + Design Tool + Loop Scheduler + AI Orchestration Canvas + Mimir's Well DB UI)
**Organization:** 47-person software company, CVKG as primary UI system
**Test class:** Integration / Stress / Visual-Regression / Resource-Budget / Concurrency-Correctness
**Premise this document operationalizes:** *"Just because a program compiles does not mean it works as intended."* Every section below exists to catch a class of failure that `cargo check` cannot.

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

All seven modules are live, visible, and interactive **simultaneously** in a single dockable workspace (think: an IDE with chat, kanban, canvas, and DB browser all as panels/tabs the user can split, float, or tile) — this is the source of nearly every hard problem in this scenario. This is not seven apps that happen to share a binary; it is one scene graph, one renderer, one frame budget, shared by seven domains of state.

### 1.1 Non-Negotiable Constraints (from the brief)

1. **VRAM is scarce and contested** — the same GPU is running local/remote AI models. The UI's own VRAM footprint must be minimized, not just "acceptable."
2. **Visual target: macOS Tahoe-like** — Liquid-glass/Mica-style translucency, depth, continuous corner radii, vibrancy-correct text contrast over blurred backgrounds, smooth physically-based motion.
3. **Universal content rendering** — SVG, GIF, video, raster images, slides, documents, PDF, markdown, "other common file types" must all render inside panels (chat attachments, design canvas imports, doc previews in the IDE, Kanban card attachments, Mimir's Well BLOB previews).
4. **Multi-agent UX must not degrade under concurrency** — N agents can be running, posting to chat, updating kanban cards, and mutating the orchestration canvas at once, and the UI must stay legible and responsive.
5. **Full wiring, no orphaned logic** — every component's event must reach the state it's supposed to mutate, and every state mutation must reach every view that depends on it, with correct ownership (no panics, no silent data loss, no use-after-free-equivalent logic bugs even though Rust prevents the literal memory bug).
6. **Compiling is not passing.** A green `cargo build` is the *entry ticket* to this test plan, not the exit criteria.

---

## 2. Personnel-to-Surface Mapping

A 47-person company touching this many CVKG subsystems will fragment ownership unless mapped explicitly. This mapping is itself a test input: Section 7's integration tests are organized along these seams, because **seams between owners are where wiring breaks**.

| Role (approx. headcount) | CVKG surfaces owned | Where they will break things if untested |
|---|---|---|
| Rendering/GPU engineers (4-6) | cvkg-render-gpu, cvkg-compositor, cvkg-svg-filters | VRAM regressions, texture atlas leaks, overdraw from glass effects |
| Layout/Components engineers (6-8) | cvkg-layout, cvkg-components, cvkg-themes | Reflow thrash, theme-toggle state loss, virtualization bugs in long lists (Mimir's Well tables, chat history) |
| Text/Typography engineers (2-3) | cvkg-runic-text | Mixed-script chat/code rendering, markdown-to-styled-run mapping, BiDi in RTL agent output |
| Animation/Physics engineers (2-3) | cvkg-anim, cvkg-physics | Kanban drag inertia fighting layout reflow, spring jank under high frame-time variance |
| Canvas/Graph engineers (4-5) | cvkg-flow, cvkg-spatial, cvkg-scene | Orchestration canvas and Design canvas both build on cvkg-flow/cvkg-spatial — divergent assumptions here cause the classic "two teams built two graph editors" bug |
| Macro/Tooling engineers (2) | cvkg-macros, cvkg-reflect | hamr!-generated views silently dropping event handlers, reflect-based inspector lying about live state |
| Data/Backend engineers (6-8) | Mimir's Well storage layer + its CVKG-facing adapter, cvkg-webkit-server | UI rendering stale data because the adapter doesn't propagate change events into the scene graph |
| Scheduler engineers (2) | cvkg-scheduler, Skuld module | Frame update ordering bugs that only appear when scheduler-driven UI updates collide with user input in the same frame |
| Accessibility engineer (1) | cvkg-accessibility | Screen-reader tree desyncing from seven simultaneously-mutating panels |
| QA/Test engineers (3-4) | cvkg-test, cvkg-certification | Authoring the very scenarios in Sections 6-9 |
| App/Product engineers (6-8) | cvkg (umbrella), cvkg-cli, cvkg-icons, cvkg-materials | Cross-module integration: this is where "all features must just work" is actually proven or disproven |
| Platform/Web engineers (3-4) | cvkg-render-native, cvkg-render-software, WASM demo targets | Fallback-path correctness when GPU is degraded/absent (rare but real on some dev VMs) |

**Test implication:** Section 7 ("Cross-Seam Integration Tests") requires pairing engineers from two different rows above for each test, specifically because each row's owner will have already convinced themselves their own slice works.

---

## 3. CVKG Knowledge Requirements (Prerequisite Gate)

Before any test in this document can be trusted, the team must demonstrate working knowledge of these CVKG internals — not API familiarity, but mechanism-level understanding. This is a gate, not a checklist to skim:

1. **View/state contract** — When does `changed()` fire, what triggers a re-layout vs. a re-paint-only pass, and what is the actual cost difference between those two paths in `cvkg-core` + `cvkg-render-gpu`.
2. **Scene graph dirty-rect semantics** — Exactly which subtree gets re-culled (`cvkg-scene` AABB culling) when one of seven docked panels mutates, and whether sibling panels are guaranteed *not* to re-layout as a result.
3. **Renderer trait surface** — Which `Renderer` methods a given visual effect (glass blur, vector shape, video frame, text run) actually calls, so engineers can predict GPU cost before profiling rather than after.
4. **Spatial index choice per use case** — When `cvkg-spatial`'s QuadTree vs BVH vs SpatialHash is correct: Kanban hit-testing (2D, sparse) vs Design Canvas selection (2D, dense, frequently mutated) vs Orchestration Canvas node-picking (2D, graph-structured) are *not* the same problem and using one index strategy for all three is a likely silent-performance-bug source.
5. **Ownership model for cross-panel shared state** — How agent state (one Hugin Chat thread, one Kanban card, one Hugsvinnur canvas node, all representing the *same* agent run) is owned without aliasing conflicts or duplicated, divergent copies. This must be a designed answer (single source of truth + view projections), not an emergent one.
6. **Frame budget arithmetic** — Given a target frame time (e.g. 16.6ms @ 60fps, or a deliberately relaxed budget while AI inference holds the GPU), what is each module's allotted slice, and what happens when one module (e.g. a video-playing chat attachment) blows its slice.
7. **Macro-expanded code is real code** — `hamr!`-generated views must be inspectable (via `cargo expand` or `cvkg-reflect`) by anyone debugging them; "the macro probably did the right thing" is not an acceptable team norm at this complexity level.

**Gate exit criteria:** each engineer on the project can, without notes, explain items 1-6 for the subsystem(s) they own, and at least one adjacent subsystem (per the pairing in Section 2).

---

## 4. Visual Target: "macOS Tahoe-like" — Operational Definition

"Looks like Tahoe" must be turned into testable properties, or it becomes an unfalsifiable design opinion that QA can't sign off on.

| Property | Test | Pass threshold |
|---|---|---|
| Background translucency (glass/Mica) | Render a panel over a busy background (e.g. Design Canvas with many colored shapes) using `cvkg-materials` Glass/Acrylic; sample text contrast over the worst-case background region | WCAG-equivalent contrast ratio ≥ 4.5:1 for body text *measured through the blur*, not on a flat mock background |
| Continuous corner radii | Inspect rendered corner geometry at 2x and 3x display scaling | No visible faceting/polygon edges at any tested scale factor; radius scales with panel size per Tahoe's squircle-like continuous curvature, not a fixed pixel radius |
| Depth/elevation hierarchy | Stack a floating panel (e.g. a detached Hugin Chat window) over docked panels | Shadow + blur depth matches the elevation scale defined for the project (see prior CVKG plan, Phase 2 item #21); no two panels at different declared elevations render with identical shadow depth |
| Vibrancy under motion | Drag a Kanban card or pan the Orchestration Canvas while a glass panel is on top | No tearing, no stale blur (blur source texture must update at the same cadence as the content behind it), no frame drops attributable to blur recompute alone |
| Color model consistency | Sample identical semantic colors (e.g. "danger red") across IDE, Chat, Kanban, Design Canvas | All four use the same `cvkg-themes` OKLCH token, not four hand-picked hex values that drift |
| Light/dark + accent consistency | Toggle system theme | All seven modules re-theme atomically in the same frame; no panel "left behind" showing the old theme for more than one frame |

**Anti-pattern this catches:** a team where each module owner approximates "looks kind of glassy" independently, producing seven subtly different translucency implementations that never quite match — the single most common failure mode in "looks like macOS" UI work.

---

## 5. VRAM / Memory Budget Test Plan

The brief is explicit: VRAM must be minimized *because the GPU is shared with AI inference*. This is the highest-stakes non-functional requirement in the whole scenario, and it needs hard numbers, not "use less memory" as a vibe.

### 5.1 Budget allocation (proposed, to be ratified by the rendering team)

| Subsystem | VRAM budget (idle, all 7 modules open) | VRAM budget (active stress, Section 6.4) |
|---|---|---|
| Texture atlas (icons, glyphs, UI chrome) | ≤ 64 MB | ≤ 96 MB |
| Glass/blur intermediate render targets | ≤ 48 MB (shared, ping-pong buffers reused across panels — not one set per panel) | ≤ 64 MB |
| Design Canvas vector tessellation buffers | ≤ 32 MB | ≤ 80 MB (large designs) |
| Video decode/playback buffers (chat/doc attachments) | ≤ 64 MB per concurrently-playing video, max 2 concurrent decode streams enforced by policy | ≤ 128 MB |
| Mimir's Well virtualized table/grid GPU buffers | ≤ 16 MB regardless of underlying row count (virtualization must decouple GPU cost from data size) | ≤ 16 MB (must NOT scale with dataset — this is the actual test) |
| Orchestration Canvas node-graph geometry | ≤ 24 MB up to 500 visible nodes | ≤ 40 MB at 2,000 nodes (see 6.4) |
| **Total UI VRAM ceiling** | **≤ 280 MB idle** | **≤ 450 MB under stress** |

These numbers are illustrative starting targets for the team to validate against actual hardware (e.g. an 8GB or 16GB unified-memory or discrete VRAM target device) — the structural requirement that matters for testing is **the ceiling exists, is measured, and is enforced in CI**, not the exact byte counts.

### 5.2 Required tests

1. **T-MEM-01 — Atlas leak test.** Open and close each of the 7 modules 200 times in sequence. Assert texture atlas size returns to baseline ± 2% after each close, not monotonically increasing (classic sign of un-evicted glyph/icon entries).
2. **T-MEM-02 — Idle steady-state.** All 7 modules open, no user interaction, no agent activity, for 10 minutes. Assert VRAM usage is flat (±1%) — any drift indicates a per-frame allocation that should be cached/pooled but isn't (this directly tests the `changed()` default fix from the prior CVKG core-stabilization plan: a view re-allocating GPU resources every frame because it never reports "unchanged").
3. **T-MEM-03 — Glass buffer sharing.** Open all 7 modules with overlapping floating glass panels. Assert blur intermediate buffer count does not scale linearly with panel count — buffers must be pooled/shared, per the budget note in 5.1.
4. **T-MEM-04 — Mimir's Well scale-independence.** Load a 50-row table, measure GPU buffer size. Load a 5,000,000-row table (virtualized). Assert GPU buffer size is within 5% of the 50-row measurement. Any growth correlated with row count is a virtualization bug, not a "needs more VRAM" situation.
5. **T-MEM-05 — Concurrent video decode ceiling.** Attempt to play 5 videos simultaneously across Chat attachments and Design Canvas embeds. Assert the system enforces the policy ceiling (e.g. pauses/queues beyond 2 concurrent decodes with a visible, non-confusing UI state) rather than uncapped VRAM growth.
6. **T-MEM-06 — VRAM pressure cooperative yield.** Simulate the AI inference subsystem requesting a large VRAM allocation while Agent Ulfhednar is at its stress ceiling. Assert the UI can shed non-essential GPU memory (e.g. drop cached glass blur textures, fall back to flat translucency) without crashing or visually breaking, and recovers visual fidelity when memory pressure clears.

---

## 6. Content-Rendering Matrix

The brief requires rendering "SVG, GIFs, videos, text, images, slides, documents, PDFs, markdown files, and other common use files" — inside a framework whose core rendering crates (`cvkg-svg-filters`, `cvkg-svg-serialize`, `cvkg-runic-text`, `cvkg-render-gpu`) do not natively cover all of these. This section makes explicit which formats are native to CVKG vs. require an adapter, because **this is the most likely place for the team to discover a gap that should have been flagged at design time, not at integration time**.

| Format | Native CVKG path | Adapter needed? | Test |
|---|---|---|---|
| SVG | `cvkg-svg-filters` + `cvkg-svg-serialize` (read path needs confirming — serialize crate is write-oriented; an SVG *parse/render* path must exist or be built) | **Likely yes** — confirm whether an SVG parse-to-scene-graph path exists; if not, this is a gap to close before claiming SVG support | T-FMT-01: render a complex multi-layer SVG (icon set + a Design Canvas export) and diff against a reference rasterization |
| GIF | No native crate | **Yes** — decode to frame sequence, drive via `cvkg-anim` timeline or a dedicated sprite-sheet view | T-FMT-02: animated GIF in a chat attachment loops correctly, frame timing matches source, memory does not grow per loop |
| Video | No native crate | **Yes** — external decode (e.g. platform media framework) feeding GPU texture updates through `cvkg-render-gpu` | T-FMT-03: scrub, pause, and resize a video panel; assert no decode-pipeline desync and VRAM stays within the 5.2/T-MEM-05 ceiling |
| Text | `cvkg-runic-text` (native, HarfBuzz/BiDi) | No | T-FMT-04: mixed LTR/RTL agent output in Hugin Chat shapes and wraps correctly |
| Images (raster: PNG/JPEG/WebP) | `cvkg-render-gpu` texture upload | No (decode libs needed, not CVKG's job) | T-FMT-05: large image (e.g. 8K screenshot attachment) downsamples for display without uploading full-res to GPU unless zoomed |
| Slides (e.g. pptx-derived) | No native crate | **Yes** — render to image/SVG per-slide upstream, display via image or SVG path | T-FMT-06: a 50-slide deck preview in the IDE's file panel scrolls smoothly via virtualization, not by keeping 50 full-res textures resident |
| Documents (e.g. docx-derived) | No native crate | **Yes** — render to a paginated text+image representation, reuse `cvkg-runic-text` for the text portions | T-FMT-07: a multi-page document preview paginates correctly and text remains selectable/searchable, not just a flattened image |
| PDF | No native crate | **Yes** — external PDF rasterizer/parser feeding pages as either vector (preferred, reuse SVG path) or raster fallback | T-FMT-08: PDF with embedded vector graphics renders crisp at zoom (vector path), not blurry (raster-only fallback used incorrectly) |
| Markdown | No native crate, but composes cleanly from `cvkg-runic-text` (styled runs) + `cvkg-components` (code blocks, tables) | **Partial** — markdown-to-view-tree compiler needed, but no exotic rendering tech required | T-FMT-09: a markdown doc with nested lists, tables, code fences, and inline math renders structurally correct, and a live-streaming markdown message (token-by-token from an agent) re-renders incrementally without full-tree rebuild per token |
| "Other common files" (e.g. CSV, JSON, log files) | Text path for raw view, `cvkg-components` DataTable for structured view | No (display logic, not new rendering tech) | T-FMT-10: a 1GB log file opens in the IDE without loading the full file into memory — virtualized line access required |

**Critical finding to flag for the project, not just test:** at least four of ten required formats (GIF, video, slides, PDF) and a fifth partially (documents) have **no native CVKG crate**. This must be raised as an architecture decision (build adapter crates now, vs. integrate a third-party decode library) before the test plan below can be executed — testing an adapter that doesn't exist yet is not possible, and discovering this gap during integration testing instead of design is exactly the kind of "compiles but wasn't actually built" risk this scenario calls out.

---

## 7. Cross-Seam Integration Tests (Wiring Correctness)

These are the tests for the brief's central warning: *"All features... must be fully wired and have impeccable logic flows."* Each test below targets a seam between two owners (Section 2) where a handler can exist, compile, and still not be connected to anything real.

| ID | Seam | Test | Failure signature if broken |
|---|---|---|---|
| T-WIRE-01 | Hugin Chat ↔ Munin Kanban | An agent in Chat says "create a task for X." Assert a Kanban card appears in the correct column within one frame-budget cycle of the agent message completing. | Card never appears; or appears but chat shows no acknowledgment (one-way wiring) |
| T-WIRE-02 | Munin Kanban ↔ Hugsvinnur Orchestration Canvas | Moving a Kanban card to "In Progress" should reflect as a state-color change on the corresponding orchestration node. | Kanban and canvas show contradictory states for the same logical task — classic divergent-copy bug from Section 3 item 5 |
| T-WIRE-03 | Hugsvinnur Orchestration Canvas ↔ Skuld Loop Scheduler | A recurring loop's next-run countdown shown in Skuld must match the same loop node's countdown badge in Hugsvinnur. | Two clocks drift because each module computes time-remaining independently instead of from one source |
| T-WIRE-04 | Forge IDE ↔ Mimir's Well | Editing a schema file in the IDE that defines a Mimir's Well table should prompt a live schema-diff in the Mimir's Well table view, not require a manual refresh. | Stale schema view; requires app restart to see changes (clear sign the adapter doesn't propagate `cvkg-scene` change events) |
| T-WIRE-05 | Sjofn Design Canvas ↔ cvkg-components | A component instance edited in the Design Canvas should be reflected if that exact component is also rendered live elsewhere in the app (e.g. a custom Kanban card style). | Visual drift between "design source of truth" and "shipped component," meaning the design tool is cosmetic only, not actually backing the live components |
| T-WIRE-06 | Multi-agent state ↔ all panels simultaneously | Spin up 12 concurrent simulated agents, each touching Chat, Kanban, and Orchestration Canvas at a randomized cadence for 30 minutes. Assert: no panel ever shows an agent in two contradictory states at once; no dropped events (cross-check an event log against final UI state). | Any state where querying "what is agent N doing" gives a different answer depending on which panel you ask — this is the core multi-agent UX correctness bar |
| T-WIRE-07 | Theme system ↔ all 7 modules | Toggle system theme while all modules are actively rendering content (video playing, agent streaming text, Kanban drag in progress). | Any module retains stale theme tokens past one frame; any module's *custom* palette overrides (per the `Theme::toggle()` fix noted in the prior plan) are lost on toggle |
| T-WIRE-08 | Accessibility tree ↔ live multi-panel mutation | With a screen reader active, run T-WIRE-06's 12-agent stress scenario. Assert the accessibility tree never exposes a stale or duplicate node, and focus does not silently jump away from the user's last interacted element when an unrelated panel updates. | Screen reader announces wrong/duplicate content, or keyboard focus gets hijacked by background agent activity — both are real, common bugs in busy multi-panel apps |

---

## 8. Strict Ownership & Resource-Usage Test Plan

The brief specifically calls for "strict object ownership and minimal resource usage," beyond Rust's compile-time guarantees (which catch memory-safety bugs but not design-level ownership bugs).

1. **T-OWN-01 — Single source of truth audit.** For each cross-panel entity (an "agent run," a "task," a "design component instance," a "DB row being edited"), document which crate/module owns the canonical state and confirm every other panel holds a *projection* (read-derived view), never a second mutable copy. Grep-level audit: search for any struct that duplicates fields from another module's canonical state without an explicit "this is a projection, sync via X event" comment.
2. **T-OWN-02 — Borrow contention under concurrency.** With 12 simulated agents (T-WIRE-06) plus live user interaction, profile for any `Mutex`/`RwLock` contention hot spots in the shared state layer. Assert no single lock is held across a frame boundary (a lock held into the render pass is a latent stutter/deadlock risk even if it "happens to work" today).
3. **T-OWN-03 — Dangling subscription test.** Open and close each floating/detachable panel (Hugin Chat windows can detach) 100 times. Assert that closing a panel actually unsubscribes its event listeners from the shared state bus — a panel that's visually closed but still subscribed is a resource leak and a source of "ghost updates" bugs.
4. **T-OWN-04 — Minimal resource usage under idle.** With all 7 modules open and zero agent activity, assert CPU usage (not just VRAM) drops to near-zero between frames — i.e., confirm the `changed()`-default fix means idle panels are not being re-laid-out or re-painted purely because a scheduler tick happens to run. This is a sister test to T-MEM-02 but on the CPU/frame-scheduling side.
5. **T-OWN-05 — Resource cleanup on agent termination.** Kill a running agent mid-task (simulated crash, not graceful shutdown). Assert all of: its Chat thread is marked terminated (not silently frozen), its Kanban card reflects a clear "failed/stopped" state, its Orchestration Canvas node stops animating its "in progress" indicator, and any GPU resources tied to its rendering (e.g. a live preview it was streaming) are released within one frame of termination detection — not "eventually," not "on next panel open."

---

## 9. Functional/Logic-Flow Tests Beyond Compilation

Direct response to "just because a program compiles does not mean it works as intended" — these are scenarios where the *logic*, not the syntax, is what's under test.

1. **T-LOGIC-01 — Out-of-order event arrival.** Deliver two state updates for the same Kanban card out of timestamp order (simulating network/agent jitter feeding the webkit-server bridge). Assert the UI converges to the *correct final state* (latest-timestamp-wins or explicit conflict resolution), not "whichever arrived last wins by accident of arrival order."
2. **T-LOGIC-02 — Partial failure mid-workflow.** In the Hugsvinnur Orchestration Canvas, run a 5-node agent chain where node 3 fails. Assert nodes 4-5 correctly show "blocked/skipped," not "pending" (which would imply they might still run) and not silently green (which would imply false success).
3. **T-LOGIC-03 — Race between user edit and agent edit.** User is editing a Mimir's Well row in the same moment an agent writes to that row. Assert there's a defined, tested resolution (e.g. optimistic-lock conflict prompt) rather than whichever write happens to land last in undefined fashion.
4. **T-LOGIC-04 — Loop scheduler boundary conditions.** Schedule a Skuld loop for a time that has already passed (clock skew / DST edge case), a loop with zero interval, and a loop whose previous run is still executing when the next trigger fires. Assert defined behavior for all three (no double-fire, no silent drop, no panic) rather than relying on "this case probably won't happen."
5. **T-LOGIC-05 — Macro-generated handler audit.** For every `hamr!`-declared view with an event handler, write a test that actually fires the event and asserts the bound state changed — not just that the view compiles and renders. This directly targets the brief's warning: a `hamr!` view can compile and render a button that does nothing if the macro's handler binding silently no-ops on a syntax form it doesn't fully support.
6. **T-LOGIC-06 — Design Canvas → live component drift detection.** Build a component in Sjofn, use it live, then modify the source design. Assert the system can correctly tell the difference between "drifted, needs sync" and "in sync" — a false "in sync" report when the two have actually diverged is worse than no sync feature at all.

---

---

## 10. False-Positive Detection: When Tests Pass But Nothing Actually Renders

**This section exists because of an observed failure mode in this exact project:** the test suite reports passing while a window opens with no GPU rendering occurring at all. This is the single most dangerous kind of result in this entire test plan, because it is *worse than a failing test* — it actively tells the team the system works when it doesn't. Every section above must be re-audited through this lens before its passing results are trusted.

### 10.1 Root cause hypothesis

Almost all of Sections 5-9's tests are written against **state and logic** (did the Kanban card appear, does VRAM stay flat, did the handler fire) rather than against **actual presented pixels**. If those tests run against:
- a `MockRenderer` (per the prior CVKG core-stabilization plan's Task 1.6) left wired in by default instead of swapped for the real GPU renderer, or
- a real `Renderer` implementation whose methods are being called but whose underlying `wgpu` device/surface/pipeline setup silently failed and is now writing into a dummy or detached target,

...then every state-correctness assertion in Sections 7-9 can pass legitimately while zero frames are actually presented to the screen. The agent running this scenario is not lying about the results — it is correctly testing the parts of the system it was told to test, and nobody told it to test "is the screen actually showing anything."

This is the test-plan equivalent of the brief's own warning: a test suite can compile and pass and still not test what it was supposed to test.

### 10.2 Required action: structured logging on system-critical crates

Add `env_logger` (or `tracing` + `tracing-subscriber` if the team prefers structured/filterable logs — either is acceptable, but pick one and apply it consistently) as a dependency to every crate in the GPU-critical path, with mandatory log lines at each lifecycle stage. A passing test run must be cross-checked against this log output, not against assertions alone.

| Crate | Required log lines (minimum) | What a missing line proves |
|---|---|---|
| `cvkg-render-gpu` | `wgpu::Instance` created; adapter requested + adapter info (name, backend); device + queue acquired; surface configured (format, size); each render pipeline created (name/id); each frame: command buffer submitted, **present called** | If "present called" never logs, no frame ever reached the screen regardless of what state tests say |
| `cvkg-render-native` | Window created (winit); surface created from window handle; event loop entered; each resize event + new surface config | If "surface created from window handle" never logs, the window and the renderer were never actually connected — this exactly matches the reported symptom (window opens, no rendering) |
| `cvkg-compositor` | Layer tree built (layer count); damage region computed per frame (region size, or explicitly "full repaint"); composite pass executed | If damage is always "full repaint" or always empty, damage tracking is broken even if visually nothing looks wrong yet |
| `cvkg-core` | `Renderer` trait implementation in use at startup (log the concrete type name) | **This is the single most important line in this table.** If startup logs `MockRenderer` instead of the real GPU renderer type, every downstream result is void by construction |
| `cvkg-scene` | Scene graph root attached to a live render target (target id/handle) | If the scene graph is never attached to a real target, it can mutate state correctly forever while drawing into nothing |
| `cvkg-svg-filters`, `cvkg-runic-text` | At least one successful draw-call dispatch per test run (count, not just "ran") | Confirms these subsystems were exercised by GPU calls, not bypassed entirely by a path that renders text/SVG as a no-op |

**Rule for the team:** these log lines are not optional debug aids to add "if useful later." They are test fixtures. A CI run that produces zero `present called` log lines is a failing run, full stop, regardless of how many `assert!`/`assert_eq!` calls passed.

### 10.3 Renderer-liveness tests (new, mandatory, run first)

These run **before** any test in Sections 5-9, and if any of these fail, the rest of the suite's results for that run are discarded as untrustworthy rather than reported as a pass:

1. **T-LIVE-01 — Concrete renderer type check.** At app startup, assert (via the `cvkg-core` log line in 11.2, or a direct type-id check if exposed) that the active `Renderer` implementation is the real GPU backend, not `MockRenderer` or any other test double. Fail loudly and immediately if not — do not let the suite proceed.
2. **T-LIVE-02 — Surface attachment check.** Assert the `cvkg-render-native` window surface was successfully created and configured (non-zero width/height, valid format) before any UI test runs. This is the exact check that would have caught the reported symptom: window opens, but the surface was never actually wired to the renderer.
3. **T-LIVE-03 — Frame presentation count.** Run the app for N seconds with the UI idle (no interaction). Assert the present-call counter (from the `cvkg-render-gpu` log line) is greater than zero and roughly matches the expected frame cadence for that duration. Zero presents over a multi-second run, with a window visibly open, is the precise bug being investigated right now and must become a permanent regression test.
4. **T-LIVE-04 — Pixel readback / non-blank check.** After the app reaches steady state with known content on screen (e.g. a themed panel with visible text), read back the actual framebuffer (via `wgpu`'s readback path, or platform screenshot capture as a fallback) and assert it is **not** a single constant color and **does** contain pixels matching the expected theme palette. A passing T-LIVE-03 with a uniformly blank or garbage framebuffer indicates frames are being presented but rendering nothing meaningful (e.g. clearing to background color and never drawing content) — a distinct failure mode from "no frames at all," and one that pure present-count testing would miss.
5. **T-LIVE-05 — Format-specific draw verification.** For each content type in Section 6's matrix, after the per-format test claims success, perform the same pixel-readback check on the specific panel region rendering that content. A markdown panel that "passes" T-FMT-09 by correctly building a view tree, but whose final pixels are blank because the draw calls for that view tree were never actually issued to the GPU, must be caught here, not assumed safe because the state-level test passed.
6. **T-LIVE-06 — MockRenderer quarantine check.** Audit the test harness configuration directly: confirm `MockRenderer` is only ever constructed inside `#[cfg(test)]` unit tests for individual components (its intended use, per the original core-stabilization plan), and never reachable from any code path used by the integration/stress/visual tests in Sections 5-10 or by the shipped application binary. Grep for `MockRenderer` construction sites and manually justify every single one.

### 10.4 Revised exit criteria addendum

Section 11.2's exit criteria is amended: **no test result from Sections 5-10 may be counted toward exit criteria unless T-LIVE-01 through T-LIVE-06 passed in the same run.** Any historical "passing" results obtained before this section was added must be considered void and re-run, since the reported symptom (window opens, no GPU rendering) means at least one prior run produced false positives — and there is no way to know how many without the instrumentation in 11.2 in place to check against.


## 11. Test Execution Plan & Exit Criteria

### 11.1 Sequencing

1. **Gate (Section 3)** — knowledge gate must pass before any test below counts as validated; a test passed by someone who doesn't understand the mechanism they're testing is not trustworthy.
2. **Format-gap resolution (Section 6)** — resolve the GIF/video/slides/PDF/document adapter-crate decision before attempting Section 6's tests; do not "test around" a missing adapter with a mock that doesn't reflect real behavior.
3. **Memory/VRAM (Section 5)** — establish baseline numbers early; every later phase should re-run T-MEM-01 through 06 as regression checks, since wiring and content-rendering work are the most likely things to silently regress VRAM.
4. **Content rendering (Section 6)** — once adapters exist, run per-format tests.
5. **Cross-seam wiring (Section 7)** — requires Sections 5-6 substantially working, since wiring tests assume content renders and memory is stable.
6. **Ownership/resource (Section 8)** — run continuously alongside Section 7, not after — ownership bugs surface fastest under the same concurrent-agent stress used in T-WIRE-06.
7. **Logic-flow correctness (Section 9)** — final gate; these are the tests most likely to reveal that something which "passed" Sections 5-8 individually still produces wrong end-to-end behavior.

### 11.2 Exit Criteria (what "ready to ship Agent Ulfhednar v1" means)

- All Section 3 gate items demonstrated by all engineers in their assigned area.
- All Section 5 VRAM tests pass against ratified budgets, under both idle and stress profiles, with CI regression tracking in place (not a one-time manual check).
- All ten Section 6 format tests pass, including the explicit resolution of the four-to-five-format gap.
- All eight Section 7 wiring tests pass, *including* the 30-minute 12-agent stress run with zero contradictory-state findings.
- All five Section 8 ownership tests pass, with the single-source-of-truth audit (T-OWN-01) documented per cross-panel entity type, not just spot-checked.
- All six Section 9 logic-flow tests pass with defined (not accidental) behavior at every boundary condition.
- Section 4's macOS-Tahoe visual properties pass measured contrast/geometry/consistency checks, not subjective "looks right" sign-off alone.

A build that compiles, runs the demo happy path, and looks visually plausible in a five-minute manual click-through **does not meet exit criteria**. Exit criteria is met only when the above is true under concurrent multi-agent load, over extended idle duration, and across every required content format.
