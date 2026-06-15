# CVKG Design Steward

**Role:** Design System Governance
**Scope:** All visual design tokens, component patterns, and theme contracts in CVKG.

---

## What the Design Steward Owns

The Design Steward is the final authority on changes to:

1. **Theme tokens** — `cvkg-themes/src/lib.rs`
   - `SemanticColors` (primary, secondary, accent, background, surface, error, warning, success, text, text_dim)
   - `SpacingScale`, `RadiusScale`, `TypographyScale`, `MotionScale`
   - `GlassMaterial` parameters
   - `AccessibilityOverrides`
   - `Density` enum

2. **Animation vocabulary** — `doc/ANIMATION_VOCABULARY.md`
   - Spring parameter presets (snappy, fluid, heavy, bouncy)
   - Interaction-to-animation mappings
   - Stagger timing rules

3. **Component patterns** — `cvkg-components/src/`
   - Theming rules (all colors via theme tokens)
   - Default component colors
   - Focus ring, disabled state, error state patterns

4. **Documentation** — `doc/`
   - `ANIMATION_VOCABULARY.md`
   - `DESIGN_STEWARD.md` (this file)
   - `ARCHITECTURE.md` (theming section)

---

## Approval Required

Any PR that touches the following **must** be reviewed and approved by the Design Steward:

- [ ] New semantic color roles or renaming existing ones
- [ ] Spacing/radius/typography scale additions or changes
- [ ] Animation vocabulary changes (new presets, modified mappings)
- [ ] New `Density` variants
- [ ] Glass material parameter defaults
- [ ] Accessibility override behavior changes
- [ ] New theme tokens in `cvkg-components/src/theme.rs`
- [ ] Changes to default component colors

## Not Approval Required

These can be merged without Design Steward review:

- Bug fixes that don't change visual appearance
- New components that use existing theme tokens
- Documentation typo fixes
- Test additions
- Performance optimizations that don't change output

---

## Review Process

1. Author opens PR with `design-steward-review` label
2. Design Steward reviews within 2 business days
3. If approved: merge
4. If changes requested: author updates, re-requests review
5. If rejected: author may escalate via RFC process

## Escalation Path

For disagreements about design direction:

1. Author writes RFC in `doc/RFCs/`
2. Design Steward responds within 3 business days
3. If unresolved: escalate to project lead
4. Project lead makes final decision

---

## Current Design Steward

**Name:** TBD (project lead serves as interim)
**Contact:** GitHub @ mention on PRs

---

## Design Principles

1. **Consistency over novelty.** Prefer existing patterns over new ones.
2. **Accessibility first.** All designs must meet APCA Lc >= 60 for text.
3. **Themeable by default.** Every color must resolve through the theme system.
4. **Physics-based motion.** No hardcoded durations; use spring parameters.
5. **Graceful degradation.** Effects must degrade cleanly on low-end hardware.

---

*Last updated: 2026-06-14 by OWL*