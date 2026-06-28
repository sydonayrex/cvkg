# CVKG Product Gap Analysis — PM Audit

> **Date:** 2026-06-28
> **Auditor:** Product Management Review
> **Scope:** CVKG v0.2.15 — all 38 workspace crates, demos, docs
> **Benchmarks:** React/Next.js, SwiftUI, Flutter, egui, Compose Multiplatform, Tauri, Slint

---

## Executive Summary

CVKG is an extraordinarily ambitious Rust UI framework with genuine GPU-first architecture, a 38-crate workspace covering scene graphs, physics simulation, SVG, text shaping (HarfBuzz + BiDi), node-graph editing, STL loading, and a rich component library. The **rendering, layout, animation, and theming foundations are production-grade** for a v0.2 release. However, when measured against what a developer needs to ship *a real app today* (not a demo), the following gaps create **P0–P1 blockers** for general-purpose adoption.

**TL;DR Scorecard (vs. industry baseline):**

| Category | Score | Notes |
|---|---|---|
| Rendering Backend | ⭐⭐⭐⭐⭐ | GPU + software, multi-backend — *best-in-class for Rust UI* |
| Layout System | ⭐⭐⭐⭐ | Taffy flexbox/grid parity — strong |
| Component Library | ⭐⭐⭐⭐ | ~170 covers 80% of common widgets — but see P1 gaps |
| State Management | ⭐⭐⭐ | ComputedSignal + State exist, but no context/provider, no async data |
| Routing / Navigation | ⭐⭐ | Menu component only — no URL router, no deep linking |
| Accessibility | ⭐⭐ | AccessKit native bridge + tree exist, focus screen-reader bridge — **not wired to all components** |
| Theming | ⭐⭐⭐⭐ | OKLCH + semantic tokens + reduce-motion/contrast/transparency — strong |
| i18n | ⭐ | Static global locale (OnceLock), no ICU, no date/number/currency formatting |
| Data Visualization | ⭐⭐⭐⭐ | 15 chart types — strong for v0.2 |
| DX / Onboarding | ⭐⭐ | README docs exist; **no tutorial book, no video, no IDE tooling, no LSP** |
| Dev Tooling | ⭐⭐ | Hot-reload server exists; no debugger, no inspector panel in-prod |
| Mobile / Web | ⭐⭐ | WebGPU via wasm32 works; **no iOS/touch-first layout指南, no Android target** |

---

## P0 — Blockers (Must Fix for Any Shipping App)

### P0-1. Router / Deep-Linking Missing
**Benchmark:** React Router, Flutter Navigator 2.0, SwiftUI NavigationStack, egui (community).

**Current state:** `cvkg-components/src/navigation.rs` has a `NavigationMenu` — a horizontal nav bar with sub-menus. This is NOT a router. There is:
- No URL-to-view mapping, no history stack (`push`/`pop`/`replace`)
- No route params, no nested routes
- No 404 handling, no lazy route loading
- No URL sync for tabs, modals, or detail panes

**Impact:** Apps cannot share links to screens, integrate with browser history, or support "back button on android". This is table-stakes.

**Recommendation:** Build `cvkg-router` crate with declarative route definitions, parameter extraction, nested layouts, and an optional history adapter (for web, use `web-sys` History API).

---

### P0-2. i18n Is Not Usable in Production
**Benchmark:** react-intl / Fluent, Flutter gen-l10n, SwiftUI Localization, ICU.

**Current state:** `lingua_tong.rs` is a static global (`OnceLock<Mutex<String>>`) with a `HashMap<String, String>` of translations and a basic `t(key)` lookup. Problems:
- **No ICU MessageFormat** — no pluralization (`one`/`other`/`few`/`many`), no gender, no select.
- **No date/number/currency formatting** (no ICU4X integration).
- **No locale negotiation** (no `Accept-Language` parsing, no locale fallback chains).
- **No lazy loading** of catalogs (everything held in a locked global HashMap).
- **Right-to-left** detection exists but BiDi layout isn't wired to layout engine (text shaping supports BiDi but flex direction won't flip).
- **No tooling** to extract/generate `.ftl`/`.json` catalogs.

**Impact:** Apps targeting >1 locale cannot ship. Enterprise/regulatory users blocked.

