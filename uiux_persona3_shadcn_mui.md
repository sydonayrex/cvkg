# Persona 3: Product Designer (shadcn/MUI Migrant)

## Executive Summary

CVKG is an ambitious Rust-based UI framework with 215+ components that uses a View/Modifier pattern familiar to React/SwiftUI developers. For a product designer migrating from shadcn/ui and MUI, the component surface is impressively broad — most shadcn primitives have equivalents — but the framework's "Cyberpunk Viking" aesthetic, Norse naming conventions, and Rust-first architecture create significant friction for building production business web apps. The theming system (OKLCH-based) is technically sophisticated but opinionated toward dark mode and glassmorphism, requiring deliberate effort to achieve the neutral, clean defaults that shadcn users expect.

## Onboarding from React

**Mental model shift: Moderate-to-high friction.**

The View/Modifier pattern maps well conceptually to React's composition model. A CVKG `View` is roughly a React component, and `.modifier()` chains are analogous to wrapping with higher-order components or applying Tailwind classes. However, several Rust-specific patterns create friction:

1. **Type-system composition vs. children props**: React's `children` prop is replaced by Rust generics and `View::Body` associated types. You can't just nest arbitrary children — the type system enforces what goes where. This is powerful but restrictive compared to React's "anything goes" children.

2. **Builder pattern everywhere**: Every component uses `.foo().bar().baz()` chaining instead of JSX props. This is ergonomic in Rust but verbose compared to JSX's declarative attribute syntax.

3. **State management is global by default**: Components like `Input`, `MimirSpotlight`, and `DropdownMenu` use a global `load_system_state()`/`update_system_state()` hash-map pattern rather than React's `useState`. This is closer to a global store (Redux-like) than local component state, which is a significant mental shift.

4. **Norse naming is a constant tax**: Every API call requires translating from standard UI vocabulary to Norse mythology. `BifrostTabs` instead of `Tabs`, `GraniSheet` instead of `Sheet`, `MimirSpotlight` instead of `CommandPalette`, `RunesCard` instead of `Card`, `RunesTable` instead of `DataGrid`. This is not just cosmetic — it makes documentation searches, Stack Overflow queries, and team communication harder.

5. **No JSX equivalent**: The closest to JSX is the `view!` macro (from `cvkg_macros`), but it's not used consistently across the codebase. Most examples show imperative builder chains.

**Estimated ramp-up time**: 2-4 weeks for a React developer to feel productive, 6-8 weeks to build production-quality apps.

## Component Parity Matrix

### shadcn/ui Components → CVKG Equivalents

