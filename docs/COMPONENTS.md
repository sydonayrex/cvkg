# CVKG Component Index

Auto-generated index of all UI components in CVKG. Browse by category or search by standard English name (alias).

## Interactive Components

| Component | Description | Module |
|---|---|---|
| **Button** | Clickable button with variants (Default, Destructive, Secondary, Ghost, Link, Glass, TintedGlass, Capsule) | `interactive::button` |
| **Checkbox** | Toggle checkbox with indeterminate state | `interactive::checkbox` |
| **RadioGroup** | Radio button group for single selection | `radio_group` |
| **Toggle** | On/off switch | `interactive::checkbox` |
| **Slider** | Range slider with custom min/max | `interactive::button` |
| **Stepper** | Numeric stepper with increment/decrement buttons | `interactive::button` |
| **Input** | Single-line text input with cursor, selection, validation | `interactive::input` |
| **Textarea** | Multi-line text input | `interactive::textarea` |
| **Select** | Dropdown select with search | `interactive::select` |
| **Picker** | Color/date/time picker | `interactive` |
| **SecureField** | Password input with visibility toggle | `interactive` |
| **SearchField** | Search input with suggestions | `form_controls` |
| **Tag** | Tag/chip component | `form_controls` |
| **Label** | Form field label | `form_controls` |
| **Link** | Hyperlink component | `form_controls` |
| **Rating** | Star rating component | `interactive` |
| **Pagination** | Pagination control | `interactive::hringrpagination` |
| **InputOTP** | One-time password input | `input_otp` |

## Layout Components

| Component | Description | Module |
|---|---|---|
| **HStack** | Horizontal stack layout | `container::stacks` |
| **VStack** | Vertical stack layout | `container::stacks` |
| **ZStack** | Z-index stack (overlay) | `layout_primitives` |
| **FlexBox** | Flexible box (Taffy wrapper) | `container::flex` |
| **Grid** | Grid layout | `grid` |
| **LazyVStack** | Lazy-loaded vertical stack | `layout_primitives` |
| **LazyHStack** | Lazy-loaded horizontal stack | `layout_primitives` |
| **LazyVGrid** | Lazy-loaded grid | `layout_primitives` |
| **LazyHGrid** | Lazy-loaded horizontal grid | `layout_primitives` |
| **ScrollView** | Scrollable container | `container::scroll` |
| **Resizable** | Resizable container | `layout_primitives` |
| **AspectRatio** | Aspect ratio box | `layout_primitives` |
| **Separator** | Dividers and separators | `layout_primitives` |
| **Group** | Group container | `layout_primitives` |
| **GroupBox** | Group box with title | `layout_primitives` |
| **DisclosureGroup** | Expandable/collapsible group | `navigation` |
| **List** | List container | `navigation` |
| **Section** | Section with header | `navigation` |

## Container Components

| Component | Description | Module |
|---|---|---|
| **Dialog** | Modal dialog | `container::modal` |
| **Sheet** | Side sheet modal | `container` |
| **Popover** | Floating popover | `popover` |
| **HoverCard** | Card that appears on hover | `hover_card` |
| **ContextMenu** | Right-click context menu | `context_menu` |
| **DropdownMenu** | Dropdown menu | `dropdown_menu` |
| **AlertDialog** | Alert dialog | `dialog` |
| **ConfirmationDialog** | Confirmation dialog | `dialog` |
| **FullScreenCover** | Full-screen overlay | `dialog` |
| **Drawer** | Bottom sheet drawer | `navigation` |
| **Menubar** | Top navigation bar | `navigation` |
| **NavigationMenu** | Nested navigation menu | `navigation` |
| **Breadcrumb** | Breadcrumb navigation | `breadcrumb` |
| **NavigationSplitView** | Split view navigation | `container` |
| **NavigationStack** | Navigation stack | `container` |

## Form Components

| Component | Description | Module |
|---|---|---|
| **Form** | Form container with validation | `form_validation` |
| **FormField** | Form field wrapper with validation | `form_validation` |
| **FormBinder** | Form state binder | `form_binder` |
| **DateTimePicker** | Date/time picker | `form_controls` |
| **DateRangePicker** | Date range picker | `form_controls` |
| **TimePicker** | Time picker | `form_controls` |
| **PhoneInput** | Phone number input | `phone_input` |
| **InputGroup** | Group of related inputs | `input_group` |
| **InputOTP** | OTP input fields | `input_otp` |
| **SearchField** | Search input | `form_controls` |
| **Tag** | Tag/chip input | `form_controls` |
| **Label** | Form label | `form_controls` |
| **Link** | Hyperlink | `form_controls` |

## Data Display

