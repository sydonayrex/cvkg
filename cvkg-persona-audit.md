# CVKG Persona-Based Usability Audit

**Date:** 2026-07-03  
**Framework version:** 0.3.0  
**Methodology:** Multi-persona evaluation per `old/uiux_audit_plan.md`. 5 independent reviewers, 9 dimensions each, consolidated findings.

---

## Executive Summary

CVKG is a 33-crate, 182-component Rust UI framework targeting GPU-accelerated, reactive UIs with first-class WASM support. It achieves near parity with React/MUI in component coverage while offering native rendering performance through wgpu. The framework's strongest assets are its GPU charts (13 chart types), animation system (SpringParams, morph, text animations), and multi-renderer architecture. Its weakest areas are discoverability (Norse naming collisions stymie both AI and human users), documentation sparseness (no per-component reference docs, no dedicated docs site), and a steep initial learning curve that compounds naming opacity with a custom macro system.

**Overall persona fit:**

| Persona | Score | Verdict |
|---------|-------|---------|
| 1 — Design Engineer (iOS Game Dev) | 5/10 | Marginal: powerful rendering, but too much friction from SwiftUI |
| 2 — Vibe Coder (AI Agentic) | 3/10 | Poor: Norse naming blocks AI inference cold |
| 3 — Product Designer (shadcn/MUI Migrant) | 6/10 | Workable: good component coverage marred by missing forms, docs |
| 4 — Frontend Engineer (Engineering-First) | 7/10 | Best fit: type-safe, testable, predictable layout |
| 5 — Marketing Designer (Ad Creative) | 5/10 | Stalled: WASM bundle size unknown, export pipeline opaque |

---

## Persona 1: The Design Engineer (iOS Game Developer)

### Profile
SwiftUI/UIKit background building game UIs, HUDs, inventory screens. Needs 165Hz performance, particle effects, fluid animation, GPU pipeline control.

### Onboarding Experience
**Moderate.** The `View` + modifier pipeline mirrors SwiftUI's `View` + `.modifier()` pattern closely — an iOS dev will recognize `VStack { ... }` and `.padding()`. The `state!{}` macro and `hamr!` reactive system map roughly to SwiftUI's `@State` and `@Binding`. However, the Norse naming is immediate friction: `MjolnirSlider`, `BifrostTabs`, `HringrPagination`. An iOS dev who types `Slider(value: $val)` has no way to guess `MjolnirSlider`. No SwiftUI migration guide exists.

### Component Coverage
**Solid for game UI.** Inventory grids → `Grid`, `FlexBox`. HUD overlays → `Hud`, `ValhallaIndicator`, `WyrdHud`. Animated transitions → `Animated<V>`, `SpringParams`, `MorphBridge`. Particle effects → `Confetti`, `spinner`, `floating_text`. Missing: a dedicated game controller input ring (DPad analog), health-bar primitive, minimap widget. 80% of game UI patterns are covered; the remaining 20% requires writing custom `View` impls, which is straightforward.

### API Design & Ergonomics
**Good at the View layer, painful at the trait layer.** The `View` trait with `render(&self, renderer: &mut dyn Renderer, rect: Rect)` is clean and intuitive. Modifier chaining works. But implementing a custom `View` requires understanding `Never`, `Rect`, `Renderer` trait, and the `body()` / `render()` split — concepts absent from SwiftUI. The `Renderer` trait has 50+ methods (fill_rect, draw_text, stroke_rect, bifrost, gungnir, etc.), which is overwhelming. The macro system (`#[derive(View)]`, `state!{}`, `hamr!`) helps but documentation is insufficient to explain when each is needed.

### Visual Design Quality
**High out of the box.** Default theme uses dark glassmorphism (Bifrost) consistently. Color palette is OKLCH-based and aesthetically modern. Components have proper rounded corners, elevation shadows, hover states, and focus rings. The GlassMaterial system rivals Apple's NSVisualEffectView. GpuCharts (13 types) render beautifully with animated transitions.

### Theming System
**Excellent.** `OklchColor` → `oklch_to_color_theme()` generates a full semantic palette from a single seed color. `GlassMaterial` with `bifrost()` for frosted-glass backgrounds. `SemanticColors` map to 10 semantic roles. Theme system supports dark/light mode switching. For an iOS game dev, this is overkill — they want to set `[0.1, 0.1, 0.15, 1.0]` and go. The system handles it, but the API surface is larger than needed.

