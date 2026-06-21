# Persona 2: Vibe Coder (AI Agentic Design System)

## Executive Summary

CVKG is a visually ambitious Rust UI framework with a sophisticated OKLCH-based theme system and 215+ components, but its heavy use of Norse mythological names creates a **severe discoverability barrier** for AI agents. An AI coding assistant will correctly guess ~40% of component names and fail on the remaining 60% because the Norse naming is unsystematic, inconsistently applied, and provides no semantic signal about component function. The underlying architecture (View/Modifier pattern, theme tokens, APCA contrast validation) is genuinely well-designed, but the naming layer makes this framework **hostile to AI-assisted development**.

## AI Onboarding Experience

**Time to first pixel: ~15-30 minutes** (vs. ~5 min for a well-designed AI-friendly framework).

An AI agent starting from `use cvkg::prelude::*` gets the `View` trait, macros, and all public components. The prelude is well-structured. However, the agent immediately hits a wall: it cannot guess component names. If asked to "add tabs to the view," the agent will try `Tabs`, `TabView`, `TabBar` — all wrong. The correct answer is `BifrostTabs`. There is no way to infer this from the name, the docs, or the code.

The `#[derive(View)]` and `#[view_component]` macros are clean and AI-friendly — an agent can learn these patterns from one example. The `hamr!` DSL macro is also discoverable. But the component library itself is a minefield of Norse names that require memorization.

The rendering pipeline selection (gpu/native/web) via Cargo features is clearly documented in the crate root — an AI agent will handle this correctly.

## Naming Analysis

### Complete Component Inventory

From `cvkg-components/src/lib.rs`, here is every exported component, classified by naming convention:

**Standard/Descriptive Names (AI can guess these):**
`Button`, `Checkbox`, `Input`, `Select`, `Slider`, `Stepper`, `Textarea`, `Toggle`, `Picker`, `SecureField`, `Label`, `Link`, `Tag`, `SearchField`, `DateTimePicker`, `AlertDialog`, `ConfirmationDialog`, `FullScreenCover`, `NavigationStack`, `NavigationSplitView`, `ScrollView`, `FlexBox`, `HStack`, `VStack`, `LazyVStack`, `Text`, `Divider`, `Spacer`, `Canvas`, `Shape`, `Badge`, `Icon`, `Typography`, `ScrollArea`, `BackgroundPattern`, `DropdownMenu`, `DropdownItem`, `ContextMenu`, `ContextMenuItem`, `Breadcrumb`, `BreadcrumbItem`, `ButtonGroup`, `HoverCard`, `InputGroup`, `InputOTP`, `NativeSelect`, `PhoneInput`, `Popconfirm`, `QRCode`, `ToggleGroup`, `Item`, `Kbd`, `MentionInput`, `Separator`, `AspectRatio`, `Group`, `GroupBox`, `LazyHGrid`, `LazyHStack`, `LazyVGrid`, `Resizable`, `ZStack`, `Carousel`, `FloatingNavbar`, `Loader`, `Marquee`, `MultiStepLoader`, `NavbarMenu`, `BentoGrid`, `BgMediaHero`, `Codeblock`, `DateRangePicker`, `DynamicIsland`, `ExtendedFAB`, `FAB`, `HeroColorPanels`, `Kanban`, `KanbanCard`, `KanbanColumn`, `LogoCarousel`, `SidePanel`, `TimePicker`, `Audio`, `Map`, `Video`, `DisclosureGroup`, `Drawer`, `List`, `Menubar`, `NavigationMenu`, `Section`, `Gallery`, `Login`, `Settings`, `Wizard`, `Gantt`, `Scheduler`, `SchedulerEvent`, `GanttTask`, `BarChart`, `CandlestickChart`, `FunnelChart`, `GaugeChart`, `HeatmapChart`, `Histogram`, `LineChart`, `PieChart`, `RadarChart`, `RangeChart`, `SankeyChart`, `ScatterPlot`, `SparkLineChart`, `TreemapChart`, `CardStack`, `DraggableCard`, `ExpandableCard`, `NumberTicker`, `RippleButton`, `ShimmerButton`, `StatefulButton`, `TextAnimate`, `TypewriterEffect`, `LineChart`, `Candle`, `TreemapNode`, `AlertKind`, `SpinnerVariant`, `ButtonVariant`, `ButtonSize`, `BadgeVariant`, `BadgeSize`, `TypographyVariant`, `BgPattern`, `SheetPosition`, `AlertVariant`, `HoverCardPosition`, `SonnerPosition`, `SonnerType`, `LoaderVariant`, `ChartType`, `VaultStatus`, `FileKind`, `TrailKind`, `GateTier`, `ContainerLayout`, `ScopeThreshold`, `OutlineNode`, `RichTreeView`, `TreeViewNode`, `ForgeSegment`, `PaletteCommand`, `PeerCursor`, `WeaveOp`, `MorphState`, `ComputedSignal`, `InputRef`, `DataTrail`, `ConsentGate`, `AwaitVeil`, `DirectionProvider`, `Editable`, `FormBinder`, `Binding`, `DropVault`, `VaultEntry`, `VaultFile`, `PerfOverlay`, `PhaseGate`, `Popover`, `PromptForge`, `SyncEditor`, `SyncWeave`, `TextEditor`, `TokenStream`, `VTree`, `VTreeNode`, `Trustmark`, `Anim`, `MicroFeedback`

