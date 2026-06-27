# CVKG Migration Audit: shadcn/MUI → CVKG

**Audit Date:** June 22, 2026  
**Auditor:** Product Design Review  
**Context:** Evaluating migration from React/shadcn/MUI to CVKG Rust UI framework

---

## Executive Summary

CVKG is a mature Rust-based UI framework that provides **strong parity** with shadcn/MUI components. The framework offers sophisticated theming (OKLCH color model), Taffy-backed layout (flexbox/grid), and spring-physics animations. Migration is viable but requires significant paradigm shift from React/TypeScript to Rust.

---

## 1. Component Parity Assessment

### shadcn/ui Components → CVKG Equivalents

| shadcn/ui | CVKG Component | Status |
|-----------|-----------------|--------|
| **Button** | `cvkg_components::Button` | ✅ Complete |
| **Input** | `cvkg_components::Input` | ✅ Complete |
| **Textarea** | `cvkg_components::Textarea` | ✅ Complete |
| **Select** | `cvkg_components::Select` | ✅ Complete |
| **Dialog** | `container::GeriDialog` | ✅ Complete |
| **Sheet** (Drawer) | `container::GraniSheet` | ✅ Complete |
| **DropdownMenu** | `dropdown_menu::DropdownMenu` | ✅ Complete |
| **Popover** | `popover::Popover` | ✅ Complete |
| **Card** | `card::RunesCard` | ✅ Complete |
| **Table** | `data_grid::RunesTable`, `virtual_table` | ✅ Complete |
| **Checkbox** | `interactive::Checkbox` | ✅ Complete |
| **RadioGroup** | `radio_group` | ✅ Complete |
| **Toggle** | `interactive::Toggle` | ✅ Complete |
| **Slider** | `interactive::Slider`, `mjolnir_slider` | ✅ Complete |
| **Tabs** | `bifrost_tabs::BifrostTabs` | ✅ Complete |
| **Accordion** | `container::SagaAccordion` | ✅ Complete |
| **Avatar** | `visual::MuninAvatar` | ✅ Complete |
| **Badge** | `visual::MerkiBadge` | ✅ Complete |
| **Toast** | `toast` | ✅ Complete |
| **AlertDialog** | `hud::GjallarAlert` | ✅ Complete |
| **Progress** | `visual::SkollProgress` | ✅ Complete |
| **Skeleton** | `visual::DraumaSkeleton` | ✅ Complete |

### Extended Components (Not in shadcn)

CVKG includes additional components not found in shadcn/MUI:
- **Charts:** 14+ chart types (BarChart, LineChart, PieChart, CandlestickChart, etc.)
- **Advanced Forms:** Calendar, DatePicker, DateTimePicker
- **Navigation:** TreeView, FileTree, Kanban, Gantt
- **AI/Dashboard:** Agent chat, workflow builders, telemetry views
- **Effects:** TextAnimate, Typewriter, Ripple, Shimmer

---

## 2. Theming Parity

### CSS Variables → Design Tokens

| shadcn/MUI | CVKG Approach |
|------------|---------------|
| CSS custom properties | Rust constants (`FONT_MD`, `SPACE_XL`, etc.) |
| `theme CSS variables` | `cvkg_themes::Theme` struct |
| Tailwind config | `cvkg_themes::ThemeBuilder` |
| `--primary`, `--muted`, etc. | `OklchColor` + `SemanticColors` |
| CSS `@media (prefers-color-scheme)` | `Theme::dark()`, `Theme::light()`, `Theme::toggle()` |

### Dark Mode Support

✅ **Full parity** - CVKG themes support dark/light switching:
- `Theme::dark()` / `Theme::light()` - preset themes
- `Theme::toggle()` - switch preserving custom scales
- `Theme::from_seed(OklchColor)` - derive palette from brand color

### Advanced Theming Features

CVKG exceeds shadcn/MUI theming:
- **OKLCH color model** - perceptually uniform, better than RGB/HSL
- **Glassmorphism** - `GlassMaterial` for frosted-glass surfaces
- **Accessibility validation** - APCA contrast checking built-in
- **Density modes** - Compact/Default/Spacious (0.75x/1.0x/1.25x)
- **State colors** - `StateColors::from_base()` auto-generates hover/active/focus/disabled states

### Design Tokens (cvkg-components)

```
Typography: FONT_XS (10px) → FONT_3XL (48px)
Spacing:    SPACE_XS (4px) → SPACE_XL (32px)  
Radius:     RADIUS_XS (2px) → RADIUS_FULL (9999px)
Line Height: LINE_HEIGHT_XS (1.4) → LINE_HEIGHT_3XL (1.2)
```

---

## 3. Layout System Comparison

### Flexbox/Grid

| shadcn/MUI | CVKG |
|------------|------|
| CSS Flexbox | `cvkg_layout::HStack`, `VStack`, `ZStack` |
| CSS Grid | `cvkg_layout::Grid` with `GridTrack` |
| CSS gap | spacing parameter on Stack/Grid |
| CSS justify-* | `cvkg_core::Distribution` enum |
| CSS align-* | `cvkg_core::Alignment` enum |
| CSS flex-grow | `cvkg_layout::Flex` |
| CSS flex-shrink | Built-in via Taffy |

### Layout Features

✅ **Full Flexbox/Grid parity** - Taffy 0.6 backend:
- Mixed track sizing: `Fixed`, `Flex`, `Auto`, `MinContent`, `MaxContent`
- Progressive/incremental layout for deep trees
- Focus order computation for keyboard navigation
- Spatial index for O(log n) hit-testing
- Safe area / aspect ratio primitives
- Touch modality (44pt minimum tap targets)

