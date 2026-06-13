# CVKG Component Pool (cvkg-com-pool.md)

## Overview

This document catalogs UI component recommendations inspired by modern open-source libraries, filtered against CVKG's existing implementation inventory. All recommended components follow CVKG's core principles:

- **Composable**: Built on the `View` trait with modifier patterns
- **Easy to maintain**: Minimal state, single responsibility, clear API
- **Themeable**: Respects `cvkg-themes` design tokens
- **Easy to implement**: Leverages existing renderer primitives

---

## Existing CVKG Components (Already Implemented)

The following components exist in `cvkg-components/src/` and should NOT be re-implemented:

| Category | Components |
|----------|------------|
| **Primitives** | Text, Shape, Divider, Spacer, Canvas, Badge |
| **Interactive** | Button, Toggle, Slider, Input, Textarea, Checkbox, Picker, Select |
| **Containers** | VStack, HStack, ScrollView, NavigationStack, NavigationSplitView, LazyVStack, Grid, GjallarSplitter, GraniSheet, GinnugapWindow, HiminnModal, YggdrasilWindow |
| **AI/Chat** | OracleOrb, RavenMessenger, MultiAgentOrchestrator, AiWorkflowBuilder, AiComponents, PromptForge |
| **Floating UI** | Tooltip (RunicTooltip), Popover, Toast, ToastManager (Sonner), HoverCard, ContextMenu, AutoComplete, ComboBox |
| **Forms** | DatePicker, InputOTP, InputGroup, ButtonGroup, ToggleGroup, Breadcrumb, Kbd |
| **Layout/Navigation** | BifrostTabs, DockWorkspace, FileTree (YggdrasilTree), CommandPalette, MimirSpotlight, BifrostLauncher |
| **Visual/Data** | SkollProgress, HatiSpinner, DraumaSkeleton, MuninAvatar, MerkiBadge, ValkyrieAnalytics, UrdrTimeline, SleipnirGait, HatiCarousel |
| **HUD/Tactical** | Vegvisir, TacticalGauge, WyrdHUD, TelemetryView, PerfOverlay, A11yBeacon, A11yInspector |
| **Editors** | RunestoneEditor, ScribingStone, TextEditor, NodeGraphEditor |
| **Advanced** | HolographicRunestone, ShieldWall, RadialMenu, MjolnirFrame, ClippedCorner |

---

## Recommended Components for Implementation

### 1. 3D & Spatial Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **3DWrapper** | badtz-ui.com | Wraps content in a 3D perspective container with hover tilt effects and depth scaling | Implement as `SpatialWrapper` - uses GPU transform matrices for rotation, configurable tilt intensity, integrated with Bifrost glass effect |
| **ThreeDCard** | aceternity.com | Interactive card with 3D flip and perspective effects on hover | Create `Runes3DCard` with front/back content, CSS-like transform API via GPU shaders |
| **AnimatedGlobe** | aceternity.com, kibo-ui | Rotating 3D globe visualization for data displays | Implement `YggdrasilGlobe` with texture mapping, auto-rotation, marker points for agent locations |
| **CloudOrbit** | badtz-ui | Circular orbiting elements around a central point | `AsgardOrbit` - particle orbit system with configurable radius, speed, and count |

### 2. Animated & Interactive Patterns

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **AnimatedList** | magicui, badtz-ui | Staggered list item entrance animations | Add to existing `virtual_list.rs` - implement staggered reveal with spring physics |
| **Marquee** | aceternity, badtz-ui | Continuous horizontal/vertical scrolling content | `FreyrMarquee` - infinite scroll with pause-on-hover, direction control, speed variants |
| **InfiniteRibbon** | badtz-ui | Animated ribbon/ticker display | `RanRibbon` - wrapped overflow text with gradient masking |
| **RetroGrid** | magicui | Animated retro-style grid background | `GridBackground` - parametric grid with scanline animation, configurable density |
| **AuroraBackground** | aceternity | Soft aurora-like animated gradient background | `AuroraField` - multi-layer gradient shader with noise displacement |

