# UI/UX Audit Specification — CVKG Design Engineering Review

## Purpose
This document defines the scope, structure, and methodology for a multi-perspective UI/UX audit of the CVKG framework. The audit evaluates the system from 5 distinct user personas, each representing a real-world use case. Subagents will produce persona-specific reviews; the primary agent consolidates into uiux.md.

## Audit Scope

### What is being audited
- **Component API surface** — 215+ exported components across 126 modules in cvkg-components
- **Theming system** — OKLCH color model, Theme/ThemeBuilder, DesignTokens, GlassMaterial, APCA contrast
- **Design tokens** — FONT_*, SPACE_*, RADIUS_* constants, typography scale, spacing scale, motion scale
- **View trait system** — Modifier pipeline, lifecycle hooks, event handling, accessibility (AriaProperties)
- **Naming conventions** — Mix of Norse mythological names (Bifrost, Mjolnir, Yggdrasil) and standard names (Button, Checkbox, Card)
- **Macro system** — #[derive(View)], #[view_component], state!{} macros
- **Material system** — Glass, Mica, Acrylic materials; Elevation levels
- **Icon system** — IconRegistry, Icon component, SVG path-based icons
- **Localization** — lingua_tong module (set_locale, t(), is_rtl)
- **Animation system** — SpringParams (snappy/fluid/heavy/bouncy), text_anim, morph_bridge
- **Multi-renderer support** — GPU (wgpu), Native (winit+AccessKit), Web/WASM VDOM
- **Demo apps** — berserker (native game-like), adele-web, niflheim-wasi, berserker-fire-web

### What is NOT being audited
- GPU shader code correctness (covered by owl_audit.md)
- Crate-level Rust safety/bugs (covered by owl_audit.md)
- Build system / Cargo.toml structure
- Test infrastructure

## Evaluate Against These Skills

Load and apply these skills during the audit:
- cvkg — CVKG framework domain knowledge
- design-review — Structured design review methodology
- design-qa — Design QA gates (token + hardcoded-value linting)
- design-code — Production-ready, accessible, token-driven component code
- design-component — Component spec quality bar
- design-taste-frontend — Anti-slop frontend standards
- frontend-design — Visual design direction
- design-system-starter — Design token architecture
- advanced-visualization-techniques — UI mockups, dashboards, advanced interactivity
- impact-designer — Distinctive, production-grade frontend interfaces
- ux-designer (bencium-controlled-ux-designer) — Expert UI/UX design guidance
- minimalist-ui — Clean editorial-style interfaces
- ui-typography — Professional typography rules for UI
- ui-ux-pro-max — UI/UX design intelligence for web and mobile
- microinteractions — Small details: triggers, rules, feedback, loops
- visual-style — Portable visual design systems
- lean-ux — Hypothesis-driven design, collaborative
- mom-test — Customer research without leading
- token-build — DTCG design token pipeline
- top-design — Award-winning, immersive web experiences
- ux-writing — UI copy (buttons, errors, empty states)
- design-sprint — Structured 5-day design process
- interactive — Interactive patterns
- logging — Library diagnostics
- requirements-clarity — Clarify ambiguous requirements
- redesign — Upgrade existing UI to premium quality
- brand — Brand voice, visual identity

## Persona Definitions

### Persona 1: The Design Engineer (iOS Game Developer)
**Profile:** Uses SwiftUI/UIKit for iOS game development. Wants to use CVKG for game UI overlays, HUDs, inventory screens, and menu systems. Values:
- Smooth 165Hz rendering performance
- Particle effects, fluid animations, shader-based visual effects
- Frame budget control and performance overlay
- Familiar SwiftUI-like declarative syntax
- Metal/GPU pipeline control
- Physics integration (cvkg-physics)

**Audit focus:**
- How does CVKG's View/Modifier pattern compare to SwiftUI's View/binding system?
- Is the animation system (SpringParams, text_anim, morph) expressive enough for game UI?
- Can you achieve Cytus/Amonite-quality visual effects?
- How steep is the learning curve from SwiftUI?
- Does the frame budget system work for 165Hz?
- Are the performance debugging tools adequate?

### Persona 2: The Vibe Coder (AI Agentic Design System)
**Profile:** Uses AI coding tools (Cursor, Copilot, Claude Code) to build apps quickly. Wants a design system that AI agents can:
- Pick up from minimal examples without reading docs
- Compose correctly from component names alone
- Theme consistently without manual color picking
- Discover available components through naming patterns
- Generate WCAG-compliant UIs by default

