# Theming / token consolidation

**Status:** addresses fix-plan task P2-3 (theming/token consolidation), current-state
map and consolidation proposal completed 2026-11-21. Static source audit; no
rendering or CSS consumption was executed. Sections 2–3 describe **current state**;
Section 4 onward is a **proposal**, not implemented code.

## 1. Problem statement

The 2026-11-20 audit found theming split across layers
(`plans/codebase-audit-2026-11-20.md:73`). The split is real: there are three
typography scales, two `SemanticColors` structs, two interactive-state models, a
string-keyed color resolver with silent fallbacks, a GPU uniform palette, and a
CSS export path. Each has a different value for at least one shared token name.
This document maps the sources, records the drift with citations, and proposes a
single semantic-token model that keeps existing public APIs compiling.

## 2. Current state — token source map

| Source | What it owns | Evidence |
|---|---|---|
| `cvkg-components` constants | Typography `FONT_XS…FONT_3XL` (10/12/14/16/20/24/32/48), line heights, spacing `SPACE_XS…SPACE_XL` (4/8/16/24/32), radius `RADIUS_XS…RADIUS_FULL` (2/4/6/8/12/16/9999), focus ring width/offset | `cvkg-components/src/lib.rs:55-96`, `:103-113` |
| `cvkg-components` theme helpers | String-key resolver `color(key)` mapping ~20 keys onto `use_theme()` colors with ad-hoc multipliers; convenience wrappers; font-size helpers; `glassmorphism_enabled()`; `Themed<V>` | `cvkg-components/src/theme.rs:19-84`, `:88-665`, `:467-505`, `:677-679`, `:699-723` |
| `cvkg-core` semantic colors | `SemanticColors` (10 roles) with `dark()`/`light()`; sRGB state derivation | `cvkg-core/src/color.rs:20-98`, `:105-144` |
| `cvkg-core` theme context | Thread-local `ThemeContext`, `use_theme()`, `set_current_theme()`, GPU→semantic bridge | `cvkg-core/src/theme.rs:15-143`, bridge at `:50-72` |
| `cvkg-core` GPU palette | `ColorTheme` shader uniform (176 bytes, layout asserted) with `primary_neon`/`glass_base`/`rune_glow` etc. | `cvkg-core/src/render_tier.rs:19-50` |
| `cvkg-core` color constants | `Color::VIKING_GOLD`, `MAGENTA_LIQUID`, `TACTICAL_OBSIDIAN` | `cvkg-core/src/env_core.rs:72-84` |
| `cvkg-themes` generation | `OklchColor` conversions/clamps, `GlassMaterial`, `oklch_to_color_theme()`, `Theme`/`ThemeBuilder`, APCA validation, CSS export | `cvkg-themes/src/lib.rs:20-170`, `:180-232`, `:234-285`, `:432-905`, `:919-1182`, `:1107-1174` |
| `cvkg-themes` scales | Apple-HIG `TypographyScale`, `RadiusScale`, `SpacingScale`, `MotionScale`, `Density`, `StateColors` (OKLCH) | `cvkg-themes/src/lib.rs:339-413`, `:1212-1350` |
| Consumers | GPU/VDOM/gallery push `ThemeContext` from `ColorTheme`; component helpers read thread-local colors | `cvkg-render-gpu/src/api/mod.rs:930`, `cvkg-vdom/src/lib.rs:1868`, `cvkg-gallery/src/app.rs:126` |

## 3. Current state — duplication and drift