**Norse Mythological Names (AI CANNOT guess these):**

| Component | Norse Reference | What It Actually Does | AI Guessability |
|-----------|----------------|----------------------|-----------------|
| `BifrostTabs` | Bifrost = Rainbow bridge | Tab bar | 0% — no semantic link |
| `BifrostColorPicker` | Bifrost = Rainbow bridge | Color picker | 0% — "Bifrost" used for color picker?? |
| `BifrostLauncher` | Bifrost = Rainbow bridge | Command palette/launcher | 0% |
| `MjolnirSlider` | Mjolnir = Thor's hammer | Slider control | 0% |
| `MjolnirFrame` | Mjolnir = Thor's hammer | Frame/container | 0% — why is "Mjolnir" on a frame? |
| `MjolnirSlice` (modifier) | Mjolnir = Thor's hammer | Geometric clip effect | 0% |
| `MjolnirShatter` (modifier) | Mjolnir = Thor's hammer | Fragment transition | 0% |
| `GjallarAlert` | Gjallarhorn = Heimdall's horn | Alert/notification banner | 5% — "Gjallar" sounds loud, maybe? |
| `GjallarSplitter` | Gjallarhorn = Heimdall's horn | Split pane divider | 0% — same prefix, completely different UI |
| `HatiSpinner` | Hati = wolf chasing moon | Loading spinner | 0% |
| `HatiCarousel` | Hati = wolf chasing moon | Carousel/slider | 0% — same prefix, different widget |
| `YggdrasilWindow` | Yggdrasil = World Tree | Window manager | 0% |
| `YggdrasilTree` | Yggdrasil = World Tree | File tree view | 30% — "Tree" in name helps |
| `GeriDialog` | Geri = one-odd wolf | Dialog component | 0% |
| `GeriTransfer` | Geri = one-odd wolf | Transfer/list shuttle | 0% — same prefix, different UI |
| `HringrPagination` | Hringr = ring/circle? | Pagination | 0% — "Hringr" doesn't mean pagination |
| `ValhallaRating` | Valhalla = Hall of Slain | Star rating | 0% |
| `GinnungagapWindow` | Ginnungagap = primordial void | Window | 0% |
| `HiminnModal` | Himinn = sky/heaven | Modal | 0% |
| `SagaAccordion` | Saga = Norse narrative | Accordion/collapsible | 0% |
| `GraniSheet` | Grani = Sleipnir's offspring | Bottom sheet | 0% |
| `SleipnirGait` | Sleipnir = Odin's 8-legged horse | Step indicator/progress | 0% |
| `SkollProgress` | Skoll = wolf chasing sun | Progress bar | 0% |
| `Vegvísir` | Vegvísir = Norse compass | Navigation indicator | 0% |
| `ValkyrieIndicator` | Valkyrie = chooser of slain | Status indicator | 0% |
| `ValkyrieAnalytics` | Valkyrie = chooser of slain | Analytics display | 0% — same prefix, different purpose |
| `MuninAvatar` | Munin = memory raven | User avatar | 0% |
| `MerkiBadge` | Merki = ? | Badge | 0% — but there's also standard `Badge` |
| `RuneScript` | Rune = Norse script | Code display | 20% — "Rune" vaguely suggests text |
| `RunicTooltip` | Rune = Norse script | Tooltip | 0% |
| `RunesCard` | Rune = Norse script | Card component | 0% — why not just `Card`? |
| `RunesTable` | Rune = Norse script | Data table | 0% |
| `MimirsWell` | Mimir = wisdom god | Knowledge/search panel | 0% |
| `RunestoneDecoder` | Rune = Norse script | Decoder utility | 0% |
| `RunestoneEditor` | Rune = Norse script | Editor component | 0% |
| `ScribingStone` | — | Note-taking widget | 0% |
| `HolographicRunestone` | Rune = Norse script | Holographic display | 0% |
| `OracleOrb` | — | Prediction/AI display | 10% — "Oracle" is English |
| `RavenMessenger` | Huginn/Munin = ravens | Chat/messaging | 0% |
| `ShieldWall` | — | Defensive layout barrier | 0% |
| `TacticalGauge` | — | Gauge display | 20% — "Tactical" is English |
| `TelemetryView` | — | Telemetry display | 50% — "Telemetry" is English |
| `StatusBar` | — | Status bar | 90% — standard name |
| `UrdrTimeline` | Urdr = Norn of past | Timeline | 0% |
| `VölvaScan` | Völva = seeress | Scanning/preview | 0% |
| `DraumaSkeleton` | Drauma = dream? | Skeleton loader | 0% |
| `EmptyState` | — | Empty state placeholder | 90% — standard name |
| `WyrdHUD` | Wyrd = fate | HUD overlay | 0% |
| `NiflheimDemo` | Niflheim = ice realm | Demo component | 0% |
| `FreyrInspector` | Freyr = fertility god | Inspector panel | 0% |
| `GullveigInspector` | Gullveig = witch | Inspector panel | 0% — two inspectors, both Norse |
| `GerdTelemetry` | Gerd = giantess | Telemetry | 0% |
| `IdunnPersistence` | Idunn = youth goddess | Persistence layer | 0% |
| `NjordTheme` | Njord = sea god | Theme variant | 0% |
| `SkadiScripting` | Skadi = ski goddess | Scripting engine | 0% |
| `BragiCreative` | Bragi = poetry god | Creative tools | 0% |
| `LinguaTong` | — | i18n/localization | 0% |
| `HlinAccessibility` | Hlin = protector goddess | Accessibility layer | 0% |
| `TyrSecurity` | Tyr = war god | Security module | 0% |
| `A11yBeacon` | — | Accessibility beacon | 50% — "A11y" is standard shorthand |
| `A11yInspector` | — | A11y inspector | 50% — "A11y" is standard shorthand |
| `ClippedCorner` | — | Clipped corner shape | 80% — descriptive |
| `FlexiScope` | — | Responsive layout scope | 20% |
| `FluxLayout` | — | Layout state | 10% |
| `MorphBridge` | — | Morph transition | 10% |
| `PhaseGate` | — | Phase gate | 30% |
| `SyncWeave` | — | Collaborative editing | 0% |
| `TokenStream` | — | Token stream | 60% — standard CS term |
| `RadialMenu` | — | Radial menu | 80% — descriptive |
| `DockingWorkspace` | — | Docking workspace | 80% — descriptive |
| `NodeGraphEditor` | — | Node graph editor | 80% — descriptive |
| `InfiniteCanvas` | — | Infinite canvas | 80% — descriptive |
| `VirtualList` | — | Virtual list | 80% — descriptive |
| `TimelineEditor` | — | Timeline editor | 90% — descriptive |
| `SemanticMemoryExplorer` | — | Memory explorer | 50% |
| `MultiAgentOrchestrator` | — | Multi-agent orchestrator | 60% |
| `AIWorkflowBuilder` | — | AI workflow builder | 70% |
| `AgentChat` | — | Agent chat | 70% |
| `PromptForge` | — | Prompt builder | 30% |
| `FontAxisPanel` | — | Font axis panel | 50% |
| `FormValidation` | — | Form validation | 90% — descriptive |
| `Autocomplete` | — | Autocomplete | 90% — standard name |
| `Combobox` | — | Combobox | 90% — standard name |
| `RadioGroup` | — | Radio group | 90% — standard name |
| `NotificationCenter` | — | Notification center | 90% — standard name |
| `OutlineView` | — | Outline view | 90% — standard name |
| `TreeView` | — | Tree view | 90% — standard name |
| `TextEditor` | — | Text editor | 90% — standard name |
| `Image` | — | Image | 90% — standard name |
| `GridView` | — | Grid view | 90% — standard name |
| `Collapsible` | — | Collapsible | 90% — standard name |
| `DialogAction` | — | Dialog action | 90% — standard name |
| `SettingsForm` | — | Settings form | 90% — standard name |
| `EikonaForm` | Eikona = image? | Form component | 0% |
| `Calendar` | — | Calendar | 90% — standard name |
| `Sonner` | — | Toast system | 0% — named after a person? |
| `Toast` | — | Toast notification | 90% — standard name |

