# CVKG Component Library & Gallery — Product Designer UX Audit

**Audit Date:** 2026-06-28  
**Scope:** cvkg-components (core interactive components), cvkg-gallery (demo), cvkg-themes, cvkg-core (design tokens/a11y), cvkg-accessibility  
**Methodology:** Static code analysis of rendering logic, color math, spacing/typography scales, accessibility primitives, and gallery interaction patterns.

---

## Executive Summary

CVKG demonstrates **strong foundational design engineering** — a perceptually uniform OKLCH color system, spring-physics motion, comprehensive accessibility primitives (focus traps, ARIA roles, APCA contrast validation), and a well-structured token system (spacing, radius, typography, elevation). However, the library is in a **"framework-first, product-second"** state: the infrastructure is more polished than the visual output. Raw RGB literals still leak into component code, the gallery lacks keyboard navigation, and several components have hardcoded decorative colors that break theme consistency.

**Overall Design Maturity: 6.5/10** — Solid architecture, inconsistent application.

---

## 1. Component Design Quality

### 1.1 Visual Hierarchy & Spacing Consistency

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Spacing scale is well-defined | Low | `SPACE_XS` (4px) through `SPACE_XL` (32px) in `lib.rs` — clean 4px grid |
| ✅ Typography scale is comprehensive | Low | `FONT_XS` (10px) through `FONT_3XL` (48px) with line-height multipliers |
| ⚠️ Components mix token and raw values | **Medium** | Button uses hardcoded `44.0`/`52.0` heights instead of a size token; Input uses raw `200.0` width |
| ⚠️ Inconsistent vertical padding in Button | **Medium** | Small=12px, Default=16px, Large=24px — not on the 4px grid (12 is fine, but 24 skips a step) |
| ❌ Gallery uses raw pixel offsets | **Medium** `rect.x + 40.0`, `title_y + 25.0`, `200.0` carousel height — no spacing tokens |

**Recommendation:** Introduce `SPACE_2XL` (48px) and `SPACE_3XL` (64px) tokens. Replace all raw `40.0`/`200.0`/`25.0` values in gallery with spacing/radius tokens.

### 1.2 Color Usage (OKLCH vs Raw RGB)

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ OKLCH color model exists in cvkg-themes | — | Full sRGB↔OKLCH conversion pipeline with `OklchColor` |
| ✅ Theme provides semantic color tokens | — | `primary()`, `accent()`, `surface()`, `error_color()`, etc. |
| ✅ `from_seed()` generates full palettes from one color | — | Perceptually harmonious theme derivation |
| ❌ Button `bg_color()` uses 12+ raw RGB literals | **Critical** | `[0.08, 0.07, 0.06, 1.0]`, `[0.20, 0.17, 0.13, 1.0]`, etc. — completely bypasses theme tokens |
| ❌ Button `text_color()` uses raw `[0.0, 1.0, 0.95, 1.0]` | **Critical** | "Neon cyan on dark iron" is hardcoded — ignores theme's `text` token |
| ❌ Button disabled state uses raw `[1.0, 0.0, 0.85, 1.0]` | **High** | Neon magenta disabled text is hardcoded — fails in light mode |
| ❌ Button border uses raw `[0.80, 0.72, 0.55, 1.0]` bevel colors | **High** | "Forged iron" aesthetic is baked to specific RGB values |
| ❌ Gallery carousel uses raw `[0.06, 0.055, 0.06, 1.0]` | **High** | Card backgrounds hardcoded — won't adapt to light theme |
| ❌ Gallery divider uses raw `[0.35, 0.12, 0.15, 0.6]` | **Medium** | Dark red divider — theme-agnostic |
| ❌ `surface_high_contrast()` returns raw `[0.45, 0.20, 0.55, 0.95]` | **Medium** | Vibrant purple hardcoded in theme.rs |
| ⚠️ `InteractiveColorStates::from_color()` operates in sRGB | **Medium** | Lightness adjustments (+15%/-15%) in sRGB are perceptually inconsistent — should use OKLCH |

**Recommendation:** 
1. Migrate Button's `bg_color()`, `text_color()`, `border_color()` to use `theme::surface()`, `theme::accent()`, `theme::text()` with alpha modulation.
2. Move `InteractiveColorStates` into `OklchColor` space — adjust `l` channel instead of raw RGB.
3. Replace all `[0.x, 0.y, 0.z, 1.0]` arrays in gallery with `theme::color()` calls.