### Animation & Interaction
**Strong.** `SpringParams` (snappy/fluid/heavy/bouncy) are well-tuned. `MorphBridge` enables shape morphing. `TextAnimate`, `TypewriterEffect`, `NumberTicker`, `CardStack`, `RippleButton`, `ShimmerButton` — quality matches mobile game standards. Canvas-backed GPU rendering means animations don't jank regardless of DOM repaints. Missing: timeline-driven animation editor, keyframe sequence definition (only value-based spring/hard-transition), particle system API (Confetti is a single component, not a system).

### Accessibility
**Functional but incomplete.** `HlinAccessibility` provides a tree-based accessibility node builder with `AriaRole` enumeration, focus trapping, high contrast, reduced motion. `A11yInspector` renders a visual tree overlay. But: no screen reader testing documented, no automated a11y regression tests, focus management is manual (no auto-focus on mount), reduced motion support depends on component authors opting in.

### Performance
**Strong for 165Hz targets.** GPU-backed rendering eliminates DOM compositing. `PerfOverlay` provides real-time frame timing and P99 tracking. `TelemetryOverlay` shows VRAM usage, draw calls, and per-phase timings. Lack of a frame budget profiler (annotating per-component render cost) means devs who hit a budget will have to eyeball which component is expensive. No GPU timeline capture integration (no RenderDoc/NSight hooks).

### Gaps & Recommendations

| # | Issue | Impact | Recommendation |
|---|-------|--------|----------------|
| P1 | No SwiftUI migration guide | Blocks adoption | Write a side-by-side reference (SwiftUI ↔ CVKG equivalents) |
| P2 | Norse naming opaque to iOS devs | Discovery friction | Add `pub use` aliases: `pub use MjolnirSlider as Slider;` |
| P3 | No health-bar / minimap / DPad | Missing game UI primitives | Add `HealthBar`, `MiniMap`, `DPadControl` components |
| P4 | Renderer trait surface too large | Steep custom View learning curve | Document the 10 essential methods; mark advanced as `_advanced` |
| P5 | No timeline/animation editor | Hard to craft multi-step sequences | Ship a `SequencePlayer` or expose `Keyframe<T>` |

### Verdict: 5/10

CVKG has the raw GPU power and animation quality a game dev needs, but the naming friction and lack of game-specific primitives make SwiftUI with Metal shaders the easier path today.

---

## Persona 2: The Vibe Coder (AI Agentic Design System)

### Profile
Uses AI coding tools (Cursor, Copilot, Claude) to build apps fast. Needs a system where AI can infer correct usage from component names alone, compose themes without manual color picking, and produce WCAG-compliant UIs by default.

### Onboarding Experience
**Frustrating.** The very first hurdle: the AI sees `pub mod bifrost_tabs` and infers nothing about "tabs." `BifrostTabs` is a Norse mythological reference (Bifröst = the rainbow bridge). No amount of prompting helps an AI guess that. The AI tries `BifrostTabs("Home", "Profile")`, gets a type error, and has no path to recovery without reading source code. `MjolnirFrame`, `HringrPagination`, `HrungnirSegmented`, `RunestoneEditor`, `ScribingStone` — each is a cognitive dead end for an LLM.

### Component Coverage
**Difficult to assess even with tooling.** 182 files across 70+ `pub mod` exports. The AI must scan every module's public API to discover what's available. There's no prelude that re-exports common components. The Norse names mean the AI can't fall back on general knowledge (it knows "Tabs" but not "BifrostTabs"). Components that DO have standard names (Button, Checkbox, Calendar, Dialog) work fine when the AI discovers them, but the AI doesn't know they exist under Norse-named modules.

### API Design & Ergonomics
**Inconsistent for AI inference.** The modifier pipeline is predictable: `.padding(8.0).background(color).corner_radius(4.0)` chains as any AI expects. But then: `.on_click::<ClickEvent>(|| ...)` uses turbofish where a plain `.on_click(|| ...)` would be AI-friendly. The `state!{}` macro uses `state!{ count: i32 = 0 }` syntax that's unlike anything in the AI's training data (React uses `useState`, SwiftUI uses `@State`). An AI that's never seen a `state!{}` invocation will hallucinate the wrong syntax.

