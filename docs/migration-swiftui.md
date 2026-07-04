# SwiftUI to CVKG Migration Guide

A side-by-side reference showing SwiftUI code alongside its CVKG equivalent.

## State Management

| SwiftUI | CVKG |
|---------|------|
| `@State var count = 0` | `state!{ count: 0 }` |
| `@Binding var isEnabled` | `hamr!{ is_enabled }` (two-way binding) |
| `@ObservedObject var model` | Component with `Arc<Mutex<T>>` for shared state |
| `StateObject` | `state!{}` in component body |

## Layout Primitives

| SwiftUI | CVKG |
|---------|------|
| `VStack { }` | `VStack::new(spacing)` |
| `HStack { }` | `HStack::new(spacing)` |
| `ZStack { }` | `ZStack::new()` |
| `Grid { }` | `Grid::new()` or `LazyVGrid` |
| `.frame(width:, height:)` | `.frame(Some(width), Some(height))` |
| `.padding()` | `.padding()` modifier |
| `.background()` | `.background(Color)` modifier |

## Modifiers

| SwiftUI | CVKG |
|---------|------|
| `.foregroundColor(.primary)` | `.color(theme.text())` |
| `.font(.title)` | `.font_size(24.0)` |
| `.cornerRadius(8)` | `.radius(8.0)` or `RADIUS_MD` |
| `.shadow(color:, radius:, x:, y:)` | `renderer.push_shadow()` internally |

## Navigation

| SwiftUI | CVKG |
|---------|------|
| `TabView { }` | `Tabs::new(options, selected, on_select)` |
| `NavigationView { }` | `NavigationStack` + `NavigationSplitView` |
| `.sheet(isPresented:)` | `.sheet(content)` modifier |
| `NavigationLink` | `Button` with navigation callback |

## Animation

| SwiftUI | CVKG |
|---------|------|
| `.animation(.spring())` | `.spring(hash, SpringParams::snappy())` |
| `.animation(.easeInOut)` | `SpringParams::gentle()` |
| `.transition(.slide)` | `Transition::slide_in()` |
| `withAnimation { }` | `update_system_state(|| ...)` |

## Effects

| SwiftUI | CVKG |
|---------|------|
| `.blur(radius:)` | `renderer.fill_glass_rect()` |
| `.opacity()` | `.opacity()` modifier |
| `.rotationEffect()` | `transform: Rotate` via custom rendering |
| `.scaleEffect()` | Scale transform in render loop |

## Forms & Controls

| SwiftUI | CVKG |
|---------|------|
| `TextField("placeholder", $text)` | `Input::new("placeholder").on_change(|t| {...})` |
| `Toggle("label", $isOn)` | `Toggle::new("label", is_on, |val| {...})` |
| `Slider(value: $value, in: 0...100)` | `Slider::new(value, 0.0..=100.0, |val| {...})` |
| `Picker("title", $selection, options)` | `Select::new("title").options(opts).on_change(...)` |
| `Button("title") { }` | `Button::new("title", || {})` |

## Data Flow

| SwiftUI | CVKG |
|---------|------|
| `ObservableObject` | `View::body()` + `state!{}` |
| `onAppear { }` | `.on_appear(|| {...})` modifier |
| `onDisappear { }` | `.on_disappear(...)` modifier |
| `task { }` | `cvkg_vdom::use_effect` patterns |

## View Lifecycle

| SwiftUI | CVKG |
|---------|------|
| `.onAppear { work() }` | `View::render()` automatically called on state change |
| `.onChange(of: value)` | State subscription via `state!{}` |
| `.id()` | `.key("unique_id")` modifier |

## Theming

| SwiftUI | CVKG |
|---------/------|
| `@Environment(\.colorScheme)` | `theme::current()` or `theme::surface()` |
| `Color.primary` | `theme::text()` |
| `Color.secondary` | `theme::text_muted()` |
| `AssetColor` | `semantic!` macro in themes |

## Examples

### Counter Example

**SwiftUI:**
```swift
struct CounterView: View {
    @State private var count = 0
    
    var body: some View {
        VStack {
            Text("Count: \(count)")
            Button("Increment") {
                count += 1
            }
        }
    }
}
```

**CVKG:**
```rust
struct CounterView;

impl View for CounterView {
    type Body = VStack;
    
    fn body(self) -> Self::Body {
        let (count, set_count) = state!{ count: 0 };
        VStack::new(8.0)
            .child(Text::new(format!("Count: {}", count)))
            .child(Button::new("Increment", move || {
                set_count(count + 1);
            }))
    }
}
```

### Form Example

**SwiftUI:**
```swift
struct FormView: View {
    @State private var email = ""
    @State private var agree = false
    
    var body: some View {
        Form {
            TextField("Email", $email)
            Toggle("I agree", $agree)
            Button("Submit") {
                submit()
            }
        }
    }
}
```

**CVKG:**
```rust
struct FormView;

impl View for FormView {
    type Body = VStack;
    
    fn body(self) -> Self::Body {
        let (email, set_email) = state!{ email: String::new() };
        let (agree, set_agree) = state!{ agree: false };
        
        VStack::new(12.0)
            .child(Input::new("Email").on_change(set_email))
            .child(Toggle::new("I agree", agree, set_agree))
            .child(Button::new("Submit", || { submit(); }))
    }
}
```

## Key Differences

1. **No view builder syntax:** CVKG uses builder pattern with `.child()`
2. **State is explicit:** `state!{}` macro creates state, no implicit `@State`
3. **No conditional view building:** Use `if` statements in render to choose views
4. **No built-in two-way binding:** Use `hamr!` macro for bindings
5. **No built-in navigation stack:** Use `NavigationStack` with manual routing

## Cheat Sheet

```
SwiftUI                    →  CVKG
@State                       →  state!{}
@Binding                     →  hamr!{}
Text("Hello")                →  Text::new("Hello")
Button("Tap") { }          →  Button::new("Tap", || {})
VStack { }                   →  VStack::new()
ForEach(0..<10) { i in }    →  (0..10).map(|i| ...)
.onAppear { }               →  .on_appear(|| {})
.animation(.spring())        →  SpringParams::snappy()
```