### 1.3 Typography Scale

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Two parallel typography systems exist | — | `cvkg-components` (FONT_*) and `cvkg-themes` (TypographyScale with Apple HIG sizes) |
| ✅ Line-height multipliers defined | Low | Per-size LINE_HEIGHT_* constants |
| ⚠️ Two typography systems are disconnected | **Medium** | Components use `FONT_SM=12`, `FONT_BASE=14` but themes use `body=17`, `caption1=12` — no single source of truth |
| ⚠️ No letter-spacing/tracking tokens | Low | Tightening for large headings, widening for small caps — missing |
| ❌ No font family fallback chain | **Medium** | Text component hardcodes "SF Pro Text" — not configurable per-platform |

**Recommendation:** Unify on `TypographyScale` as the canonical system. Add `letter_spacing` field. Make font family configurable via theme.

### 1.4 Elevation & Depth

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Elevation scale is well-documented | — | 7 named levels (FLAT→TOAST) with blur/offset/opacity mappings |
| ✅ Z-index layers properly layered | — | BASE(0) → CONTENT(100) → DROPDOWN(1000) → STICKY(2000) → MODAL(3000) → TOAST(4000) → TOOLTIP(5000) → DEBUG(99999) |
| ✅ Shadow system supports layered shadows | — | Up to 4 layers, inset, spread |
| ⚠️ Button elevation is minimal | Low | Only `push_shadow(1.0, ...)` — very subtle, almost imperceptible |
| ❌ No colored shadow support | **Medium** | Shadows are always black — modern UIs use tinted shadows (e.g., accent-colored glow for focused elements) |

**Recommendation:** Add `push_tinted_shadow(color, ...)` to renderer. Increase Button shadow depth to use elevation level RESTING (blur=2.0, offset_y=1.0).

### 1.5 Motion & Animation Quality

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Spring physics with RK4 integration | — | `SpringSolver` with snappy/fluid/heavy/bouncy presets |
| ✅ Reduced motion support | — | `is_reduced_motion()`, `effective_duration()` |
| ✅ Accessibility preferences integration | — | `AccessibilityPreferences` with reduce_motion/transparency/contrast |
| ⚠️ Button uses two SpringSolvers (hover+press) | Low | Correct approach but no `is_settled()` check to stop ticking |
| ⚠️ Cursor blink uses raw `sin()` without spring | Low | Functional but feels mechanical — could use eased interpolation |
| ❌ No shared element transitions | **Medium** | `SharedElementModifier` exists but gallery doesn't use it for carousel→detail transition |
| ❌ No scroll-linked animations | Low | Gallery carousel has no parallax or scroll-driven effects |

**Recommendation:** Use `is_settled()` to pause idle spring computations. Add shared element transition between carousel card and detail panel.

### 1.6 Interaction Feedback (Hover/Press/Focus)

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Button has full state machine | — | Hover, pressed, focused, disabled, loading — all with distinct visuals |
| ✅ Focus rings on all interactive components | — | `draw_focus_ring()` with theme-aware color, 2px width, 2px offset (WCAG 2.4.7) |
| ✅ Magnetic pointer warping on Button | — | Subtle attraction effect within 120px radius |
| ✅ Haptic + audio feedback | — | `haptic_impact()` + `play_sound("success_chime")` on click |
| ⚠️ Checkbox has no hover state | **Medium** | Only checked/unchecked visual — no hover preview |
| ⚠️ Input has no animated border transition | Low | Border width changes 1px→2px on focus but no interpolation |
| ❌ Toggle has no hover state | **Medium** | No visual change between rest and hover |
| ❌ No pressed state animation on Checkbox | Low | Missing scale-down feedback |

**Recommendation:** Add hover states to Checkbox (subtle bg shift) and Toggle (track brighten). Animate Input border width with spring.

---

## 2. Accessibility

### 2.1 Keyboard Navigation

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Button handles Enter/Space | — | `on_key_event()` correctly triggers click |
| ✅ Checkbox handles Enter/Space | — | Toggles state |
| ✅ Toggle handles Enter/Space | — | Toggles state |
| ✅ Breadcrumb has full arrow key nav | — | ArrowLeft/Right/Home/End + Enter/Space activation |
| ✅ FocusManager in cvkg-core | — | Tab/Shift+Tab with wrapping, focus traps |
| ✅ FocusManager in cvkg-accessibility | — | Visual-position-based tab order (y, then x sort) |
| ❌ Gallery carousel has no keyboard nav | **Critical** | No arrow key support, no Enter to select — mouse-only |
| ❌ Tabs component has no arrow key nav | **High** | BifrostTabs doesn't handle ArrowLeft/Right for tab switching |
| ❌ No skip navigation in gallery | **Medium** | No "skip to content" link — tab through all carousel cards |
| ❌ Command palette not keyboard-accessible from gallery | **High** | No global shortcut (Ctrl+K) to open it |

