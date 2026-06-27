# CVKG Naming Map — Norse to English

**Audit recommendation #9.** This map lists every Norse-named public type found across the workspace
and its proposed English alias. Entries are grouped by crate and marked as **ALIASED** (alias exists)
or **MISSING** (alias does not exist yet).

## cvkg-components

| Norse name | Proposed English | Status | Notes |
|---|----|--------|-------|
| `SagaAccordion<V>` | `Accordion` | **ALIASED** | `pub type Accordion = SagaAccordion<AnyView>` |
| `GjallarAlert` | `Alert` | **ALIASED** | `pub type Alert = GjallarAlert` |
| `ValkyrieAnalytics` | `Analytics` | **ALIASED** | `pub type Analytics = ValkyrieAnalytics` |
| `MuninAvatar` | `Avatar` | **ALIASED** | `pub type Avatar = MuninAvatar` |
| `GraniBreadcrumb<V>` | `Breadcrumb` | **MISSING** | File: `advanced.rs:638` |
| `BifrostColorPicker` | `ColorPicker` | **ALIASED** | `pub type ColorPicker = BifrostColorPicker` |
| `MimirSpotlight` | `CommandPalette` | **ALIASED** | `pub type CommandPalette = MimirSpotlight` |
| `BragiCreative` | `CreativeTools` | **ALIASED** | `pub type CreativeTools = BragiCreative` |
| `RunestoneDecoder` | `Decoder` | **ALIASED** | `pub type Decoder = RunestoneDecoder` |
| `GeriDialog<V>` | `Dialog` | **ALIASED** | `pub type Dialog = GeriDialog<AnyView>` |
| `RunestoneEditor` | `Editor` | **MISSING** | File: `runestone_editor.rs:6` |
| `BifrostLauncher` | `Launcher` | **ALIASED** | `pub type BifrostLauncher = Launcher` (deprecated) |
| `HolographicRunestone` | `HolographicDisplay` | **ALIASED** | `pub type HolographicDisplay = HolographicRunestone` |
| `WyrdHUD` | `HUD` | **ALIASED** | `pub type HUD = WyrdHUD` |
| `ValkyrieIndicator` | `Indicator` | **ALIASED** | `pub type Indicator = ValkyrieIndicator` |
| `SagaItem<V>` | `AccordionItem` | **MISSING** | File: `container/disclosure.rs:359` |
| `FenrirCode` | `CodeBlock` | **MISSING** | File: `ai_components.rs:456` |
| `FenrirNode` | `AINode` | **MISSING** | File: `ai_components.rs:1032` |
| `RavenMessenger` | `Messenger` | **ALIASED** | `pub type Messenger = RavenMessenger` |
| `OracleOrb` | `Orb` | **ALIASED** | `pub type Orb = OracleOrb` |
| `HringrPagination` | `Pagination` | **ALIASED** | `pub type Pagination = HringrPagination` |
| `SkollProgress` | `Progress` | **ALIASED** | `pub type Progress = SkollProgress` |
| `PromptForge` | `PromptBuilder` | **ALIASED** | `pub type PromptBuilder = PromptForge` |
| `GraniRate` | `Rate` | **MISSING** | File: `ai_components.rs:576` |
| `ValhallaRating` | `Rating` | **ALIASED** | `pub type Rating = ValhallaRating` |
| `ScribingStone` | `ScribingNote` | **ALIASED** | `pub type ScribingNote = ScribingStone` |
| `GraniSheet<V>` | `Sheet` | **ALIASED** | `pub type Sheet = GraniSheet<AnyView>` |
| `DraumaSkeleton` | `Skeleton` | **MISSING** | File: `visual/decorators.rs:253` |
| `HatiSpinner` | `Spinner` | **ALIASED** | `pub type Spinner = HatiSpinner` |
| `GjallarSplitter<V1,V2>` | `Splitter` | **ALIASED** | `pub type Splitter = GjallarSplitter<AnyView, AnyView>` |
| `SleipnirGait` | `StepIndicator` | **ALIASED** | `pub type StepIndicator = SleipnirGait` |
| `HatiStream` | `Stream` | **MISSING** | File: `ai_components.rs:399` — name conflict with `std::stream::Stream`? |
| `HatiSwipe<V>` | `Swipe` | **MISSING** | File: `advanced.rs:1104` |
| `RunesTable<D>` | `Table` | **MISSING** | File: `data_grid.rs:18` |
| `BifrostTabs` | `Tabs` | **ALIASED** | `pub type Tabs = BifrostTabs` |
| `SkollPulse` | `Pulse` | **MISSING** | File: `ai_components.rs:1117` |
| `SkollTime<V>` | `Time` | **MISSING** | File: `advanced.rs:1085` |
| `SkollTimeline<V>` | `Timeline` | **MISSING** | File: `advanced.rs:255` — separate from `UrdrTimeline` |
| `UrdrTimeline` | `Timeline` | **ALIASED** | `pub type Timeline = UrdrTimeline` |
| `Sonner` | `Toast` | **MISSING** | File: `sonner.rs:131` — the manager |
| `SonnerToast` | `ToastMessage` | **MISSING** | File: `sonner.rs:64` — a single toast |
| `SonnerType` | `ToastStyle` | **MISSING** | File: `sonner.rs:14` — success/error/info |
| `SonnerPosition` | `ToastPosition` | **MISSING** | File: `sonner.rs:53` |
| `ValkyrieToolbar` | `Toolbar` | **MISSING** | File: `chrome/valkyrie_toolbar.rs:128` |
| `RunicTooltip<V>` | `Tooltip` | **ALIASED** | `pub type Tooltip = RunicTooltip<AnyView>` |
| `YggdrasilTree` | `TreeView` | **ALIASED** | `pub type TreeView = YggdrasilTree` |
| `MimirsWell` | `Well` | **ALIASED** | `pub type Well = MimirsWell` |
| `YggdrasilWindow<V>` | `Window` | **ALIASED** | `pub type Window = YggdrasilWindow<AnyView>` |
| `HatiCarousel` | `Carousel` | **MISSING** | File: `visual/carousel.rs:7` |
| `RunesCard<V>` | `Card` | **MISSING** | File: `card.rs:18` |

