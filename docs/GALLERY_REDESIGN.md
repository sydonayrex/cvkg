# cvkg Component Showcase — Redesign Spec

**Date:** 2026-07-12  
**Status:** Draft  
**Related:** `cvkg-gallery/`, `cvkg-components`

---

## 1. Problem Statement

The current `cvkg-gallery` is a 3D carousel tech demo that obscures the components it claims to showcase. It's poorly planned from a UI/UX standpoint:

- Carousel is a liability (distracts, not demonstrates)
- No discoverability (search/filter)
- Minimal previews (1-2 states max)
- No API documentation
- Hardcoded entry list
- No theming/variant exploration
- Poor keyboard accessibility

**Goal:** Build a showcase that actually helps developers discover, understand, and test cvkg components.

---

## 1.1 Design Direction

**Hybrid: Storybook + macOS Tahoe**

- Use **NiflheimSidebar** (existing source-list component) as the navigation — macOS Finder-style collapsible groups
- Use **MimirSpotlight** (existing command palette) for search — invoke with `Cmd+K` / `Ctrl+K`
- **macOS Tahoe design language** — translucent vibrancy, rounded corners, muted color palette, SF-style typography
- **Storybook layout** — canvas preview on the left/center, props panel on the right

---

## 2. Target Users

1. **Internal developers** — browse available components, copy-paste usage
2. **API consumers** — understand props, variants, behavior
3. **QA/testers** — verify component states, edge cases
4. **Designers** — review component consistency, theming

---

## 3. Design Principles

| Principle | Description |
|-----------|-------------|
| **Component-first** | The component is the hero, not the container |
| **OS-native feel** | Use existing cvkg components that feel like macOS native apps |
| **Discoverable** | Spotlight search + sidebar categories — find anything in 2 clicks |
| **Interactive** | Live props manipulation, not just static previews |
| **Documented** | Show API inline with the component |
| **Accessible-first** | Keyboard nav, screen reader landmarks |

---