| Component | Description | Module |
|---|---|---|
| **DataGrid** | Table/grid component | `data_grid` |
| **TreeView** | Hierarchical tree view | `tree_view` |
| **Calendar** | Calendar widget | `calendar` |
| **Scheduler** | Timeline scheduler | `scheduler` |
| **Gantt** | Gantt chart | `scheduler` |
| **Kanban** | Kanban board | `m3_components` |
| **Chart** | Various chart types | `gpu_charts` |

## Visual Components

| Component | Description | Module |
|---|---|---|
| **Text** | Text display | `primitive` |
| **Typography** | Typography display | `display` |
| **Avatar** | User avatar | `visual` |
| **Badge** | Badge/stamp | `visual` |
| **Spinner** | Loading spinner | `visual` |
| **ProgressBar** | Progress bar | `visual` |
| **Carousel** | Image/content carousel | `visual` |
| **Marquee** | Scrolling text | `layout_components` |
| **Loader** | Loading indicator | `layout_components` |
| **Toast** | Toast notification | `toast` |
| **Alert** | Alert/notification | `feedback::alert` |
| **Card** | Card container | `card` |

## Navigation

| Component | Description | Module |
|---|---|---|
| **Tabs** | Tab navigation (alias for BifrostTabs) | `bifrost_tabs` |
| **Accordion** | Expandable sections | `container` |
| **Drawer** | Side drawer | `navigation` |
| **Menubar** | Top menu bar | `navigation` |

## Advanced Components

| Component | Description | Module |
|---|---|---|
| **FlowGraph** | Node graph editor | `node_graph_editor` |
| **Codeblock** | Code display | `text_editor` |
| **TextEditor** | Rich text editor | `text_editor` |
| **Map** | Map component | `multimedia` |
| **Video** | Video player | `multimedia` |
| **Audio** | Audio player | `multimedia` |
| **QRCode** | QR code generator | `qrcode` |
| **FileTree** | File browser tree | `file_tree` |
| **CommandPalette** | Command palette | `command_palette` |

## Animation & Effects

| Component | Description | Module |
|---|---|---|
| **Motion** | Animation preset library | `motion` |
| **TextAnimate** | Text animations | `text_anim` |
| **TypewriterEffect** | Typewriter animation | `text_anim` |
| **ShimmerButton** | Shimmer effect button | `text_anim` |
| **RippleButton** | Ripple effect button | `text_anim` |
| **NumberTicker** | Number counting animation | `text_anim` |
| **CardStack** | Stack flip animation | `text_anim` |
| **DraggableCard** | Draggable card animation | `text_anim` |
| **ExpandableCard** | Expand/collapse animation | `text_anim` |

## Game UI

| Component | Description | Module |
|---|---|---|
| **HealthBar** | Health bar with gradient | `game` |
| **MiniMap** | Mini map display | `game` |
| **DPadControl** | Directional pad | `game` |

## Landing Page

| Component | Description | Module |
|---|---|---|
| **Hero** | Hero section | `landing` |
| **FeatureGrid** | Feature grid | `landing` |
| **PricingTable** | Pricing table | `landing` |
| **TestimonialCard** | Testimonial card | `landing` |

## Accessibility

| Component | Description | Module |
|---|---|---|
| **AccessibilityTree** | Accessibility tree | `hlin_accessibility` |
| **A11yBeacon** | Accessibility beacon | `a11y_beacon` |
| **A11yInspector** | Accessibility inspector | `a11y_inspector` |

## DevTools

| Component | Description | Module |
|---|---|---|
| **Inspector** | Component inspector | `freyr_inspector` |
| **DevToolsInspector** | DevTools inspector | `gullveig_inspector` |
| **Telemetry** | Telemetry display | `gerd_telemetry` |

---

## How to Use

### Import from Prelude (Standard Names)

```rust
use cvkg::prelude::*;

let button = Button::new("Click me");
let tabs = Tabs::new(vec!["Tab 1".to_string(), "Tab 2".to_string()]);
let dialog = Dialog::new();
let sheet = Sheet::new();
```

### Import from Component Module

```rust
use cvkg_components::{BifrostTabs, MjolnirSlider, GeriDialog};

let tabs = BifrostTabs::new(vec!["A".to_string(), "B".to_string()]);
let slider = MjolnirSlider::new(0.0, 100.0);
```

### Import from Individual Modules

```rust
use cvkg_components::container::modal::GeriDialog;
use cvkg_components::interactive::button::Button;
use cvkg_components::interactive::input::Input;
```

---

*Generated from source files in `cvkg-components/src/`. Use `cvkg::prelude::*` for standard English names or Norse names for the canonical implementations.*