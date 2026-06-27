# Gallery Component Audit — Interactive vs. Non-Interactive

## Current Gallery Entries (18 total)

### Forms (5)
| Entry | Current State | Verdict |
|---|---|---|
| Button | Interactive (click handler, disabled state) | **KEEP** — primary interactive component |
| Checkbox | Interactive (toggle state) | **KEEP** — but needs label fix (overlapping boxes) |
| Input | Interactive (text entry) | **KEEP** — core interactive |
| Toggle | Interactive (boolean switch) | **KEEP** — core interactive |
| Slider | Interactive (drag value) | **KEEP** — core interactive |
| Select | **NON-interactive** (static text, no dropdown) | **REPLACE** — should use real Combobox or be removed |

### Layout (3)
| Entry | Current State | Verdict |
|---|---|---|
| VStack | **NON-interactive** (layout primitive) | **REMOVE** — not a gallery-worthy component |
| HStack | **NON-interactive** (layout primitive) | **REMOVE** — not a gallery-worthy component |
| Text | **NON-interactive** (static display) | **REMOVE** — primitive, not a component demo |

### Navigation (1)
| Entry | Current State | Verdict |
|---|---|---|
| Tabs | Interactive (BifrostTabs with selection) | **KEEP** — premium interactive component |

### Overlays (2)
| Entry | Current State | Verdict |
|---|---|---|
| Tooltip | **SEMI-interactive** (visible=true forced, no hover) | **KEEP** — but make it hover-triggered |
| Dialog | **NON-interactive** (static, no open/close) | **REPLACE** — make it a real openable modal |

### Data Display (4)
| Entry | Current State | Verdict |
|---|---|---|
| Progress | **NON-interactive** (animated bar, no user input) | **KEEP** — but as a "Feedback" category, not interactive |
| Spinner | **NON-interactive** (animated, no input) | **KEEP** — but should show loading→loaded transition |
| Badge | **NON-interactive** (static label) | **REMOVE** — too trivial for gallery |
| Avatar | **NON-interactive** (static text) | **REMOVE** — too trivial for gallery |

### Feedback (1)
| Entry | Current State | Verdict |
|---|---|---|
| Alert | **NON-interactive** (static colored text) | **KEEP** — but add auto-dismiss + trigger button |

---

## Missing Interactive Components (in cvkg-components but NOT in gallery)

These are the components that showcase CVKG's interactive capabilities:

| Component | Why It Belongs in Gallery |
|---|---|
| **Combobox** | Real dropdown selection — replaces the fake "Select" entry |
| **CommandPalette** (MimirSpotlight) | Premium search+select overlay — flagship component |
| **AutoComplete** | Interactive typeahead — demonstrates input + suggestion fusion |
| **MentionInput** | Rich input with @mention dropdown — complex interaction |
| **PhoneInput** | Formatted input with validation — real-world form component |
| **DatePicker** / **Calendar** | Complex date selection — high-value interactive |
| **Toast** | Auto-dismissing feedback — demonstrates state lifecycle |
| **HoverCard** | Hover-triggered overlay — demonstrates pointer interaction |
| **ToggleGroup** | Multi-select boolean group — forms upgrade |
| **ButtonGroup** | Segmented control — demonstrates mutual exclusion |
| **Breadcrumb** | Navigation hierarchy — demonstrates path-based interaction |
| **ThemeSwitch** | Dark/light toggle — demonstrates global state mutation |
| **NodeGraphEditor** | Complex drag-and-drop graph editing — flagship demo |
| **MarkdownEditor** | Rich text editing — complex input with formatting |
| **Scheduler** | Gantt-style scheduling — complex data interaction |
| **Slider** (MjolnirSlider) | Already have basic Slider — consider the premium variant |
| **Editor** (SyncWeave) | Collaborative editing — demonstrates multi-cursor |
| **QRCode** | Scannable code generation — demonstrates visual output |

---

## Recommended Gallery (18 entries, interactive-first)

### Forms (6)
1. **Button** — variants: Default, Disabled, Loading, TintedGlass, Ghost
2. **Checkbox** — with proper labels, tri-state (indeterminate)
3. **Input** — text entry with placeholder, validation states
4. **Combobox** — dropdown selection (replaces fake "Select")
5. **DatePicker** — calendar popup with date selection
6. **Toggle** — boolean switch with labels

### Navigation (2)
7. **Tabs** — BifrostTabs with animated indicator (already in gallery)
8. **Breadcrumb** — clickable hierarchy navigation

### Overlays (3)
9. **CommandPalette** — search-driven overlay with keyboard nav
10. **Dialog** — openable modal with confirm/cancel actions
11. **Toast** — auto-dismissing notifications with variants

### Feedback (3)
12. **Alert** — dismissible with trigger button + auto-dismiss
13. **Progress** — with loading→complete transition
14. **Spinner** — Ouroboros variant with loading state

### Data & Input (4)
15. **AutoComplete** — typeahead with suggestion list
16. **PhoneInput** — formatted input with country code
17. **NodeGraphEditor** — drag-and-drop node editing (flagship)
18. **ThemeSwitch** — dark/light mode toggle with live preview

---

## Entries to Remove (6 current entries)

| Entry | Reason |
|---|---|
| VStack | Layout primitive — not a component demo |
| HStack | Layout primitive — not a component demo |
| Text | Primitive — not a component demo |
| Badge | Too trivial — static label only |
| Avatar | Too trivial — static text only |
| Select | Fake (non-interactive text) — replace with Combobox |

## Entries to Fix (2 current entries)

| Entry | Fix |
|---|---|
| Checkbox | Labels overlap each other (click target bug) — needs 22px height + proper spacing |
| Tooltip | Currently forced `visible=true` — should demonstrate hover-triggered appearance |

---

## Design Principles Applied

1. **Every gallery entry must be interactive** — if a user can't click/drag/type/toggle it, it doesn't belong
2. **Showcase state transitions** — loading→loaded, collapsed→expanded, empty→filled
3. **Demonstrate the 8-state system** — default, hover, focus, active, disabled, loading, error, selected
4. **Progressive complexity** — simple forms → complex overlays → flagship editors
5. **No layout primitives** — VStack/HStack/Text are implementation details, not user-facing components
6. **Replace static demos with real interactions** — Dialog should open/close, Toast should auto-dismiss, Alert should trigger
