//! Declarative Form Binder for CVKG Components.
//!
//! Provides state binding primitives (`Binding<T>`) and a controller (`FormBinder<T>`)
//! to achieve parity with SwiftUI `@Binding` and React `useForm`.

use std::collections::HashMap;
use std::sync::Arc;

/// A state binding that wraps a value getter and setter callback to notify parent components of updates.
///
/// This provides a declarative, state-sharing interface similar to SwiftUI's `@Binding`
/// or React input handler state links.
#[derive(Clone)]
pub struct Binding<T> {
    /// Retrieve the current value of the state.
    pub get: Arc<dyn Fn() -> T + Send + Sync>,
    /// Update the state to a new value.
    pub set: Arc<dyn Fn(T) + Send + Sync>,
}

impl<T: Clone + 'static> Binding<T> {
    /// Create a new binding from explicit getter and setter functions.
    ///
    /// # Contract
    /// - `get` must return the current state of the bound variable.
    /// - `set` must invoke the state mutation logic in the parent component.
    pub fn new(
        get: impl Fn() -> T + Send + Sync + 'static,
        set: impl Fn(T) + Send + Sync + 'static,
    ) -> Self {
        Self {
            get: Arc::new(get),
            set: Arc::new(set),
        }
    }

    /// Retrieve the current value of the bound state.
    pub fn get(&self) -> T {
        (self.get)()
    }

    /// Mutate the bound state to a new value.
    pub fn set(&self, val: T) {
        (self.set)(val);
    }

    /// Project/map the binding to a different type using a bidirectional mapping function.
    ///
    /// This is highly useful for binding enum or integer fields to text inputs.
    pub fn project<U: Clone + 'static, F, G>(self, to: F, from: G) -> Binding<U>
    where
        F: Fn(T) -> U + Send + Sync + 'static,
        G: Fn(U) -> T + Send + Sync + 'static,
    {
        let get_fn = self.get.clone();
        let set_fn = self.set.clone();
        Binding::new(
            move || to(get_fn()),
            move |val| set_fn(from(val)),
        )
    }
}

/// A declarative form controller with validation capabilities and state bindings.
///
/// Achieves parity with React `useForm` and SwiftUI `@Binding` by linking input states,
/// tracking validation rules, rendering error status, and providing serialization hooks.
pub struct FormBinder<T> {
    /// The current state data of the form.
    pub state: T,
    /// Errors mapped by field name.
    pub errors: HashMap<String, String>,
    /// Validation rules to apply to the state.
    pub rules: HashMap<String, Vec<Arc<dyn Fn(&T) -> Result<(), String> + Send + Sync>>>,
}

impl<T: Clone + Send + Sync + 'static> FormBinder<T> {
    /// Create a new FormBinder with the initial state.
    pub fn new(state: T) -> Self {
        Self {
            state,
            errors: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    /// Add a validation rule for a specific field name.
    ///
    /// # Contract
    /// - The validator closure evaluates the whole form state and returns `Ok(())` or `Err(error_msg)`.
    pub fn add_rule(
        &mut self,
        field: impl Into<String>,
        rule: impl Fn(&T) -> Result<(), String> + Send + Sync + 'static,
    ) {
        self.rules
            .entry(field.into())
            .or_default()
            .push(Arc::new(rule));
    }

    /// Validate the current state against all registered rules.
    ///
    /// Returns true if all rules pass, false otherwise.
    /// Updates the internal error map for individual field lookup.
    pub fn validate(&mut self) -> bool {
        let mut is_valid = true;
        self.errors.clear();
        for (field, rules) in &self.rules {
            for rule in rules {
                if let Err(err) = rule(&self.state) {
                    self.errors.insert(field.clone(), err);
                    is_valid = false;
                    break;
                }
            }
        }
        is_valid
    }

    /// Returns whether the form currently has any errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Retrieve the validation error message for a given field.
    pub fn error_for(&self, field: &str) -> Option<&String> {
        self.errors.get(field)
    }

    /// Create a binding directly to a field of the form state.
    ///
    /// # Contract
    /// - `get_field` returns the specific field value from `T`.
    /// - `set_field` modifies the specific field value in `T`.
    /// - Invokes `on_change` in the parent to trigger component re-render.
    pub fn bind_field<U: Clone + 'static>(
        &self,
        get_field: impl Fn(&T) -> U + Send + Sync + 'static,
        set_field: impl Fn(&mut T, U) + Send + Sync + 'static,
        on_change: impl Fn(T) + Send + Sync + 'static,
    ) -> Binding<U> {
        let state_val = self.state.clone();
        let get_fn = Arc::new(get_field);
        let set_fn = Arc::new(set_field);
        let change_fn = Arc::new(on_change);

        let get_state = state_val.clone();
        let get_field_fn = get_fn.clone();
        let get = move || get_field_fn(&get_state);

        let set = move |val: U| {
            let mut new_state = state_val.clone();
            set_fn(&mut new_state, val);
            change_fn(new_state);
        };

        Binding::new(get, set)
    }

    /// Serialize the current state using a serialization function.
    pub fn serialize<S>(&self, serializer: impl Fn(&T) -> S) -> S {
        serializer(&self.state)
    }
}