**Recommendation:** 
1. Add ArrowLeft/Right handler to gallery carousel
2. Add ArrowLeft/Right to BifrostTabs
3. Add Ctrl+K keyboard shortcut to open command palette
4. Add skip-nav link

### 2.2 Focus Management

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Focus trap infrastructure exists | — | `push_focus_trap`/`pop_focus_trap` in renderer trait |
| ✅ Focus ring rendering | — | Consistent 2px offset, theme-colored |
| ⚠️ Focus trap not used by Dialog | **High** | Dialog component exists but no evidence of focus confinement |
| ⚠️ No focus restoration on dialog close | **Medium** | Should return focus to trigger element |
| ❌ Gallery carousel cards not focusable | **Critical** | Cards are drawn with raw `draw_text` — not in tab order |

**Recommendation:** Implement focus trap in Dialog. Make carousel cards focusable with `tabindex=0`.

### 2.3 ARIA Roles

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Comprehensive AriaRole enum | — | 50+ roles including Tree, Grid, Tab, Switch |
| ✅ Button sets `role="button"` | — | With label and disabled state |
| ✅ Checkbox sets `role="checkbox"` | — | With checked state |
| ✅ Toggle sets `role="switch"` | — | With checked and value |
| ✅ Input sets `role="textbox"` | — | With label |
| ✅ Slider sets `role="slider"` | — | With valuemin/valuemax/valuenow |
| ✅ Breadcrumb sets `role="navigation"` | — | With label |
| ❌ Tabs missing `role="tablist"` | **High** | No tablist/tab/tabpanel role hierarchy |
| ❌ Command palette missing roles | **High** | Should use `role="listbox"` + `role="option"` |
| ❌ Gallery carousel missing semantics | **Medium** | Should use `role="region"` with `aria-roledescription="carousel"` |

**Recommendation:** Add ARIA roles to Tabs (tablist/tab/tabpanel), Command Palette (listbox/option), and carousel.

### 2.4 Color Contrast

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ APCA contrast validation exists | — | `validate_accessibility()` in Theme with font-size-aware thresholds |
| ✅ WCAG relative luminance calculation | — | `relative_luminance()` in accessibility.rs |
| ✅ Contrast ratio function | — | `contrast_ratio()` in accessibility.rs |
| ⚠️ Tests exist but no runtime enforcement | **Medium** | Validation is test-only — no debug-mode overlay showing failing pairs |
| ❌ Button disabled text contrast likely fails | **Critical** | `[1.0, 0.0, 0.85, 1.0]` (magenta) on `[0.05, 0.045, 0.05, 1.0]` — likely < 3:1 for small text |
| ❌ Gallery title `[0.9, 0.2, 0.25, 0.85]` on dark bg | **Medium** | Red text at 0.85 alpha reduces effective contrast |

**Recommendation:** Run APCA validation on all component color pairs. Fix disabled text to use `theme::disabled_text()` which is designed for this purpose.

### 2.5 Screen Reader Support

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ ScreenReaderBridge trait exists | — | With Announcement, Priority levels |
| ✅ AriaProperties comprehensive | — | Role, label, description, value, pressed, checked, expanded, disabled, level, live, atomic |
| ❌ No live regions for dynamic content | **Medium** | Progress updates, toast notifications — no `aria-live` announcements |
| ❌ No `aria-describedby` for error messages | **Medium** | Input error messages not programmatically linked to input |

**Recommendation:** Add `aria-live="polite"` to Progress and Toast. Link Input error messages with `aria-describedby`.

---

## 3. Gallery UX

### 3.1 Carousel Interaction

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Scroll wheel cycles cards | — | `pointerwheel` handler with delta_y threshold |
| ✅ Click selects card | — | Direct click on card sets `selected` |
| ✅ Visual depth via scale + z-index | — | Cards scale and layer correctly |
| ❌ No keyboard navigation | **Critical** | Arrow keys do nothing |
| ❌ No touch/swipe support | **High** | No pointer drag gesture for mobile |
| ❌ No autoplay/pause | **Low** | Could auto-rotate when idle |
| ❌ No transition animation | **High** | Card position changes are instant — no spring animation between states |
| ❌ No pagination indicators | **Medium** | User can't see how many items or which is selected |
| ❌ Cards clip each other awkwardly | **Medium** | `push_clip_rect` logic is complex and produces visual artifacts at certain angles |