## 4. Proposed Layout (macOS Tahoe + Storybook)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  TOOLBAR (translucent, 48px, blur backdrop)                                   │
│  [◉] [Search components...            ] [⊙] Theme [⊞] View                  │
│      └───────── Cmd+K spotlight trigger ──────────┘                         │
├──────────────┬──────────────────────────────────────────────────────────────┤
│  SIDEBAR     │  CANVAS (center, flexible, checkered/lite/dark bg)           │
│  (240px)     │                                                               │
│  Niflheim    │    ┌─────────────────────────────────────────────────────┐   │
│  Source List │    │                                                             │   │
│              │    │         Live Component Render                           │   │
│  ▼ Forms     │    │                                                             │   │
│    • Button  │    │         (interactive, full bleed, scaleable)           │   │
│    • Input   │    │                                                             │   │
│    • Select  │    └─────────────────────────────────────────────────────┘   │
│  ▼ Overlays  │                                                               │
│    • Dialog  │  ┌───────────────────────────────────────────────────────┐   │
│    • Popover │  │  PROPS PANEL (collapsible, 280px, right sidebar)        │   │
│  ▼ Layout    │  │  ├─ variant: [Primary ▼]                                │   │
│    • VStack  │  │  ├─ disabled:  [○]                                      │   │
│    • HStack  │  │  └─ on_click: () => {...}                                │   │
│              │  └───────────────────────────────────────────────────────┘   │
│              │                                                               │
│              │  ┌───────────────────────────────────────────────────────┐   │
│              │  │  API DOCS (tabbed, below canvas)                        │   │
│              │  │  [Props] [Usage] [Accessibility] [Changelog]           │   │
│              │  └───────────────────────────────────────────────────────┘   │
└──────────────┴──────────────────────────────────────────────────────────────┘
```

### Visual Design (macOS Tahoe)

| Element | Style |
|---------|-------|
| **Window chrome** | Translucent blur, rounded corners (12px), full-height sidebar |
| **Sidebar** | `NiflheimSidebar` with `SidebarVibrancy::Translucent`, source-list style |
| **Toolbar** | Translucent material, separator line, inline search field |
| **Colors** | System palette — `systemBackground`, `secondarySystemBackground`, `tertiarySystemFill` |
| **Typography** | SF Pro — 13px body, 11px caption, semibold headers |
| **Spacing** | 8px grid — sidebar item height 28px, section gap 16px |
| **Animations** | Spring-based (existing `motion` component), 200-300ms |
| **Shadows** | `shadow(.elevation(3))` for floating panels |

---

## 5. Core Features

### 5.1 Sidebar Navigation (NiflheimSidebar)

- **Use existing `NiflheimSidebar`** — macOS source-list style with collapsible groups
- **Auto-generated from component metadata** — walks the component registry
- **Category tree** — forms, overlays, layout, data-display, feedback
- **Collapsible groups** — click triangle to expand/collapse (use existing DisclosureGroup pattern)
- **Recent items** — last 5 viewed (persisted in localStorage)
- **Keyboard nav** — arrow keys, Enter to select, Esc to collapse
- **Vibrancy** — `SidebarVibrancy::Translucent` for the glass effect

### 5.2 Search / Command Palette (MimirSpotlight)

- **Use existing `MimirSpotlight`** — invoke with `Cmd+K` (macOS) / `Ctrl+K` (Windows)
- **Fuzzy match** — "btn" finds Button, "inp" finds Input
- **Actions** — type `>` for commands (e.g., `>theme dark`, `>scale 150%`)
- **Recent searches** — show last 5 searches
- **Category prefixes** — type category name to filter (e.g., `forms button`)

### 5.3 Canvas (Live Preview)

- **Full-bleed render** — component fills available space
- **Background toggle** — light / dark / checkered / transparent
- **Scale control** — 50% / 75% / 100% / 150% / 200%
- **Responsive preview** — resize viewport to mobile/tablet/desktop presets
- **Screenshot mode** — capture current state as PNG (for docs)

### 5.4 Props Panel (Interactive)

- **Live manipulation** — change props, see instant updates
- **Type-aware editors**:
  - `bool` → toggle switch
  - `enum` → dropdown
  - `string` → text input
  - `number` → slider + input
  - `callback` → code editor or event log
- **Preset states** — buttons: "default", "hover", "active", "disabled"
- **Export current config** — copy as Rust/JSON

### 5.5 API Docs Tab

- **Props table** — name, type, required, default, description
- **Usage example** — copy-pasteable code block with syntax highlighting
- **Accessibility notes** — ARIA attributes, keyboard interactions
- **Version history** — when props were added/changed

### 5.6 Theme Switcher

- **In-theme preview** — toggle light/dark/high-contrast
- **Scale** — text size multiplier (for accessibility testing)
- **Motion** — toggle reduced-motion (respects `prefers-reduced-motion`)

---

## 6. Leveraged cvkg Components

This redesign intentionally reuses existing cvkg components rather than building new ones:

| cvkg Component | Use In |
|---------------|--------|
| `NiflheimSidebar` | Main navigation sidebar (source-list style) |
| `MimirSpotlight` | Search / command palette (`Cmd+K` trigger) |
| `Dialog` | Props panel modal for complex prop editing |
| `VStack` / `HStack` | Layout structure |
| `Input` / `Select` | Props editors |
| `Toggle` / `Checkbox` | Boolean prop editors |
| `DisclosureGroup` | Collapsible sidebar sections |
| `motion` | Smooth panel transitions |
| system state | Theme persistence, recent items |
| `sonner` (toast) | Notifications (copy success, errors) |

This approach:
1. **Dogfoods your own components** — the showcase demonstrates cvkg's own UI quality
2. **Reduces dev effort** — no new primitives needed
3. **Ensures consistency** — if a component is good enough for the showcase, it's good enough for users

### 6.1 Auto-Registration

Instead of hardcoding entries in `main.rs`, derive them automatically:

```rust
// pseudocode — in cvkg-components or a separate proc-macro
#[component]
pub struct Button {
    /// Button variant style
    pub variant: ButtonVariant,
    /// Whether the button is interactive
    pub disabled: bool,
    /// Click handler
    pub on_click: Option<Callback<()>>,
}

