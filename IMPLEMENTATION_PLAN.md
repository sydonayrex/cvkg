# CVKG Implementation Plan
## Physics, Layout Engine & Accessibility Layer

---

## 1. PHYSICS ENGINE (cvkg-physics)

### 1.1 GJK Warm-Start for EPA
**File**: `cvkg-physics/src/narrowphase.rs`

**Problem**: `epa()` creates an arbitrary equilateral triangle as its initial polytope, ignoring the GJK termination simplex that already contains the origin. This wastes ~10-15 EPA iterations.

**Solution**: 
- Change `gjk()` to return `Option<([Vec2; 3], usize)` (the simplex points and vertex count) instead of just `bool`
- Add `epa_with_simplex()` that accepts the GJK simplex as its initial polytope
- Keep `epa()` as a fallback that creates an arbitrary triangle
- Update `collide()` to pass the GJK simplex through to EPA

**Impact**: Faster contact manifold generation, tighter penetration depth estimates.

### 1.2 Compound Shapes
**File**: `cvkg-physics/src/shape.rs`, `cvkg-physics/src/collider.rs`

**Problem**: A collider can only have a single shape (circle, AABB, capsule, or convex hull). Real-world objects need compound shapes (e.g., a table = 5 rectangles).

**Solution**:
- Add `ShapeKind::Compound(Vec<(Vec2, f32, Shape)>)` variant -- offset, rotation, and child shape
- Update `Shape::support()` to iterate children and return the farthest support point in the given direction
- Update `Shape::bounding_radius()` to account for child offsets
- Add `Collider::with_compound_shape()` convenience method
- `moment_of_inertia()` for compound: sum child inertias using parallel axis theorem

**Impact**: Richer collision geometry without changing the narrowphase interface.

### 1.3 Swept Collision Detection (CCD)
**File**: `cvkg-physics/src/world.rs`, `cvkg-physics/src/narrowphase.rs`

**Problem**: Fast-moving bodies tunnel through thin objects because the solver only checks discrete positions.

**Solution** (already partially done via velocity clamping in world.rs):
- Add `WorldConfig::ccd_enabled: bool` and `WorldConfig::ccd_max_substeps: u32`
- In `step_substep()`, when CCD is enabled, cast rays from each body's previous position to current position
- Use `Shape::swept_support()` to find the maximum extent along the sweep direction
- If a sweep intersects another body's expanded AABB, compute the time-of-impact and place the body at the collision point
- Add `gjk_swept()` that takes position deltas and returns the time of impact

**Impact**: Prevents tunneling for fast-moving projectiles, physics-based cursors, etc.

### 1.4 Persistent Contact Cache
**File**: `cvkg-physics/src/world.rs` (new module)

**Problem**: Contact manifolds are recomputed every frame from scratch. Bodies that remain in contact (e.g., a box sitting on a floor) should reuse the previous frame's contact data for better stability.

**Solution**:
- Add `ContactCache: HashMap<(BodyId, BodyId), Vec<Contact>>` to `PhysicsWorld`
- After narrow phase, merge new contacts with cached contacts using a proximity test
- Warm-start the contact impulse solver with accumulated impulses from the cache
- Expire cache entries that haven't been refreshed in N frames

**Impact**: Stacking stability, less jitter in resting contact.

### 1.5 Improved Broadphase: Swept AABB
**File**: `cvkg-physics/src/broadphase.rs`

**Problem**: Spatial hash uses static AABBs. Fast-moving bodies may skip cells between frames.

**Solution**:
- Expand each body's AABB by its velocity * dt before inserting into the spatial hash
- Add `SpatialHash::insert_swept(body_id, min, max, velocity, dt)` that expands the AABB
- This is a simple, zero-allocation change

**Impact**: Prevents fast-moving bodies from missing collisions.

---

## 2. LAYOUT ENGINE (cvkg-layout)

### 2.1 ScrollView
**File**: `cvkg-layout/src/new/mod.rs` (new module)

**Problem**: No scrolling layout. Content that overflows its bounds is clipped with no way to scroll.

**Solution**:
- Add `ScrollView { axis: Axis, content: Box<dyn LayoutView> }`
- Implement `LayoutView` that measures content without size constraints, then clips to the given bounds
- Add `scroll_offset: Vec2` state tracking
- Integrate with `cvkg-core` `ScrollCommand` for scroll events

**Impact**: Foundation for scrollable lists, text editors, etc.

### 2.2 Overlay / Z-Index Layer
**File**: `cvkg-layout/src/new/mod.rs`

**Problem**: `ZStack` layers children at the same z-level. Modals, tooltips, and popovers need to render above all other content.

**Solution**:
- Add `Overlay { content: Box<dyn LayoutView>, anchor: Rect }` for positioning relative to an anchor rect
- Add `Popover { anchor: Rect, content: Box<dyn LayoutView>, position: PopoverPosition }` 
- Implement automatic repositioning when the popover would overflow the screen edge

**Impact**: Tooltips, dropdown menus, context menus, modal overlays.

