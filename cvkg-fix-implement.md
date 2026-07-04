# CVKG Fix Implementation Plan

**Derived from:** `cvkg-persona-audit.md` (multi-persona usability audit, 2026-07-03)  
**Framework version:** 0.3.0  
**Overall health:** 5.9/10 → Target: 8/10  

---

## How to read this document

Each fix is organised by priority tier (P0 → P5). Every item follows the same structure:

| Field | Meaning |
|-------|---------|
| **Why** | The audit evidence that motivates this fix, including which personas are affected |
| **Rationale** | The design reasoning behind the chosen approach — why this fix, and why this way |
| **Implementation** | Concrete steps: files to touch, code to write, tests to add |
| **Success criteria** | How to tell when this fix is done and working |
| **Effort** | Estimated person-days or person-hours |

---

## P0 — Critical (blocks adoption for 2+ personas)

### P0.1 AI-friendly prelude (`cvkg::prelude`)

**Why.** Persona 2 (Vibe Coder, score 3/10) cannot discover components through Norse names. An AI agent asked to build a tab component searches for `Tabs` — it does not find `BifrostTabs`. The AI can infer function from standard names, but Norse names are semantically opaque. Persona 3 (Product Designer, 6/10) also benefits: `pub use MjolnirSlider as Slider;` means standard muscle memory works. Per the audit, this is "the single highest-impact fix."

**Rationale.** The original Norse names stay — they are part of CVKG's identity, documented in code, and tested. Aliases cost zero maintenance: they are `pub use` re-exports that the compiler inlines. A `cvkg::prelude` module re-exports the 25 most common components under standard names, importable in one line (`use cvkg::prelude::*;`). This is the same pattern `std::prelude` uses: the real names are unchanged; the prelude is a convenience.

**Implementation.**

```
1. Create cvkg-components/src/prelude.rs
   Re-export every component under its standard alias:
     pub use crate::bifrost_tabs::BifrostTabs as Tabs;
     pub use crate::interactive::button::Button;
     pub use crate::interactive::checkbox::Checkbox;
     pub use crate::interactive::select::Select;
     pub use crate::interactive::input::Input;
     pub use crate::interactive::textarea::TextArea;
     pub use crate::mjolnir_slider::MjolnirSlider as Slider;
     pub use crate::hringrpagination::HringrPagination as Pagination;
     pub use crate::hrungnirsegmented::HrungnirSegmented as SegmentedControl;
     pub use crate::radial_menu::RadialMenu;
     pub use crate::docking_workspace::DockingWorkspace;
     pub use crate::container::modal::Modal as Dialog;
     pub use crate::container::stacks::VStack;
     pub use crate::container::stacks::HStack;
     pub use crate::container::stacks::ZStack;
     pub use crate::container::scroll::ScrollView;
     pub use crate::container::flex::FlexBox;
     pub use crate::card::Card;
     pub use crate::data_grid::DataGrid;
     pub use crate::tree_view::TreeView;
     pub use crate::grid::Grid;
     pub use crate::toast::Toast;
     pub use crate::hover_card::HoverCard;
     pub use crate::popover::Popover;
     pub use crate::datepicker::DatePicker;
     pub use crate::calendar::Calendar;

2. Add `pub mod prelude;` to cvkg-components/src/lib.rs

3. Re-export from umbrella cvkg crate (cvkg/src/lib.rs):
     pub use cvkg_components::prelude::*;
```

**Success criteria.** `use cvkg::prelude::*;` compiles. AI agent asked to write a tab bar can discover `Tabs` instead of `BifrostTabs`. `rust-analyzer` resolves `Tabs` to `cvkg_components::bifrost_tabs::BifrostTabs`.

**Effort.** 1 day (including testing each alias compiles).

---

### P0.2 Component index (`COMPONENTS.md`)

**Why.** Persona 2 has no way to discover what components exist without scanning 182 source files. An AI agent cannot efficiently `grep -r "pub struct"` across 70 modules. A generated component index gives both AI and human a single file to reference. Persona 3 and 4 also benefit: they can see the full surface area at a glance.