**Recommendation:** Integrate ICU4X or `fluent-rs`. Add MessageFormat interpolation, locale-aware formatters, and lazy catalog loading at app startup (from disk or HTTP). Add RTL layout flip in cvkg-layout. Provide CLI catalog extraction tooling.

---

### P0-3. Accessibility Not Wired End-to-End
**Benchmark:** React ARIA / ARIA Authoring Practices, SwiftUI a11y API, Flutter Semantics.

**Current state:** `cvkg-accessibility` exists with `AccessibilityTree`, `FocusManager`, `ScreenReaderBridge`. Components like `NavigationMenu`, `calendar.rs`, `combobox.rs` call `set_aria_role(...)`. However:
- **No component-level a11y audit** — many components (data_grid, virtual_table, charts, calendar, autocomplete, slider, dialog) lack real ARIA roles/properties.
- **Focus ring rendering** is inconsistent; `keyboard_nav.rs` exists but tab stops are not auto-derived from view tree.
- **Live regions** (aria-live) not implemented for async updates.
- **Screen-reader bridge** is platform-abstracted but untested with actual screen readers.
- No automated a11y test harness (no `tarpaulin` style scan, no axe-core equivalent).

**Impact:** Non-compliant with WCAG 2.1 AA; blocks public-sector procurement and excludes users.

**Recommendation:** Mandate ARIA role tab on every PR. Build automated a11y scanner in `cvkg-certification`. Run CI with screen-reader smoke tests (NVDA on Linux via speech-dispatcher). Adopt ARIA Authoring Practices Guide patterns.

---

### P0-4. Getting Started Friction & Missing Tutorial Book
**Benchmark:** docs.rs + mdBook (Tauri), Flutter codelabs, SwiftUI tutorials, egui book.