### 3. Advanced Input & Selection

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **ChoiceBox** | kibo-ui | Checkbox/radio hybrid with visual selection cards | `ValkyrieChoice` - multi/single-select with icon and description support |
| **DropZone** | kibo-ui | Drag-and-drop file upload zone | Extend `DropVault` - add file type validation, preview thumbnails, progress states |
| **ImageCrop** | kibo-ui | Interactive image cropping interface | `SkadiCrop` - gesture-based crop rectangle, aspect ratio presets, rotation controls |
| **ColorPicker** | kibo-ui, magicui | HSV/HSL color selection wheel | Extend `BifrostColorPicker` - add preset palettes, alpha control, hex/RGB inputs |
| **MorphingInput** | uselayouts | Input field that morphs into different states | `MorphField` - smooth state transitions for search, filter, or command inputs |

### 4. Notification & Feedback

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **Announcement** | kibo-ui | Prominent banner for important notifications | `MjolnirBanner` - dismissible banner with action button, urgency levels |
| **StatusIndicator** | kibo-ui | Real-time status visualization | Extend `AvatarStatus` - add pulse animation, status history, tooltip details |
| **SocialProofAvatars** | badtz-ui | Stack of user avatars showing collective activity | `HuginGroup` - avatar stack with overlap, hover expansion, tooltip member list |

### 5. Media Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **VideoPlayer** | kibo-ui | Embedded video with custom controls | `VidarPlayer` - glassmorphic controls, fullscreen support, loading states |
| **ImageTrail** | badtz-ui | Sequential image reveal on scroll/hover | `ImageCascade` - staggered reveal with fade and scale transitions |
| **ImageSplit** | badtz-ui | Split-view image comparison slider | `RuneCompare` - draggable divider with before/after labels |

### 6. Layout & Structure

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **BentoGrid** | aceternity, magicui | Asymmetric grid layout for dashboard cards | Extend `Grid` - add span support, auto-fit columns, gap variants |
| **DialogStack** | kibo-ui | Stack of modal dialogs with history | Extend `GraniSheet` - add dialog history navigation, swipe-to-dismiss from edges |
| **DiscreteTab** | uselayouts | Minimal tab navigation | Add to `BifrostTabs` - compact style variant, icon-only mode |
| **VerticalTabs** | uselayouts | Side-by-side vertical tab navigation | `RanVerticalTabs` - sidebar tab strip with indicator line |

### 7. Calendar & Date Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **MiniCalendar** | kibo-ui | Compact month view calendar | Extend `Calendar` - mini mode with day indicators, click callbacks |
| **DayPicker** | uselayouts | Week-focused date picker | Add inline week view mode to `DatePicker` |

### 8. Typography & Text Effects

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **TextReveal** | magicui | Line-by-line text reveal animation | `RunesReveal` - animated text appearance with typewriter or fade effects |
| **KineticText** | magicui | Continuously animated text movement | `TextFlow` - marquee-style single-line text with gradient mask |
| **WordRotate** | magicui | Rotating word replacement animation | `RuneCycle` - timed word transitions with cross-fade |
| **TypewriterEffect** | aceternity | Character-by-character text typing | Extend `RunicText` - add typing animation with cursor blink |
| **HyperText** | magicui | Animated text with gradient on hover | `EitrText` - shimmer gradient on hover, smooth transitions |

### 9. Progress & Data Visualization

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **NumberTicker** | magicui | Animated counting to target number | `TalonCounter` - smooth interpolation with easing, prefix/suffix support |
| **Ticker** | kibo-ui | Continuous scrolling ticker display | Extend `FreyrMarquee` - add reverse direction, hover pause |
| **Gantt** | kibo-ui | Project timeline visualization | `LokiTimeline` - horizontal bar timeline with dependencies, zoom levels |

### 10. Overlay & Modal Patterns

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **CursorCard** | badtz-ui | Card that follows pointer movement | `PointerCard` - follow offset with smooth lag, context-aware positioning |
| **ExpandableGallery** | uselayouts | Grid that expands selected items | `ExpandableGrid` - modal transition with backdrop, thumbnail navigation |
| **MultiStepForm** | uselayouts | Wizard-style form with progress indicator | Add to `advanced_forms.rs` - step management, validation, progress dots |