**Rationale.** A hand-written index goes stale immediately. A generated one (via a build script or `cargo doc` scanning) is always correct. Since CVKG already uses `#[doc]` annotations on most components, the index can be extracted from doc metadata. The simplest approach: a one-shot script that walks `cvkg-components/src`, parses `pub struct` declarations with their doc comments, and emits a markdown table.

**Implementation.**

```
1. Write tools/gen_component_index.sh (or .py):
   - Walk cvkg-components/src/*.rs
   - For each file, extract:
       pub struct Name
       pub fn new(...)
       /// Doc comment
   - Emit markdown table: | Component | Description | Module |
   - Sort alphabetically by *standard* name (not Norse) if it has a prelude alias,
     otherwise by original name

2. Generate:
     ./tools/gen_component_index.sh > COMPONENTS.md

3. Add to CI: check that COMPONENTS.md is up-to-date on every PR
```

**Success criteria.** `COMPONENTS.md` lists every component with a one-line description. An AI agent can search for "tabs" and find it. CI fails if `COMPONENTS.md` is stale.

**Effort.** 4 hours (script + CI check).

---

## P1 — High (affects 3+ personas or blocks a critical workflow)

### P1.1 `pub use` aliases for Norse-named components

**Why.** 25% of the component surface uses Norse names. The audit found this affects all 5 personas — from blocked AI discovery (Persona 2) to iOS dev friction (Persona 1, who types `Slider` and gets nothing) to memorability (Persona 4, who can't remember if it's `MjolnirSlider` or `MjolnirFrame`). This is effectively a lighter-weight version of P0.1 without the prelude module. Where P0.1 creates a single import point, P1.1 creates aliases in each component's own module so that both the Norse name and the standard name resolve.

**Rationale.** Each `pub use X as Y` is a one-liner in the source module. The compiler generates identical code for both names. Tools like rust-analyzer will show both names in autocomplete. Existing code using Norse names continues to compile unchanged. New users can use whichever name they discover first.

**Implementation.**

```
For each Norse-named component, add a pub use alias in its module file:

  cvkg-components/src/bifrost_tabs.rs:
    pub use BifrostTabs as Tabs;

  cvkg-components/src/mjolnir_slider.rs:
    pub use MjolnirSlider as Slider;

  cvkg-components/src/mjolnir_frame.rs:
    pub use MjolnirFrame as Frame;

  cvkg-components/src/hringrpagination.rs:
    pub use HringrPagination as Pagination;

  cvkg-components/src/hringrpagination.rs:
    pub use HrungnirSegmented as SegmentedControl;

  cvkg-components/src/hlin_accessibility.rs:
    pub use HlinAccessibility as AccessibilityTree;

  cvkg-components/src/gerd_telemetry.rs:
    pub use GerdTelemetry as Telemetry;

  cvkg-components/src/freyr_inspector.rs:
    pub use FreyrInspector as Inspector;

  cvkg-components/src/gullveig_inspector.rs:
    pub use GullveigInspector as DevToolsInspector;

  cvkg-components/src/idunn_persistence.rs:
    pub use IdunnPersistence as Persistence;

  cvkg-components/src/njord_theme.rs:
    pub use NjordTheme as ThemeConfig;

  cvkg-components/src/skadi_scripting.rs:
    pub use SkadiScripting as Scripting;

  cvkg-components/src/wyrd_hud.rs:
    pub use WyrdHud as Hud;

  cvkg-components/src/runestone_editor.rs:
    pub use RunestoneEditor as CodeEditor;

  cvkg-components/src/runestone_decoder.rs:
    pub use RunestoneDecoder as Decoder;

  cvkg-components/src/scribing_stone.rs:
    pub use ScribingStone as Markdown;

  cvkg-components/src/valkyrie_indicator.rs:
    pub use ValkyrieIndicator as Indicator;

  cvkg-components/src/shield_wall.rs:
    pub use ShieldWall as SecurityGate;
```

**Success criteria.** Both `MjolnirSlider` and `Slider` resolve to the same type. Existing Norse-using code compiles. New user can type `Slider` and get autocomplete.

**Effort.** 2 hours (one alias per file, 18 files).

---

### P1.2 Integrated form validation