// Generates metadata:
Button::metadata() -> ComponentMetadata {
    name: "Button",
    category: "Forms",
    props: [
        PropMeta { name: "variant", ty: "ButtonVariant", required: false, default: "Primary" },
        PropMeta { name: "disabled", ty: "bool", required: false, default: "false" },
        PropMeta { name: "on_click", ty: "Option<Callback>", required: false, default: "None" },
    ],
    // ...
}
```

- **Option A:** Proc-macro in `cvkg-components` that generates `fn metadata()` per component
- **Option B:** Reflection-like trait `ComponentMeta` implemented manually (lower effort, higher maintenance)

### 6.2 State Management

- **URL-based state** — `/button?variant=secondary&disabled=true` — shareable, bookmarkable
- **Local persistence** — recent items, theme preference, last viewport size
- **Event log** — capture all callback invocations for debugging

### 6.3 Component Isolation

Each showcase component runs in its own render context:

```rust
fn render_showcase<R: Renderer>(
    component: &dyn ComponentMeta,
    props: &ComponentProps,
    canvas_rect: Rect,
    renderer: &mut R,
) {
    // Create fresh component instance with current props
    let instance = component.instantiate(props);
    // Render into canvas — no shared state pollution
    instance.render(renderer, canvas_rect);
}
```

### 6.4 Integration Points

| Existing Component | Reuse In |
|--------------------|----------|
| `MimirSpotlight` | Search / command palette |
| `Dialog` | Props panel modal |
| `VStack` / `HStack` | Layout structure |
| `Input` / `Select` | Props editors |
| `Toggle` / `Checkbox` | Boolean prop editors |
| system state | Theme persistence, recent items |

---

## 7. Phased Implementation Plan

### Phase 1: Foundation (MVP)

- [ ] New layout scaffold — sidebar + canvas + props panel structure
- [ ] Reuse `NiflheimSidebar` for component navigation (populated from hardcoded list for now)
- [ ] Basic component switching (click sidebar → render in canvas)
- [ ] Kill the 3D carousel (remove complexity from old gallery)
- [ ] Toolbar with search field

### Phase 2: Search Integration

- [ ] Integrate `MimirSpotlight` for `Cmd+K` search
- [ ] Fuzzy matching on component names
- [ ] Recent searches (persisted)

### Phase 3: Interactivity

- [ ] Props panel with type-aware editors (reuse `Input`, `Select`, `Toggle`)
- [ ] Live component updates on prop change
- [ ] Preset state buttons
- [ ] Background / scale toggles

### Phase 4: Documentation

- [ ] Props table rendering
- [ ] Usage code block generation
- [ ] Copy-to-clipboard buttons (show success via `sonner` toast)

### Phase 5: Auto-Registration (Future)

- [ ] Proc-macro or metadata trait for components
- [ ] Remove hardcoded entry list
- [ ] Auto-generate sidebar from metadata

**Note:** Each phase builds on existing cvkg components, so the showcase itself demonstrates the library's quality.

---

## 8. File Structure

```
cvkg-gallery/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, window setup
│   ├── app.rs            # Main showcase app (new)
│   ├── sidebar.rs        # Category nav, recent (new)
│   ├── canvas.rs         # Live render area (new)
│   ├── props_panel.rs    # Props editor (new)
│   ├── docs_tab.rs      # API docs display (new)
│   ├── state.rs          # Shared GalleryState
│   ├── registry.rs      # Component metadata (new)
│   └── commands.rs      # Spotlight commands (new)
├── tests/
│   └── ...
└── examples/
    └── ...