| shadcn Component | CVKG Equivalent | Gaps / Notes |
|---|---|---|
| `Button` | `Button` (from `interactive`) | ✅ More variants than shadcn (Glass, TintedGlass, Capsule). Missing `outline` — `Secondary` is the closest but not identical. |
| `Card` | `RunesCard` | ⚠️ Generic over a single type `V` — header/content/footer must be the same view type. shadcn's Card accepts different children types per slot. |
| `Dialog` | `GeriDialog`, `AlertDialog`, `ConfirmationDialog` | ⚠️ Three separate components instead of one compound component. No compound component pattern (Dialog.Trigger, Dialog.Content, etc.). AlertDialog uses fixed 400×180px size. |
| `Sheet` | `GraniSheet` | ✅ Supports Left/Right/Top/Bottom positions. Uses glassmorphic backdrop by default (may be too stylized for business). |
| `Tabs` | `BifrostTabs` | ⚠️ Glassmorphic background with "jelly physics" wobble animation. No plain/minimal style option. Closable tabs supported. |
| `Command` | `MimirSpotlight`, `BifrostLauncher` | ✅ Fuzzy matching, keyboard navigation. MimirSpotlight is a full-screen overlay (like Raycast/Spotlight). BifrostLauncher is simpler. |
| `Calendar` | `Calendar` (from `advanced_forms`), `DatePicker` (from `datepicker`) | ⚠️ `Calendar` is simplified (only renders first week). `DatePicker` is more complete with Single/Range modes. |
| `Accordion` | `SagaAccordion` | ⚠️ Named differently. Basic accordion functionality present. |
| `Alert` | `AlertDialog`, `GjallarAlert` | ⚠️ `AlertDialog` is modal-only. No inline/banner alert variant (GjallarAlert is HUD-styled). |
| `Avatar` | `MuninAvatar` | ⚠️ Norse naming. Basic avatar with status indicator. |
| `Badge` | `Badge` (from `primitive`), `MerkiBadge` | ✅ Badge has variants: Default, Secondary, Destructive, Outline, Success, Warning, Info. |
| `Breadcrumb` | `Breadcrumb` | ✅ Direct equivalent. |
| `Checkbox` | `Checkbox` (from `interactive`) | ✅ With label support. |
| `Collapsible` | `Collapsible` (from `container`) | ✅ Direct equivalent. |
| `Context Menu` | `ContextMenu` | ✅ With items and dividers. |
| `Dropdown Menu` | `DropdownMenu` | ✅ Glassmorphic dropdown with keyboard navigation. |
| `Hover Card` | `HoverCard` | ✅ Direct equivalent. |
| `Input` | `Input` (from `interactive/input`) | ✅ Full-featured with cursor, selection, clipboard, undo, IME support. |
| `Label` | `Label` (from `form_controls`) | ✅ With required indicator. |
| `Menubar` | `Menubar` | ✅ Horizontal menu bar with dropdowns. |
| `Navigation Menu` | `NavigationMenu` | ✅ Hierarchical navigation. |
| `Pagination` | `HringrPagination` | ⚠️ Norse naming. |
| `Popover` | `Popover` | ✅ Direct equivalent. |
| `Progress` | `SkollProgress` | ⚠️ Norse naming. |
| `Radio Group` | `RadioGroup` | ✅ With keyboard navigation and ARIA roles. |
| `Scroll Area` | `ScrollView`, `ScrollArea` | ✅ Both exist. ScrollArea has custom scrollbar. |
| `Select` | `Select` (from `interactive`), `Combobox`, `NativeSelect` | ✅ Multiple select variants. Combobox has search. |
| `Separator` | `Separator` | ✅ Direct equivalent. |
| `Skeleton` | `DraumaSkeleton` | ⚠️ Norse naming. |
| `Slider` | `Slider` (from `interactive`), `MjolnirSlider` | ✅ Two variants. |
| `Switch` | `Toggle` (from `interactive`) | ⚠️ Named Toggle, not Switch. |
| `Table` | `RunesTable` | ✅ See MUI DataGrid comparison below. |
| `Textarea` | `Textarea` (from `interactive`) | ✅ Direct equivalent. |
| `Toast` | `Toast` (from `toast`), `Sonner` | ✅ Two toast systems. Sonner is more feature-rich (positions, types, countdown). |
| `Toggle Group` | `ToggleGroup` | ✅ Single/multi-select modes. |
| `Tooltip` | `RunicTooltip` | ⚠️ Norse naming. |
| `OTP Input` | `InputOTP` | ✅ Direct equivalent. |
| `Phone Input` | `PhoneInput` | ✅ Direct equivalent. |
| `Mention Input` | `MentionInput` | ✅ Direct equivalent. |
| `Kbd` | `Kbd` | ✅ Direct equivalent. |
| `QR Code` | `QRCode` | ✅ Direct equivalent. |
| `Carousel` | `HatiCarousel`, `Carousel` | ✅ Two variants. |
| `Drawer` | `Drawer` | ✅ Slide-in panel. |
| `Form` | `Form` (from `form_validation`), `FormBinder` | ⚠️ Two separate systems. FormBinder is closer to react-hook-form. |
| `Input Group` | `InputGroup` | ✅ Direct equivalent. |
| `Toggle` | `Toggle` (from `interactive`) | ✅ Direct equivalent. |
| `Slider` | `Slider` (from `interactive`) | ✅ Direct equivalent. |
| `Stepper` | `Stepper` (from `interactive`) | ✅ Direct equivalent. |
| `Rating` | `ValhallaRating` | ⚠️ Norse naming. |
| `Color Picker` | `BifrostColorPicker` | ⚠️ Norse naming. |
| `Date Picker` | `DatePicker` | ✅ Single/Range modes with i18n month names. |
| `Time Picker` | `TimePicker` (from `m3_components`) | ✅ Clock face UI. |
| `Date Range Picker` | `DateRangePicker` | ✅ Calendar grid with range selection. |
| `Autocomplete` | `AutoComplete` | ✅ Glassmorphic dropdown with keyboard nav. |
| `Multi Select` | `MultiSelect` (from `advanced_forms`) | ✅ Basic multi-select. |
| `Tag Input` | `TagInput` | ✅ Tag input with dismiss. |
| `File Upload` | `DropVault` | ⚠️ Norse naming. Glassmorphic drop zone. |
| `Resizable` | `Resizable` | ✅ Draggable resize. |
| `Aspect Ratio` | `AspectRatio` | ✅ Direct equivalent. |
| `Empty State` | `EmptyState` | ✅ Direct equivalent. |
| `Banner` | `GjallarAlert` | ⚠️ HUD-styled, not a clean banner. |
| `Skeleton` | `DraumaSkeleton` | ⚠️ Norse naming. |
| `Spinner` | `HatiSpinner`, `Loader` | ✅ Multiple loader variants. |
| `Progress` | `SkollProgress` | ⚠️ Norse naming. |
| `Command` | `Command` (from `command`) | ✅ Command pattern implementation. |

