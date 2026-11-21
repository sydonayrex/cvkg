# Component parity audit program

**Status:** addresses fix-plan task P2-1 (component parity audit program), rubric and
first scorecards completed 2026-11-21. This is a **static source audit only**: no
browser run, no rendering, no focus/keyboard event injection, no screen-reader
check. "Met" means source evidence exists; it does not mean runtime verified.

## 1. Scope and reference points

The comparison target is the baseline expected of MUI, Radix, shadcn/ui and
Fluent: a component family publishes interaction states, a keyboard contract,
semantic roles/properties, theme consumption, adaptive layout, density, API docs
and runnable examples. Only seven priority families from `plans/fix-plan-2026-11-20.md:38`
are scored: buttons, inputs, selects, dialogs/overlays, menus, tabs, tables.

Method: read `cvkg-components/src/lib.rs`, the implementation files for each
family, `cvkg-components/src/theme.rs`, `cvkg-core/src/view.rs` and `aria.rs`,
`cvkg-components/tests/`, `cvkg-components/examples/`, and the existing component
docs. Claims in docs are checked against code; a doc statement is never counted
as evidence for itself. Historical claims such as `docs/MIGRATION_AUDIT_shadcn_MUI.md:21-41`
("Complete" for every family) and its "95%+ parity" verdict
(`docs/MIGRATION_AUDIT_shadcn_MUI.md:270`) are **not** adopted here; they predate
this audit and are contradicted by several findings below.

## 2. Rubric (reusable for the next family)

| Dimension | Met when | Static check |
|---|---|---|
| States | Default, hover, focus, active/pressed, disabled and loading/error where applicable are represented in render or state code; visual state has a semantic counterpart. | grep state fields/enums and branch conditions in render path |
| Keyboard | Component registers focus + key handling with actual dispatch (`renderer.register_handler("keydown", …)`), covers activation, arrows, Escape where relevant, and traps focus for modals. | grep handlers; confirm the trait hook has a call site |
| Roles/ARIA | `with_role`/`register_a11y`/`aria_properties` publish role, label and state (checked/selected/expanded/invalid/disabled) through `cvkg-core::AriaProperties`. | compare emitted properties with `cvkg-core/src/aria.rs:61-78` |
| Theming | Colors come from `crate::theme::*` or `use_theme()`; no literal RGBA arrays in the render path. | grep `[f32; 4]` literals per file |
| Responsiveness | Family adapts layout through `Responsive`/`Breakpoint` (`cvkg-components/src/responsive.rs:78-126`) or equivalent; fixed pixel math alone is not enough. | grep family for breakpoint consumption |
| Density | Spacing/radius resolved through `Density::multiplier` (`cvkg-themes/src/lib.rs:395-413`) or a density-aware helper. | grep family for `Density`/`multiplier()` |
| Docs | A `docs/components/<Family>.md` matches the current public signature and makes only verifiable claims. | diff docs vs struct/impl |
| Examples/tests | At least one runnable example in `cvkg-components/examples/` and a test exercising the family. | list files, grep test names |

## 3. Scorecard A — interaction and semantics