### 2.3 Aspect Ratio Layout
**File**: `cvkg-layout/src/new/mod.rs`

**Problem**: No way to constrain a child to an aspect ratio (e.g., 16:9 video, square avatar).

**Solution**:
- Add `AspectRatio { ratio: f32, child: Box<dyn LayoutView> }`
- During `size_that_fits()`, compute the largest rect that fits within the proposal while maintaining the ratio

**Impact**: Media displays, image containers, game viewports.

### 2.4 Safe Area / Padding
**File**: `cvkg-layout/src/new/mod.rs`

**Problem**: No concept of safe areas (notches, status bars) or uniform padding.

**Solution**:
- Add `SafeArea { insets: EdgeInsets, child: Box<dyn LayoutView> }`
- Add `Padding { insets: EdgeInsets, child: Box<dyn LayoutView> }`
- `EdgeInsets { top, right, bottom, left }` struct
- Modify child's proposal by subtracting insets

**Impact**: Proper layout on devices with notches, status bars, etc.

### 2.5 Intrinsic Content Size
**File**: `cvkg-core/src/layout.rs` (trait modification)

**Problem**: Layout views don't report their intrinsic minimum/maximum sizes. This makes it impossible for parents to size themselves based on content.

**Solution**:
- Add to `LayoutView` trait:
  - `fn min_size(&self) -> Size` -- minimum size the view needs
  - `fn max_size(&self) -> Size` -- maximum size before scrolling/clamping
  - `fn content_size(&self) -> Size` -- "natural" size for the content
- These have default implementations returning `Size::ZERO` / `Size::MAX`
- HStack/VStack use these to compute their own intrinsic sizes

**Impact**: Self-sizing buttons, labels that wrap, tables that size to content.

---

## 3. ACCESSIBILITY LAYER (cvkg-components + cvkg-core)

### 3.1 ARIA Trait for Components
**File**: `cvkg-core/src/a11y.rs` (new module)

**Problem**: There's no standardized way for components to declare their accessibility properties.

**Solution**:
- Define `AriaProperties` struct with fields for:
  - `role: AriaRole` (enum: Button, Link, Text, Image, Heading, List, ListItem, Form, Input, Navigation, Main, Banner, ContentInfo, etc.)
  - `label: String`
  - `description: String`
  - `value: Option<String>`
  - `checked: Option<bool>` (for checkboxes, toggles)
  - `expanded: Option<bool>` (for collapsible sections)
  - `disabled: bool`
  - `hidden: bool`
  - `level: Option<u32>` (for headings)
  - `shortcut: Option<String>` (keyboard shortcut)
- Define `AriaRole` enum with all standard ARIA roles
- Add `View::aria_properties() -> Option<AriaProperties>` with default `None`

**Impact**: Every component can declare its semantics in a standard way.

### 3.2 Focus Management System
**File**: `cvkg-core/src/focus.rs` (new module)

**Problem**: Focus state is scattered. There's no central focus manager, no tab order, no focus traps.

**Solution**:
- Define `FocusManager` struct with:
  - `focus_order: Vec<FocusableId>` -- ordered list of focusable elements
  - `focused_index: Option<usize>` -- currently focused element
  - `traps: Vec<FocusTrap>` -- active focus traps (for modals)
  - `history: Vec<FocusableId>` -- focus history for restoration
- Define `FocusableId(String)` newtype for unique focusable element IDs
- Define `FocusTrap { id: FocusableId, inner_order: Vec<FocusableId> }`
- Add `View::focus_properties() -> Option<FocusProperties>` to the View trait
- Implement Tab/Shift+Tab navigation, focus trapping for modals
- Add `FocusManager::handle_event(event: &InputEvent) -> bool` for processing Tab, Escape, etc.

**Impact**: Proper keyboard navigation, modal focus trapping, screen reader compatibility.

### 3.3 Screen Reader Bridge
**File**: `cvkg-core/src/screen_reader.rs` (new module)

**Problem**: No way for assistive technologies (screen readers) to discover UI state.

**Solution**:
- Define `LiveRegion` struct with:
  - `content: String`
  - `politeness: Politeness` (Off, Polite, Assertive)
  - `atomic: bool` -- whether to read the entire region or just changes
- Announce changes via platform APIs (Linux: AT-SPI, macOS: NSAccessibility, Windows: UIA)
- For now, implement a logging fallback that prints announcements to the console
- Add `View::live_regions() -> Vec<LiveRegion>` to the View trait

**Impact**: Screen readers can announce dynamic content changes.

### 3.4 Keyboard Navigation Trait
**File**: `cvkg-core/src/keyboard_nav.rs` (new module)

**Problem**: Each component handles keyboard input independently with no standardization.

**Solution**:
- Define `KeyboardNav` trait with:
  - `fn handle_key(&mut self, key: Key, modifiers: Modifiers) -> KeyResult` 
  - `fn key_bindings() -> Vec<KeyBinding>` -- declares what keys the component responds to