**Why.** Persona 3 (Product Designer) and Persona 4 (Frontend Engineer) both identified form handling as the single biggest gap. `form_binder`, `form_validation`, and `form_controls` exist as three separate modules with no integration. A user who wants a validated `<input>` must manually wire: form state → binder → validation rules → error display → submit handler. Shadcn/MUI users get this in one component: `<FormField name="email" rules={[required, email]}>`.

**Rationale.** Integration means: (a) `Input`, `Select`, `Checkbox` accept a `validation` prop that takes `Vec<ValidationRule>`, (b) error text is rendered automatically below the field, (c) a `Form<Values>` wrapper manages dirty/pristine/submitting state without manual `state!{}` scaffolding. The existing `form_binder` and `form_validation` modules provide the primitives — this fix wires them in rather than rewriting.

**Implementation.**

```
1. Extend Input, Select, Checkbox to accept:
     .validation(vec![Rule::Required("Email is required")])
     .validator(my_custom_fn)  // for cross-field validation

2. Create Form<Values> wrapper component:
     Form::new(|ctx| {
         let email = ctx.field("email").string().required();
         let age = ctx.field("age").number().min(18);
         form! { "Email: " (email.input()) (email.error()) }
     })
     .on_submit(|values| async { /* submit */ })

3. Wire form_validation error display into field components
   (render red border + error text below the field)

4. Add Form example to demos (adele-web or berserker)
```

**Success criteria.** A validated form can be written in ~15 lines without manual `state!{}` or error display wiring. Examples compile and display validation errors correctly.

**Effort.** 2 days (design + implementation + demo).

---

### P1.3 WASM bundle size and startup documentation

**Why.** Persona 5 (Marketing Designer) cannot evaluate CVKG for web deployment because bundle size and startup time are undocumented. Persona 3 (Product Designer) also needs to know the payload for web apps. Without this data, neither persona can make a build-vs-buy decision. The audit scored WASM Readiness at 5/10 specifically because of this documentation gap.

**Rationale.** The fix is measurement and documentation, not code change. Three representative apps (minimal, typical landing page, full dashboard) are built for WASM and measured. Bundle sizes are reported split by crate (core + GUI framework + app code). Cold-start time on mobile (Chrome on Moto G) and desktop is measured. The results go into a `WASM.md` document that future releases keep updated.

**Implementation.**

```
1. Define three measurement targets:
     - minimal: one Button + one Text ("Hello, CVKG!")
     - typical: BentoGrid + Carousel + Card grid + Form (Persona 5 landing page)
     - full: Dashboard with DataGrid + GpuCharts + Navigation + ThemeSwitch

2. Build each for wasm32-unknown-unknown with --release

3. Measure:
     - .wasm binary size (wasm-opt -O4)
     - JS glue size (wasm-bindgen output)
     - Total transferred (gzipped)
     - Cold-start: time to first frame on Moto G (Chrome DevTools throttling)
     - Cold-start: desktop Chrome

4. Write WASM.md with results table and analysis
```

**Success criteria.** `WASM.md` exists with reproducible measurement methodology and results for all three targets. A prospective user can estimate their app's payload.

**Effort.** 1 day (builds + measurement + documentation).

---

## P2 — Medium (meaningful quality-of-life improvements)

### P2.1 Responsive breakpoint utilities (`Responsive<T>`)

**Why.** Persona 3 and 4 both noted the lack of responsive design primitives. There is no way to say "show this layout on desktop, that layout on mobile." CVKG's FlexBox and Grid components have no breakpoint concept. For business apps (Persona 4) and landing pages (Persona 5), responsive layout is a hard requirement.

**Rationale.** A `Responsive<T>` wrapper takes four values for xs/sm/md/lg breakpoints and picks one based on available width (which the `Renderer` provides). This mirrors CSS media queries without introducing stringly-typed breakpoints. The breakpoint thresholds match Material Design: xs=0, sm=600, md=900, lg=1200. The component re-renders on resize via a signal.

**Implementation.**