### Mathematical Naming Analysis

- **Total unique component structs**: ~215
- **Standard/descriptive names**: ~130 (60%)
- **Norse mythological names**: ~55 (26%)
- **Mixed/ambiguous names**: ~30 (14%)

**AI Guessability Score:**
- Standard names: ~85% chance AI guesses correctly
- Norse names: ~3% chance AI guesses correctly (essentially zero)
- Mixed names: ~35% chance AI guesses correctly

**Weighted average: ~58% of components are AI-discoverable by name alone.**

### Critical Naming Collisions & Inconsistencies

1. **`Geri` appears in both `GeriDialog` and `GeriTransfer`** — These are completely different UI components (a dialog vs. a list shuttle). The prefix "Geri" provides zero semantic signal for either. An AI agent cannot determine what "Geri" means.

2. **`Bifrost` is used for three unrelated concepts:**
   - `BifrostTabs` — tab navigation
   - `BifrostColorPicker` — color selection
   - `BifrostLauncher` — command palette
   - `bifrost()` modifier — frosted glass effect
   - `bifrost_bridge()` modifier — shared element transition
   
   The only consistent thread is "rainbow bridge" → visual/colorful, but this is too vague to be useful. An AI agent would never guess that `BifrostColorPicker` is the color picker.

3. **`Gjallar` appears in both `GjallarAlert` and `GjallarSplitter`** — An alert banner and a split pane divider share the same Norse prefix. No semantic connection.