| Family | States | Keyboard | Roles/ARIA |
|---|---|---|---|
| Buttons | **Partial** — hover/pressed/disabled/loading branches at `cvkg-components/src/interactive/button.rs:92-170`, `:246-266`. | **Partial** — `on_key_event` handles Enter/Space (`:679-689`) and pointer handlers at `:600-660`, but `View::on_key_event` has no in-tree call site (`cvkg-core/src/view.rs:337`). | **Partial** — role/label at `:51`, `aria_properties` sets `disabled` at `:672-678`; `pressed` is now emitted from VDOM press state. |
| Inputs | **Met (static)** — `InputState` and error/success/focus paths at `cvkg-components/src/interactive/input.rs:9-30`, `:102-117`, `:200-215`. | **Partial** — character insert, Backspace/Delete, arrows, Home/End, Enter keydown at `:381-435`; no `on_key_event` dispatch. | **Partial** — `with_role("textbox")` at `:75`, `register_a11y` at `:276`, properties at `:627-640`; `invalid` now exists in `cvkg-core/src/aria.rs` and Input emits it when `InputState::Error`, so `docs/components/Input.md:40` is now supported. |
| Selects | **Partial** — selected/hover rendering at `cvkg-components/src/interactive/select.rs:203-253`; `native_select.rs:103-181` has disabled/hover colors only. | **Partial** — ArrowDown/Enter/Escape handler at `select.rs:336-390`, `on_key_event` at `:639-648`; `native_select.rs` has no key handler. | **Partial** — `with_role("combobox")` at `select.rs:63`, `aria_properties` at `:454-456`; `expanded`/`value`/`invalid` are now emitted from live state; `listbox`/`option` roles still absent; `native_select.rs` has no role call. |
| Dialogs/overlays | **Partial** — scrim/surface/variant colors at `cvkg-components/src/dialog.rs:139-221`; focus trap in `cvkg-components/src/container/modal.rs:192`, `:443`. | **Partial** — Escape/Enter/Space at `dialog.rs:224-240`, `:422-440`; `popover.rs` has no keydown handler (`:197-330`). | **Partial** — role at `dialog.rs:63`, `register_a11y("alertdialog")` at `:135`, `:355`; `popover.rs:51` claims `aria-expanded`/`aria-modal` but no ARIA call exists in the file. |
| Menus | **Partial** — hover item state via system state at `cvkg-components/src/dropdown_menu.rs:163-204`; navigation menu selection at `cvkg-components/src/navigation.rs:221-247`. | **Partial** — keydown in `navigation.rs:249`, `:393`, `:513`, `:729`; `dropdown_menu.rs:94-283` registers pointer handlers only; `context_menu.rs` has no key handler. | **Missing** — no `with_role`/`register_a11y`/`aria_properties` in `dropdown_menu.rs`, `context_menu.rs` or `navigation.rs`. `keyboard_nav.rs:210-242` (`FocusTrap`) and `:375-430` (`ArrowNav`) are opt-in wrappers, not defaults. |
| Tabs | **Partial** — selected tab alpha/color at `cvkg-components/src/bifrost_tabs.rs:170-196`. | **Partial** — ArrowLeft/Right + W keydown at `bifrost_tabs.rs:260-289`. | **Partial** — `with_role("tablist")` at `:64`, but `aria_properties` returns only `Tablist` + static label (`:306-308`); no per-tab role or `selected` state. |
| Tables | **Partial** — selected/bulk-selected row visuals at `cvkg-components/src/data_grid.rs:309-324`. | **Missing** — only `pointerclick` handlers at `data_grid.rs:261`, `:390`; the keyboard claim at `:21-22` has no implementation. | **Missing** — no role, `aria-sort` or `aria-selected` emitted; `data_grid.rs:23` claims all three. |

## 4. Scorecard B — system and delivery

| Family | Theming | Responsiveness | Density | Docs | Examples/tests |
|---|---|---|---|---|---|
| Buttons | **Partial** — `theme::*` used, but 14 literal RGBA arrays remain, e.g. `button.rs:94`, `:252-253`. | **Missing** — no breakpoint use in `button.rs`. | **Missing** — no `Density` use. | **Partial** — `docs/components/Button.md` exists; `:3` calls `BifrostButton` canonical, but the struct is `Button` (`button.rs:17`). | **Met** — `cvkg-components/examples/basic_button_demo.rs`, `test_button.rs`, `interactive_demo.rs`; tests in `cvkg-components/tests/component_tests.rs`. |
| Inputs | **Met** — all colors via `theme::*` in `input.rs` (no literal arrays). | **Missing** — fixed 200x44 size at `input.rs:619-626`. | **Missing** — no density use. | **Partial** — docs exist; `aria-invalid` claim at `Input.md:40` is now implemented. | **Met** — multiple examples; `cvkg-components/tests/component_tests.rs`. |
| Selects | **Met** — `theme::*` throughout `select.rs:154-253`, `native_select.rs:103-181`. | **Missing** — no breakpoint use. | **Missing** — no density use. | **Partial** — `docs/components/Select.md` exists and documents the generic constraint, but has no keyboard/ARIA section. | **Partial** — one example references `Select`; no family-specific test. |
| Dialogs/overlays | **Met** — `theme::*` in `dialog.rs:139-221`, `popover.rs:197-330`. | **Missing** — no breakpoint use. | **Missing** — no density use. | **Missing** — `docs/components/README.md:33-34` marks Dialog/Popover "coming soon". | **Missing** — no example or test file names the family. |
| Menus | **Met** — `theme::*` in `dropdown_menu.rs:154-236`, `navigation.rs:98-128`. | **Missing** — no breakpoint use. | **Missing** — no density use. | **Missing** — README marks navigation docs "coming soon" (`docs/components/README.md:31`). | **Missing** — no example/test names the family. |
| Tabs | **Partial** — `theme::*` plus 3 literal RGBA arrays in `bifrost_tabs.rs`. | **Missing** — no breakpoint use. | **Missing** — no density use. | **Missing** — README marks Tabs "coming soon" (`docs/components/README.md:31`). | **Missing** — no example/test names the family. |
| Tables | **Met** — `theme::*` in `data_grid.rs:220-324`. | **Missing** — no breakpoint use. | **Missing** — no density use. | **Missing** — no `docs/components` file; index lists DataGrid only in `docs/COMPONENTS.md:93`. | **Missing** — no example/test names the family. |