### 11. Cult-UI Hero & Animated Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **HeroColorPanels** | cult-ui.com | Hero section with color-shifting gradient panels | `HeroRune` - full-width gradient hero with animated color transitions, content overlay, CTA placement |
| **AnimatedBadge** | cult-ui | Badge with pulse/glow animation on status changes | Extend `MerkiBadge` - add entry/exit animation, status transition effects |
| **AnimatedButton** | cult-ui | Button with smooth state transitions and micro-interactions | Extend `Button` - add hover/active state animations with scale/tilt effects |
| **AnimatedCard** | cult-ui | Card with reveal and hover animations | Extend `RunesCard` - add staggered content reveal, hover lift effects |
| **AnimatedDropzone** | cult-ui | Drop zone with visual feedback during drag operations | Extend `DropVault` - add hover state highlight, progress visualization |
| **AnimatedField** | cult-ui | Form field with floating labels and validation animation | Extend `Input` - add floating label transition, error shake animation |
| **AnimatedSearch** | cult-ui | Search input with expanding animation and results dropdown | Extend `AutoComplete` - add smooth width expansion, quick-search shortcuts |
| **AnimatedTabs** | cult-ui | Tab navigation with smooth indicator transitions | Extend `BifrostTabs` - add sliding indicator animation, tab reorder gestures |
| **AnimatedToast** | cult-ui | Toast notifications with enter/exit animations | Extend `Sonner` - add slide/fade animations, stacking behavior |
| **AIBlobWarp** | cult-ui | AI-themed blob/warp background effect for hero sections | `SurtrWarp` - SVG filter-based liquid blob background with organic movement |
| **AnalyticsChart** | cult-ui | Interactive analytics dashboard charts | Extend `ValkyrieAnalytics` - add chart type switching, tooltip values, loading skeletons |

### 12. MUI X & Joy UI Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|---------------|
| **MUI Charts** | mui.com/x | Data visualization with zoom, pan, and interaction | Extend `ValkyrieAnalytics` - add zoom gestures, data point tooltips, export capabilities |
| **MUI Chat** | mui.com/x | Enterprise-grade chat interface with attachments | Extend `RavenMessenger` - add attachment previews, typing indicators, message actions |
| **MUI Scheduler** | mui.com/x | Calendar timeline for event planning | Extend `Calendar` - add event drag-drop, resource views, timezone support |
| **MUI TreeView** | mui.com/x | Hierarchical data display with expand/collapse | Extend `YggdrasilTree` - add multi-select, checkbox integration, lazy loading nodes |
| **Joy Sheet** | mui.com/joy-ui | Bottom sheet with swipe gestures | Extend `GraniSheet` - add swipe-dismiss, snap points, detent animations |
| **Joy Chip** | mui.com/joy-ui | Compact selection/removable tags | Extend `MerkiBadge` - add removable action, multi-select mode, avatar integration |
| **MUI Stepper** | mui.com/material-ui | Wizard progress indicator with validation | Add to `advanced_forms.rs` - vertical/horizontal modes, step icons, error states |
| **MUI AspectRatio** | mui.com/joy-ui | Maintains content aspect ratio | `AspectRune` - responsive container with predefined ratios (16/9, 1/1, 4/3) |
| **MUI Snackbar** | mui.com/material-ui | Temporary notification at screen edge | Extend `Sonner` - add action button, queue stacking, position variants |
| **MUI Modal** | mui.com/material-ui | Accessible modal dialog with backdrop | Extend `HiminnModal` - add focus trap, escape handling, size variants |

### 13. Material Design 3 Components