### MUI Components → CVKG Equivalents

| MUI Component | CVKG Equivalent | Gaps / Notes |
|---|---|---|
| `TextField` | `Input`, `SearchField`, `Textarea` | ✅ Multiple input types. Input has full cursor/selection/IME. |
| `Select` | `Select`, `Combobox`, `NativeSelect` | ✅ Combobox adds search. |
| `Autocomplete` | `AutoComplete` | ✅ With glassmorphic dropdown. |
| `DataGrid` | `RunesTable` | ⚠️ See detailed comparison below. |
| `DatePicker` | `DatePicker` | ✅ With Single/Range modes. |
| `TimePicker` | `TimePicker` | ✅ Clock face. |
| `DateTimePicker` | `DateTimePicker` | ✅ Combined date+time. |
| `Tabs` | `BifrostTabs` | ⚠️ No plain style option. |
| `Dialog` | `GeriDialog`, `AlertDialog` | ⚠️ Fixed sizes, no compound component pattern. |
| `Drawer` | `Drawer` | ✅ Slide-in panel. |
| `AppBar` | `FloatingNavbar`, `NavbarMenu` | ✅ Glassmorphic navbar. |
| `Snackbar` | `Toast`, `Sonner` | ✅ Sonner has positions and countdown. |
| `Alert` | `AlertDialog`, `GjallarAlert` | ⚠️ No inline alert variant. |
| `Avatar` | `MuninAvatar` | ⚠️ Norse naming. |
| `Badge` | `Badge`, `MerkiBadge` | ✅ Multiple variants. |
| `Chip` | `Tag` | ✅ Dismissible tag. |
| `Divider` | `Divider`, `Separator` | ✅ Both exist. |
| `List` | `List` | ✅ Selectable list. |
| `Menu` | `DropdownMenu`, `Menubar` | ✅ Both exist. |
| `Pagination` | `HringrPagination` | ⚠️ Norse naming. |
| `Rating` | `ValhallaRating` | ⚠️ Norse naming. |
| `Slider` | `Slider`, `MjolnirSlider` | ✅ Two variants. |
| `Switch` | `Toggle` | ⚠️ Named Toggle. |
| `Tooltip` | `RunicTooltip` | ⚠️ Norse naming. |
| `Accordion` | `SagaAccordion` | ⚠️ Norse naming. |
| `Card` | `RunesCard` | ⚠️ Single generic type constraint. |
| `Paper` | `Shape::rounded_rect()` | ⚠️ No direct equivalent. |
| `Breadcrumbs` | `Breadcrumb` | ✅ Direct equivalent. |
| `Speed Dial` | `FAB`, `ExtendedFAB` | ✅ M3 FAB variants. |
| `Toggle Button` | `ToggleGroup` | ✅ Single/multi-select. |
| `Image List` | `Gallery` | ✅ Grid gallery. |
| `Skeleton` | `DraumaSkeleton` | ⚠️ Norse naming. |
| `Timeline` | `UrdrTimeline` | ⚠️ Norse naming. |
| `Tree View` | `RichTreeView`, `TreeViewNode` | ✅ Hierarchical tree. |
| `Virtualized List` | `VirtualList` | ✅ Direct equivalent. |

### Missing Components (No CVKG Equivalent)

| shadcn/MUI Component | Status |
|---|---|
| `Slider` (range slider) | ❌ No range slider (min/max handles) |
| `Command` (Cmd+K palette with groups) | ⚠️ Partial — MimirSpotlight has flat list, no grouped sections |
| `Calendar` (full month grid) | ⚠️ Partial — Calendar only renders first week |
| `Table` (with column resizing) | ⚠️ Partial — RunesTable has fixed column widths |
| `Table` (with pagination) | ❌ No built-in pagination |
| `Table` (with filtering) | ❌ No built-in column filtering |
| `Table` (with row grouping) | ❌ No row grouping |
| `Table` (with export) | ❌ No CSV/Excel export |
| `Transfer List` | ❌ Not available |
| `Rich Text Editor` | ⚠️ Partial — RunestoneEditor exists but API unclear |
| `Color Picker` (full) | ⚠️ BifrostColorPicker exists but may be basic |
| `Image Cropper` | ❌ Not available |
| `Signature Pad` | ❌ Not available |
| `Tour / Walkthrough` | ❌ Not available |
| `Watermark` | ❌ Not available |
| `Float Button` (MUI) | ✅ FAB exists |
| `Timeline` (MUI-style) | ⚠️ UrdrTimeline exists |
| `Stack` (MUI) | ✅ HStack, VStack exist |
| `Box` (MUI) | ⚠️ No direct equivalent (use Group or Shape) |
| `Container` (MUI) | ⚠️ No direct equivalent |
| `Grid` (MUI responsive) | ⚠️ Grid exists but no breakpoint-based props |
| `Hidden` (MUI) | ❌ Not available |
| `NoSsr` (MUI) | ❌ Not applicable (Rust) |
| `Portal` (MUI) | ⚠️ Partial — overlay system exists |
| `Modal` (MUI) | ✅ GeriDialog, HiminnModal |
| `Popover` (MUI) | ✅ Popover exists |
| `Popper` (MUI) | ⚠️ Partial — overlay system |
| `ClickAwayListener` | ⚠️ Manual in each component |
| `Autocomplete` (MUI freeSolo) | ⚠️ AutoComplete exists, freeSolo unclear |
| `Alert` (MUI inline) | ❌ No inline alert variant |
| `Backdrop` (MUI) | ⚠️ Manual in dialogs |
| `CircularProgress` (MUI) | ✅ Loader with Spinner variant |
| `LinearProgress` (MUI) | ✅ SkollProgress |
| `Skeleton` (MUI variants) | ⚠️ DraumaSkeleton — variants unclear |
| `Speed Dial` (MUI) | ❌ Not available |
| `Toggle Button` (MUI) | ✅ ToggleGroup |
| `Tooltip` (MUI followCursor) | ⚠️ RunicTooltip — followCursor unclear |

