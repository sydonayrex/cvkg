# CVKG AI Agent Quickstart

This guide is designed for AI coding assistants (Cursor, Copilot, Claude Code)
to quickly understand and use CVKG effectively.

## One-Line Import

```rust
use cvkg::prelude::*;
```

This gives you: `View`, `State`, `Binding`, `Rect`, `AppState`, `FlexBox`, `Grid`,
`HStack`, `ScrollView`, `VStack`, `Button`, `Checkbox`, `Input`, `Select`, `Slider`,
`Text`, `Color`, and all English-aliased components.

## Rendering Pipeline Selection

Select exactly one rendering pipeline in your `Cargo.toml`:

```toml
cvkg = { workspace = true, features = ["native"] }   # Desktop (winit)
cvkg = { workspace = true, features = ["web"] }      # Browser (WASM)
cvkg = { workspace = true, features = ["gpu"] }      # Direct wgpu
```

**Never enable more than one.** The features are mutually exclusive and will
produce a compile-time error if combined.

## Five Macros

### `#[state]` -- Derive traits for state structs

```rust
#[state]
struct MyState {
    count: u32,
    name: String,
}
```

Derives: `Clone`, `Debug`, `Default`, `Serialize`, `Deserialize`.

### `#[derive(View)]` -- Implement View for a struct

```rust
#[derive(View)]
struct MyWidget { label: String }
```

### `#[view_component]` -- Function to View struct

```rust
#[view_component]
fn greeting(name: &str) -> impl View {
    Text::new(format!("Hello, {}!", name))
}
```

### `#[binding]` -- Derive traits for binding structs

```rust
#[binding]
struct FormBinding { email: String }
```

### `#[cvkg_component]` -- Generate struct + builder pattern

```rust
#[cvkg_component]
struct Card {
    title: String,
    content: String,
}

// Generates:
// Card::builder().title("Hi").content("Body").build()
```

## Norse-to-English Alias Table

All components below are available through `cvkg::prelude::*`. Use the English
name; the Norse name is the canonical type.

| English Name | Canonical (Norse) Type |
|---|---|
| `Accordion` | `SagaAccordion<AnyView>` |
| `Alert` | `GjallarAlert` |
| `Analytics` | `ValkyrieAnalytics` |
| `Avatar` | `MuninAvatar` |
| `ColorPicker` | `BifrostColorPicker` |
| `CommandPalette` | `MimirSpotlight` |
| `CreativeTools` | `BragiCreative` |
| `Decoder` | `RunestoneDecoder` |
| `Dialog` | `GeriDialog<AnyView>` |
| `HolographicDisplay` | `HolographicRunestone` |
| `HUD` | `WyrdHUD` |
| `Indicator` | `ValkyrieIndicator` |
| `Messenger` | `RavenMessenger` |
| `Orb` | `OracleOrb` |
| `Pagination` | `HringrPagination` |
| `Progress` | `SkollProgress` |
| `PromptBuilder` | `PromptForge` |
| `Rating` | `ValhallaRating` |
| `ScribingNote` | `ScribingStone` |
| `Sheet` | `GraniSheet<AnyView>` |
| `Spinner` | `HatiSpinner` |
| `Splitter` | `GjallarSplitter<AnyView, AnyView>` |
| `StepIndicator` | `SleipnirGait` |
| `Tabs` | `BifrostTabs` |
| `Timeline` | `UrdrTimeline` |
| `Tooltip` | `RunicTooltip<AnyView>` |
| `TreeView` | `YggdrasilTree` |
| `Well` | `MimirsWell` |
| `Window` | `YggdrasilWindow<AnyView>` |

## Three Complete Examples

### 1. Counter App

```rust
use cvkg::prelude::*;

struct Counter {
    count: State<u32>,
}

impl Counter {
    fn new() -> Self {
        Self { count: State::new(0) }
    }
}

impl View for Counter {
    type Body = VStack;
    fn body(self) -> Self::Body {
        VStack::new(16.0)
            .child(Text::new(format!("Count: {}", self.count.get())))
            .child(Button::new("+", {
                let count = self.count.clone();
                move || count.set(count.get() + 1)
            }))
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(Counter::new(), None);
}
```

### 2. Form with Validation

```rust
use cvkg::prelude::*;

struct LoginForm {
    email: State<String>,
    password: State<String>,
}

impl LoginForm {
    fn new() -> Self {
        Self {
            email: State::new(String::new()),
            password: State::new(String::new()),
        }
    }
}

impl View for LoginForm {
    type Body = VStack;
    fn body(self) -> Self::Body {
        VStack::new(12.0)
            .child(Input::new("Email").value(&self.email.get()).on_change({
                let email = self.email.clone();
                move |v| email.set(v)
            }))
            .child(Input::new("Password").value(&self.password.get()).on_change({
                let password = self.password.clone();
                move |v| password.set(v)
            }))
            .child(Button::new("Login", || println!("Login!")))
    }
}
```

### 3. Dashboard with Theme

```rust
use cvkg::prelude::*;

struct Dashboard;

impl View for Dashboard {
    type Body = VStack;
    fn body(self) -> Self::Body {
        VStack::new(16.0)
            .child(
                Text::new("Dashboard")
                    .font_size(28.0)
                    .color([0.0, 0.8, 1.0, 1.0]),
            )
            .child(
                HStack::new(12.0)
                    .child(Progress::new(0.7))
                    .child(Spinner::new().size(16.0)),
            )
            .child(
                HStack::new(8.0)
                    .child(Button::new("Refresh", || {}))
                    .child(Button::new("Export", || {})),
            )
    }
}
```

## Common Pitfalls

1. **Don't mix rendering features.** Select only one of `gpu`, `native`, or `web`.
2. **Use English names in new code.** The Norse names are canonical but the
   English aliases are preferred for readability.
3. **State requires `Clone + Send + Sync + 'static`.** Use `Arc<T>` for
   non-Clone types.
4. **Views must implement `Clone`.** Use `.erase()` to type-erase views
   when heterogeneous children are needed in stacks.
