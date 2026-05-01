# CVKG Components Implementation Plan

**Generated:** 2026-04-30  
**Purpose:** Implementation roadmap for shadcn/ui-compatible components in cvkg-components

---

## Table of Contents

1. [Tier 1: High Priority Components](#tier-1-high-priority)
2. [Tier 2: Medium Priority Components](#tier-2-medium-priority)
3. [Tier 3: Low Priority Components](#tier-3-low-priority)
4. [Design Principles](#design-principles)
5. [Testing Strategy](#testing-strategy)

---

## Tier 1: High Priority

### 1.1 Card Component

**File:** `cvkg-components/src/card.rs`

```rust
/// Container component with header, content, and footer sections
pub struct Card<V> {
    header: Option<V>,
    content: Option<V>,
    footer: Option<V>,
}

impl<V: View> Card<V> {
    pub fn new() -> Self { ... }
    pub fn header(mut self, header: V) -> Self { ... }
    pub fn content(mut self, content: V) -> Self { ... }
    pub fn footer(mut self, footer: V) -> Self { ... }
}
```

**API:**
- `Card::new()` - Create empty card
- `header(content)` - Set header view
- `content(view)` - Set main content
- `footer(view)` - Set footer view

**Styling:** Glassmorphic with subtle elevation

---

### 1.2 Input Component

**File:** `cvkg-components/src/interactive.rs` (extend existing)

```rust
pub struct Input {
    placeholder: String,
    value: String,
    on_change: Arc<dyn Fn(String) + Send + Sync>,
    is_focused: bool,
}

impl Input {
    pub fn new(placeholder: impl Into<String>) -> Self { ... }
    pub fn value(mut self, value: impl Into<String>) -> Self { ... }
    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self { ... }
}
```

**Features:**
- Placeholder text
- Focus state styling
- Text selection support

---

### 1.3 Textarea Component

**File:** `cvkg-components/src/interactive.rs` (extend existing)

```rust
pub struct Textarea {
    placeholder: String,
    value: String,
    rows: usize,
    on_change: Arc<dyn Fn(String) + Send + Sync>,
}
```

---

### 1.4 Checkbox Component

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Checkbox {
    is_checked: bool,
    on_change: Arc<dyn Fn(bool) + Send + Sync>,
    label: Option<String>,
}

impl Checkbox {
    pub fn new(is_checked: bool) -> Self { ... }
    pub fn label(mut self, label: impl Into<String>) -> Self { ... }
}
```

**Visual:** Square with checkmark when checked

---

### 1.5 Radio Group

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct RadioGroup<V> {
    options: Vec<RadioOption<V>>,
    selected_index: usize,
    on_change: Arc<dyn Fn(usize) + Send + Sync>,
}

struct RadioOption<V> {
    label: String,
    value: V,
}
```

---

### 1.6 Dialog Component

**File:** `cvkg-components/src/container.rs` (extend existing Sheet)

```rust
pub struct Dialog<V> {
    is_presented: bool,
    title: Option<String>,
    content: V,
    actions: Vec<DialogAction>,
}

struct DialogAction {
    label: String,
    style: DialogActionStyle,
    on_click: Arc<dyn Fn() + Send + Sync>,
}
```

---

### 1.7 Alert Dialog

**File:** `cvkg-components/src/container.rs`

```rust
pub struct AlertDialog {
    is_presented: bool,
    title: String,
    description: String,
    on_confirm: Arc<dyn Fn() + Send + Sync>,
    on_cancel: Arc<dyn Fn() + Send + Sync>,
}
```

---

### 1.8 Tabs Component

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Tabs<V> {
    tabs: Vec<TabItem<V>>,
    selected_index: usize,
}

struct TabItem<V> {
    label: String,
    content: V,
    icon: Option<String>,
}
```

---

### 1.9 Dropdown/Select

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Select<V> {
    placeholder: String,
    options: Vec<SelectOption<V>>,
    selected_index: Option<usize>,
    on_change: Arc<dyn Fn(usize) + Send + Sync>,
    is_open: bool,
}
```

---

## Tier 2: Medium Priority

### 2.1 Avatar

**File:** `cvkg-components/src/image.rs`

```rust
pub struct Avatar {
    src: Option<String>,
    fallback: String,
    size: AvatarSize,
}

enum AvatarSize {
    Sm,
    Md,
    Lg,
    Xl,
}
```

---

### 2.2 Badge

**File:** `cvkg-components/src/primitive.rs`

```rust
pub struct Badge {
    text: String,
    variant: BadgeVariant,
}

enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}
```

---

### 2.3 Tooltip

**File:** `cvkg-components/src/container.rs`

```rust
pub struct Tooltip<V> {
    content: V,
    text: String,
    position: TooltipPosition,
}

enum TooltipPosition {
    Top,
    Right,
    Bottom,
    Left,
}
```

---

### 2.4 Popover

**File:** `cvkg-components/src/container.rs`

```rust
pub struct Popover<T, C> {
    trigger: T,
    content: C,
    is_open: bool,
    position: PopoverPosition,
}
```

---

### 2.5 Calendar

**File:** `cvkg-components/src/calendar.rs` (new file)

```rust
pub struct Calendar {
    selected_date: Date,
    on_date_select: Arc<dyn Fn(Date) + Send + Sync>,
    min_date: Option<Date>,
    max_date: Option<Date>,
}
```

---

### 2.6 Date Picker

**File:** `cvkg-components/src/calendar.rs`

```rust
pub struct DatePicker {
    selected_date: Date,
    placeholder: String,
    on_date_change: Arc<dyn Fn(Date) + Send + Sync>,
}
```

---

### 2.7 Breadcrumb

**File:** `cvkg-components/src/navigation.rs` (new file)

```rust
pub struct Breadcrumb {
    items: Vec<BreadcrumbItem>,
}

struct BreadcrumbItem {
    label: String,
    href: Option<String>,
}
```

---

### 2.8 Pagination

**File:** `cvkg-components/src/navigation.rs`

```rust
pub struct Pagination {
    current_page: usize,
    total_pages: usize,
    on_page_change: Arc<dyn Fn(usize) + Send + Sync>,
}
```

---

### 2.9 Data Table

**File:** `cvkg-components/src/virtual_table.rs` (extend existing)

```rust
pub struct DataTable<T> {
    columns: Vec<ColumnSpec>,
    data: Vec<T>,
    sort_column: Option<usize>,
    sort_direction: SortDirection,
}
```

---

### 2.10 Skeleton

**File:** `cvkg-components/src/primitive.rs`

```rust
pub struct Skeleton {
    width: Option<f32>,
    height: Option<f32>,
    rounded: bool,
}
```

---

## Tier 3: Low Priority

### 3.1 Accordion

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Accordion<V> {
    items: Vec<AccordionItem<V>>,
    expanded_indices: Vec<usize>,
    on_change: Arc<dyn Fn(Vec<usize>) + Send + Sync>,
}
```

---

### 3.2 Collapsible

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Collapsible<V> {
    is_open: bool,
    trigger: V,
    content: V,
}
```

---

### 3.3 Carousel

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Carousel<V> {
    items: Vec<V>,
    current_index: usize,
    auto_play: bool,
}
```

---

### 3.4 Resizable

**File:** `cvkg-components/src/container.rs`

```rust
pub struct Resizable<V> {
    panels: Vec<ResizablePanel<V>>,
    orientation: Orientation,
}
```

---

### 3.5 Command (Cmd+K Palette)

**File:** `cvkg-components/src/interactive.rs`

```rust
pub struct Command<V> {
    is_open: bool,
    placeholder: String,
    items: Vec<CommandItem<V>>,
}
```

---

## Design Principles

### Visual Style
- **Glassmorphism:** Frosted glass effect with transparency
- **Neon accents:** Cyan/blue borders for interactive elements
- **Elevation:** Subtle shadows for depth perception
- **Dark theme:** Default dark backgrounds with light text

### Interaction Patterns
- Hover states with glow effects
- Press animations with scale/translation
- Smooth transitions using ease-in-out curves
- Accessible keyboard navigation

### API Consistency
```rust
// Builder pattern with chained methods
Component::new()
    .property(value)
    .callback(|e| ...)
    .build()

// View trait implementation for all components
impl View for Component {
    type Body = Never;
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) { ... }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size { ... }
}
```

---

## Testing Strategy

### Unit Tests
- Location: `cvkg-components/tests/component_tests.rs`
- Test each component's intrinsic size calculation
- Verify render output with mock renderer

### Snapshot Tests
- Location: `cvkg-components/tests/snapshots/`
- Visual snapshots for each component state
- Compare against reference images

### Accessibility Tests
- Location: `cvkg-components/tests/accessibility_tests.rs`
- ARIA role verification
- Keyboard navigation support
- Focus order validation

### Integration Tests
- Location: `cvkg-components/examples/`
- Demo applications combining components
- Cross-component interaction verification

---

## File Structure

```
cvkg-components/
├── src/
│   ├── lib.rs              # Re-exports
│   ├── primitive.rs        # Text, Divider, Spacer, Badge, Skeleton
│   ├── interactive.rs      # Button, Toggle, Slider, Input, Checkbox, Radio
│   ├── container.rs        # Sheet, Dialog, Alert, Navigation
│   ├── layout.rs           # HStack, VStack, ZStack, Grid
│   ├── navigation.rs       # Tabs, Breadcrumb, Pagination, Sidebar
│   ├── overlay.rs          # Tooltip, Popover, ContextMenu
│   ├── data.rs             # Card, Table, Avatar, Calendar
│   └── ...
├── tests/
│   ├── component_tests.rs
│   ├── accessibility_tests.rs
│   └── snapshots/
└── examples/
    ├── form_demo.rs
    ├── data_display_demo.rs
    └── navigation_demo.rs
```

---

## Estimated Timeline

| Tier | Components | Estimated Days |
|------|------------|----------------|
| Tier 1 | 9 components | 12-15 days |
| Tier 2 | 12 components | 15-18 days |
| Tier 3 | 9 components | 10-12 days |
| **Total** | **30 components** | **37-45 days** |

---

## Dependencies to Consider

- `cvkg-core` - Core View trait and Renderer
- `cvkg-layout` - Layout algorithms
- `cvkg-themes` - Color scheme and styling
- `cvkg-anim` - Animation utilities (for transitions)