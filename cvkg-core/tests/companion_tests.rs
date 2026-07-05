use cvkg_core::{A11yCompanion, Companion, FocusState, FocusableCompanion};

#[test]
fn test_companion_trait_is_default_constructible() {
    let c = FocusableCompanion::default();
    assert_eq!(c.state, FocusState::Unfocused);
    assert_eq!(c.tab_index, 0);
}

#[test]
fn test_companion_type_name() {
    let c = FocusableCompanion::default();
    assert_eq!(c.type_name(), "FocusableCompanion");

    let a = A11yCompanion::default();
    assert_eq!(a.type_name(), "A11yCompanion");
}

#[test]
fn test_focusable_companion_new() {
    let c = FocusableCompanion::new();
    assert_eq!(c.state, FocusState::Unfocused);
    assert_eq!(c.tab_index, 0);
}

#[test]
fn test_a11y_companion_new() {
    let c = A11yCompanion::new();
    assert!(c.role.is_empty());
    assert!(c.label.is_empty());
    assert!(c.description.is_empty());
    assert!(!c.disabled);
}

#[test]
fn test_a11y_companion_builder() {
    let c = A11yCompanion::new()
        .with_role("button")
        .with_label("Submit")
        .with_description("Submit the form");
    assert_eq!(c.role, "button");
    assert_eq!(c.label, "Submit");
    assert_eq!(c.description, "Submit the form");
    assert!(!c.disabled);
}

#[test]
fn test_focus_state_variants() {
    assert_eq!(FocusState::default(), FocusState::Unfocused);
    assert!(matches!(FocusState::Unfocused, FocusState::Unfocused));
    assert!(matches!(FocusState::Focused, FocusState::Focused));
    assert!(matches!(FocusState::FocusVisible, FocusState::FocusVisible));
}