## Composition Model Comparison

### React Children Props vs. CVKG View/Modifier

**React pattern:**
```jsx
<Card>
  <CardHeader>Title</CardHeader>
  <CardContent>Body</CardContent>
  <CardFooter>Actions</CardFooter>
</Card>
```

**CVKG pattern:**
```rust
RunesCard::new()
    .header(Text::new("Title").font_size(16.0))
    .content(Text::new("Body"))
    .content(Text::new("Actions"))  // ⚠️ Can't add footer separately!
```

**Key differences:**

1. **Generic type constraint**: `RunesCard<V>` is generic over a single type `V`. All slots (header, content, footer) must be the same type. In React, each slot accepts any `ReactNode`. This is a significant limitation — you can't put a `Text` header with a `Button` footer without erasing to `AnyView`.

2. **No compound components**: shadcn's Dialog uses compound components (`Dialog.Trigger`, `Dialog.Content`, `Dialog.Header`). CVKG has separate components (`AlertDialog`, `ConfirmationDialog`, `GeriDialog`) with no shared compound pattern.

3. **Modifier chain vs. props**: CVKG uses `.modifier()` chains instead of props. This is idiomatic Rust but less readable than JSX for complex UIs:
   ```rust
   // CVKG
   Button::new("Click me")
       .variant(ButtonVariant::Destructive)
       .size(ButtonSize::Large)
       .on_click(|| { /* ... */ })
       .padding(16.0)
       .background([1.0, 0.0, 0.0, 1.0])
       .border([0.0, 0.0, 0.0, 1.0], 1.0)
       .elevation(4.0)
       .on_appear(|| { /* ... */ })
   ```

4. **Render props**: React's render props pattern (`renderItem={(item) => <Item {...item} />}`) maps to CVKG's `cell_builder` pattern in `RunesTable`:
   ```rust
   RunesTable::new(data)
       .column("Name", 200.0, true, |item| Text::new(&item.name))
   ```
   This works but is less flexible than React's render props which can return any component tree.

5. **No fragments**: React's `<>...</>` fragment has no direct equivalent. CVKG uses `Group` or `HStack`/`VStack` as container substitutes.

6. **Type erasure for heterogeneous children**: To mix different view types, you must call `.erase()` to get `AnyView`. This is similar to `React.createElement` boxing but explicit and manual.

## Theming Migration

### CSS Variables / Tailwind → OKLCH

**Current shadcn approach:**
```css
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --primary: 222.2 47.4% 11.2%;
  --muted: 210 40% 96.1%;
  --border: 214.3 31.8% 91.4%;
  --radius: 0.5rem;
}
```

**CVKG approach:**
```rust
// Theme::dark() — hardcoded Norse palette
Theme::dark()  // Viking Gold primary, Deep Void background, etc.

// OR derive from seed color
Theme::from_seed(OklchColor::new(0.55, 0.12, 260.0, 1.0))

// OR use ThemeBuilder (not shown in detail)
```

**Migration pain points:**

1. **No CSS variable equivalent**: CVKG themes are Rust structs, not CSS. You can't override a single variable in a media query — you must construct a new `Theme` or use `Theme::toggle()`.

2. **OKLCH is unfamiliar**: Most designers think in HEX/RGB/HSL. OKLCH (Lightness, Chroma, Hue) is perceptually uniform but requires learning a new mental model. The `OklchColor::new(0.55, 0.12, 260.0, 1.0)` API is not designer-friendly.