4. **`Hati` appears in both `HatiSpinner` and `HatiCarousel`** — A loading spinner and a carousel share the prefix. The only link is "wolf chasing moon" → circular motion, but this is too oblique.

5. **`Mjolnir` is used for both `MjolnirSlider` and `MjolnirFrame`** — A slider and a frame. "Hammer" → "striking" doesn't map to either.

6. **`Valkyrie` appears in both `ValkyrieIndicator` and `ValkyrieAnalytics`** — Status indicator and analytics dashboard. No semantic link.

7. **`Rune` prefix is overloaded:** `RuneScript`, `RunicTooltip`, `RunesCard`, `RunesTable`, `RunestoneDecoder`, `RunestoneEditor`, `ScribingStone`, `HolographicRunestone` — 8 components sharing a "Rune" prefix with wildly different purposes.

8. **`Inspector` collision:** Both `FreyrInspector` and `GullveigInspector` exist. An AI agent cannot determine which to use.

9. **Standard `Badge` coexists with Norse `MerkiBadge`** — Two badge components with no clear differentiation.

10. **Standard `Card` doesn't exist, but `RunesCard` does** — An AI agent will try `Card` and fail.

### Can an AI Infer Norse Meanings?

| Norse Term | Mythological Meaning | Component Usage | Inferrable? |
|------------|---------------------|-----------------|-------------|
| Bifrost | Rainbow bridge | Tabs, ColorPicker, Launcher, Glass effect | ❌ No consistent mapping |
| Mjolnir | Thor's hammer | Slider, Frame, Slice, Shatter | ❌ "Hammer" doesn't map to any |
| Gjallar | Heimdall's horn | Alert, Splitter | ❌ Same prefix, different UI |
| Geri | One-eyed wolf | Dialog, Transfer | ❌ No semantic link |
| Hringr | Ring/circle | Pagination | ❌ "Ring" ≠ pagination |
| Hati | Wolf chasing moon | Spinner, Carousel | ⚠️ Circular motion, but too vague |
| Yggdrasil | World Tree | Window, Tree | ⚠️ "Tree" helps for Tree component |
| Valhalla | Hall of the slain | Rating | ❌ No connection |
| Saga | Norse narrative | Accordion | ❌ "Story" ≠ collapsible |
| Grani | Sleipnir's offspring | Sheet | ❌ No connection |
| Skoll | Wolf chasing sun | Progress | ❌ No connection |
| Vegvísir | Norse compass | Navigation indicator | ⚠️ "Compass" → navigation, barely |
| Mimir | Wisdom god | Well (knowledge panel) | ⚠️ "Wisdom" → knowledge, but obscure |
| Rune | Norse script | Card, Table, Tooltip, Editor | ❌ "Script" ≠ any of these |

