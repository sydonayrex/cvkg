use cvkg_core::{Direction, DirectionKey, Environment, Never, Rect, Renderer, View};

/// Direction context component.
/// Wraps content and provides direction context for RTL/LTR layout.
#[derive(Clone)]
pub struct DirectionProvider {
    pub(crate) direction: Direction,
}

impl DirectionProvider {
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }

    pub fn ltr() -> Self {
        Self {
            direction: Direction::LTR,
        }
    }

    pub fn rtl() -> Self {
        Self {
            direction: Direction::RTL,
        }
    }
}

impl View for DirectionProvider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
        // Set the direction in the environment for children to read
        cvkg_core::env::insert::<DirectionKey>(self.direction);
    }
}

/// Get the current text direction from the environment.
pub fn current_direction() -> Direction {
    Environment::<DirectionKey>::new().get()
}

/// Check if the current direction is RTL.
pub fn is_rtl() -> bool {
    current_direction() == Direction::RTL
}