# hamr! DSL

The `hamr!` macro provides a declarative, JSX-like syntax for building CVKG UI trees.
It parses a custom `HamrRoot` syntax (nested `Type::new(...) { child child }` blocks)
and expands to plain Rust constructor chains.

## Basic Syntax

```rust
hamr! {
    VStack::new(16.0) {
        Text::new("Hello")
        Button::new("Click", || {})
    }
}
```

This expands to:

```rust
VStack::new(16.0)
    .child(Text::new("Hello"))
    .child(Button::new("Click", || {}))
```

## Rules

1. **Leaf nodes** are expressions without braces: `Text::new("Hello")`
2. **Branch nodes** are expressions with braces containing children: `VStack::new(16.0) { ... }`
3. Children are separated by whitespace (newlines or spaces)
4. The macro expands to `.child()` calls on the parent

## Examples

### Simple Layout

```rust
hamr! {
    VStack::new(12.0) {
        Text::new("Title")
        Text::new("Subtitle")
    }
}
```

### Nested Layout

```rust
hamr! {
    VStack::new(16.0) {
        HStack::new(8.0) {
            Text::new("Label")
            Text::new("Value")
        }
        HStack::new(8.0) {
            Button::new("OK", || {})
            Button::new("Cancel", || {})
        }
    }
}
```

### Forms

```rust
hamr! {
    VStack::new(12.0) {
        Input::new("Email")
        Input::new("Password")
        Checkbox::new(false, |_| {})
        Button::new("Submit", || {})
    }
}
```

### With State

```rust
hamr! {
    VStack::new(16.0) {
        Text::new(format!("Count: {}", count.get()))
        HStack::new(8.0) {
            Button::new("+", move || count.set(count.get() + 1))
            Button::new("-", move || count.set(count.get() - 1))
        }
    }
}
```

### Mixed Leaf and Branch

```rust
hamr! {
    VStack::new(12.0) {
        Text::new("Header")
        HStack::new(8.0) {
            Text::new("Left")
            Text::new("Right")
        }
        Text::new("Footer")
    }
}
```

### Empty Container

```rust
hamr! {
    VStack::new(0.0) { }
}
```

### Single Child

```rust
hamr! {
    VStack::new(16.0) {
        Text::new("Only child")
    }
}
```

### Deep Nesting

```rust
hamr! {
    VStack::new(16.0) {
        HStack::new(8.0) {
            VStack::new(4.0) {
                Text::new("A")
                Text::new("B")
            }
            VStack::new(4.0) {
                Text::new("C")
                Text::new("D")
            }
        }
    }
}
```

### With Closures

```rust
hamr! {
    VStack::new(12.0) {
        Button::new("Action", || println!("clicked"))
        Slider::new(0.5, 0.0..=1.0, |v| println!("value: {}", v))
    }
}
```

## Limitations

- All children must implement `View + Clone + 'static` (same as `.child()`)
- The macro does not support conditional rendering or loops -- use Rust control flow outside the macro
- Type annotations may be needed for closure arguments in some contexts

## Implementation

Defined in `cvkg-macros/src/lib.rs` (lines 256-273). The parser uses `syn` to parse
a custom `HamrRoot` syntax tree, then generates `.child()` calls via `quote!`.
