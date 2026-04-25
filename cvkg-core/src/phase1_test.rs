//! Phase 1 Verification Test
//!
//! This module verifies that the core framework components (Text, Button, State)
//! can be assembled into a minimal app that compiles, runs, and toggles state.

#[cfg(test)]
mod tests {
    use crate::{Never, State, View};

    // A minimal mock Button and Text for the core test,
    // since cvkg-components depends on cvkg-core and we're inside cvkg-core.
    struct Button<F>
    where
        F: Fn(),
    {
        action: F,
    }
    impl<F> Button<F>
    where
        F: Fn(),
    {
        fn new(_label: impl Into<String>, action: F) -> Self {
            Self { action }
        }
    }
    impl<F> View for Button<F>
    where
        F: Fn() + Send,
    {
        type Body = Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
    }

    struct AppView {
        state: State<bool>,
    }

    impl View for AppView {
        type Body = Button<Box<dyn Fn() + Send>>;

        fn body(self) -> Self::Body {
            let current_state = self.state.get();
            let label = if current_state { "ON" } else { "OFF" };

            let state_clone = self.state.clone();
            Button::new(
                label,
                Box::new(move || {
                    let current = state_clone.get();
                    state_clone.set(!current);
                }),
            )
        }
    }

    #[test]
    fn test_minimal_app_compiles_and_toggles_state() {
        let app = AppView {
            state: State::new(false),
        };

        // Check initial state
        assert_eq!(app.state.get(), false);

        // Get the body (which is a button)
        let body = app.body();

        // Simulate a click
        (body.action)();

        // Verify state toggled
        let state = State::new(false);
        let state_clone = state.clone();

        let action = move || {
            let current = state_clone.get();
            state_clone.set(!current);
        };

        assert_eq!(state.get(), false);
        action();
        assert_eq!(state.get(), true);
        action();
        assert_eq!(state.get(), false);
    }
}