```
1. Create Responsive<T> in cvkg-core or cvkg-components:
     pub enum Breakpoint { Xs, Sm, Md, Lg }
     pub struct Responsive<T> {
         values: [T; 4],
         current: Signal<Breakpoint>,
     }
     impl Responsive<T> {
         pub fn new(values: [T; 4]) -> Self;
         pub fn current(&self) -> &T;
     }

2. Implement View for Responsive<T>:
     - On first render, read available width from Renderer
     - Select value by breakpoint
     - Subscribe to resize signal for re-evaluation

3. Add helper for common patterns:
     .responsive() modifier on View
```

**Success criteria.** `Responsive::new([mobile_view, tablet_view, desktop_view, wide_view])` compiles and picks the correct variant on window resize. Example in demos.

**Effort.** 1 day.

---

### P2.2 Sheet / Drawer / Accordion components

**Why.** Persona 3 identified three missing structural primitives that shadcn/MUI provides out of the box: Sheet (side panel), Drawer (bottom sheet), Accordion (expandable sections). These are among the most-used components in business applications (settings panels, filter drawers, FAQ accordions).

**Rationale.** Each component follows existing patterns:
- `Sheet` = `Modal` + slide-in animation + side positioning (left/right)
- `Drawer` = `Modal` + bottom-attached + drag-to-dismiss
- `Accordion` = toggle group with expand/collapse animation + icon rotation

All three can reuse `Modal` for overlay/dismiss and `Animated<V>` for slide/expand transitions. No new rendering primitives are needed.

**Implementation.**

```
1. cvkg-components/src/container/sheet.rs:
     Sheet { title, children, side: Left|Right, width }

2. cvkg-components/src/container/drawer.rs:
     Drawer { children, snap_points: Vec<f64>, on_dismiss }

3. cvkg-components/src/accordion.rs:
     Accordion { items: Vec<AccordionItem> }
     AccordionItem { title, content, expanded }

4. Register in lib.rs and add to prelude
5. Examples in demos
```

**Success criteria.** All three components compile, render, and animate correctly. Examples show: settings sheet, filter drawer, FAQ accordion.

**Effort.** 3 days.

---

### P2.3 Light mode theme audit and improvement

**Why.** The audit noted light mode as "clearly a secondary concern." Shadow depths, surface colors, and transition timing are less refined than the dark mode. Persona 3 (Product Designer) and Persona 5 (Marketing Designer) both use light mode primarily (business apps, landing pages). A system that ships with a poor default light theme undermines professional credibility.

**Rationale.** The fix is a systematic pass through the `SemanticColors` generated by `oklch_to_color_theme()` for the light variant, tuning:
- Surface elevation colors (currently too similar to background)
- Shadow opacity (needs reduction — light mode shadows look heavy with dark-mode values)
- Border contrast (increase from dark mode values)
- Text contrast ratios (ensure WCAG AA across all surface levels)

The `apca_contrast()` function in `cvkg-themes` already exists — use it to verify each color pair numerically.

**Implementation.**

```
1. For each semantic role (surface, surface_elevated, text, text_muted, border, etc.):
     - Read the light variant value from oklch_to_color_theme()
     - Compute APCA contrast against expected background
     - Adjust OKLCH lightness (L) until WCAG AA is met
     - Document the target value

2. Tune shadow parameters:
     Reduce shadow opacity in light mode by ~40%
     Soften blur radius

3. Verify with ApcaResult in tests:
     assert!(contrast.ok_for_body_text());
```

**Success criteria.** Light mode components are visually comfortable at all elevation levels. All semantic color pairs meet WCAG AA. Shadow depth is proportionate (lighter in light mode).

**Effort.** 2 days.

---

### P2.4 CSS variables export function

**Why.** Persona 3 and 5 both want to preview CVKG themes in browser design tools (Figma, browser DevTools). Currently theming exists only in Rust code — a designer cannot copy a color value into a mockup without manually converting OKLCH to hex. A `ThemeBuilder::to_css_variables() -> String` method generates CSS custom properties (`--cvkg-bg`, `--cvkg-text`, etc.) that can be pasted directly into a browser or design tool.

**Rationale.** This is a pure function on `ThemeBuilder` that serialises the semantic color palette to CSS custom property format. It requires no new dependencies (string formatting only). The output is a static string that designers can copy without touching Rust.

**Implementation.**

