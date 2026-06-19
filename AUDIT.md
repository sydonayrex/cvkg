# macOS Tahoe Parity Audit — cvkg UI Framework

Scope: `cvkg-materials`, `cvkg-components` (window, chrome/*, navigation, theme), `cvkg-core` rendering trait defaults, `cvkg-render-native` / `cvkg-render-gpu` backends.

---

## 1. Window Chrome

- `cvkg-components/src/window.rs` (`YggdrasilWindow`, `GinnungagapWindow`, `HiminnModal`): no traffic-light controls (close/minimize/zoom) are drawn or hit-tested anywhere in the component layer, despite `cvkg-core` exposing `Minimize`/`Zoom` menu actions (`cvkg-core/src/lib.rs:7619`, `:7468`). There is no component that renders the three-dot control cluster, no hover/press states for it, and no traffic-light layout offset reserved in the header rect (`window.rs:36-92`).
- `cvkg-core/src/window.rs:40-56`: `decorations: bool` is a boolean flag only — no associated drawing path renders title bar buttons when `true`.
- Title bar header height is hardcoded to `28.0` in three different places (`window.rs:37`, `:132`) with no shared constant and no DPI/scale-aware unification with `NornirBar`'s own hardcoded `28.0` (`chrome/nornir_bar.rs:55`) or `ValkyrieToolbar`'s `40.0` (`chrome/valkyrie_toolbar.rs:144`).
- `GinnungagapWindow` "fold" transform path is only active when `Realm::Midgard` is false; the `Midgard` branch returns early with a hard cross-fade switch with no transition at all (`window.rs:178-186`), i.e. two visually unrelated behaviors gated by an unrelated enum (`Realm`) rather than an animation/reduced-motion setting.

## 2. Corner Geometry / "Liquid Glass" Shape Language

- `cvkg-components/src/clipped_corner.rs`: `ClippedCornerNode` draws **chamfered/octagonal** corners (straight diagonal cuts via `draw_line` segments, `:44-93`), not rounded or continuous-curvature corners. This is used directly by `YggdrasilWindow`'s window frame (`window.rs:43-46`).
- A proper `fill_squircle`/`stroke_squircle` (superellipse) API exists (`cvkg-render-gpu/src/api.rs:135-175`, `material_id::SQUIRCLE_STROKE` in `renderer.rs:39`) but:
  - The default trait fallback in `cvkg-core/src/lib.rs:2174-2182` does **not** implement a superellipse — it silently substitutes a plain rounded-rect (`fill_rounded_rect(... width.min(height) * 0.22 ...)`), discarding the `n` parameter entirely (`_n: f32` is unused).
  - No chrome/window component (`window.rs`, `clipped_corner.rs`, `dialog.rs`, `popover.rs`) calls `fill_squircle`/`stroke_squircle` at all — the only call sites are icon-silhouette code paths. Window corners, dialogs, popovers, and toolbar platters all use circular-arc `fill_rounded_rect`/`stroke_rounded_rect` instead of squircle continuity, producing visibly different corner curvature than Tahoe's continuous corner system.
- No shared corner-radius design-token scale exists. `grep` for radius/corner constants in `cvkg-components/src/theme.rs` returns nothing — every component hardcodes its own literal: `4.0`, `6.0`, `8.0`, `10.0`, `12.0`, `16.0` are all used independently across `window.rs`, `clipped_corner.rs`, `niflheim_sidebar.rs`, `valkyrie_toolbar.rs`, `chrome/heimdall_dock.rs`, with no derivation rule tying nested-container radius to parent radius (Tahoe's concentricity rule).

## 3. Glass Material

- `cvkg-materials/src/glass.rs`: `GlassMaterial` has no edge "lensing"/specular rim-light parameter and no light/dark-adaptive tint variant — only a single static linear `tint` regardless of system appearance.
- `roughness`/`refraction` are static scalars with no dependency on view depth, content motion, or scroll position (Tahoe's Liquid Glass reacts to underlying content motion/parallax).
- `fill_glass_rect_with_pressure` (`cvkg-core/src/lib.rs:2163-2169`) documents that "Desktop stub: pressure is always 1.0 for mouse clicks, 0.0 otherwise" but no call site in `cvkg-components` or `cvkg-render-native` actually sets pressure based on click state — the codepath is unreachable from any pointer-event handler found in the audited components.

## 4. Accessibility / System Preference Integration

- `AccessibilityPreferences` (`cvkg-core/src/lib.rs:6538-6633`) correctly detects `reduce_motion`, `reduce_transparency`, and `increase_contrast` from the OS on macOS/Windows/Linux. However:
  - `grep` across `cvkg-components/src` and `cvkg-materials/src` finds **zero** references to `reduce_transparency` or `increase_contrast`. No glass/blur material is ever swapped for an opaque surface, and no border/contrast boost is ever applied — the detection layer is fully disconnected from rendering.
  - `reduce_motion` is referenced once at `cvkg-core/src/lib.rs:725`, but none of the spring/bounce-style animations in `cvkg-anim` or the dock magnification/bounce logic in `chrome/heimdall_dock.rs` check it.
- `Appearance` (light/dark) default is hardcoded: `cvkg-core/src/lib.rs:3971-3973` — `AppearanceKey::default()` returns `Appearance::Dark` unconditionally ("Default to Dark... for Berserker aesthetic") rather than querying the OS appearance setting. Apps that don't explicitly override this env key will not follow the system's actual light/dark setting or "Auto" schedule.

## 5. Menu Bar (`chrome/nornir_bar.rs`)

- `render_submenu` (`:109-150`) only matches `MenuItem::Action` and `MenuItem::Separator`; nested `MenuItem::Submenu` falls into the `_ => {}` catch-all and renders nothing — multi-level (cascading) submenus are silently dropped.
- `MenuItem::Action` carries a `shortcut: Option<KeyboardShortcut>` field (`cvkg-core/src/lib.rs:7409-7414`), but neither the top-level bar loop (`:60-92`) nor `render_submenu` (`:126-148`) ever draws it — no right-aligned keyboard-shortcut glyphs anywhere in the menu system.
- No checkmark/radio-state rendering for toggleable items (no boolean "checked" field/branch at all in `MenuItem`).
- No disabled-state dimming for top-level `MenuItem::Action` items in the bar loop (`:88-92` always draws with `theme::text()`, ignoring `enabled`) even though the submenu renderer does respect `enabled` (`:128-133`) — inconsistent behavior between the two render paths for the same enum variant.
- No app-icon/leading glyph slot reserved at the start of the bar (the bar starts directly with the first `Submenu` label at `x = rect.x + 8.0`).
- No keyboard navigation (arrow-key traversal between top-level menus / into submenus) — only `toggle_menu`/`close_menu` driven by index, no `handle_key` method on `NornirBar`.

## 6. Sidebar (`chrome/niflheim_sidebar.rs`)

- `SidebarItem.children` and `is_expanded` are defined and have builder methods (`:41-49`), but `View::render` (`:193-211`) only iterates `self.items` directly — it never recurses into `item.children`, so nested/collapsible sidebar sections never actually render their child rows regardless of `is_expanded`.
- No disclosure triangle is drawn for items that do have children.
- `render_row`'s `is_selected` branch in the text-color computation is a no-op tautology: `if is_selected { theme::text() } else { theme::text() }` (`:148-152`) — both branches return the same value, so selected/unselected rows never get the distinct selected-row text color real source-list behavior requires.
- Row background highlight radius (`6.0`, `:131`) is a circular-arc rounded rect, not a squircle, and doesn't match `ValkyrieToolbar`'s `12.0` platter radius or `clipped_corner`'s chamfer language elsewhere in the same chrome module — no consistent selection-pill shape across chrome components.
- Separator line at the sidebar's trailing edge is drawn at a fixed `rect.x + rect.width - 0.5` (`:101`) with `1.0`-unit width, with no device-pixel-ratio awareness — at non-1x scale factors this will not align to a crisp hairline.

## 7. Toolbar (`chrome/valkyrie_toolbar.rs`)

- `ToolbarItem::FlexSpace` returns `0.0` from `item_width` (`:192`) and the render loop in `View::render` (`:222-253`) never computes total available width vs. consumed width to redistribute remaining space into `FlexSpace` entries — `FlexSpace` is a declared API with no actual flexible-layout behavior; it behaves identically to a `0`-width spacer.
- Search field renders a literal `"*"` glyph as the search icon (`:319`) rather than a magnifying-glass glyph/icon — placeholder glyph shipped in place of real iconography.
- Segmented control's sliding "pill" indicator is computed purely from `seg.selected` with no interpolation/animation state stored anywhere in `ToolbarSegmented` — selection changes will jump discretely with no slide transition.
- Toolbar item corner radii are independently hardcoded per item type (`6.0` for buttons at `:276`, `h/2.0` pill radius for segmented/search at `:286`/`:315`) with no shared token, and don't match the platter's configurable `self.radius` (default `12.0`).

## 8. Dock (`chrome/heimdall_dock.rs`)

- `dock_item_magnification` only takes a 1D `pointer_x`/`item_center` pair; for `DockPosition::Left`/`DockPosition::Right` the render loop (`:147-150`) still passes `self.pointer_x` as the proximity axis instead of a vertical coordinate — magnification will not respond correctly to pointer position on vertically-oriented docks.
- Item layout positions (`item_center`, `:148`) are computed from a fixed `base_size` step and never adjusted for the current frame's magnification scale of neighboring items — magnified icons will overlap their neighbors instead of displacing them, unlike the real dock's elastic layout.
- `auto_hide: bool` field exists and has a builder (`:92-95`), but `View::render` never branches on it — there is no slide-away/reveal behavior implemented; the field is inert.
- Module doc comment claims "bounce animations" (`:1`) but no bounce/launch animation state exists anywhere in the struct or render function.
- `handle_pointer_move`'s dock-hit detection is `y > 0.0` with a `// Simplified` comment (`:100`), meaning `pointer_in_dock` is `true` for almost any pointer position, not just when the pointer is over the dock platter.

## 9. Cross-Backend Rendering Parity

- `fill_squircle`/`stroke_squircle` have a real superellipse implementation only on the GPU backend (`cvkg-render-gpu/src/api.rs`); the trait-default fallback used elsewhere (`cvkg-core/src/lib.rs:2174-2182`) substitutes a circular rounded-rect with a fixed `0.22` radius ratio that ignores the `n` (squareness) parameter — any caller relying on `n` to vary squircle character will see different shapes depending on which renderer backend is active.
- `cvkg-render-native/src/lib.rs:1836-1847` forwards `fill_squircle`/`stroke_squircle` to an underlying native call, but it is not verified here whether that native path honors `n` either — flagged for follow-up given the GPU/default-trait divergence already confirmed.