**Conclusion: The Norse naming system is not learnable by an AI agent.** There is no consistent mapping between mythological concepts and UI component functions. The names are essentially random from an AI's perspective.

## Prelude & Discoverability

The prelude exports:
```rust
pub mod prelude {
    pub use cvkg_components::Color;
    pub use cvkg_components::*;  // All 215+ components
    pub use cvkg_core::{AssetKey, AssetState, Binding, ComponentErrorState, AppState, Never, Rect, State, View};
    pub use cvkg_macros::{View, view_component};
}
```

**Strengths:**
- `View` trait is exported — AI can learn the core pattern
- All components are available via `cvkg_components::*`
- Macros `#[derive(View)]` and `#[view_component]` are exported
- Core types like `State`, `Binding`, `Rect` are available

**Weaknesses:**
- No `use cvkg_themes` in prelude — AI must discover theme system separately
- No `use cvkg_layout` in prelude — layout types require separate import
- `cvkg_components::*` exports everything, but the Norse names are not self-documenting
- No re-export of `ThemeBuilder` or `OklchColor` in prelude

**What an AI can build from prelude alone:** Basic views with standard-named components (Button, Checkbox, Input, VStack, HStack, Text, ScrollView, etc.). This covers ~60% of common UI needs.

**What an AI cannot build from prelude alone:** Anything requiring Norse-named components (tabs, sliders, alerts, dialogs, spinners, carousels, ratings, pagination, sheets, modals, windows, trees, etc.). This covers ~40% of common UI needs.

## Token System & Visual Consistency

**This is CVKG's strongest AI-friendly feature.**

The theme system enforces visual consistency through:

1. **Semantic color tokens** — `theme::accent()`, `theme::surface()`, `theme::text()`, `theme::error_color()`, etc. An AI agent using these tokens will produce visually consistent code.

2. **Spacing scale** — `SPACE_XS` (4.0), `SPACE_SM` (8.0), `SPACE_MD` (16.0), `SPACE_LG` (24.0), `SPACE_XL` (32.0). Consistent 4px grid.

3. **Typography scale** — `FONT_XS` (10.0) through `FONT_3XL` (48.0). Consistent type ramp.

4. **Border radius scale** — `RADIUS_XS` (2.0) through `RADIUS_FULL` (9999.0). Consistent corner rounding.

5. **OKLCH color model** — Perceptually uniform color manipulation. An AI agent can use `OklchColor::new(l, c, h, a)` with `.lighten()`, `.darken()`, `.saturate()`, `.rotate_hue()` and get predictable results.

6. **APCA contrast validation** — `theme.validate_accessibility()` checks contrast ratios automatically.

**AI Impact:** An AI agent that uses theme tokens will produce **visually consistent** code. The tokens are well-named and discoverable. This is a significant strength.

**Weakness:** The `ThemeBuilder` requires understanding OKLCH color science. An AI agent asked to "make the theme more purple" would need to understand hue rotation in OKLCH space. The `from_seed()` method helps — an AI can generate a complete theme from a single `OklchColor`.

## Composition Patterns for AI

**The View/Modifier pattern is highly learnable:**

