// =============================================================================
// COLOR MODULE -- Standalone semantic colors type
// =============================================================================
//
// This module provides `SemanticColors`, a self-contained color palette that
// components can use without depending on `cvkg-themes`. The `use_theme()`
// function returns the current `SemanticColors` from thread-local storage.

use crate::{load_system_state, update_system_state};

// =============================================================================
// USE_STATE HOOK -- Local component state with automatic re-render
// =============================================================================
//
// Components call `use_state(id, initial)` to get a `(getter, setter)` pair.
// The setter updates the global system state and triggers a re-render.
//
// This is the minimal state primitive needed for interactive components.
// For complex state, use the global `AppState` directly.

/// Local state hook for components.
///
/// Returns a `(getter, setter)` pair:
/// - `getter()` returns the current value of type `T`
/// - `setter(value)` updates the value and triggers a re-render
///
/// The `id` must be unique per component instance (use a hash of the
/// component's label or a generated UUID).
pub fn use_state<T: Clone + Send + Sync + 'static>(
    id: u64,
    initial: T,
) -> (impl Fn() -> T, impl Fn(T)) {
    // Initialize the state if not already present
    let already_exists = load_system_state().get_component_state::<T>(id).is_some();
    if !already_exists {
        update_system_state(|s| {
            let mut ns = s.clone();
            ns.set_component_state(id, initial.clone());
            ns
        });
    }

    let getter = move || -> T {
        load_system_state()
            .get_component_state::<T>(id)
            .map(|arc_lock| {
                arc_lock
                    .read()
                    .ok()
                    .map(|guard| (*guard).clone())
                    .unwrap_or_else(|| initial.clone())
            })
            .unwrap_or_else(|| initial.clone())
    };

    let setter = {
        move |value| {
            update_system_state(|s| {
                let mut ns = s.clone();
                ns.set_component_state(id, value);
                ns
            });
        }
    };

    (getter, setter)
}

/// Generate a stable hash ID from a string key.
///
/// Use this to create unique IDs for `use_state` based on component labels
/// or other stable identifiers.
///
/// # Example
/// ```no_run
/// use cvkg_core::{use_state, use_state_hash};
/// let id = use_state_hash("my-checkbox");
/// let (value, set_value) = use_state(id, false);
/// ```
pub fn use_state_hash(key: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut s);
    s.finish()
}