### Visual Design Quality
**Accidentally good.** When the AI successfully composes a component, the default dark glassmorphism theme ensures it looks professional regardless of layout choices. OKLCH seed-color theming means the AI can say `.theme(color_seed: [0.6, 0.3, 0.8])` and get a coherent palette — no manual color picking. This is the system's strongest AI-friendly property.

### Theming System
**AI-friendly.** A single `OklchColor` seed generates a full semantic palette deterministically. The AI can write `OklchColor::new(0.6, 0.15, 270.0, 1.0).to_rgba()` and get WCAG-compatible colors. `glass_material_to_gpu_patch()` converts abstractions to GPU uniforms — the AI doesn't need to understand wgpu. This is excellent.

### Animation & Interaction
**Predictable but verbose.** `Animated::new(content).spring(hash, params).duration(0.3).easing(Easing::EaseOut)` is a construction the AI can manage. But the `hash` parameter for spring identity is opaque — what hash? Why? The AI will guess `0` or `random()`, both wrong.

### Accessibility
**Invisible to AI.** The a11y system (`HlinAccessibility`, `AriaRole`) is a builder pattern that the AI will probably never discover because it's in `hlin_accessibility.rs` (Hlin = Norse goddess of protection). No component auto-registers with an accessibility tree. The AI must manually construct a11y nodes for each component — something no AI will think to do.

### Performance
**Irrelevant to this persona.** AI-generated prototypes don't hit performance ceilings.

### Gaps & Recommendations

| # | Issue | Impact | Recommendation |
|---|-------|--------|----------------|
| P0 | Norse names block AI discovery | Critical | Create an AI-friendly prelude: `pub use bifrost_tabs::BifrostTabs as Tabs;` for every component. Add a `cvkg::prelude` module. |
| P1 | No component index for AI to scan | High | Generate a `COMPONENTS.md` listing every component with its name, purpose, and example usage as a prompt-context file |
| P2 | `hash` parameter on spring is opaque | Medium | Default `hash` to `0` and document when override is needed |
| P3 | a11y is manual and hidden | Medium | Auto-register components in `AriaRole` tree when they have a `label` prop |
| P4 | `state!{}` syntax is unique | Medium | Document the exact grammar in a machine-parseable form (like a BNF comment) |
| P5 | No prelude module | High | Add `cvkg::prelude` re-exporting 20 most common components with standard aliases |

### Verdict: 3/10

The Norse naming convention is a hard blocker for AI agent adoption. An AI cannot infer `BifrostTabs` from general knowledge, cannot discover it through naming patterns, and gets no help from the module structure. Without an AI-friendly prelude and aliases, this system requires a human to translate names. For vibe-coding, this is a non-starter.

---

## Persona 3: The Product Designer (shadcn/MUI Migrant)

### Profile
React ecosystem background (shadcn/ui, MUI, Chakra). Wants production web apps with familiar component patterns, Tailwind-like composition, dark/light mode, responsive design, form handling, WCAG a11y.

### Onboarding Experience
**Mixed.** The View/modifier pipeline maps well to JSX chaining: `Button::new("Click").on_click(...).variant(ButtonVariant::Primary)` ≈ `<Button onClick={...} variant="primary">Click</Button>`. `FlexBox`, `Grid`, `Stack` map to CSS Flexbox/Grid mental models. But: no `style` prop — everything uses modifier methods. No CSS variables — theming is Rust code. The OKLCH color model is unfamiliar to designers used to hex/rgb/hsl. The `state!{}` macro feels alien compared to `useState`.

### Component Coverage
**Broad but shallow in places.** Card ✅, Button (multiple variants) ✅, Dialog ✅, Checkbox ✅, Radio ✅, Select ✅, Calendar ✅, DatePicker ✅, DataGrid ✅, TreeView ✅, Navigation (navbar, breadcrumb, tabs) ✅. Missing compared to shadcn/MUI: 
- Form with validation (`form_binder` and `form_validation` exist but are not component-integrated)
- Sheet/Drawer (no bottom-sheet or side-sheet)
- Accordion (no built-in)
- Command palette exists but is not keyboard-searchable out of the box
- Combobox and Autocomplete exist but are separate, unlinked
- Skeleton/shimmer loading states exist (`ShimmerButton`) but no generic Skeleton component