| ID | Drift | Evidence |
|---|---|---|
| D1 | **Two `SemanticColors` structs** with the same ten fields: `cvkg_core::SemanticColors` and `cvkg_themes::SemanticColors`. No `From`/`Into` conversion exists. `Theme::dark()` populates the themes-side struct; components read the core-side struct. | `cvkg-core/src/color.rs:20-41`, `cvkg-themes/src/lib.rs:293-304`, `cvkg-themes/src/lib.rs:454-500`, `cvkg-components/src/theme.rs:16-20` |
| D2 | **GPU→semantic bridge hardcodes three roles.** `from_color_theme()` maps neon fields to primary/secondary/accent/background/surface/text but sets `error`, `warning`, `success` to literal RGBA instead of reading a theme palette. | `cvkg-core/src/theme.rs:50-72` (literals at `:62-64`) |
| D3 | **Typography values disagree for the same role.** `FONT_BASE=14`, `FONT_MD=16`, `FONT_LG=20` vs `TypographyScale.body=17`, `callout=16`, `title3=20`; `theme::font_body()` returns 16 and `font_code()` 14, while `Theme::dark()` legacy `code=12`. | `cvkg-components/src/lib.rs:57-61`, `cvkg-themes/src/lib.rs:508-519`, `cvkg-components/src/theme.rs:491-505` |
| D4 | **Spacing labels disagree.** `SPACE_MD=16` vs `SpacingScale.m=12`; `SPACE_SM=8` matches `s=8`, `SPACE_XL=32` matches `xxl=32` but no `SPACE_XXXL` exists for 48. | `cvkg-components/src/lib.rs:79-83`, `cvkg-themes/src/lib.rs:521-529` |
| D5 | **Radius labels disagree.** Components: `RADIUS_SM=4`, `MD=6`, `LG=8`, `XL=12`. Themes: `xs=4`, `s=6`, `m=8`, `l=10`, `xl=12`. The token named `RADIUS_MD` therefore carries the value of the themes `s` step, not `m`. | `cvkg-components/src/lib.rs:90-96`, `cvkg-themes/src/lib.rs:530-538` |
| D6 | **Two state-color models with different math and variants.** Core: sRGB ±15% lightness, disabled alpha 0.4, focus ring alpha 0.7, five states. Themes: OKLCH +0.08/−0.10, disabled alpha 0.38, text-on-state computation, seven states (`Error`, `Success`). | `cvkg-core/src/color.rs:100-154`, `cvkg-themes/src/lib.rs:1189-1350` |
| D7 | **String keys collapse distinct roles and fail silently.** `surface_elevated`, `surface_overlay` and `code_bg` all resolve to `surface`; `info`, `active` and `focus_ring` all resolve to `accent`; unknown keys default to `surface`. | `cvkg-components/src/theme.rs:24-25`, `:38`, `:60`, `:69`, `:74-75` |
| D8 | **CSS export disagrees with Rust constants.** `to_css_variables()` emits `--cvkg-font-body: 17px`, `--cvkg-space-md: 12px`, `--cvkg-radius-md: 8px`, while Rust consumers use 16/16/6. It also emits no density or motion/reduced-motion variables. | `cvkg-themes/src/lib.rs:1136-1164`, `cvkg-components/src/lib.rs:57-58`, `:81`, `:92` |
| D9 | **Literal RGBA remains in themed components** despite the module contract at `cvkg-components/src/theme.rs:4-5`: 14 arrays in `interactive/button.rs` (e.g. `:94`, `:252-253`), 3 in `bifrost_tabs.rs`, 1 each in `data_grid.rs`, `dialog.rs`, `dropdown_menu.rs`. | `cvkg-components/src/theme.rs:4-5`, `cvkg-components/src/interactive/button.rs:92-264` |
| D10 | **Density is defined but unconsumed.** `Density` and its multiplier exist on `Theme`/`ThemeBuilder`; no `Density` reference appears in `cvkg-components/src`. `ThemeBuilder::build()` also claims APCA validation but only returns `self.base`. | `cvkg-themes/src/lib.rs:395-413`, `:441`, `:1066-1069`, `:1178-1181` |

## 4. Proposal — single semantic-token model

This section is a **proposal only**. No source file is changed by this document.

```text
primitive colors (OKLCH generation, clamped)
        cvkg-themes::OklchColor / Theme::from_seed
                     │  From<cvkg_themes::SemanticColors>
                     ▼
canonical semantic roles (one struct)  cvkg_core::SemanticColors
        primary / secondary / accent / background / surface / error /
        warning / success / text / text_dim
                     │  typed ComponentTokens resolver (no string keys)
                     ▼
component aliases  button.primary.bg / input.border.focus / table.row.selected …
                     │  ThemeContext (thread-local) + density multiplier
                     ▼
render calls  cvkg-components::theme::*  →  cvkg_core::Renderer
```

Rules for the proposed model:

1. **One canonical role set.** `cvkg_core::SemanticColors` is the only public
   semantic color type. `cvkg_themes::SemanticColors` becomes either a private
   alias or converts via `From`/`Into`; `Theme::colors` stays source-compatible
   but is explicitly marked as the generation-time view of the same roles.
2. **One scale table.** A single `TokenScale` (typography, spacing, radius,
   motion) is generated once and consumed by `theme::font_*`, `SPACE_*`,
   `RADIUS_*` and `to_css_variables()`. Existing names remain as deprecated
   constants that read from this table, so no downstream code breaks.
3. **Typed component aliases replace `color(&str)`.** Define an enum or struct
   with named fields for `surface_elevated`, `input_border_focus`,
   `table_row_selected`, etc. No silent `_ => surface` fallback; an unknown key
   must be unrepresentable at compile time.
4. **One state derivation.** Keep `cvkg-themes::StateColors` (OKLCH) as the
   generator and expose `InteractiveState` with a single variant set. The core
   sRGB `InteractiveColorStates` becomes a deprecated adapter that calls the
   canonical derivation.
5. **Density at resolve time.** `ThemeContext` carries density; spacing/radius
   helpers multiply by `Density::multiplier()`. CSS export emits the same
   resolved values, including reduced-motion and contrast flags.
6. **GPU palette stays separate.** `cvkg_core::ColorTheme` is a 176-byte WGSL
   uniform with a compile-time layout assertion
   (`cvkg-core/src/render_tier.rs:46-52`); it remains a shader parameter block,
   populated from the canonical roles via `oklch_to_color_theme()`
   (`cvkg-themes/src/lib.rs:234-285`) rather than becoming a token source.

## 5. Migration path (preserves public APIs)

- **Phase 1 — bridge, no renames.** Add `From<cvkg_themes::SemanticColors> for
  cvkg_core::SemanticColors`; make `ThemeContext::from_color_theme()` take
  error/warning/success from the theme palette instead of literals (D1, D2).
  Existing signatures unchanged.
- **Phase 2 — reconcile values.** Choose one typography/spacing/radius table,
  make the existing constants delegate to it, and document the chosen numeric
  values in this file. Expect visual diffs; gate them behind a documented
  release note (D3–D5, D8).
- **Phase 3 — typed aliases.** Introduce `ComponentTokens` and migrate the
  `theme::*` wrappers one family at a time; keep `color(&str)` as a deprecated
  compatibility shim with no new callers (D7).
- **Phase 4 — states, density and cleanup.** Unify `InteractiveState`, apply
  density in resolvers, and replace the 20 remaining literal RGBA arrays with
  token calls (D6, D9, D10). Add a CI grep that fails on new literal RGBA arrays
  in `cvkg-components/src`.

## 6. Acceptance criteria for P2-3

- One documented semantic-token model with a named owner crate for every layer.
- `cvkg-themes::SemanticColors` → `cvkg_core::SemanticColors` conversion exists
  and is exercised by a test; no hardcoded error/warning/success in the GPU→CPU
  bridge.
- Every priority component family resolves colors through typed aliases; no new
  literal RGBA arrays in `cvkg-components/src`.
- `to_css_variables()` output and Rust helpers produce the same numeric values
  for font, space and radius keys.
- All existing public constants and `theme::*` functions still compile for
  downstream users; removals are deferred and marked deprecated.

## 7. Verification

```bash
# duplicate semantic types / conversions
rg 'pub struct SemanticColors|impl From<.*SemanticColors' cvkg-core/src cvkg-themes/src

# hardcoded colors in component render paths
rg -c '\[[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+\]' \
  cvkg-components/src --glob '*.rs'

# density consumers (currently none expected)
rg 'Density|multiplier\(\)' cvkg-components/src --glob '*.rs'

cargo test -p cvkg-themes --lib
cargo test -p cvkg-components --test theming_test
```

## 8. Non-goals

- No new token crate and no breaking rename of `Color`, `Theme`, `ThemeBuilder`
  or any `FONT_*`/`SPACE_*`/`RADIUS_*` constant.
- No change to the WGSL `ColorTheme` layout or shader uniform packing.
- No claim that runtime themes are visually consistent: that requires rendered
  snapshots and contrast measurement, which this static audit did not perform.