**Recommendation:** Add spring animation to carousel position transitions. Add arrow key navigation. Add dot indicators.

### 3.2 Detail Panel Layout

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Component renders in detail area | — | Centered VStack layout |
| ✅ Title shows selected component name | — | "GALLERY / BUTTON" format |
| ❌ No code snippet view | **High** | Developer can't see/copy the Rust code to reproduce the component |
| ❌ No props/variants panel | **Medium** | Can't interactively toggle component variants |
| ❌ No responsive behavior | **Medium** | Fixed `rect.x + 40.0` margins — doesn't adapt to narrow windows |
| ❌ Detail panel has no scroll | **Medium** | Tall components will overflow |

**Recommendation:** Add a code snippet panel below the live preview. Add scrollable detail area.

### 3.3 Component Discoverability

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Components organized by category | — | Forms, Navigation, Overlays, Data Display, Feedback |
| ✅ 15+ components in catalog | — | Good coverage |
| ❌ No search/filter | **High** | As catalog grows, finding a specific component requires scrolling |
| ❌ No component descriptions | **Medium** | Name + category only — no usage guidance |
| ❌ No "copy snippet" button | **Medium** | Developer onboarding friction |

**Recommendation:** Add a search bar. Add component descriptions and usage snippets.

### 3.4 Developer Onboarding Experience

| Finding | Severity | Detail |
|---------|----------|--------|
| ✅ Examples in doc comments | — | `Button::new("Click me", ...)` in rustdoc |
| ✅ Gallery is runnable | — | `cvkg-gallery` binary exists |
| ❌ No "Getting Started" documentation | **High** | No README in cvkg-components with setup instructions |
| ❌ No interactive playground | **Medium** | Can't tweak props and see live update |
| ❌ No design token reference | **Medium** | Token values only visible in source code |

**Recommendation:** Create a design token reference page in the gallery. Add a "playground" mode where props are adjustable.

---

## 4. Missing Design Tokens

### 4.1 Semantic Colors

| Token | Status | Notes |
|-------|--------|-------|
| `primary` | ✅ | Viking Gold |
| `secondary` | ✅ | Magenta Liquid |
| `accent` | ✅ | Crimson Flash / NiflCyan |
| `background` | ✅ | Deep Void |
| `surface` | ✅ | Tactical Obsidian |
| `surface_elevated` | ✅ | |
| `surface_overlay` | ✅ | |
| `error` | ✅ | |
| `warning` | ✅ | |
| `success` | ✅ | |
| `text` | ✅ | |
| `text_muted` | ✅ | |
| `text_dim` | ✅ | |
| `border` | ✅ | |
| `border_strong` | ✅ | |
| `focus_ring` | ✅ | |
| `shadow` | ✅ | |
| `code_bg` | ✅ | |
| `info` | ✅ | |
| ❌ `text_inverse` | **Missing** | Text on colored backgrounds (e.g., accent button) |
| ❌ `text_link` | **Missing** | Link color distinct from accent |
| ❌ `surface_hover` | **Missing** | Separate from `hover` — subtle bg for list items |
| ❌ `surface_active` | **Missing** | Selected state bg distinct from hover |
| ❌ `disabled_bg` | **Missing** | Separate from `disabled` text color |
| ❌ `interactive_hover` | **Missing** | Generic hover overlay (10% white) |

### 4.2 Spacing Scale

| Token | Status | Notes |
|-------|--------|-------|
| `SPACE_XS` (4px) | ✅ | |
| `SPACE_SM` (8px) | ✅ | |
| `SPACE_MD` (16px) | ✅ | |
| `SPACE_LG` (24px) | ✅ | |
| `SPACE_XL` (32px) | ✅ | |
| ❌ `SPACE_2XL` (48px) | **Missing** | Used in gallery as raw `40.0` |
| ❌ `SPACE_3XL` (64px) | **Missing** | Section spacing |

### 4.3 Radius Scale

| Token | Status | Notes |
|-------|--------|-------|
| `RADIUS_XS` (2px) | ✅ | |
| `RADIUS_SM` (4px) | ✅ | |
| `RADIUS_MD` (6px) | ✅ | |
| `RADIUS_LG` (8px) | ✅ | |
| `RADIUS_XL` (12px) | ✅ | |
| `RADIUS_2XL` (16px) | ✅ | |
| `RADIUS_FULL` (9999px) | ✅ | |
| ❌ `RADIUS_3XL` (24px) | **Missing** | Large sheets/modals |

### 4.4 Shadow Tokens