## cvkg-core

| Norse name | Proposed English | Status | Notes |
|---|----|--------|-------|
| `BifrostRegistry` | `Registry` | **MISSING** | File: `scene_graph.rs:13` |
| `KvasirId` | `Id` | **MISSING** | File: `identity.rs:22` — identity wrapper over `u64` |
| `YggdrasilKey` | `Key` | **MISSING** | File: `env_core.rs:17` — singleton unit struct |

## cvkg-anim

| Norse name | Proposed English | Status | Notes |
|---|----|--------|-------|
| `RunicParticle` | `Particle` | **MISSING** | File: `particles.rs:7` |
| `RunicEmitter` | `Emitter` | **MISSING** | File: `particles.rs:19` |

## cvkg-components (ornamental)

| Norse name | Proposed English | Status | Notes |
|---|----|--------|-------|
| `RunicStyle` | `Style` | **MISSING** | File: `ornamental/aetti_frame.rs:19` |

## Total

- **49** Norse-named public types found across the workspace
- **27** already have English aliases
- **22** still need English aliases (marked MISSING)

## Notes

- `Popover`, `RadioGroup`, `FormBinder`, `FormBinding` — already English, no Norse counterpart.
- `Skeleton` exists as a standalone type (`primitive.rs:663`) — the Norse one is `DraumaSkeleton`.
- `SkollTimeline<V>` and `UrdrTimeline` are two distinct types with different APIs.
- `HatiStream` may conflict with Rust's `std::stream::Stream` — `AIStream` is the proposed alternative.
