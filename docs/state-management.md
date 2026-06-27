# Reactive State Management

CVKG's reactive state system is built on `State<T>` and `Binding<T>` types,
which use atomic reference counting (`ArcSwap`) for lock-free reads and
software transactional memory (STM) for atomic writes on native targets.

## Core Types

### `State<T>`

The primary reactive state container. Defined in `cvkg-core/src/lib.rs`.

```rust
pub struct State<T: Clone + Send + Sync + 'static> {
    swap: Arc<ArcSwap<T>>,           // Lock-free atomic value storage
    subscribers: SubscriberList<T>,   // List of effects to notify on change
    version: Arc<AtomicU64>,          // Monotonic version counter
    // ... STM fields on native targets
}
```

**Creating state:**
```rust
let count: State<u32> = State::new(0);
```

**Reading state:**
```rust
let value: u32 = count.get();  // Lock-free read via ArcSwap
```

**Writing state:**
```rust
count.set(42);  // Atomic write + notifies all subscribers
```

When `set()` is called:
1. The new value is stored in the `ArcSwap` (lock-free)
2. On native targets, the STM `TVar` is updated atomically
3. The version counter is incremented
4. All subscribed effects are notified synchronously

### `Binding<T>`

A read/write reference to a `State<T>`. Created from a state handle.

```rust
pub struct Binding<T: Clone + Send + Sync + 'static> {
    swap: Arc<ArcSwap<T>>,  // Shared with the parent State
    version: Arc<AtomicU64>, // Shared version counter
}
```

**Creating a binding:**
```rust
let binding: Binding<u32> = Binding::from_state(&count);
```

**Using a binding:**
```rust
let value = binding.get();   // Read through shared ArcSwap
binding.set(100);            // Write through shared ArcSwap
```

Bindings share the same underlying `ArcSwap` and version counter as their
parent `State`, so changes through either handle are visible to all.

## How Re-rendering Works

The reactive pipeline connects state changes to VDOM updates:

1. **State mutation**: `State::set()` increments the version counter and
   notifies subscribers.

2. **Effect execution**: Subscribed effects (registered during VDOM build)
   re-run when their tracked signals change. Effects call `Signal::get()`
   which auto-subscribes the running effect to the signal.

3. **VDOM diffing**: The VDOM layer (`cvkg-vdom/src/diff.rs`) compares the
   previous and current VDOM trees, producing a list of `VDomPatch` operations
   (Create, Update, Remove, Replace, Move, SetRoot, ClearHandlers).

4. **Render**: The renderer applies the patches to the scene graph.

## Complete Example

```rust
use cvkg::prelude::*;
use cvkg_core::{Binding, State, View};

struct CounterApp {
    count: State<u32>,
}

impl CounterApp {
    fn new() -> Self {
        Self { count: State::new(0) }
    }
}

impl View for CounterApp {
    type Body = VStack;

    fn body(self) -> Self::Body {
        // Create a binding to the state for this render pass
        let count_binding = Binding::from_state(&self.count);
        let current_value = count_binding.get();

        VStack::new(16.0)
            .child(
                Text::new(format!("Count: {}", current_value))
                    .font_size(24.0)
            )
            .child(
                Button::new("Increment", {
                    let count = self.count.clone();
                    move || {
                        let current = count.get();
                        count.set(current + 1);
                    }
                })
            )
    }
}
```

## Conflict Resolution (Multi-Agent)

On native targets, `State` supports STM-based conflict resolution for
multi-agent scenarios:

```rust
use cvkg_core::agents::ConflictResolution;

let state = State::new(0)
    .with_resolution(ConflictResolution::PriorityWins);
```

When multiple agents write concurrently, the resolution strategy determines
which write succeeds. The `PriorityWins` strategy keeps the write with the
highest priority and notifies the losing agent via `ConflictEvent`.

## Key Files

- `cvkg-core/src/lib.rs` -- `State<T>` (line 3316), `Binding<T>` (line 3743)
- `cvkg-vdom/src/signals.rs` -- `Signal<T>` primitive (fine-grained reactivity)
- `cvkg-vdom/src/diff.rs` -- `VDomPatch` enum (VDOM diffing operations)