| Token | Status | Notes |
|-------|--------|-------|
| Elevation shadow blur/offset/opacity | ✅ | Via `elevation::to_blur_radius()` etc. |
| ❌ Named shadow tokens | **Missing** | `shadow_sm`, `shadow_md`, `shadow_lg` for direct use |
| ❌ Colored shadows | **Missing** | Accent glow, error glow |
| ❌ Inner shadow token | **Missing** | Inset shadow for pressed states |

### 4.5 Motion Tokens

| Token | Status | Notes |
|-------|--------|-------|
| Spring presets (snappy/fluid/heavy/bouncy) | ✅ | |
| ❌ Duration tokens | **Missing** | `duration_fast` (150ms), `duration_normal` (250ms), `duration_slow` (400ms) |
| ❌ Easing curves | **Missing** | `ease_out`, `ease_in_out`, `ease_spring` |
| ❌ Transition tokens | **Missing** | `transition_hover`, `transition_toggle`, `transition_modal` |

---

## 5. Priority Action Items (Ranked by Impact)

### 🔴 Critical (Fix Immediately)

1. **Migrate Button colors to theme tokens** — 12+ raw RGB arrays make theming impossible
2. **Add keyboard navigation to gallery carousel** — currently completely inaccessible
3. **Fix disabled text contrast** — neon magenta on near-black fails WCAG
4. **Make carousel cards focusable** — screen reader users can't discover components

### 🟠 High (Next Sprint)

5. **Add arrow key navigation to BifrostTabs** — standard tab pattern
6. **Add focus trap to Dialog** — accessibility requirement for modals
7. **Add code snippet panel to gallery** — core developer experience
8. **Add search/filter to gallery** — scalability as catalog grows
9. **Unify typography systems** — single source of truth for font sizes
10. **Add transition animations to carousel** — spring physics between states

### 🟡 Medium (Next Month)

11. **Replace all raw RGB in gallery with theme tokens**
12. **Add hover states to Checkbox and Toggle**
13. **Add ARIA roles to Tabs, Command Palette, Carousel**
14. **Add live regions for Progress/Toast**
15. **Add missing semantic tokens** (text_inverse, surface_hover, etc.)
16. **Add duration/easing tokens**

### 🟢 Low (Backlog)

17. **Add autoplay/pause to carousel**
18. **Add pagination dots to carousel**
19. **Add skip navigation link**
20. **Add interactive playground mode**
21. **Add design token reference page**

---

## 6. Architecture Observations

### Strengths
- **OKLCH color science** is genuinely advanced for a UI framework — perceptually uniform palette generation from a seed color
- **Spring physics motion** with RK4 integration shows engineering rigor
- **Accessibility is architected, not bolted on** — focus management, ARIA, contrast validation, reduced motion all have dedicated modules
- **Theme builder pattern** with `ThemeBuilder` allows runtime customization
- **Multi-platform accessibility detection** (macOS, Linux, Windows)

### Weaknesses
- **Two parallel systems** for everything (cvkg-core vs cvkg-accessibility FocusManager, cvkg-components FONT_* vs cvkg-themes TypographyScale) — indicates organic growth without consolidation
- **Components don't consistently use the token system they define** — the infrastructure exists but adoption is patchy
- **Gallery is a prototype** — demonstrates rendering but not interaction patterns
- **No runtime token validation** — tokens are constants, not enforced at compile time
- **No visual regression testing** — no screenshot comparison tests

---

## Appendix: Contrast Analysis (Quick Math)

Key color pairs in the dark theme:

| Pair | Foreground | Background | Est. Ratio | WCAG AA (4.5:1) | WCAG AAA (7:1) |
|------|-----------|------------|------------|-----------------|-----------------|
| text on bg | `#F2F2FF` | `#05050D` | ~18:1 | ✅ | ✅ |
| text_dim on bg | `#9999B3` | `#05050D` | ~5.2:1 | ✅ | ❌ |
| primary on bg | `#FFD700` | `#05050D` | ~13:1 | ✅ | ✅ |
| accent on bg | `#FF0066` | `#05050D` | ~6.5:1 | ✅ | ❌ |
| disabled text | `#FF00D9` | `#0D0B0D` | ~4.8:1 | ✅ (barely) | ❌ |
| button text | `#00FFF2` | `#14120A` | ~10:1 | ✅ | ✅ |
| error on bg | `#FF3333` | `#05050D` | ~5.8:1 | ✅ | ❌ |

The disabled text color (`#FF00D9` on `#0D0B0D`) is borderline — it passes AA for normal text but fails for small text and all AAA levels.