3. **Dark mode is the default**: `Theme::dark()` is the primary constructor. `Theme::light()` exists but the entire framework aesthetic (glassmorphism, neon glow, cyberpunk) is designed around dark mode. Building a light-mode-first business app requires fighting the defaults.

4. **No Tailwind-like utility classes**: Tailwind's `p-4`, `mt-2`, `rounded-lg` have no direct equivalent. CVKG uses modifier methods: `.padding(16.0)`, `.margin_top(8.0)` (not shown but implied), `.border_radius(RADIUS_LG)`.

5. **Spacing scale comparison**:
   | Tailwind | CVKG | Match? |
   |---|---|---|
   | `1` (4px) | `SPACE_XS` (4.0) | ✅ |
   | `2` (8px) | `SPACE_SM` (8.0) | ✅ |
   | `4` (16px) | `SPACE_MD` (16.0) | ✅ |
   | `6` (24px) | `SPACE_LG` (24.0) | ✅ |
   | `8` (32px) | `SPACE_XL` (32.0) | ✅ |
   | `0.5` (2px) | — | ❌ No 2px spacing |
   | `1.5` (6px) | — | ❌ No 6px spacing |
   | `3.5` (14px) | — | ❌ No 14px spacing |

6. **Radius scale comparison**:
   | Tailwind | CVKG | Match? |
   |---|---|---|
   | `rounded-sm` (2px) | `RADIUS_XS` (2.0) | ✅ |
   | `rounded` (4px) | `RADIUS_SM` (4.0) | ✅ |
   | `rounded-md` (6px) | `RADIUS_MD` (6.0) | ✅ |
   | `rounded-lg` (8px) | `RADIUS_LG` (8.0) | ✅ |
   | `rounded-xl` (12px) | `RADIUS_XL` (12.0) | ✅ |
   | `rounded-2xl` (16px) | `RADIUS_2XL` (16.0) | ✅ |
   | `rounded-full` (9999px) | `RADIUS_FULL` (9999.0) | ✅ |

### Dark/Light Mode Comparison

| Feature | next-themes | CVKG |
|---|---|---|
| Toggle | `<ThemeProvider attribute="class">` | `Theme::toggle()` or `Theme::dark()`/`Theme::light()` |
| System preference | `defaultTheme="system"` | `AccessibilityOverrides` + env var detection |
| CSS variables | Automatic | N/A (no CSS) |
| Per-component override | `class="dark:bg-white"` | `.background(theme::color("key"))` |
| Transition | CSS transition | Spring animation (Sleipnir) |
| Persistence | `localStorage` | Manual (no built-in) |

## Form Handling

### form_binder vs. react-hook-form

**react-hook-form pattern:**
```tsx
const { register, handleSubmit, formState: { errors } } = useForm();
<input {...register("email", { required: "Email is required" })} />
{errors.email && <span>{errors.email.message}</span>}
```

**CVKG FormBinder pattern:**
```rust
let mut form = FormBinder::new(MyFormState { email: String::new() });
form.add_rule("email", |state| {
    if state.email.is_empty() {
        Err("Email is required".to_string())
    } else {
        Ok(())
    }
});
let email_binding = form.bind_field(
    |s| s.email.clone(),
    |s, val| s.email = val,
    |new_state| update_state(new_state),
);
```

**Comparison:**

| Feature | react-hook-form | CVKG FormBinder | Verdict |
|---|---|---|---|
| Registration | `register("field")` | `bind_field(get, set, on_change)` | ⚠️ More verbose |
| Validation rules | Built-in + custom | `add_rule()` with closures | ✅ Similar |
| Error display | `formState.errors` | `form.errors` HashMap | ✅ Similar |
| Field arrays | `useFieldArray` | ❌ Not available | ❌ Missing |
| Nested objects | `register("user.name")` | Manual closure composition | ⚠️ Possible but manual |
| Schema validation | Zod/Yup integration | ❌ No schema integration | ❌ Missing |
| Dirty tracking | `formState.isDirty` | `FormField.is_dirty` | ✅ Present |
| Submit handling | `handleSubmit(onSubmit)` | Manual | ⚠️ More work |
| Reset | `reset()` | Manual state replacement | ⚠️ More work |
| Watch | `watch("field")` | `Binding::project()` | ✅ Present |
| Performance | Minimal re-renders | Global state updates | ⚠️ Less granular |

**FormField wrapper** (from `form_validation.rs`) provides a higher-level API closer to shadcn:
```rust
FormField::new("Email", Input::new("Enter email"))
    .required()
    .rule(ValidationRule::Pattern("@".to_string()))
```
This is closer to shadcn's `<FormItem><FormControl><Input /></FormControl><FormMessage /></FormItem>` pattern.

## Responsive & Density

### FlexiScope (Container Queries)