```

---

## 10. Comparison: Current vs. Proposed

| Aspect | Current (`cvkg-gallery`) | Proposed (Storybook + macOS Tahoe) |
|--------|--------------------------|------------------------------------|
| **Layout** | 3D carousel + split panel | Sidebar + canvas + props panel |
| **Navigation** | Rotating cards | NiflheimSidebar (source-list) |
| **Search** | None | MimirSpotlight (Cmd+K) |
| **Interactivity** | Static | Live props manipulation |
| **Documentation** | None | Inline API docs |
| **Entries** | Hardcoded ~25 | Auto-registered (future) |
| **Theming** | None | Light/dark/scale toggles |
| **Accessibility** | Poor | Full keyboard + ARIA |
| **Shareability** | None | URL-based state |
| **Visual style** | Generic | macOS Tahoe native |
| **Dev effort** | ~1300 lines | ~2500 lines (but reuses existing components) |
| **Dogfooding** | No | Yes — uses NiflheimSidebar, MimirSpotlight, etc. |

---

## 11. Open Questions

1. **Registry strategy:** Proc-macro (auto) vs. manual trait impl (simpler)?
2. **Props editing:** How to handle complex callback props in the UI?
3. **Responsive preview:** Need preset viewport sizes — which breakpoints?
4. **Screenshot export:** Use `gpu` capture or render-to-texture?
5. **Backward compat:** Keep old gallery as hidden demo mode?
6. **Window chrome:** Should we use native title bar or custom translucent toolbar?
7. **Accent color:** Pull from system or allow custom in showcase?
8. **Sidebar width:** Fixed 240px or resizable?
9. **Slider selection:** Use `Slider` (interactive/button.rs) or `MjolnirSlider` (mjolnir_slider.rs)?

---

## 12. Gap Analysis — Components Needed

The redesign reuses existing cvkg components. Here's what's available vs. what's needed:

| Feature | Needed For | Available? | Component |
|---------|-----------|------------|-----------|
| Tab navigation | Props/Usage/Accessibility tabs | ✅ Yes | `BifrostTabs` |
| Code display | Usage examples | ✅ Yes | `Codeblock` |
| Code editing | Advanced prop editing | ✅ Yes | `RunestoneEditor` |
| Slider | Number prop editor | ✅ Yes | `Slider` or `MjolnirSlider` |
| Segmented buttons | View mode, scale toggles | ✅ Yes | `SegmentedControl` |
| Toast notifications | Copy success feedback | ✅ Yes | `Sonner` |
| Color picker | Accent color setting | ✅ Have | `ColorPicker` |
| Copy button | Copy props/code to clipboard | ✅ Have | `CopyButton`, `CutButton`, `PasteButton` (newly created) |

### Components Added

- **`CopyButton`** (`interactive/clipboard.rs`) — copies text to clipboard, shows "Copied!" feedback
- **`CutButton`** — cuts text to clipboard (copy + delete pattern)
- **`PasteButton`** — pastes from clipboard, callback receives the pasted text

All three use `arboard` crate (already a dependency) and include ARIA labels, disabled states, and visual feedback icons.

---

## 13. System Integration Review

### The cvkg Rendering Architecture

cvkg has a sophisticated rendering stack that components should integrate with:

| Layer | Purpose | Key APIs |
|-------|---------|----------|
| **cvkg-vdom** | Virtual DOM tree, scene graph | `VNode`, `VDom`, `NodeId` |
| **cvkg-layout** | Constraint-based layout | `LayoutView`, `SizeProposal`, `LayoutCache` |
| **cvkg-anim** | Spring-based animations | `Spring`, `SpringParams` |
| **cvkg-flow** | Flow/layout computation | Flow nodes |
| **cvkg-render-gpu** | GPU rendering | `push_vnode`, world-space panels |
| **cvkg-render-native** | Native 2D rendering | Same renderer trait |

### Key cvkg-Specific Features NOT Used by cvkg-components

1. **`Companion` states** — Auto-initialized per-VNode state (like Bevy's components)
   - `FocusableCompanion` — focus management
   - `A11yCompanion` — accessibility properties
   - Custom companions can be defined
   
2. **`WorldSpacePanel`** — Renders VDOM subtree to offscreen texture and composites as 3D quad
   - Enables true 3D UI (not just 2D UI in 3D space)
   - Supports glass materials, physics, spring settling
   
3. **`Transform3D`** — Full 3D transform (position, rotation, scale)
   
4. **`sdf_shape`** — Precise hit-testing via signed distance fields

5. **Proper `LayoutView` integration** — Components should:
   - Accept subviews via the layout system
   - Use `place_subviews` to position children
   - NOT implement stub LayoutView that ignores subviews

### Current Component Issues

#### Issue 1: Stub LayoutView Implementations

Most components implement `LayoutView` but completely ignore subviews:

```rust
// Typical pattern in cvkg-components (BAD):
impl LayoutView for Button {
    fn size_that_fits(&self, proposal, subviews, cache) -> Size { ... }
    