| Component | Source | Intent | Recommendation |
|-----------|--------|--------|----------------|
| **BottomNavigation** | m3.material.io | Navigation bar for mobile/focused contexts | `RanNavigation` - add to `Navigation` - icon + label bottom bar with shift animation |
| **NavigationBar** | m3.material.io | Side navigation rail for tablet/desktop | Extend `NavigationSplitView` - compact icon-only mode, badge indicators |
| **NavigationDrawer** | m3.material.io | Modal/side sheet navigation | Extend `GraniSheet` - add navigation item binding, scrim treatment |
| **Cards** | m3.material.io | Elevated content containers with states | Extend `RunesCard` - add pressed state elevation, hover lift, media variants |
| **Chips** | m3.material.io | Compact selection/input chips | Extend `MerkiBadge` - add input variant (removable), filter variant, avatar chip |
| **DatePicker** | m3.material.io | Calendar date selection with range | Add to `DatePicker` - range mode, week numbers, custom date formatting |
| **Search** | m3.material.io | Search field with voice/mic support | Extend `AutoComplete` - add voice input capability, clear button, recent queries |
| **ProgressIndicator** | m3.material.io | Determinate/indeterminate progress | Extend `SkollProgress` - add circular determinate, linear variants, stop indicator |
| **Snackbar** | m3.material.io | Brief message with optional action | Extend `Sonner` - add action button, fixed positioning, queue management |
| **Switch** | m3.material.io | On/off toggle with thumb animation | Extend `Toggle` - add thumb slide animation, icon states, disabled styling |
| **TextField** | m3.material.io | Text input with supporting text | Extend `Input` - add supporting/help text area, error text, character counter |
| **Menus** | m3.material.io | Dropdown menu with icon support | Extend `DropdownMenu` - add icon placement, checkbox/radio items, submenu support |
| **List** | m3.material.io | Repeated items with leading/trailing widgets | Extend `FileTree` - add leading/trailing templates, divider variants, dense mode |
| **Slider** | m3.material.io | Range selection with tick marks | Extend `MjolnirSlider` - add tick marks, value labels, range selection |

---

## Component Enhancement Recommendations

### Existing Components to Improve

| Component | Current State | Improvement Opportunity |
|-----------|--------------|----------------------|
| **Avatar (MuninAvatar)** | Basic avatar with status | Add `AvatarStack` grouping, `AvatarGroup` overflow handling, status pulse animation |
| **Progress (SkollProgress)** | Linear and circular variants | Add determinate/indeterminate states, segmented/track variants, gradient fills |
| **Toast (Sonner)** | Basic positioning | Add swipe-to-dismiss gestures, action buttons, queue management, animated entry/exit |
| **Calendar** | Full calendar | Add mini mode, range selection, event dots, dark theme variants |
| **Card (RunesCard)** | Basic container | Add hover lift effect, glass variant, media overlay support, staggered reveal animation |
| **Popover** | Basic anchor | Add arrow pointer, auto-alignment, nested popovers, smooth transitions |
| **Tooltip** | Basic on-hover | Add interactive mode (hoverable tooltip), positioning animation, rich content support |
| **NavigationSplitView** | Sidebar layout | Add overlay mode on mobile, snap-to-edge, persistent collapse state |
| **Button** | Multiple variants | Add animated variants (scale, tilt, ripple), micro-interaction feedback |
| **AutoComplete/Input** | Basic dropdown | Add floating labels, clear button animation, validation shake effects |

### Cult-UI Specific Enhancements

| Component | Current State | Improvement Opportunity |
|-----------|--------------|----------------------|
| **MjolnirFrame** | Decorative runic frame | Add gradient color animation support, border pulse effects, corner accent customization |
| **HatiSpinner** | Basic loading indicator | Add branded spinners matching Cult's aesthetic - blob warp, neural, kinetic variants |
| **OracleOrb** | AI state indicator | Add pulsing animation on thinking state, particle effects, audio-reactive variants |

### MUI-Specific Enhancements

| Component | Current State | Improvement Opportunity |
|-----------|--------------|----------------------|
| **RunesCard** | Basic container | Add `Chip` integration (removable tags), elevation levels, hover lift with shadow spread, pressed state |
| **GraniSheet** | Modal bottom sheet | Add swipe gestures for dismiss, snap points (top/half/bottom), detent animations, NavigationDrawer mode |
| **Calendar** | Full calendar | Add `Scheduler` event integration, drag-to-create events, range selection, week numbers |
| **YggdrasilTree** | File tree display | Add `TreeView` checkbox mode, multi-select, lazy loading, List item templates |
| **ValkyrieAnalytics** | Chart visualization | Add `Charts` zoom/pan gestures, data point markers, export to image capability |
| **Sonner** | Toast notification | Add `Snackbar` action button support, queue management, swipe-dismiss gestures |
| **Toggle** | Basic on/off | Add thumb slide animation, icon states, Switch styling variants |
| **Input** | Basic text field | Add supporting text, error text, character counter, TextField variants |
| **MjolnirSlider** | Basic slider | Add tick marks, value labels, range selection, Slider variants |
| **NavigationSplitView** | Sidebar layout | Add NavigationBar (rail) mode, badge indicators, compact icon mode |