CVKG's `FlexiScope` is a container query system — components respond to their own width, not the viewport. This is more powerful than CSS container queries because it can switch entire layout modes, not just CSS properties.

```rust
FlexiScope::new(
    |mode| match mode {
        LayoutMode::Compact => VStack::new(/* mobile layout */),
        LayoutMode::Expanded => HStack::new(/* desktop layout */),
    },
    vec![
        ScopeThreshold { min_width: 0.0, mode: LayoutMode::Compact },
        ScopeThreshold { min_width: 600.0, mode: LayoutMode::Expanded },
    ],
)
```

**Comparison with Tailwind/MUI breakpoints:**

| System | Approach | Granularity |
|---|---|---|
| Tailwind | `sm:`, `md:`, `lg:`, `xl:` | CSS classes per property |
| MUI | `xs`, `sm`, `md`, `lg`, `xl` | Props per component |
| CVKG FlexiScope | Custom thresholds | Entire layout switch |

**Verdict**: FlexiScope is conceptually superior to MUI's breakpoint system but requires defining custom `ContainerLayout` enums. There's no built-in equivalent to Tailwind's `md:flex-row` — you must define the layout switch yourself.

### Density System

CVKG has a three-tier density system:

```rust
pub enum Density {
    Compact,   // 0.75x spacing/radius
    Default,   // 1.0x
    Spacious,  // 1.25x
}
```

**Comparison:**

| System | Options | Granularity |
|---|---|---|
| MUI | `density="compact"` (theme-level) | Theme-wide |
| shadcn | None (manual spacing) | Per-component |
| CVKG | `Density::Compact/Default/Spacious` | Theme-level |

**Verdict**: The three-tier system is sufficient for most use cases but less granular than MUI's density system which affects component sizing (not just spacing). CVKG's density only multiplies spacing and radius — it doesn't change font sizes or component heights.

## Accessibility Comparison

### WCAG 2.1 AA Compliance

**What CVKG does well:**

1. **Focus rings**: Every interactive component draws a focus ring via `draw_focus_ring()` using `FOCUS_RING_WIDTH` (2.0px) and `FOCUS_RING_OFFSET` (2.0px). This meets WCAG 2.4.7 (Visible Focus).

2. **ARIA roles**: Components set ARIA roles via `renderer.set_aria_role()` — `alertdialog`, `button`, `textbox`, `radiogroup`, `radio`, `menubar`, `navigation`, `list`, `listbox`, etc.

3. **ARIA properties**: The `AriaProperties` struct supports `role`, `label`, `description`, `value`, `pressed`, `checked`, `expanded`, `disabled`, `hidden`, `level`, `shortcut`, `focused`, `live`, `atomic` — covering most WCAG requirements.

4. **Keyboard navigation**: Most interactive components support keyboard navigation — Arrow keys, Enter, Space, Escape, Tab. BifrostTabs supports ArrowLeft/Right+W to close. MimirSpotlight supports ArrowUp/Down+Enter+Escape.

5. **Screen reader announcements**: `A11yBeacon` provides live region announcements with `AnnouncementPriority::Polite` and `Assertive`.

6. **Reduced motion**: `HlinAccessibility` detects OS-level reduced motion preferences via environment variables and gsettings.

7. **High contrast**: `AccessibilityOverrides::increase_contrast` and `HlinAccessibility::high_contrast` mode.

8. **APCA contrast**: The theme system includes APCA (Accessible Perceptual Contrast Algorithm) contrast evaluation.

**What's missing or unclear:**

1. **Focus trap management**: `HlinAccessibility` has `trap_active` but it's unclear if this is automatically applied to modals/dialogs. In React, Radix UI's Dialog automatically traps focus.

2. **Focus restoration**: No clear evidence that focus returns to the trigger element when a dialog/popover closes. This is a WCAG 2.1 requirement.

3. **Skip links**: No built-in skip navigation component.

4. **Landmark roles**: No explicit `<main>`, `<aside>`, `<nav>` semantic components. `NavigationMenu` sets `role="navigation"` but there's no `Main` or `Contentinfo` wrapper component.

5. **Form error association**: `FormField` renders error messages but doesn't set `aria-describedby` or `aria-invalid` on the input. This is a WCAG 3.3.1 requirement.

6. **Color contrast in default theme**: The dark theme uses `[0.95, 0.95, 1.0]` text on `[0.02, 0.02, 0.05]` background — this is high contrast. But the accent color (`#00FFFF` cyan) on dark background may not meet WCAG AA for small text.

7. **No automated a11y testing**: No built-in axe-core or pa11y integration. The `A11yInspector` component visualizes the a11y tree but doesn't validate compliance.

### Comparison with Radix UI Primitives