---

## 4. Animation Patterns

### Framer Motion → cvkg-anim

| Framer Motion | CVKG |
|---------------|------|
| `animate()` | `SpringSolver` with RK4 integration |
| `useSpring()` | `SpringSolver::new()` |
| `transition={{ type: "spring", ... }}` | `SpringParams::snappy()`, `fluid()`, `heavy()`, `bouncy()` |
| `layoutId` animations | `AnimationEngine` + spring-driven rect transitions |
| `AnimatePresence` | `Animation::Sequence` / `Parallel` |
| `staggerChildren` | `Animation::Stagger` |
| `layoutId` shared element | Spring interpolation via `AnimationValue::lerp` |
| `easeInOut` | `Easing::EaseInOut.evaluate()` |
| Scroll-linked animations | `ProgressDriver::Scalar` |
| Gesture-driven animations | `RubberBand` (elastic over-scroll clamp) |

### cvkg-anim Capabilities

**Spring Physics (RK4):**
- Stiffness, damping, mass parameters
- Four presets: snappy, fluid, heavy, bouncy
- Accessibility: `set_reduce_motion(true)` snaps instantly

**Advanced Effects:**
- `MjolnirShatter` / `MjolnirSlice` - physical break effects
- `Momentum` - inertial scroll decay
- `Skeletal` - bone animation
- `Particles` - particle systems
- `Morph` - morph-target animation

**Limitations:**
- No animation blending/cross-fade (compose via `Animation::Parallel`)
- No CSS keyframe equivalent (use `Animation::Linear` sequences)
- `ProgressDriver::Scalar` ignored by physics animations

---

## 5. Migration Complexity Estimate

### Low Effort (Direct Mapping)
- Button, Input, Textarea, Checkbox, Toggle, Slider
- Typography tokens
- Basic spacing/radius

### Medium Effort (API Adaptation)
- Select, Dropdown, Popover - different selection model
- Dialog, Sheet - Rust closure-based callbacks
- Card, Container layouts - builder pattern differs

### High Effort (Architecture Change)
- **State management** - React hooks → Rust ownership model
- **Component composition** - JSX children → Rust builder chaining
- **Context** - React Context → `cvkg_vdom::use_state` with hash-based keys
- **Theme switching** - CSS variables → `Theme` struct mutation
- **Animations** - declarative → imperative spring physics

### Complexity Breakdown

| Aspect | Effort | Notes |
|--------|--------|-------|
| Component API | Medium | Builder pattern, closures vs props |
| Theming | Low | More sophisticated, better outcomes |
| Layout | Low | Nearly identical flexbox/grid model |
| Animations | Medium-High | Physics-based requires mindset shift |
| State Management | High | No React hooks equivalent |
| Type Safety | High (benefit) | Rust's ownership model |
| Bundle Size | High (benefit) | Native binary vs JS bundle |

---

## 6. Key Architectural Differences

### 1. Language Paradigm
- **shadcn/MUI:** TypeScript/React - JSX, hooks, virtual DOM
- **CVKG:** Rust - structs, traits, ownership, no GC

### 2. Component Definition
```typescript
// shadcn
function Button({ children, onClick }) {
  return <button onClick={onClick}>{children}</button>
}
```

```rust
// CVKG
pub struct Button {
    label: String,
    on_click: Arc<dyn Fn() + Send + Sync>,
    variant: ButtonVariant,
}
```

### 3. Theme Application
```typescript
// shadcn - CSS variables
<div className="bg-primary text-primary-foreground" />
```

```rust
// CVKG - Direct color values
renderer.fill_rect(rect, theme::accent());
```

### 4. Animation Trigger
```typescript
// Framer Motion
<motion.div animate={{ scale: pressed ? 0.95 : 1 }} />
```

```rust
// CVKG - Spring solver per component
let mut solver = SpringSolver::new(params, target, current);
let scale = 1.0 - (0.03 * solver.tick(dt));
```

---

## 7. Skills & Resources Available

- **minimalist-ui** - Design system principles
- **ux-writing** - Component naming and documentation
- **design-qa** - Quality assurance workflows
- **design-review** - Review processes

---

## 8. Recommendations

### For Migration

1. **Start with static components** - Button, Input, Card, Typography
2. **Adopt design tokens early** - Replace all magic numbers
3. **Embrace spring physics** - It's more powerful than Framer Motion once learned
4. **Leverage OKLCH** - Better accessibility outcomes than RGB
5. **Use English aliases** - `Alert` = `GjallarAlert`, `Dialog` = `GeriDialog`

### For Team

- **Training needed:** Rust ownership, Spring physics tuning, Taffy layout
- **Tooling:** Ensure IDE has Rust analyzer working
- **Testing:** Component tests require runtime rendering (different from Jest)

---

## 9. Verdict

| Criteria | Assessment |
|----------|------------|
| Component Coverage | ✅ 95%+ parity |
| Theming Capability | ✅ Superior (OKLCH, glassmorphism) |
| Layout Parity | ✅ Full flexbox/grid |
| Animation Power | ✅ More capable (physics-based) |
| Migration Effort | ⚠️ Medium-High |
| **Overall Viability** | ✅ **Recommended** |

**CVKG is a production-ready alternative** that exceeds shadcn/MUI in theming sophistication and animation capabilities. The primary migration cost is the Rust learning curve and paradigm shift from React's declarative model to Rust's ownership model.

---

*Audit conducted using cvkg-components v*, cvkg-themes, cvkg-layout, and cvkg-anim source documentation.