**Audit focus:**
- Are component names self-documenting for AI? (BifrostTabs vs Tabs?)
- Can an AI agent compose a full app from the prelude alone?
- Are the design tokens consistent enough that AI won't create visual inconsistency?
- Is the naming convention systematic enough for AI to infer patterns?
- What's the "time to first pixel" for an AI agent with no CVKG knowledge?

### Persona 3: The Product Designer (shadcn/MUI Migrant)
**Profile:** Comes from React ecosystem (shadcn Chakra MUI). Wants to build production web apps with CVKG. Values:
- Familiar component patterns (Button variants, Card compositions, Dialog/Sheet)
- Tailwind-like composition model
- Dark/light mode that works out of the box
- Responsive design support
- Form handling with validation
- Accessibility compliance (WCAG 2.1 AA)
- Component customization without fighting the system

**Audit focus:**
- How does CVKG's compositional model compare to shadcn/ui or Material UI?
- Is the theming migration from CSS variables to OKLCH painless?
- Does the component set cover all shadcn/MUI primitives?
- Are the default aesthetics neutral enough for business software?
- How accessible is the default component output?
- Is the Density system sufficient for different form factors?

### Persona 4: The Frontend Engineer (Engineering-First Design)
**Profile:** A software engineer tasked with implementing frontend. Doesn't want to be a designer but needs the UI to look professional. Values:
- Type-safe component APIs
- Predictable layout behavior (knows CSS Flexbox/Grid)
- Clear error messages when composition is wrong
- Minimal magic — explicit over implicit
- Easy testing integration
- Good documentation with working examples
- Patterns for common layouts (dashboard, CRUD, settings, auth)

**Audit focus:**
- How type-safe is the View composition pipeline?
- Does the layout system (FlexBox, Grid, Stack) map to CSS mental models?
- Are error messages helpful when the type system rejects bad composition?
- Is there a clear mental model for the rendering pipeline?
- How testable are components (unit + visual regression)?
- Is the component API discoverable through rust-analyzer?

### Persona 5: The Marketing Designer (Ad Interactions)
**Profile:** Works in marketing department. Wants to evaluate UI systems for generating highly polished advertisement interactions — landing pages, product demos, interactive brand experiences. Values:
- Visual polish and "wow factor"
- Smooth, performant animations
- Easy theming for brand colors
- Image/media handling
- Responsive layouts for different ad formats
- Export to web/WASM
- Minimal code for maximum visual impact

**Audit focus:**
- Can CVKG produce visually stunning landing pages?
- How easy is it to apply a brand's color palette?
- Is the animation system capable of ad-quality motion?
- How does the visual output compare to Framer/Webflow?
- What's the WASM bundle size for web deployment?
- Is the BentoGrid/Carousel/Marquee sufficient for ad layouts?

## Audit Output Format

Each subagent produces a review in this structure:

```
# Persona: [Name]

## Executive Summary
[2-3 sentences: overall fit for this persona]

## Onboarding Experience
[How easy is it to get started? First impressions?]

## Component Coverage
[Does the component set cover all needs? What's missing?]

## API Design & Ergonomics
[How pleasant is the API to use day-to-day?]

## Visual Design Quality
[How good do things look out of the box?]

## Theming System
[How flexible is theming for this persona's needs?]

## Animation & Interaction
[Quality and expressiveness of motion/interaction system?]

## Accessibility
[WCAG compliance, screen reader support, keyboard nav?]

## Performance
[Rendering performance for this persona's targets?]

## Gaps & Recommendations
[Specific, prioritized recommendations with examples]

## Verdict
[Score 1-10, would this persona adopt CVKG?]
```

## Consolidation Plan

After all subagents complete, the primary agent will produce uiux.md with:

1. **Executive Summary** — Overall CVKG design assessment across all personas
2. **Persona-Specific Reviews** — Each persona's full review
3. **Cross-Cutting Themes** — Issues that affect multiple personas:
   - Naming convention duality (Norse vs standard)
   - Mythological terminology as a barrier to adoption
   - Component name discoverability
   - API consistency patterns
4. **Prioritized Recommendation Matrix** — Table of all recommendations ranked by:
   - Impact (how many personas benefit)
   - Effort (implementation complexity)
   - Priority (must/should/could)
5. **Design System Health Scorecard** — Quantitative metrics:
   - Component coverage score
   - Naming consistency score
   - Token coverage score
   - Accessibility score
   - Documentation score