| Feature | Radix UI | CVKG |
|---|---|---|
| Focus management | Automatic in Dialog/Dropdown | Manual per component |
| Focus trap | Built-in | `HlinAccessibility::trap_active` |
| Focus restoration | Automatic | Unclear |
| Portal rendering | `<Portal>` | Overlay system |
| Scroll lock | Automatic in Dialog | Not visible |
| ARIA attributes | Comprehensive | Good but incomplete |
| Keyboard nav | Full WAI-ARIA patterns | Good but inconsistent |
| Screen reader | Tested with NVDA/VoiceOver | Unclear testing |
| Reduced motion | `prefers-reduced-motion` | Env var detection |

## Default Aesthetics

### Neutral Enough for Business? Or Too "Game-y"?

**Verdict: Too game-y for most business software out of the box.**

The default aesthetic is "Cyberpunk Viking" — a dark theme with:

1. **Glassmorphism everywhere**: Cards (`RunesCard`), sheets (`GraniSheet`), dropdowns (`DropdownMenu`), toasts (`Sonner`), command palette (`MimirSpotlight`), and tabs (`BifrostTabs`) all use `renderer.bifrost()` (frosted glass effect) by default. This is visually striking but inappropriate for most business dashboards.

2. **Neon accents**: The default accent color is `#00FFFF` (NiflCyan) with a "neon glow" effect (`renderer.gungnir()`). The primary color is `#FFD700` (Viking Gold). These are not neutral business colors.

3. **Animated everything**: BifrostTabs has a "jelly physics" wobble animation (`(t * 4.0).sin() * 2.0`). Sheets use spring animations. Loaders have animated spinners. This creates a "game HUD" feel rather than a professional business tool.

4. **Hardcoded dark theme**: `Theme::dark()` is the default. The light theme exists but the entire component library is designed around dark mode aesthetics.

5. **The Login component** renders "SYSTEM LOGIN // AUTHORIZE" with "TRANSMIT ACCESS CODES" button text. This is thematic but not suitable for a SaaS product.

6. **The Settings component** renders "SETTINGS // {CATEGORY}" in uppercase with accent color. Again, thematic but not business-neutral.

**Can you override it?**

Yes, but it requires work:
- Use `Theme::from_seed()` with a neutral color (e.g., `OklchColor::new(0.55, 0.05, 260.0, 1.0)` for a muted blue)
- Override glass effects by not calling `.bifrost()` — but many components call it internally
- Use `AccessibilityOverrides::reduce_transparency: true` to replace glass with solid backgrounds
- Override colors via the theme system — but you can't easily change individual component styles without modifying source

**The fundamental problem**: The aesthetic is baked into the component implementations, not just the theme. `RunesCard` always calls `renderer.bifrost()`. `BifrostTabs` always has the wobble animation. You'd need to fork components to get truly neutral styling.

## Gaps & Recommendations

### P0: Critical for Business Use

1. **Provide a "business" theme preset**
   ```rust
   // RECOMMENDED: Add to cvkg-themes
   impl Theme {
       pub fn business_light() -> Self {
           Self {
               is_dark: false,
               colors: SemanticColors {
                   primary: Color::new(0.20, 0.40, 0.60, 1.0),    // Muted blue
                   secondary: Color::new(0.50, 0.50, 0.55, 1.0),  // Gray
                   accent: Color::new(0.20, 0.45, 0.65, 1.0),     // Professional blue
                   background: Color::new(0.98, 0.98, 0.99, 1.0), // Near-white
                   surface: Color::new(0.95, 0.95, 0.97, 1.0),    // Light gray
                   error: Color::new(0.70, 0.15, 0.15, 1.0),      // Muted red
                   warning: Color::new(0.75, 0.55, 0.0, 1.0),     // Amber
                   success: Color::new(0.15, 0.60, 0.30, 1.0),    // Green
                   text: Color::new(0.10, 0.10, 0.15, 1.0),       // Near-black
                   text_dim: Color::new(0.45, 0.45, 0.50, 1.0),   // Medium gray
               },
               // ... rest of theme with NO glass effects
           }
       }
   }
   ```

2. **Add a "plain" component variant or style flag**
   ```rust
   // RECOMMENDED: Add style flags to components
   impl BifrostTabs {
       pub fn plain(self) -> Self { /* no glass, no wobble */ }
       pub fn animated(self, enabled: bool) -> Self { /* toggle animation */ }
   }
   
   impl RunesCard {
       pub fn solid(self) -> Self { /* no glassmorphism */ }
   }
   ```

3. **Fix RunesCard to accept heterogeneous children**
   ```rust
   // RECOMMENDED: Use AnyView for different slot types
   pub struct RunesCard {
       header: Option<AnyView>,
       content: Option<AnyView>,
       footer: Option<AnyView>,
   }
   ```

### P1: Important for Production