    fn place_subviews(&self, bounds, subviews, cache) {
        // Empty! Ignores all subviews completely
    }
}
```

This defeats the purpose of the layout system. A proper integration would:
- Actually place content in the button
- Support child views passed to the component

#### Issue 2: No Companion State Usage

None of the components implement `companion_states()`:

```rust
// Missing from ALL cvkg-components:
impl View for Button {
    fn companion_states(&self) -> Vec<Box<dyn Companion>> {
        vec![Box::new(FocusableCompanion::new())]
    }
}
```

This means:
- No automatic focus management
- No auto-initialized accessibility state
- Components don't participate in the VDOM companion system

#### Issue 3: No 3D/World-Space Support

Components have no way to:
- Render to a `WorldSpacePanel`
- Participate in 3D transforms
- Enable glass materials on their subtrees
- Be positioned in 3D scene space

This is the "2D-only" problem you identified.

#### Issue 4: No VDOM-Aware Rendering

Components use `push_vnode` (good) but:
- Don't set `sdf_shape` for precise hit-testing
- Don't configure `world_space` for 3D panels
- Don't participate in portal/portal_target patterns

### Recommendations for Proper cvkg Integration

#### For LayoutView:

```rust
// Proper pattern (GOOD):
impl LayoutView for Button {
    fn size_that_fits(&self, proposal, subviews, cache) -> Size {
        // If we have subviews, propose their size and add padding
        // If no subviews, use intrinsic size based on label
    }
    
    fn place_subviews(&self, bounds, subviews, cache) {
        // Place subviews centered or per content distribution
        for (i, subview) in subviews.iter_mut().enumerate() {
            subview.place(bounds, cache);
        }
    }
}
```

#### For Companion States:

```rust
impl View for Button {
    fn companion_states(&self) -> Vec<Box<dyn Companion>> {
        vec![
            Box::new(FocusableCompanion::new()),
            Box::new(A11yCompanion { role: "button".into(), ..Default::default() }),
        ]
    }
}
```

#### For 3D Support (future):

Add a `world_panel` prop to enable world-space rendering:

```rust
pub struct Button {
    // ... existing props
    pub world_panel: Option<WorldSpacePanel>,
}

impl View for Button {
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if let Some(panel) = &self.world_panel {
            renderer.set_world_space_panel(panel.clone());
        }
        // ... rest of render
    }
}
```

### Gap Summary

| Feature | Status in cvkg-components |
|---------|---------------------------|
| Meaningful LayoutView | ❌ Mostly stub implementations |
| Companion states | ❌ Not implemented anywhere |
| WorldSpacePanel/3D | ❌ Not available |
| SDF shapes | ❌ Not used |
| Portal targets | ❌ Not used |
| Focus management | Manual, not auto-inited |

This confirms your suspicion: components were built for 2D rendering and only have a "slapped on" LayoutView that doesn't meaningfully integrate with cvkg's systems.

---

## 14. References

- [Storybook](https://storybook.js.org/) — industry standard, main inspiration
- [macOS Tahoe](https://developer.apple.com/design/whats-new/human-interface-guidelines/ios) — translucent materials, sidebar navigation
- [Radix UI Primitives](https://www.radix-ui.com/) — accessible component library with great docs
- [Shadcn/ui](https://ui.shadcn.com/) — copy-paste component model
- [Blueprint (Palantir)](https://blueprintjs.com/) — desktop-style component showcase
- **Existing cvkg components:**
  - `NiflheimSidebar` — `/cvkg-components/src/chrome/niflheim_sidebar.rs`
  - `MimirSpotlight` — `/cvkg-components/src/command_palette.rs`
  - `DisclosureGroup` — `/cvkg-components/src/navigation.rs`
- **cvkg core systems:**
  - `Companion` trait — `/cvkg-core/src/companion.rs`
  - `LayoutView` trait — `/cvkg-core/src/layout.rs`
  - `WorldSpacePanel` — `/cvkg-vdom/src/vnode.rs`
  - `VNodeRenderer` — `/cvkg-vdom/src/lib.rs`

---

*End of spec.*