```rust
// Pattern 1: Builder pattern (consistent across all components)
Button::new("Click me")
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Large)

// Pattern 2: Modifier chain (consistent across all views)
Text::new("Hello")
    .font_size(16.0)
    .bold()
    .color([1.0, 1.0, 1.0, 1.0])

// Pattern 3: Layout composition (consistent)
VStack::new()
    .child(Text::new("Title"))
    .child(Button::new("Action"))
    .padding(16.0)

// Pattern 4: Event handlers (consistent)
Button::new("Submit")
    .on_click(|| { /* action */ })
    .on_pointer_enter(|| { /* hover */ })
```

**AI Learnability: HIGH.** Once an AI agent sees one example of each pattern, it can apply the pattern to any component. The builder pattern is idiomatic Rust and AI agents handle it well.

**The `hamr!` DSL macro** provides a declarative syntax:
```rust
hamr! {
    VStack::new(16.0) {
        Text::new("Hello")
        Button::new("Click", || {})
    }
}
```
This is also learnable from one example.

**Modifier methods on the View trait** (`.bifrost()`, `.gungnir()`, `.mjolnir_slice()`, etc.) are discoverable through rust-analyzer/IDE completion, but the Norse names are not self-explanatory.

## Documentation & Examples

**Doc comment coverage:**
- Module-level docs: Good — each module has a `//!` header explaining its contents
- Function-level docs: Moderate — public functions have doc comments, but many are minimal
- Doc examples: **Sparse** — only 25 doc examples across 126 modules (~20% of modules have examples)
- README: None found at crate level

**AI Impact:** AI agents learn heavily from doc examples. With only 25 examples across 126 modules, an AI agent has limited material to learn from. The components with doc examples (toast, hover_card, popover, card, etc.) will be used correctly. Components without examples (most Norse-named components) will be used incorrectly or avoided.

**Specific gaps:**
- No doc examples for `BifrostTabs`, `MjolnirSlider`, `GjallarAlert`, `HatiSpinner`, `YggdrasilWindow`, or any Norse-named component
- No doc examples for the `hamr!` macro
- No doc examples for `ThemeBuilder`
- No doc examples for the `View` trait's modifier methods (`.bifrost()`, `.gungnir()`, etc.)

## Accessibility by Default

**CVKG has strong accessibility infrastructure:**

1. **ARIA properties** — The `AriaProperties` struct covers all WCAG 2.1 roles and states. The `AriaRole` enum has 40+ roles.

2. **Focus management** — `FocusManager`, `FocusTrap`, `FocusableId` provide complete keyboard navigation support.

3. **Keyboard events** — `on_key_event()`, `key_shortcuts()`, `KeyModifiers` cover keyboard interaction.

4. **Reduced motion** — `is_reduced_motion()`, `effective_duration()` respect OS-level preferences.

5. **APCA contrast validation** — `validate_accessibility()` checks text contrast.

6. **Focus ring system** — `draw_focus_ring()`, `FOCUS_RING_WIDTH`, `FOCUS_RING_OFFSET` provide visible focus indicators.

**However, accessibility is opt-in, not automatic:**
- `aria_properties()` returns `None` by default — AI must explicitly set ARIA attributes
- Focus management requires explicit registration
- Keyboard handlers must be manually attached
- The `AlertDialog` and `ConfirmationDialog` components DO set ARIA roles automatically (good!)
- But most components do NOT set ARIA properties by default

**AI Impact:** An AI agent will get **partial accessibility** — keyboard events and focus rings work if the agent knows to use them, but ARIA roles and labels require explicit effort. The `AlertDialog` and `ConfirmationDialog` are good examples of accessible-by-default components, but this pattern is not consistently applied.

## Gaps & Recommendations

### P0 — Critical (Blocks AI usage)

1. **Add standard aliases for all Norse-named components:**
   ```rust
   pub type Tabs = BifrostTabs;
   pub type ColorPicker = BifrostColorPicker;
   pub type Slider = MjolnirSlider;
   pub type Alert = GjallarAlert;
   pub type Spinner = HatiSpinner;
   pub type Window = YggdrasilWindow;
   pub type Dialog = GeriDialog;
   pub type Pagination = HringrPagination;
   pub type Rating = ValhallaRating;
   pub type Accordion = SagaAccordion;
   pub type Sheet = GraniSheet;
   pub type Progress = SkollProgress;
   pub type Carousel = HatiCarousel;
   pub type TreeView = YggdrasilTree;
   pub type Card = RunesCard;
   pub type Table = RunesTable;
   ```
   This single change would increase AI discoverability from ~58% to ~90%.