---

## Implementation Priority Matrix

| Priority | Components | Rationale |
|----------|------------|-----------|
| **P0 - High Value, Low Effort** | AnimatedList, Marquee, AuroraBackground, TextReveal, NumberTicker, AnimatedBadge, AnimatedTabs, HeroColorPanels, AspectRatio, Snackbar, Switch, Chips | Pure visual enhancements, minimal state, strong UX impact |
| **P1 - Core Missing** | 3DWrapper, ChoiceBox, DropZone, BentoGrid, DialogStack, AnimatedSearch, AnimatedField, Sheet, Stepper, Search, TextField | Fill gaps in input patterns and layout primitives |
| **P2 - AI-Specific** | CloudOrbit, YggdrasilGlobe, SocialProofAvatars, AIBlobWarp, MUI Chat, TreeView, NavigationBar, BottomNavigation | Align with CVKG's agentic/tactical aesthetic |
| **P3 - Advanced** | VideoPlayer, ImageCrop, Gantt, MultiStepForm, AnalyticsChart, Scheduler, Charts, NavigationDrawer, Menus | Higher complexity, specialized use cases |

## Animation Enhancement Opportunities

The Cult-UI library demonstrates a pattern of animating existing components rather than creating new primitives. CVKG should follow this approach:

1. **Shared Animation System** - Centralize spring physics and timing in `cvkg-anim`
2. **Staggered Reveal** - Add to any container with children (VStack, Grid, BentoGrid)
3. **Shared Layout** - Smooth transitions when components rearrange
4. **Gestures** - Tap, hover, drag animations with haptic feedback

---

## Design Tokens Alignment

All new components should use the existing theme tokens:

```rust
// From cvkg-themes
theme::surface()           // Background surfaces
theme::surface_elevated() // Elevated panels
theme::accent()            // Primary action color
theme::text()              // Primary text
theme::text_muted()        // Secondary text
theme::border_strong()     // Strong borders
theme::success()           // Success state
theme::warning()           // Warning state
theme::error_color()       // Error state

// From lib.rs constants
FONT_XS, FONT_SM, FONT_BASE, FONT_MD, FONT_LG, FONT_XL
SPACE_XS, SPACE_SM, SPACE_MD, SPACE_LG
RADIUS_XS, RADIUS_SM, RADIUS_MD, RADIUS_LG
```

---

## Composability Patterns

### Recommended Modifier Extensions

```rust
// ViewExt additions to consider
trait ViewExt {
    fn floating<V: View>(self, position: FloatingPosition) -> FloatingView<Self, V>;
    fn reveal(self, delay_ms: u32) -> RevealModifier<Self>;
    fn orbit(self, radius: f32, speed: f32) -> OrbitModifier<Self>;
    fn tilt(self, max_tilt: f32) -> TiltModifier<Self>;
}
```

### State Management Integration

All interactive components should follow the existing pattern:
- Event handlers via `renderer.register_handler()`
- State via `cvkg_core::load_system_state()` / `update_system_state()`
- Re-render triggered automatically by state changes

---

## Notes

- Components marked with Norse mythology naming (`Runes`, `Yggdrasil`, `Valkyrie`, etc.) maintain CVKG's thematic consistency
- 3D components should leverage `cvkg-render-gpu` shader capabilities for optimal performance
- Animation components should integrate with `cvkg-anim` for timing consistency
- Accessibility should follow existing `A11y*` patterns with proper ARIA roles

---

*Generated: 2026-06-12*
*Sources: agent-elements.21st.dev, ui.aceternity.com, assistant-ui, stackzero-labs/ui, badtz-ui.com, magicui.design, kibo-ui.com, tailark.com, uselayouts.com, cult-ui.com, mui.com/material-ui, mui.com/joy-ui, mui.com/x, m3.material.io*