4. **Add standard component names as aliases**
   ```rust
   // RECOMMENDED: Type aliases for discoverability
   pub type Tabs = BifrostTabs;
   pub type Sheet = GraniSheet;
   pub type CommandPalette = MimirSpotlight;
   pub type Card = RunesCard;
   pub type DataGrid = RunesTable;
   ```

5. **Implement compound component pattern for Dialog**
   ```rust
   // RECOMMENDED: shadcn-style compound components
   Dialog::new()
       .trigger(Button::new("Open"))
       .content(
           DialogContent::new()
               .header(DialogHeader::new("Title"))
               .body(Text::new("Content"))
               .footer(DialogFooter::new().actions(vec![/* ... */]))
       )
   ```

6. **Add form error ARIA attributes**
   ```rust
   // RECOMMENDED: In FormField::render()
   if !self.is_valid && self.is_dirty {
       renderer.set_aria_invalid(true);
       if let Some(ref msg) = self.error_message {
           renderer.set_aria_describedby(&format!("{}-error", self.label));
       }
   }
   ```

7. **Add DataGrid pagination and filtering**
   ```rust
   // RECOMMENDED: Extend RunesTable
   impl<D> RunesTable<D> {
       pub fn paginated(mut self, page_size: usize, current_page: usize) -> Self { /* ... */ }
       pub fn filterable(mut self, enabled: bool) -> Self { /* ... */ }
       pub fn column_filter(mut self, column: &str, filter: impl Fn(&D) -> bool) -> Self { /* ... */ }
   }
   ```

8. **Add inline Alert component**
   ```rust
   // RECOMMENDED: Non-modal alert for form-level messages
   pub struct Alert {
       pub title: String,
       pub description: String,
       pub variant: AlertVariant, // Info, Success, Warning, Error
       pub icon: bool,
   }
   ```

### P2: Nice to Have

9. **Add breakpoint-based responsive props**
   ```rust
   // RECOMMENDED: MUI-style responsive props
   Button::new("Click")
       .size_for_breakpoint((
           (0.0, ButtonSize::Small),
           (600.0, ButtonSize::Default),
           (1024.0, ButtonSize::Large),
       ))
   ```

10. **Add Zod-like schema validation for FormBinder**
    ```rust
    // RECOMMENDED: Schema-based validation
    let schema = FormSchema::new()
        .field("email", Schema::string().required().email())
        .field("age", Schema::number().min(18).max(120));
    let form = FormBinder::with_schema(state, schema);
    ```

11. **Add CSS variable export for web target**
    ```rust
    // RECOMMENDED: Generate CSS variables from theme
    impl Theme {
        pub fn to_css_variables(&self) -> String {
            format!(":root {{\n  --color-primary: {};\n  --color-background: {};\n  ...}}", 
                self.colors.primary.to_hex(),
                self.colors.background.to_hex())
        }
    }
    ```

12. **Add Storybook-like component preview system**
    A component playground for visual testing — critical for designers.

## Verdict

**Score: 5.5/10** for a product designer migrating from shadcn/MUI.

**Breakdown:**
- Component breadth: 8/10 (215+ components, most shadcn/MUI equivalents exist)
- Composition model: 6/10 (Type-safe but restrictive vs. React's flexibility)
- Theming: 5/10 (OKLCH is powerful but opinionated; no CSS variable escape hatch)
- Accessibility: 6/10 (Good foundation but gaps in focus management and form errors)
- Default aesthetics: 3/10 (Cyberpunk Viking is too game-y for business; hard to override)
- Developer experience: 5/10 (Norse naming is a constant friction; no JSX equivalent)
- Documentation: 4/10 (Doc comments exist but no user-facing docs site)
- Form handling: 6/10 (FormBinder is capable but verbose; no schema validation)
- Responsive design: 7/10 (FlexiScope is genuinely good; density system is basic)
- Production readiness: 4/10 (Missing pagination, filtering, inline alerts, focus traps)

**Migration willingness: Conditional.**

I would consider CVKG for:
- Internal tools where the cyberpunk aesthetic is acceptable
- Rust-native desktop apps (the `native` feature with winit/AccessKit)
- Projects where performance is critical (GPU rendering via wgpu)

I would NOT choose CVKG for:
- Customer-facing SaaS products (aesthetic is too opinionated)
- Teams without Rust experience (learning curve is steep)
- Projects requiring rapid iteration (no hot reload, no CSS quick-fixes)
- Projects needing extensive third-party integrations (no npm ecosystem)

**The core tension**: CVKG is technically impressive — the OKLCH theming, FlexiScope container queries, and GPU rendering are genuinely innovative. But the "Cyberpunk Viking" identity is baked so deeply into the component implementations that using it for business software feels like wearing armor to a board meeting. The framework needs a "business mode" — a theme preset + component style flag that strips the glassmorphism, neon effects, and Norse naming — before it can be recommended for mainstream product design.
