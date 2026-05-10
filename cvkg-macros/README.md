# cvkg-macros

**cvkg-macros** provides procedural macros for view generation in CVKG.

## What This Crate Does

- Provides `#[view]` attribute macro for component definition
- Provides `view!` macro for inline view construction
- Generates boilerplate for View trait implementation

## What This Crate Does NOT Do

- Does not provide runtime functionality
- Does not provide layout or rendering

## Public API Overview

### view! Macro

```rust
use cvkg_macros::view;

// Create views inline
let view = view! {
    VStack(spacing: 16.0) {
        Text("Hello")
        Button("Click Me")
    }
};
```

### #[view] Attribute

```rust
use cvkg_macros::view;

#[view]
struct MyComponent {
    count: i32,
}
impl MyComponent {
    fn new(count: i32) -> Self {
        Self { count }
    }
}
```

## Usage Example

```rust
use cvkg_macros::view;
use cvkg_components::{VStack, Text, Button};

let component = view! {
    VStack(spacing: 16.0) {
        Text("Title").size(24.0)
        Button("OK")
    }
};
```

## Known Limitations

- Macros generate verbose code; inspect expanded output with `cargo expand`
- Error messages from macro failures may be unclear
- Complex nested views may hit Rust's macro recursion limits