### API Design & Ergonomics
**Type-safe but verbose.** `Button::new("Save").on_click(|| save()).variant(Primary).disabled(!valid)` vs shadcn's `<Button onClick={save} variant="primary" disabled={!valid}>Save</Button>`. CVKG requires more keystrokes per prop. The builder pattern is consistent, which helps muscle memory. Missing: compound props (shadcn's `size="sm"` is `Button::new("X").small()` in CVKG — inconsistent). Missing: slot/children composition — CVKG uses generic `View: Sized` bounds which limit heterogeneous children without trait objects.

### Visual Design Quality
**Good for business software.** Default dark theme is professional. Glassmorphism is restrained (not overused). Typography uses system fonts (no custom font loading required). Color palette is muted — intentional. Components have proper hover, focus, active, disabled states. However: light mode aesthetics are less polished (darker shadows, less refined transitions). The `ThemeSwitch` component exists but default light mode feels like an afterthought.

### Theming System
**Powerful but unfamiliar.** OKLCH color theory is correct and produces superior results to hex-based theming. But every designer knows `--primary: #6366f1` — telling them `OklchColor::new(0.58, 0.23, 285.0, 1.0)` instead requires learning a new mental model. The `DesignTokens` (FONT_*, SPACE_*, RADIUS_*) are well-structured and comprehensive. `ThemeBuilder` would benefit from a CSS-variables export function so designers can preview in browser tools.

### Animation & Interaction
**Satisfying.** `Transition`, `Easing`, `SpringParams` match CSS transition-timing-function concepts. `HoverCard`, `Popover`, `Toast` (Sonner-style), `ContextMenu` — all animate smoothly. Missing: `motion` preset library (Predefined transitions for common patterns like "fade in", "slide up", "scale in").

### Accessibility
**Spotty.** Keyboard navigation exists (`FocusState`, `keyboard_nav` module) but is manual. No auto-focus-on-mount for dialogs. No ARIA live regions for toast/alert announcements. `Reduced motion` support exists but depends on each component checking `is_reduced_motion()`. WCAG contrast: OKLCH-based colors should score well, but no automated contrast checking is documented. APCA calculation exists (ApcaResult) but is not exposed as a dev tool.

### Performance
**Excellent for business apps.** Canvas GPU rendering means no DOM bottleneck. VirtualList supports large data sets. GpuCharts render thousands of points at 60fps. Frame budget is predictable. WASM target enables web deployment without Electron.

### Gaps & Recommendations

| # | Issue | Impact | Recommendation |
|---|-------|--------|----------------|
| P1 | No integrated form validation | High | Wire `form_binder` + `form_validation` into form components (Input, Select, Checkbox) |
| P2 | No Sheet/Drawer/Accordion | Medium | Add three missing structural primitives |
| P3 | CSS variables export missing | Medium | Add `ThemeBuilder::to_css_variables() -> String` for browser tool preview |
| P4 | Light mode less polished | Medium | Audit light mode palette; refine shadows, surfaces, transitions |
| P5 | No generic Skeleton component | Low | `Skeleton` wrapper that renders placeholder shapes for loading |
| P6 | No motion preset library | Low | Pre-built presets for fade/slide/scale/bounce |

### Verdict: 6/10

Component coverage is solid but form handling, missing primitives, and unfamiliar theming model raise the migration cost. A designer willing to learn OKLCH and the Rust build pipeline will find a powerful system. Most won't.

---

## Persona 4: The Frontend Engineer (Engineering-First Design)

### Profile
Software engineer who needs UIs to look professional without being a designer. Values type safety, predictable layout, clear error messages, minimal magic, testability, good docs.

### Onboarding Experience
**Best of the five.** Rust engineers appreciate type safety. `View` trait with explicit `render()` method eliminates hidden rendering logic. Modifier chain is predictable: each method returns `Self`, enabling `.padding(8.0).background(color).rounded(4.0)` — no surprises. Layout system (FlexBox, Grid, Stack) maps to CSS Flexbox/Grid mental model directly. Error messages from the type system are standard Rust — not enhanced, but not worse.