## 5. Prioritized gap list

1. **G1 — keyboard trait hook is dead code.** `View::on_key_event` is declared at
   `cvkg-core/src/view.rs:337` and implemented by `button.rs:679`, `select.rs:639`,
   `checkbox.rs:312`, but no call site exists in `cvkg-core`, `cvkg-vdom` or any
   renderer. Runtime keyboard behavior depends entirely on ad-hoc
   `register_handler("keydown", …)` per component; Button has one, Select has one,
   Table has none.
2. **G2 — ARIA state coverage is shallow (partially addressed).** `invalid` was
   added to `cvkg-core/src/aria.rs` and Input now emits it; Select/Tabs still
   returns only role/label/value/focused/input disabled (`input.rs:627-633`,
   `select.rs:454-456`, `bifrost_tabs.rs:306-308`). Menus and tables publish no
   role at all.
3. **G3 — docs drift creates false parity claims.** `docs/components/Button.md:3`
   names a non-existent canonical type; `docs/components/Input.md:40` claims an
   ARIA attribute the data model cannot express; `data_grid.rs:21-23` and
   `popover.rs:51` claim keyboard/ARIA behavior absent from their code;
   `docs/components/README.md:11-28` uses lowercase file links that do not match
   the actual `Button.md`/`Input.md` filenames, and `:30-34` marks Tabs, Dialog
   and Popover "coming soon".
4. **G4 — density and responsiveness are defined but not consumed.**
   `Density::multiplier` (`cvkg-themes/src/lib.rs:404-412`) has no reference in
   `cvkg-components/src`; `Responsive` (`responsive.rs:78-126`) is not used by any
   priority family.
5. **G5 — hardcoded colors in a theme-aware library.** 14 literal RGBA arrays in
   `button.rs`, 3 in `bifrost_tabs.rs`, 1 each in `data_grid.rs`, `dialog.rs`,
   `dropdown_menu.rs`, against the `theme.rs:4-5` contract that components never
   hardcode colors.
6. **G6 — delivery gaps for overlays, menus, tabs and tables.** No docs, no
   examples and no dedicated tests for these families (Section 4).

## 6. Recommended fix order

1. **Wire or remove `View::on_key_event`** (G1): add one dispatch point in the
   VDOM/native event path, then make each family use it instead of duplicating
   `register_handler` logic. Add a test that injects Enter/Space/Tab/Escape.
2. **Emit `selected` state and listbox/option roles** (`invalid`/`expanded`/`value` are done for Input/Select) and emit
   them from Input, Select, Tabs (G2); add menu/table roles and per-item state.
3. **Correct docs to match code** (G3) and delete or qualify the parity verdict in
   `docs/MIGRATION_AUDIT_shadcn_MUI.md:270,277` before it is used by adopters.
4. **Add per-family docs, examples and tests** for dialogs/overlays, menus, tabs
   and tables (G6); re-run this scorecard after each family lands.
5. **Apply density and route literal colors through tokens** (G4, G5), coordinated
   with `docs/quality/token-consolidation.md`.

## 7. Verification

```bash
# confirm the keyboard trait hook has no dispatch call site (expect definitions only)
rg 'on_key_event' --glob '*.rs' --glob '!old/**' --glob '!target/**' .

# confirm ARIA fields available to components
sed -n '61,78p' cvkg-core/src/aria.rs

# re-count literal RGBA arrays in priority families
for f in cvkg-components/src/interactive/button.rs \
         cvkg-components/src/bifrost_tabs.rs \
         cvkg-components/src/data_grid.rs; do
  rg -c '\[[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+, *[0-9]+\.[0-9]+\]' "$f"
done

cargo test -p cvkg-components --tests
```

A dynamic follow-up (browser/native focus injection, screen-reader tree dump,
reduced-motion and high-contrast checks) is required before any rating in this
document is upgraded from static to runtime-verified.
