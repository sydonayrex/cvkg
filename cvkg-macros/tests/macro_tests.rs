#![feature(impl_trait_in_assoc_type)]
use cvkg_macros::view_component;

#[allow(non_snake_case)]
#[view_component]
fn MyView(label: String) {
    cvkg_components::Text::new(label)
}

#[test]
fn test_view_macro_expansion() {
    let v = MyView("Hello".to_string());
    // The view_component macro generates a struct with the fields from the function arguments
    assert_eq!(v.label, "Hello");
}
