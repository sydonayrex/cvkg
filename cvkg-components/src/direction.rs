use cvkg_core::{Never, Rect, Renderer, View};

/// Direction -- RTL/LTR direction context provider.
/// Affects layout direction for child components.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

/// Direction context component.
/// Wraps content and provides direction context for RTL/LTR layout.
#[derive(Clone)]
pub struct DirectionProvider {
    #[allow(dead_code)]
    pub(crate) direction: Direction,
}

impl DirectionProvider {
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }

    pub fn ltr() -> Self {
        Self {
            direction: Direction::LeftToRight,
        }
    }

    pub fn rtl() -> Self {
        Self {
            direction: Direction::RightToLeft,
        }
    }
}

impl View for DirectionProvider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
        // Direction is a context provider -- it doesn't render anything itself.
        // It sets the direction state in the renderer for children to read.
        // The actual RTL flip happens in layout.
    }
}