### Component Coverage
**Sufficient for professional apps.** Data display (DataGrid, TreeView, VirtualList, VirtualTable) covers enterprise needs. Form components (Input, Select, Checkbox, Radio, DatePicker, Calendar) handle CRUD. Navigation (Navbar, Tabs, Breadcrumb, CommandPalette) covers app chrome. Missing: settings page layout pattern, responsive breakpoint utilities (there's no `Responsive<T>` wrapper).

### API Design & Ergonomics
**Strong.** Builder pattern is Rust-idiomatic. Modifier chaining is discoverable through rust-analyzer autocomplete. The `View` trait's split between `body()` (layout declaration) and `render()` (imperative drawing) is a clean separation. Testability is good: `RenderContext` can be mocked, and `Snapshot` captures rendered output as a field map. `#[derive(View)]` reduces boilerplate.

However:
- `state!{}` macro consumes non-Copy types by value (surprising to Rust engineers used to `use_state` patterns)
- `hamr!` reactive system is not documented at the component level — you discover it exists when you need cross-component reactivity
- `Rect` struct has `zero()` instead of `default()` (surprising — documented but still a papercut)
- The `Never` type used for `body()` return in leaf components (`type Body = Never;`) is confusing — new users don't know when to use `()`, `Never`, or a concrete View type

### Visual Design Quality
**Good enough.** The default dark theme looks professional with zero effort. No designer needed. OKLCH seed-color theming means branding can be reduced to a single number (hue angle). This persona values "good enough" over "stunning" — CVKG delivers.

### Theming System
**Engineer-friendly.** `OklchColor::new(0.6, 0.15, 270.0, 1.0)` is deterministic and reproducible. `oklch_to_color_theme()` generates 10+ semantic colors algorithmically — no manual palette design. `GlassMaterial::default_glass()` produces usable glass effects without tuning. The `DesignTokens` constants are well-named and consistent.

### Animation & Interaction
**Expressive enough.** `Animated<V>` wrapper with SpringParams is composable and predictable. `Transition` enum covers enter/exit/change. For most business apps, this is sufficient. Over-engineered animations (Confetti, TypewriterEffect) are available but not required.

### Accessibility
**Documented but manual.** `HlinAccessibility` builder is explicit — no magic. `AriaRole` is typed (no stringly-typed roles). Focus management via `FocusState` is explicit. This persona appreciates explicit over magical. However: no `Accessibility` section per component, so the engineer must build their own a11y tree from scratch.

### Performance
**Excellent.** GPU rendering eliminates DOM. `PerfOverlay` gives frame timing. `VirtualTable` handles tens of thousands of rows. FrameManifest system allows per-crate phase scheduling. This persona can reason about performance predictably.

### Gaps & Recommendations

| # | Issue | Impact | Recommendation |
|---|-------|--------|----------------|
| P1 | Component docs missing per-component a11y guidance | Medium | Add `// Accessibility` section to each component's module doc |
| P2 | No responsive breakpoint utilities | Medium | Add `Responsive<T>` that takes `[T; 4]` for xs/sm/md/lg breakpoints |
| P3 | state!{} macro surprising for non-Copy types | Low | Document the ownership semantics clearly; consider `state!{ val: clone }` syntax |
| P4 | Never type for leaf Views is confusing | Low | Add `view_leaf!` macro that sets `type Body = Never;` automatically |
| P5 | No settings/dashboard layout patterns | Medium | Add `SettingsLayout` and `DashboardLayout` compound components |

### Verdict: 7/10

This is CVKG's strongest persona. Type-safe, explicit, testable, professionally styled out of the box. The framework's design philosophy (explicit over implicit, type-safe over dynamic) aligns directly with a senior Rust engineer's preferences.

---

## Persona 5: The Marketing Designer (Ad Creative)

### Profile
Works in marketing. Wants polished landing pages, product demos, interactive brand experiences. Values visual polish, smooth animation, easy branding, responsive layouts, WASM export.

### Onboarding Experience
**Hardest path.** This persona has the least tolerance for Rust toolchain setup and macro syntax. They expect a visual editor or a component browser, not `cargo new` and `state!{}`. CVKG has no design tool integration (no Figma plugin, no Webflow export). The only path to a landing page is writing Rust code, which most marketing designers do not do. Even the WASM export path requires understanding web-sys, wasm-bindgen, and bundler configuration.

### Component Coverage
**Niche-appropriate.** `BentoGrid` for magazine layouts. `Carousel` for product showcases. `Marquee` for brand ticks. `Card` with hover/expand interactions. `Visual/floating_text` for headline animations. `Confetti` for celebratory interactions. `Animated<V>` for page transitions. Missing: hero section primitive, testimonial card, pricing table, feature grid (3-column icon layout), sticky header.

### API Design & Ergonomics
**Prohibitively low-level.** This persona needs to place a "Buy Now" button and change its color. Instead they must: create a Rust struct, derive `View`, call `Button::new("Buy Now")`, chain modifiers, return it from `body()`, compile with `cargo build --target wasm32-unknown-unknown`, bundle with wasm-bindgen, serve via a web server. The `draw_text` / `fill_rect` / `bifrost` primitives in `Renderer` are powerful but inappropriate for this persona — they need 20 high-level layout components, not 50 drawing methods.

### Visual Design Quality
**High.** When someone with Rust skills assembles a page, the visual output is premium. GPU rendering means no font-rendering inconsistencies, no subpixel scroll jank. Bifrost glassmorphism rivals Apple's marketing materials. The `njord_theme` module and `ThemeSwitch` give proper brand integration.

### Theming System
**Excellent for branding.** A single `OklchColor` seed generates a complete brand palette. `oklch_to_color_theme()` creates light + dark variants. Designers can specify brand colors as OKLCH values (once they learn what OKLCH is). Missing: export to CSS custom properties for design handoff, brand font family configuration (uses system fonts only).

### Animation & Interaction
**Capable but no motion designer tools.** `SpringParams` produces beautiful, natural motion. `TextAnimate` and `TypewriterEffect` are engaging. `Confetti` adds celebration. But there's no timeline editor, no scrubber, no visual animation curve editor — all tools a marketing designer expects. Animations must be coded.

### Accessibility
**Unlikely priority.** Marketing landing pages traditionally deprioritize a11y. CVKG's manual a11y system means they'll likely skip it entirely, which is acceptable for this persona's context but not ideal.

### Performance
**WASM unknowns.** Bundle size is undocumented. A landing page with animations, images, and GPU charts could be 5 MB or 15 MB — nobody has measured. WASM startup time on mobile data is unknown. No lazy-loading or code-splitting infrastructure documented. This is a blocker for marketing sites where load time directly impacts conversion.

### Gaps & Recommendations

| # | Issue | Impact | Recommendation |
|---|-------|--------|----------------|
| P1 | No visual editor / component browser | Critical | Ship a web-based component gallery (`cargo run --example component_gallery`) that renders every component with props |
| P2 | WASM bundle size undocumented | High | Profile and document bundle sizes for a minimal page, a typical landing page, and a full app |
| P3 | No hero/pricing/feature-grid components | High | Add 4 landing-page-specific compound components |
| P4 | No CSS variable export for Figma handoff | Medium | `ThemeBuilder::to_css_variables()` — designers can copy-paste into a style guide |
| P5 | System fonts only | Low | Add custom font configuration to Theme |
| P6 | WASM startup performance unknown | High | Benchmark and document cold-start time on mobile/desktop WASM |

### Verdict: 5/10

The visual output quality is there, but the pipeline cost (Rust toolchain, WASM build, no visual tools) is prohibitive for the target persona. A marketing designer would choose Framer, Webflow, or even plain HTML/CSS over Rust for a landing page.

---

## Cross-Cutting Themes

### 1. Norse Naming (Blocking for Personas 2, 3; friction for 1, 4, 5)
The most impactful issue across all personas. `HlinAccessibility`, `BifrostTabs`, `MjolnirSlider`, `HringrPagination`, `HrungnirSegmented`, `RunestoneEditor`, `ScribingStone`, `WyrdHud`, `FreyrInspector`, `GerdTelemetry`, `GullveigInspector`, `IdunnPersistence`, `NjordTheme`, `SkadiScripting` — approximately 25% of the component surface uses Norse names an AI cannot guess and a human cannot remember.

**Impact:** Discoverability, memorability, AI inference, migration cost.

### 2. Documentation Gap (Affects all personas)
No per-component reference documentation exists in the crate docs. No user-facing docs site. No examples directory for common patterns. The `#[doc]` comments exist on most public items but there's no narrative documentation — no "Thinking in CVKG" guide, no migration guides, no architecture overview. AI agents have no documentation to retrieve.

**Impact:** Learning curve, onboarding time, adoption barrier.

### 3. Form Handling Immaturity (Affects personas 3, 4)
`form_binder`, `form_validation`, `form_controls` exist as separate modules but are not integrated into form components. There's no `Form<Values>` wrapper that handles submission, validation display, dirty state, and field-level errors automatically. Shadcn/MUI users expect form handling "out of the box."

**Impact:** Enterprise adoption, CRUD app readiness.

### 4. Light Mode Neglect (Affects personas 3, 5)
The default light theme is clearly a secondary concern. Shadow depths, surface colors, and transition timing differ in quality from the dark theme. For B2B SaaS adoption (persona 3, 4), light mode is primary.

**Impact:** Business app adoption, professional credibility.

### 5. Accessibility is Manual (Affects personas 3, 4)
No component auto-registers with the accessibility tree. Developers must manually construct `HlinAccessibility` nodes. Most won't. This means default output is not WCAG-compliant for screen readers, despite OKLCH colors providing good visual contrast.

**Impact:** WCAG compliance, enterprise procurement, legal compliance.

---

## Prioritized Recommendation Matrix

| Priority | Recommendation | Impact (personas) | Effort | Category |
|----------|---------------|-------------------|--------|----------|
| P0 | Create AI-friendly prelude (`cvkg::prelude` with standard aliases) | 2, 3 | 1 day | Discoverability |
| P0 | Generate component index (`COMPONENTS.md`) | 2, 3, 4 | 4 hours | Discoverability |
| P1 | Add `pub use` aliases for Norse-named components | 1, 2, 3, 4, 5 | 2 hours | Discoverability |
| P1 | Wire integrated form validation into Input/Select/Checkbox | 3, 4 | 2 days | Form handling |
| P1 | Document WASM bundle sizes and startup time | 5, 3 | 1 day | Performance |
| P2 | Add responsive breakpoint utilities (`Responsive<T>`) | 3, 4 | 1 day | Layout |
| P2 | Add Sheet/Drawer/Accordion components | 3 | 3 days | Components |
| P2 | Audit and improve light mode theme | 3, 5 | 2 days | Visual |
| P2 | Add CSS variables export function | 3, 5 | 4 hours | Theming |
| P3 | Add per-component Accessibility section to docs | 4 | 2 days | Docs |
| P3 | Auto-register components in AriaRole tree | 3, 4 | 1 day | Accessibility |
| P3 | Add hero/pricing/feature-grid landing components | 5 | 2 days | Components |
| P3 | Publish web component gallery (wasm app) | 5, 1, 2 | 3 days | DX |
| P4 | Add SwiftUI migration side-by-side reference | 1 | 2 days | Docs |
| P4 | Add motion preset library (fade/slide/scale presets) | 3 | 1 day | Animation |
| P4 | Add generic Skeleton loading component | 3, 4 | 4 hours | Components |
| P5 | Add game UI primitives (HealthBar, MiniMap, DPad) | 1 | 2 days | Components |

---

## Design System Health Scorecard

| Metric | Score | Notes |
|--------|-------|-------|
| **Component Coverage** | 8/10 | 182 components cover most UI patterns; missing Sheet, Accordion, landing-page compounds |
| **Naming Consistency** | 5/10 | Dual naming (Norse + standard) creates confusion; 25% of components use opaque names |
| **Token Coverage** | 7/10 | DesignTokens exist (FONT_, SPACE_, RADIUS_) but aren't used uniformly across all components |
| **Accessibility Score** | 4/10 | OKLCH contrast is good but a11y tree is manual, no auto-registration, no automated tests |
| **Documentation Score** | 3/10 | Code doc comments exist but no narrative docs, no migration guides, no doc site, no examples directory |
| **Animation Quality** | 8/10 | SpringParams, MorphBridge, text animations are production-quality; missing keyframe system |
| **Theming Flexibility** | 9/10 | OKLCH seed-color system is best-in-class; GlassMaterial rivals native platform materials |
| **Performance Infrastructure** | 7/10 | PerfOverlay, Telemetry, GPU rendering excellent; missing per-component profiler and GPU capture |
| **WASM Readiness** | 5/10 | Multiple WASM demos exist, but bundle size, startup perf, and code-splitting are undocumented |
| **AI Agent Friendliness** | 3/10 | Norse naming blocks discovery; no prelude; no machine-parseable component index |

**Overall Health:** 5.9/10

---

*Audit conducted by 5 persona-reviewers against the full CVKG framework v0.3.0. Each persona evaluated onboarding, component coverage, API design, visual quality, theming, animation, accessibility, performance, and overall fit for their use case.*