```
1. Add to cvkg-themes/src/lib.rs:
     impl ThemeBuilder {
         pub fn to_css_variables(&self) -> String {
             let mut css = String::new();
             css.push_str(":root {\n");
             for (name, color) in self.semantic_colors() {
                 let [r, g, b, a] = color.to_rgba();
                 css.push_str(&format!(
                     "  --cvkg-{name}: rgba({r},{g},{b},{a});\n"
                 ));
             }
             css.push_str("}\n");
             css
         }
     }

2. Output uses hex when alpha = 1.0, rgba otherwise
```

**Success criteria.** `ThemeBuilder::default().to_css_variables()` produces valid CSS. Designer can copy-paste into browser DevTools and see CVKG colors.

**Effort.** 4 hours.

---

## P3 — Standard (important, lower urgency)

### P3.1 Per-component Accessibility section in docs

**Why.** Persona 4 (Frontend Engineer) must build the a11y tree manually for each component because no component documents its ARIA role, keyboard interactions, or focus behaviour. Adding a standard `// Accessibility` section to each component's module doc gives engineers the information they need without reading the source.

**Rationale.** Each component already has a doc comment. Adding a structured subsection:
```
/// ## Accessibility
/// - Role: `button`
/// - Keyboard: Enter/Space to activate, Tab to focus
/// - Focus: auto-focused on mount when `auto_focus` is true
/// - ARIA: `aria-label` from `label` prop, `aria-disabled` from `disabled` prop
/// - Reduced motion: respects `is_reduced_motion()` for press animation
```
This is documentation-only, no code changes. AI agents scanning docs can use this to generate correct a11y code.

**Implementation.**

```
1. Define the template:
     Role | Keyboard | Focus | ARIA | Reduced motion

2. Audit the 25 most common components (Button, Input, Select, Checkbox, Radio,
   Slider, Tabs, Dialog, Card, DataGrid, TreeView, DatePicker, Calendar, etc.)

3. Add ## Accessibility subsection to each module doc

4. For components with missing a11y features (e.g., Dialog without auto-focus),
   add a "### Missing" note so users know the limitation
```

**Success criteria.** 25+ components have documented accessibility behaviour. A developer can learn a component's ARIA role from its doc comment.

**Effort.** 2 days.

---

### P3.2 Auto-register components in AriaRole tree

**Why.** The audit scored Accessibility at 4/10 because developers must manually construct `HlinAccessibility` nodes for every component. Most won't, resulting in apps with zero screen reader support. If components auto-register when they have a `label` prop, default output is usable by screen readers out of the box.

**Rationale.** The `View` trait's `render()` method already receives a `&mut dyn Renderer`. The `Renderer` can expose an accessibility node builder (`renderer.a11y_node(id, role, label)`). When a component sets a `label` prop, it calls this method during render. Components that don't set a label skip registration (unlabeled elements are correctly omitted from the a11y tree). This is opt-in per component but requires no external builder.

**Implementation.**

```
1. Add to Renderer trait:
     fn register_a11y(&mut self, id: u64, role: AriaRole, label: &str);

2. In Button::render(), when self.label is Some:
     renderer.register_a11y(self.id, AriaRole::Button, &self.label);

3. Same for Input, Select, Checkbox, Radio, Slider, Tabs

4. Per-component: this is a one-line addition to each render() method
```

**Success criteria.** A `Button::new("Submit")` appears in the a11y tree without additional code. Screen reader can navigate the app.

**Effort.** 1 day.

---

### P3.3 Landing page compound components

**Why.** Persona 5 (Marketing Designer) needs Hero, PricingTable, FeatureGrid, and TestimonialCard for landing pages. Each is a compound component (multiple subcomponents working together). Without them, building a marketing page requires custom layout code for every section.

**Rationale.** Each component is a thin container that composes existing primitives:
- `Hero` = VStack with title text, subtitle, CTA button, optional background image
- `PricingTable` = Grid of PricingCards
- `FeatureGrid` = 3-column Grid of FeatureCards (icon + title + description)
- `TestimonialCard` = Card with quote, author, avatar slot

All use existing `Grid`, `Card`, `Button`, `Text` components. Zero new rendering primitives.

