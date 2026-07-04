use cvkg_core::{View, Companion, Never, erased_view::ErasedView};
use std::any::Any;

#[derive(Clone, Default)]
struct WithCompanion;

impl Companion for WithCompanion {
    fn type_name(&self) -> &'static str { "WithCompanion" }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Clone)]
struct HasCompanions;

impl View for HasCompanions {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn companion_states(&self) -> Vec<Box<dyn Companion>> {
        vec![Box::new(WithCompanion)]
    }
}

#[test]
fn test_erased_view_delegates_companion_states() {
    let view = HasCompanions;
    let erased: Box<dyn ErasedView> = Box::new(view);
    let companions = erased.companion_states_erased();
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].type_name(), "WithCompanion");
}