**Current state:** README has a mermaid graph + quick start. `docs/onboarding.md` is a 7-step terminal walkthrough with no architecture explanation. `docs/recipes/` and `docs/howto/` have limited content. Notable gaps:
- No equivalent of "Your First App" with step-by-step screenshots.
- No `cvkg new` CLI command that produces a runnable scaffold (scaffold.rs exists but isn't wired to a CLI entry).
- No "conceptual overview" document (signals → rendering → layout → platform).
- No YouTube video or interactive sandbox.
- Workspace compile time from cold: enormous (~38 crates). No `sccache` / `cargo-nextest` config suggestion.

**Impact:** Developer drop-off in first 30 minutes is extremely high.

**Recommendation:** Add `cvkg new <name> --template=<blank|dashboard|ai-copilot>` as a one-liner (templates already exist in scaffold.rs). Author a `src/docs/book/` mdBook with 8 chapters. Add `cargo-nextest` config. Ship a "starter" template pre-pinned to avoid workspace-wide rebuilds.

---

## P1 — High Priority (Required for Real-World Apps)

### P1-1. Context Provider / Dependency Injection
**Current state:** No `use_context<T>()` or `Provider<T>` pattern. State is created locally with `State::new()` and threaded manually. No global store (Zustand/Redux equivalent), no scoped providers.

**Recommendation:** Add a typed `Context<T>` (similar to `leptos::provide_context` or Yew's `ContextProvider`). Support scoped providers for multi-pane apps.

---

### P1-2. Async Data Fetching & Suspense
**Current state:** `AwaitVeil` exists (likely a loading overlay) but there is no Suspense boundary, no `use_future` hook, no request-dedup/caching, no error retry. No SWR/React Query equivalent.

**Recommendation:** Add `use_future` (with `Future` cancel-on-drop). Add `<Suspense fallback={view}>`. Add async error boundary integration.

---

### P1-3. Responsive Layout Primitives
**Current state:** `Flexiscope` has a local breakpoint-selection pattern but it's not a global system. No `use_breakpoint()` hook derived from viewport CSS-like breakpoints, no container queries, no mobile-specific layout helpers. Taffy supports flex but responsiveness is not declarative.

**Recommendation:** Add a global `use_breakpoint()` derived from viewport size (xs/sm/md/lg/xl). Provide responsive-first components (`<Show when={breakpoint.mdPlus()}>`). Document mobile layout patterns.

---

### P1-4. Theming & Dark Mode Persistence
**Current state:** OKLCH color science is strong, semantic tokens exist, accessibility overrides (reduce_motion/transparency/contrast) are built-in. However:
- No runtime theme switching API (`theme_switch.rs` exists but unclear if it's a one-shot or observable).
- No persistent theme storage (no `localStorage` sync on web, no `confy`/platform registry on native).
- No design-token export to CSS/Figma.

**Recommendation:** Add `ThemeProvider` with persisted preference. Add CSS/Figma token export in `cvkg-cli`. Document theming lifecycle.

---

### P1-5. Form Validation UX
**Current state:** `FormBinder<T>` + `FormValidation` exist with a `validate_all()` method. Rudimentary — no field-level error display binding, no schema validation (no Zod-equivalent), no async validation, no debounced validation on keystrokes.

**Recommendation:** Add a schema-validation crate (`cvkg-valid` with serde-compatible derive). Wire field-level error messages into inputs (see `SecureField`, `InputGroup`). Add per-field `on_blur` validation.

---

### P1-6. Data Grid Missing Features
**Current state:** `DataGrid` has columns, sorting, virtualization. Missing:
- Column resizing, reordering, pinning/frozen columns.
- Row selection (multi/single), row expansion.
- Inline cell editing.
- CSV/Excel export.
- Grouping/aggregation.

**Recommendation:** Prioritize column resize + row selection + CSV export as P1. Inline editing and grouping as P2.

---

### P1-7. Mobile / Touch Target Sizes
**Current state:** `cvkg-inputs` has touch input handling, but no documented minimum touch target sizes (44×44pt iOS HIG, 48×48dp Material). No haptic feedback API on touch. No gesture recognizer (swipe, pinch, long-press) abstraction.

**Recommendation:** Audit all interactive components for minimum 44×44pt targets. Add `Gesture` recognizer (tap, double-tap, long-press, swipe, pinch). Add haptic feedback abstraction.

---

### P1-8. Dev Tooling — Inspector & Debugger
**Current state:** `cvkg-cli` has a dev server with hot-reload and a dashboard HTML. `devtools.rs` exists. Missing:
- Runtime view-tree inspector (like React DevTools).
- Signal/state time-travel debugger.
- Performance overlay (frame time, draw calls, layout cost) — `perf_overlay.rs` exists but likely local-only.
- WGPU debug markers / render graph visualization.

**Recommendation:** Build a standalone DevTools panel (web-based, served by `cvkg-webkit-server`) that connects via WebSocket to a running app. Show view tree, signal graph, frame budget.

---

## P2 — Medium Priority (Competitive Parity)

### P2-1. Animation Orchestration
**Current state:** Spring physics, particles, morph, growth exist. Missing: staggered lists, layout animations (FLIP), gesture-driven animations (drag-to-dismiss), scroll-linked animations.

### P2-2. Rich Text Editor (WYSIWYG)
**Current state:** `RichText` is a display-only segment model. `TextEditor` is a plain multi-line editor. No WYSIWYG with inline formatting toolbar, no markdown input mode, no collaborative editing.

### P2-3. Maps (Real Geospatial)
**Current state:** `Map` is a tactical sonar-style grid (decorative). No tile-based map (MapLibre/Leaflet equivalent), no GeoJSON rendering, no markers/polylines.

### P2-4. Media Playback
**Current state:** `Video` and `Audio` components exist in `multimedia.rs`. Likely stubs — no actual decoder (no `ffmpeg`/`symphonia` integration), no playback controls, no streaming, no subtitle track.

### P2-5. Undo/Redo System
**Current state:** `cvkg-core/src/undo.rs` has `UndoManager` + `UndoGroup`. Not wired to forms, text editor, or node graph. Needs integration.

### P2-6. Drag and Drop
**Current state:** No DnD abstraction. `DropVault` exists but likely a file-drop zone. No reorderable lists, no cross-window DnD.

### P2-7. Internationalization — RTL Layout
**Current state:** BiDi text shaping exists in `cvkg-runic-text`. Layout direction (flex row ↔ row-reverse) is not auto-flipped for RTL locales.

### P2-8. Testing Utilities
**Current state:** `cvkg-test` has visual regression. Missing: component-level unit testing harness (mount + simulate click + assert rendered text), accessibility test helpers, async test utilities.

### P2-9. Package / Crate Publishing
**Current state:** `publish_dry_run.py` exists. No crates.io release yet. No semver policy documented. No MSRV (Minimum Supported Rust Version) policy.

### P2-10. iOS / Android Target
**Current state:** `platform-ios-feasibility.md` exists. No actual iOS crate. No Android JNI bridge. No mobile-specific shell.

---

## P3 — Nice to Have (Differentiators)

### P3-1. AI Copilot Integration
**Current state:** `AgentChat`, `AIWorkflowBuilder`, `PromptForge`, `MultiAgentOrchestrator` exist — these are genuinely novel. Polish and document as a flagship feature.

### P3-2. Physics-Based UI
**Current state:** `cvkg-physics` (XPBD rigid body) exists. Unique in UI frameworks. Could enable game-like interfaces.

### P3-3. Node Graph Editor
**Current state:** `cvkg-flow` is a full node-graph editor with bezier edges. Competitive with React Flow / Flutter Node Editor.

### P3-4. 3D Scene Integration
**Current state:** `cvkg-scene` has 3D camera, `cvkg-stl` loads STL files. Could enable 3D product configurators.

### P3-5. Collaboration / CRDT
**Current state:** `Collaboration` component exists. Could be expanded to real-time multi-user editing.

### P3-6. Design Token Export to Figma
**Current state:** `design-token-export.md` exists. Could be a CLI command.

### P3-7. WASI Headless Rendering
**Current state:** `niflheim-wasi` exists. Could enable server-side rendering for PDF generation.

### P3-8. Certification / Compliance Suite
**Current state:** `cvkg-certification` exists. Could be marketed as "built-in compliance testing".

---

## Recommended Roadmap (12-Month)

### Q1 2026 (Foundation for Adoption)
- [ ] **P0-4** Ship `cvkg new` CLI + mdBook tutorial (8 chapters)
- [ ] **P0-1** Build `cvkg-router` with URL sync + nested routes
- [ ] **P0-3** A11y audit of all 170 components + automated scanner in CI
- [ ] **P1-1** Context Provider / DI system
- [ ] **P1-4** ThemeProvider with persistence + CSS/Figma export

### Q2 2026 (Real-World App Readiness)
- [ ] **P0-2** Integrate fluent-rs / ICU4X, add formatters, lazy catalogs
- [ ] **P1-2** Async data fetching + Suspense
- [ ] **P1-3** Responsive layout primitives + mobile layout guide
- [ ] **P1-5** Schema validation + field-level error binding
- [ ] **P1-7** Touch target audit + gesture recognizers

### Q3 2026 (Competitive Parity)
- [ ] **P1-6** Data grid: column resize, row selection, CSV export
- [ ] **P1-8** DevTools inspector panel (WebSocket-connected)
- [ ] **P2-2** Rich text editor (WYSIWYG)
- [ ] **P2-4** Media playback (symphonia integration)
- [ ] **P2-6** Drag and drop abstraction
- [ ] **P2-8** Component testing harness

### Q4 2026 (Differentiation & Platforms)
- [ ] **P2-10** iOS shell (winit + Metal surface)
- [ ] **P3-1** Polish AI Copilot as flagship feature
- [ ] **P3-3** Node graph editor marketplace/templates
- [ ] **P3-5** Real-time collaboration (CRDT)
- [ ] Publish v1.0 to crates.io with semver policy

---

## Risks & Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Scope creep — 38 crates already, adding more | High | Freeze new crate creation until P0 items ship |
| Rust UI ecosystem fragmentation (Tauri, Slint, egui, Iced, Leptos) | High | Differentiate on GPU-first + AI + physics; don't compete on simplicity |
| Maintainer bandwidth (single-author feel) | High | Add CONTRIBUTING.md, good-first-issue labels, maintainer contact |
| Compile times | Medium | Document `sccache`, `cargo-nextest`, `mold` linker; consider workspace splitting |
| No production reference app | High | Ship a non-demo app (e.g., a settings/dashboard app with real routing, forms, i18n) |

---

## Conclusion

CVKG has **world-class rendering and animation foundations** that exceed most Rust UI frameworks and rival the GPU capabilities of Flutter/SwiftUI. The component library breadth (~170 components) is impressive for v0.2. However, the framework is currently **demo-grade, not app-grade**. The four P0 blockers (router, i18n, a11y, onboarding) must be resolved before any developer can ship a production application. The recommended 12-month roadmap prioritizes these foundations first, then builds toward competitive parity and differentiation.

**Bottom line:** CVKG is a *platform* looking for its *killer app*. Ship the P0s, build a reference app that uses every feature, and the framework's genuine technical advantages (GPU rendering, physics, AI integration, node graphs) will sell themselves.