**Implementation.**

```
1. cvkg-components/src/landing/hero.rs
2. cvkg-components/src/landing/pricing_table.rs
3. cvkg-components/src/landing/feature_grid.rs
4. cvkg-components/src/landing/testimonial_card.rs
5. cvkg-components/src/landing/mod.rs
6. Register in lib.rs, add to demos/adele-web as a landing page example
```

**Success criteria.** A complete landing page can be assembled from 4 compound components + navigation. Demo compiles and renders.

**Effort.** 2 days.

---

### P3.4 Web component gallery (WASM app)

**Why.** Persona 5 (Marketing Designer), Persona 1 (Game Dev), and Persona 2 (Vibe Coder) all benefit from seeing CVKG components rendered in a browser without writing code. A `cargo run --example component_gallery` that builds a WASM app showing every component with its props panel would serve as both documentation and a design tool.

**Rationale.** The gallery app mirrors shadcn/ui's component page: sidebar navigation → component selection → live rendering with a props panel. It uses CVKG's own components (Tabs, Navigation, TreeView, Input) to build the UI — eating the dogfood. The WASM target means it deploys to any static host. The existing `adele-web` demo structure can be extended rather than starting from scratch.

**Implementation.**

```
1. Create cvkg-gallery component_gallery example:
     - Sidebar listing all components (from COMPONENTS.md index)
     - Main area renders the selected component with default props
     - Props panel: each component's public API surfaced as Input/Slider/Checkbox controls

2. Build for wasm32:
     cargo build -p cvkg-gallery --target wasm32-unknown-unknown --release

3. Deploy to GitHub Pages or document local serve instructions

4. Each component entry also shows its source code snippet
```

**Success criteria.** `cargo run --example component_gallery` serves a web page. All 182 components render. Props panel edits update the live preview.

**Effort.** 3 days.

---

## P4 — Low (nice-to-have improvements)

### P4.1 SwiftUI migration reference

**Why.** Persona 1 (iOS Game Dev, score 5/10) has no migration path from SwiftUI. A side-by-side reference showing SwiftUI code alongside its CVKG equivalent bridges the knowledge gap and gives the dev a concrete learning path.

**Rationale.** A markdown document in the repo (`docs/migration-swiftui.md`) organised by concept: state, layout, modifiers, navigation, animation, effects. Each row shows SwiftUI on the left, CVKG on the right. This is documentation-only and can be written incrementally.

**Implementation.**

```
Write docs/migration-swiftui.md covering:
   - @State ↔ state!{}
   - @Binding ↔ hamr! binding
   - VStack/HStack/ZStack ↔ VStack/HStack/ZStack
   - .padding() ↔ .padding()
   - .animation(.spring) ↔ .spring(hash, SpringParams::snappy())
   - SwiftUI ViewBuilder ↔ CVKG View trait
   - NavigationView ↔ Tabs / Navigation
   - @Environment ↔ Theme / ThemeContext
```

**Success criteria.** Document covers 15+ common patterns. An iOS dev can find their starting point.

**Effort.** 2 days.

---

### P4.2 Motion preset library

**Why.** Persona 3 noted there is no preset library for common transitions (fade, slide, scale, bounce). Each developer must hand-configure `SpringParams` and `Transition` values. A `Motion` module with presets makes animations accessible to developers who don't think about spring stiffness and damping.

**Rationale.** Each preset is a well-tuned `Animated<V>` configuration:

```
pub enum MotionPreset {
    FadeIn,     // opacity 0→1, 300ms, ease-out
    FadeOut,    // opacity 1→0, 200ms, ease-in
    SlideUp,    // translate_y 20→0, 350ms, spring(fluid)
    SlideDown,  // translate_y -20→0, 350ms, spring(fluid)
    ScaleIn,    // scale 0.8→1.0, 300ms, spring(snappy)
    BounceIn,   // scale 0→1, 500ms, spring(bouncy)
    RotateIn,   // rotate -15°→0, 400ms, spring(snappy)
}
```

These are pre-defined `Animated` configurations that users can apply with `.animate(MotionPreset::FadeIn)`.

**Implementation.**

