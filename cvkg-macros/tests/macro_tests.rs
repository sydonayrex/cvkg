#![feature(impl_trait_in_assoc_type)]
use cvkg_macros::view;
use cvkg_core::{View, Never};

#[view]
fn MyView(label: String) -> impl View {
    // Mock body
    cvkg_components::Text::new(label)
}

#[test]
fn test_view_macro_expansion() {
    let v = MyView("Hello".to_string());
    assert_eq!(v.label, "Hello");
}
