# CVKG Implementation Plan
**Derived from:** `uiux-prime.md` (Composite Prime Audit, CVKG v0.2.15)
**Goal:** Move CVKG from "impressive but not production-ready" to a shippable framework for at least one target persona (User 2: vibe coder / AI-agentic dashboard builder), per the audit's own recommended path.
**Target window:** 8-12 weeks core stabilization, with longer-tail platform work tracked separately.

---

## 0. Guiding Principle

The audit's "Honest Verdict" gives the order of operations — stabilize the core before investing further in surface features:

1. Stabilize the core (architecture, testing, CI)
2. Invest in the design system (naming, tokens, gradients)
3. Build one complete platform (rather than three partial ones)
4. Fix the AI agent experience (hamr!, derive macros, API spec)

This plan follows that sequencing, mapped onto the audit's P0/P1/P2 recommendation list (items #1–32) and the Lean UX hypotheses (H1–H7).

---

## Phase 0: Pre-work / Decisions (Week 0)

Before writing code, lock in decisions that later items depend on, to avoid rework:

| Decision | Why it blocks downstream work | Owner action |
|---|---|---|
| English-canonical naming policy (item #9) | Every module split (#2) and prelude change (#12) touches names — decide once | Approve naming map: `GjallarAlert→Alert`, `HatiSpinner→Spinner`, `RunesCard→Card`, `DraumaSkeleton→Skeleton`, etc. |
| Module boundary map for `cvkg-core` (item #2) | Renderer sub-trait enforcement (#3) and mock renderer (#10) need final module locations | Draft target file tree before extraction begins |
| CI scope (item #11) | Gate for every subsequent PR | Decide: GitHub Actions, matrix (stable/beta), required checks |
| Video export dependency (ffmpeg vs rav1e) (item #8) | Long-lead item, decide early to unblock procurement/licensing review | Confirm EPL-2.0/MPL-2.0 compatibility with chosen encoder |

---

## Phase 1: Core Stabilization (Weeks 1–4)

This phase addresses the audit's three named "architectural time bombs": the 9,014-line core, the 119-method Renderer trait, and opt-out reactivity — plus CI and a mock renderer, since nothing else can be safely tested without them.

| Order | Item (audit #) | Work | Acceptance criteria | Effort |
|---|---|---|---|---|
| 1 | #11 — CI workflow | Add `.github/workflows/ci.yml`: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace`, `cargo fmt --check` | Green run on a clean clone; required check on PRs | 1-2 days |
| 2 | #6 — `changed()` default fix | Flip default to `false`; add `View::needs_update()` escape hatch for views requiring per-frame updates | Static label/border draws 0 calls/frame after first render (H6 metric) in GPU profiler | 1-2 days |
| 3 | #5 — `#[derive(View)]` panic fix | Replace `unreachable!()` with `static_assertions` / compile-time check | Misuse of derive macro fails `cargo build`, not at runtime | 1-2 days |
| 4 | #2 — Split `cvkg-core/lib.rs` | Extract Renderer trait, layout, animation, geometry, color, events, focus, keyboard, clipboard, undo, virtual list, asset mgmt, knowledge graph, window mgmt, runtime, agents into separate files/modules per Phase 0 map | `lib.rs` under ~2,000 lines; `cargo check` time measurably improved | 2-4 weeks |
| 5 | #3 — Enforce Renderer sub-traits | Convert documented "aspirational" sub-traits (`RendererShapes`, `RendererText`, etc.) into real supertraits | A backend missing a capability fails to compile, not silently passes | 1-2 weeks |
| 6 | #10 — Mock Renderer | Implement `MockRenderer` recording `Vec<DrawCall>`; ship `assert_draw_call_count`, `assert_text_rendered`, `assert_color_at` | Test suite for 5 sample view components runs in <1s, no GPU context (H7 metric) | 1 week |

**Dependency note:** #4 (core split) should land before #3 (sub-trait enforcement) and #10 (mock renderer), since both touch Renderer trait location. #1 (CI) should land first so all subsequent PRs in this phase are checked.

**Phase 1 exit criteria:** CI green; `lib.rs` modularized; Renderer capability slices compile-checked; mock renderer in place; static views report 0 draw calls.

---

## Phase 2: Design System & Naming (Weeks 3–6, overlaps Phase 1 tail)

Addresses the single highest cross-persona-impact item (#9) plus the design-system gaps that block Users 3 and 5.

| Order | Item (audit #) | Work | Acceptance criteria | Effort |
|---|---|---|---|---|
| 7 | #9 — Canonical/alias swap | Make English names canonical per Phase 0 map; demote Norse names to `pub type` aliases; update error messages, doc examples, stack traces | Compiling a component by English name produces English names everywhere in diagnostics (H1 metric: time-to-first-component <15 min for non-Rust designers) | 2 days |
| 8 | #12 — Complete English aliases in prelude | Add `RunesTable`, `DraumaSkeleton`, `Sonner`/`ToastManager`, `FormField`/`Binding`/`FormBinder`, `Popover`, `RadioGroup` to `cvkg::prelude` | `use cvkg::prelude::*;` resolves all commonly-used components without Norse lookups | 1-2 days |
| 9 | #17 — Resolve `Binding` name collision | Rename `cvkg_components::form_binder::Binding` | No ambiguous-import errors when both crates are in scope | 1 day |
| 10 | #7 — GPU shader gradients | Replace CPU-tessellated 16-band gradient with WGSL fragment shader sampler; band count becomes quality slider (16-256) | Marketing-grade gradient quality rated >3/5 at 1920×1080 (H5 metric) with no visible banding | 1-2 weeks |
| 11 | #13 — Shadow system | Implement layered `box-shadow`, `text-shadow`, inset shadow, spread, as a View modifier | Designers can apply elevation shadows without custom shaders | 1-2 weeks |
| 12 | #14 — Easing curve library | Add standard CSS easing curves + cubic bezier alongside RK4 springs; `EasingAnimation<T>` over `TimingFunction` | Both spring-based and curve-based animations are first-class APIs | 3-5 days |
| 13 | #21 — Elevation/z-index system | 5-25 level elevation scale + documented z-index constants | Stacking conflicts resolved via documented levels, not magic numbers | 3-5 days |
| 14 | #23 — `Theme::toggle()` fix | Preserve custom palette modifications across light/dark toggle | Toggling theme twice returns to the exact prior custom state | 2-3 days |

**Phase 2 exit criteria:** Naming barrier resolved (audit's "highest-impact DX improvement"); gradient/shadow/easing gaps that blocked Users 3 and 5 closed.

---

## Phase 3: AI Agent Experience (Weeks 4–6, parallel to Phase 2)

Targets the audit's #1 cited AI-agent failure mode and other LLM-facing friction. Can run in parallel with Phase 2 since it's mostly isolated to the `hamr!` macro and docs.

| Order | Item (audit #) | Work | Acceptance criteria | Effort |
|---|---|---|---|---|
| 15 | #4 — `hamr!` conditionals/loops | Add `if`/`match`/`for` support directly, or ship `hamr_if!`/`hamr_for!` with clear error messages | LLMs (GPT-4, Claude, DeepSeek) generate a `VStack` with conditional visibility correctly on first attempt >90% of the time (H2 metric, up from ~10%) | 1-2 weeks |
| 16 | #30 — Machine-readable API spec | Produce JSON/TOML structured API reference | Tool-augmented agents can query component signatures without parsing Rust source | 1-2 weeks |
| 17 | #27 — "Recipes" docs | Master-detail view, settings page, login w/ validation, async loading patterns | New devs/agents have a working template for each common pattern | 1 week |
| 18 | #28 — Component docs for Navigation, Overlays, Animation, Multimedia | Fill documentation gaps flagged by Users 2/3 | Doc coverage parity with existing crate-level README template | 1-2 weeks |

**Phase 3 exit criteria:** H2 hypothesis validated via the minimum-viable experiment (3-model prompt test); machine-readable spec published.

---

## Phase 4: Production Completeness (Weeks 6–10)

Closes remaining P1 gaps needed for general production use (forms, responsive design, components) and the iOS path for User 1.

| Order | Item (audit #) | Work | Acceptance criteria | Effort |
|---|---|---|---|---|
| 19 | #15 — Form validation framework | Schema-based validation, error display, form state management | A login/signup form with validation ships without custom plumbing | 1-2 weeks |
| 20 | #16 — Responsive breakpoint tokens | Define sm/md/lg/xl/2xl constants; integrate with FlexiScope | Layouts respond to breakpoint changes without hand-rolled media-query logic | 1 week |
| 21 | #18 — DataTable, Toast, Tooltip, Skeleton | Build out remaining shadcn/MUI-parity components | Component parity checklist complete | 1-2 weeks |
| 22 | #19 — Wire animation engine to HUD components | RK4 spring transitions on `HealthBar`/`ManaBar`/`CooldownIndicator`; tween/fade on `DamageNumber` | HUD components animate smoothly under load, matching User 1's game-feel bar | 1 week |
| 23 | #20 — Configurable atlas size in `forge()` | Expose atlas size as a `forge()` parameter | iOS memory-constrained builds can shrink the atlas without forking the crate | 1-2 days |
| 24 | #3 (H3) — `cvkg-render-subview` `from_external()` | Implement `GpuRenderer::from_external(device, surface)`, no event-loop dependency | 3 integration patterns (iOS MTKView, Electron webview, creative-tool surface) compile and pass smoke test within 1 week of release (H3 metric) | per H3 |

**Note on iOS:** the audit frames iOS as 7-12 weeks of dedicated backend work beyond this item — `from_external()` is the unlock, not the full backend. Track iOS completion as its own follow-on workstream once this lands, rather than inside this 10-week window.

---

## Phase 5: Marketing/Export & Longer-Tail (Weeks 8–12, lowest urgency)

These items are real but the audit scores marketing-designer fit lowest overall (3.0/10) and notes a visual editor (6-12 months) is the real unlock — out of scope for this plan's window. Track only the achievable subset:

| Order | Item (audit #) | Work | Acceptance criteria | Effort |
|---|---|---|---|---|
| 25 | #8 — Video export (MP4/WebM) | Extend `cvkg-export-raster` with frame-sequence + MP4/WebM via optional ffmpeg/rav1e | Exported clip plays correctly in standard players | 2-4 weeks |
| 26 | #31 — Lottie export | Ad-serving pipeline integration | Exported Lottie JSON renders correctly in a reference player | 2-3 weeks |
| 27 | #32 — Design token pipeline | Import from Figma Tokens / Style Dictionary | Round-trip a token set from Figma without manual transcription | 1-2 weeks |
| 28 | #24 — Scroll-linked spring physics | Enables scroll-driven interactive ads | Demo: scroll position drives a spring target smoothly | 1 week |

**Explicitly deferred (not in this plan's window):** visual WYSIWYG editor (H4) — 6-12 months, its own initiative; full iOS backend — 7-12 weeks, follow-on workstream after Phase 4 item #24; i18n framework (#25) — 2-3 weeks, deferred until an internationalization need is concrete; CHANGELOG/API stability policy (#26) — process work, schedule alongside Phase 1 CI rollout rather than here; `Send + Sync` on `ActiveAnimation` (#29) — bundle into Phase 1's core split as a small follow-up.

---

## Sequencing Summary (Gantt-style view)

```
Week:        0  1  2  3  4  5  6  7  8  9  10 11 12
Phase 0      █
Phase 1         █  █  █  █
Phase 2                █  █  █  █
Phase 3                   █  █  █
Phase 4                            █  █  █  █  █
Phase 5                                  █  █  █  █
```

Phases 2 and 3 overlap Phase 1's tail once the core split (#2) lands, since naming and `hamr!` work don't depend on Renderer internals. Phase 4 starts once Phase 1's mock renderer exists, so new components ship with tests from day one. Phase 5 is opportunistic — pull items forward only if Phase 1-4 finish early.

---

## Validation Checkpoints (tie back to Lean UX Hypotheses)

| Checkpoint | Hypothesis | Run when | Pass condition |
|---|---|---|---|
| Naming survey | H1 | End of Phase 2 | 3 designers without Rust background compile a component in <15 min |
| LLM codegen test | H2 | End of Phase 3 | 3 models, conditional-visibility prompt, >90% first-attempt success |
| Integration smoke test | H3 | End of Phase 4 | iOS MTKView, Electron webview, creative-tool surface all pass within 1 week of `from_external()` release |
| Static-view profiling | H6 | End of Phase 1 | 0 draw calls/frame for static elements in GPU profiler |
| Mock renderer adoption | H7 | End of Phase 1 | Full test suite for 5 view components runs <1s without GPU context |
| Gradient quality rating | H5 | End of Phase 2 | Marketing designer rates gradient >3/5 at 1920×1080 |

---

## Risk Register

| Risk | Source in audit | Mitigation |
|---|---|---|
| Core split (#2) introduces regressions across 35 crates | "Every developer ... must read, understand, and not break this file" | Land CI (#1) before starting the split; split incrementally per-subsystem with tests after each extraction |
| Renderer sub-trait enforcement (#3) breaks existing backends | Sub-traits currently "NOT enforced ... to avoid method ambiguity" | Prototype against one backend first; fix ambiguity before rolling out workspace-wide |
| Naming swap (#9) breaks external code depending on Norse names | Norse names are currently canonical everywhere (docs, errors) | Keep Norse names as permanent type aliases, never remove them — only swap which is canonical |
| Video/Lottie export licensing | EPL-2.0/MPL-2.0 mixed licensing noted in audit | Confirm encoder dependency license compatibility in Phase 0 before writing code |
| Scope creep into iOS full backend | 7-12 weeks estimated, separate from this plan's 8-12 week core focus | Treat `from_external()` (item in Phase 4) as the sole iOS-related deliverable in this plan; everything else is a follow-on workstream |

---

## What success looks like at the end of this plan

Per the audit's own framing: if Phases 1-4 land, CVKG should be production-ready for **User 2 (vibe coder / AI-agentic dashboard builder)** — already scored 7.2/10 and the audit's named "most adoptable today" persona — with the architectural debt, naming barrier, and AI-agent friction that were holding even that persona back resolved. Users 3 and 4 should see meaningful score improvement from the design-system and testing work in Phases 1-2. Users 1 and 5 remain gated on larger initiatives (full iOS backend, visual editor) intentionally deferred past this plan's window.