```
1. cvkg-components/src/motion.rs:
     pub enum MotionPreset { ... }
     impl ViewAnimExt for MotionPreset { fn apply(self, view: impl View) -> Animated<impl View> }

2. Add .motion(preset) modifier

3. Demo showing all presets on the same component
```

**Success criteria.** `.motion(MotionPreset::FadeIn)` compiles and animates correctly.

**Effort.** 1 day.

---

### P4.3 Generic Skeleton loading component

**Why.** Persona 3 and 4 need loading placeholders. `ShimmerButton` exists but there is no generic `Skeleton` that wraps any content and renders a placeholder shape while data loads.

**Rationale.** `Skeleton` is a wrapper that, when `loading` is true, renders a pulsing rectangle/glyph matching the layout of its children. When `loading` transitions to false, it crossfades to the real content. The pulsing animation uses the existing spring system.

**Implementation.**

```
cvkg-components/src/skeleton.rs:
  Skeleton<T: View> {
      loading: bool,
      shape: SkeletonShape(Rect | Circle | TextLine),
      content: T,
  }
```

**Success criteria.** `Skeleton::new(content).loading(true).shape(Rect)` renders an animated placeholder.

**Effort.** 4 hours.

---

## P5 — Stretch (single-persona, low urgency)

### P5.1 Game UI primitives (HealthBar, MiniMap, DPad)

**Why.** Persona 1 (iOS Game Dev) identified these as missing. The audit notes that 80% of game UI patterns are covered; these three fill the remaining gap for common game screens (HUD, inventory, settings).

**Rationale.** Each is a composable View:
- `HealthBar` = horizontal bar with fill, color gradient (green→yellow→red), optional text
- `MiniMap` = rectangular viewport with positioned markers, zoom
- `DPadControl` = cross-shaped input control with 4 directional buttons

All three use existing primitives (fill_rect, draw_text, stroke_rect) and the animation system for smooth fill transitions.

**Implementation.**

```
1. cvkg-components/src/game/mod.rs + 3 submodules
2. Register in lib.rs
3. Demo in berserker app
```

**Success criteria.** All three components render in the berserker demo game HUD with animation.

**Effort.** 2 days.

---

## Implementation order

The recommended sequence maximises adoption impact per unit of effort:

```
Phase 1 (P0) — Discoverability unlock
  ├── P0.1 AI-friendly prelude         → 1 day  (unblocks Persona 2)
  ├── P0.2 Component index             → 4 hrs  (supports all personas)
  └── P1.1 Norse-name aliases          → 2 hrs  (completes the discovery fix)

 Phase 2 (P1) — Critical gaps
  ├── P1.2 Integrated form validation  → 2 days (unblocks Persona 3, 4)
  ├── P1.3 WASM documentation          → 1 day  (unblocks Persona 5)
  └── P2.4 CSS variables export        → 4 hrs  (supports Persona 3, 5)

 Phase 3 (P2) — Quality of life
  ├── P2.1 Responsive breakpoints      → 1 day
  ├── P2.2 Sheet/Drawer/Accordion      → 3 days
  └── P2.3 Light mode audit            → 2 days

 Phase 4 (P3) — Accessibility & docs
  ├── P3.1 Per-component a11y docs     → 2 days
  ├── P3.2 Auto-register in a11y tree  → 1 day
  ├── P3.3 Landing components          → 2 days
  └── P3.4 Component gallery           → 3 days

 Phase 5 (P4) — Polish
  ├── P4.1 SwiftUI migration doc       → 2 days
  ├── P4.2 Motion preset library       → 1 day
  └── P4.3 Skeleton component          → 4 hrs

 Phase 6 (P5) — Stretch
  └── P5.1 Game primitives             → 2 days
```

**Total estimated effort:** ~24 days  
**Target persona score after Phase 1+2:** Persona 2 → 6/10, Persona 3 → 7/10, Persona 5 → 6/10  
**Target overall health after all phases:** 8.0/10  

---

*This plan is derived from `cvkg-persona-audit.md`. Priorities and effort estimates assume one experienced Rust developer. Phase 1 alone (P0 + P1.1) resolves the critical discovery blocker for Persona 2 and can be done in ~1.5 days.*