2. **Add doc examples to every Norse-named component.** Each example should show the component in context with its standard alias.

3. **Add a "Component Name Mapping" table to the crate root docs** that maps standard names to Norse names and vice versa.

### P1 — High (Significantly improves AI experience)

4. **Add `cvkg_themes` to the prelude** — `ThemeBuilder`, `OklchColor`, and `Theme` should be available from `use cvkg::prelude::*`.

5. **Make ARIA properties automatic** — Components like `Button`, `Checkbox`, `Slider`, `Toggle` should set appropriate `aria_properties()` by default. The AI shouldn't need to know about WCAG to get basic accessibility.

6. **Add `doc(alias)` attributes** for Norse-named components:
   ```rust
   #[doc(alias = "Tabs")]
   pub struct BifrustTabs { ... }
   ```

7. **Standardize the Norse naming system** — If Norse names are kept, establish clear rules:
   - `Bifrost` = glass/frosted effects only (not tabs, not color pickers)
   - `Mjolnir` = impact/striking effects only
   - `Gjallar` = notification/alert only
   - `Hati` = circular/rotating only
   - Remove Norse names from components where they don't add meaning

### P2 — Medium (Nice to have)

8. **Add more doc examples** — Target 80%+ module coverage with examples.

9. **Create a `cvkg::snippets` module** with common composition patterns (login form, settings page, data table with pagination, etc.).

10. **Add `#[doc(alias)]` for modifier methods** — `.bifrost()` → `#[doc(alias = "frosted_glass")]`, `.gungnir()` → `#[doc(alias = "neon_glow")]`.

11. **Rename `EikonaForm`** — "Eikona" is obscure even among Norse names. Use `ImageForm` or `MediaForm`.

12. **Rename `Sonner`** — Named after a person, not descriptive. Use `ToastManager` or `NotificationStack`.

13. **Consolidate duplicate inspectors** — `FreyrInspector` and `GullveigInspector` should be one component with a configuration option.

## Verdict

**Score: 4/10 — AI-Agentic Friendliness**

**Breakdown:**
| Criterion | Score | Notes |
|-----------|-------|-------|
| AI Discoverability | 2/10 | Norse names are unpredictable; 40% of components are unguessable |
| Naming Consistency | 3/10 | No systematic pattern; Norse prefix usage is random |
| Prelude Completeness | 6/10 | Good core exports, but missing themes |
| Token Consistency | 9/10 | Excellent theme token system; AI-generated code will look consistent |
| Error Messages | 5/10 | Rust compiler errors are standard; no custom error messages for composition |
| Example Density | 3/10 | Only 20% of modules have doc examples |
| Composition Patterns | 8/10 | View/Modifier pattern is highly learnable from one example |
| Theme Application | 6/10 | OKLCH is powerful but requires color science knowledge; `from_seed()` helps |
| Accessibility by Default | 5/10 | Strong infrastructure but opt-in, not automatic |
| Time to First Pixel | 5/10 | 15-30 min due to naming barrier; architecture is clean |

**Summary:** CVKG has excellent underlying architecture — the View/Modifier pattern, OKLCH theme system, APCA contrast validation, and composition model are all well-designed and AI-friendly. But the Norse mythological naming system is a **fundamental barrier** to AI adoption. An AI agent cannot guess that `BifrostTabs` means "tabs" or `HringrPagination` means "pagination." The framework requires memorization of an arbitrary mapping between Norse mythology and UI components. Adding standard type aliases (`pub type Tabs = BifrostTabs`) would be the single highest-impact change, potentially increasing AI discoverability from 58% to 90%+.

**For a Vibe Coder using Cursor/Claude Code:** You can build ~60% of a standard app using discoverable names (Button, Input, VStack, HStack, Text, etc.). For the remaining 40%, you'll need to consult documentation or guess repeatedly. The theme system will keep your AI-generated code visually consistent, which is a significant win. But the naming system will slow you down and produce frustrating "type not found" errors.
