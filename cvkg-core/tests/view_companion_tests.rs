use cvkg_core::{View, Never};

#[derive(Clone)]
struct NoCompanions;

impl View for NoCompanions {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
}

#[test]
fn test_view_default_companion_states_is_empty() {
    let view = NoCompanions;
    let companions = view.companion_states();
    assert!(companions.is_empty(), "default must return empty vec");
}