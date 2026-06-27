# How to Create Components (Macros)

## Goal

Create components using the `hamr!` declarative macro from `cvkg-macros`.

## Prerequisites

- `cvkg-macros` available in your workspace
- Understanding of `View` trait basics

## Steps

### 1. Define State with `#[state]`

```rust
use cvkg_macros::state;

#[state]
struct CounterState {
    count: i32,
}
```

The `#[state]` macro derives `Clone`, `Debug`, `Default`, `Serialize`, and `Deserialize`.

### 2. Define a View with `#[derive(View)]`

```rust
use cvkg_macros::View;

#[derive(View)]
struct Counter {
    value: i32,
}
```

This implements `cvkg_core::View` with `Body = Never`.

### 3. Use in Composition

```rust
use cvkg_core::{State, View};

fn app() -> impl View {
    let count = State::new(0);

    Counter {
        value: *count.get(),
    }
}
```

## Expected Output

A component that can be composed into a view tree and responds to state changes.

## What Can Go Wrong

- **`#[state]` on enums**: The macro only works on structs.
- **Missing `body` method**: If you need a composite view (with children), implement `View` manually rather than using `#[derive(View)]`.
- **Macro expansion errors**: Ensure `cvkg-macros` is in your `[dependencies]` with the correct path.