- Define `KeyBinding { key: Key, modifiers: Modifiers, action: KeyAction, label: String }`
- Define `KeyResult` enum (Consumed, Bubbled, Handled)
- Implement for all interactive components in cvkg-components
- Add keyboard shortcut registry: `ShortcutRegistry` maps global shortcuts to actions

**Impact**: Consistent keyboard navigation, discoverable shortcuts, conflict resolution.

### 3.5 Reduced Motion Integration
**File**: `cvkg-core/src/reduced_motion.rs` (new module)

**Problem**: `HlinAccessibility` detects reduced motion, but the information isn't propagated to the animation system.

**Solution**:
- Define `ReducedMotion` singleton that reads the OS preference at startup
- Add `ReducedMotion::is_active() -> bool` 
- Add `ReducedMotion::effective_duration(duration: Duration) -> Duration` -- returns 0 if reduced motion is active
- Integrate with `cvkg-anim` so Sleipnir springs, transitions, and particle systems respect the preference
- Connect to `Animation::update()` calls

**Impact**: Users with vestibular disorders get a comfortable experience.

### 3.6 High Contrast Theme Support
**File**: `cvkg-themes/src/high_contrast.rs` (new module)

**Problem**: `HlinAccessibility` has a `high_contrast` flag but there's no corresponding theme.

**Solution**:
- Define `HighContrastTheme` with:
  - Thicker borders (2px minimum)
  - Higher contrast color pairs (minimum 7:1 ratio per WCAG AAA)
  - Underlined links instead of color-only distinction
  - Visible focus rings always on (never hidden)
  - No reliance on color alone for information
- Add `Theme::high_contrast_variant() -> Theme`
- Add `Theme::is_high_contrast() -> bool`
- Ensure all components check the theme variant and adapt

**Impact**: WCAG AAA compliance for users with low vision.

### 3.7 Component-Level Accessibility Wiring
**File**: All interactive components in `cvkg-components/src/`

**Problem**: The A11yInspector shows a hardcoded demo tree. Individual components don't report their accessibility properties.

**Solution**: Wire accessibility into every interactive component:
- **Button**: `role: Button`, `label: text`, `disabled: !enabled`, `shortcut: keyboard_shortcut`
- **Checkbox/Toggle**: `role: Checkbox`, `label: text`, `checked: value`
- **Slider**: `role: Slider`, `label: text`, `value: format!("{}", value)`, `min/max`
- **Input/TextArea**: `role: TextBox`, `label: placeholder`, `value: text`, `disabled: !enabled`
- **Select/Dropdown**: `role: ListBox`, `label: placeholder`, `expanded: is_open`
- **Navigation**: `role: Navigation`, `label: title`
- **Heading**: `role: Heading`, `level: heading_level`
- **Image**: `role: Image`, `label: alt_text`
- **Table**: `role: Table`, role: Row/ColumnHeader for header cells

**Impact**: Assistive technologies can navigate and interact with all UI components.

---

## 4. IMPLEMENTATION ORDER

### Phase 1: Physics (Week 1-2)
1. GJK warm-start for EPA (1 day)
2. Compound shapes (2 days)
3. Swept broadphase (1 day)
4. CCD with swept GJK (2 days)
5. Persistent contact cache (2 days)
6. Tests + verification (2 days)

### Phase 2: Layout (Week 3)
1. EdgeInsets + Padding (0.5 day)
2. SafeArea (0.5 day)
3. AspectRatio (0.5 day)
4. Overlay/Popover (1 day)
5. ScrollView (1 day)
6. Intrinsic content sizes (1 day)
7. Tests + verification (1 day)

### Phase 3: Accessibility (Week 4-5)
1. AriaProperties trait (1 day)
2. Focus management system (2 days)
3. Keyboard navigation trait (1 day)
4. Screen reader bridge (1 day)
5. Reduced motion integration (1 day)
6. High contrast theme (1 day)
7. Component-level wiring (3 days -- one file per component)
8. Tests + verification (2 days)

### Phase 4: Integration + Verification (Week 6)
1. Full workspace `cargo check` + `cargo clippy` + `cargo fmt` + `cargo test`
2. Fix any issues found
3. Performance benchmarking
4. Run accessibility audit

---

## 5. RISKS & MITIGATIONS

| Risk | Impact | Mitigation |
|------|--------|-----------|
| `remove_body` swap-remove breaks existing indices | BodyId lookups fail | Already implemented and tested. 5 unit tests verify correctness. |
| Compound shapes slow down GJK | Narrowphase takes longer | Cache support direction across frames. Coarse AABB test first. |
| CCD adds per-frame cost | FPS drop | Gate behind `ccd_enabled` config. Only sweep bodies exceeding velocity threshold. |
| Accessibility trait changes break existing components | Compilation errors | Add trait methods with default implementations. Gradual adoption. |
| Focus manager conflicts with existing input handling | Keyboard input lost | Make focus manager opt-in via `Focusable` wrapper trait. |
| Theme changes affect visual appearance | UI looks different | High contrast theme is a separate opt-in theme, not a mutation of the default. |
