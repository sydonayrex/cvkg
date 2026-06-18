//! MorphBridge -- Shared Element Transition System.
//!
//! Provides shared element transitions where a view "morphs" from one
//! position to another when the component tree changes, creating spatial
//! continuity for the user.

use cvkg_core::{Never, Rect, Renderer, View};

/// A shared element transition that animates a view from one rect to another.
///
/// When the active element visually changes position (e.g., a card expands
/// into a detail view), MorphBridge animates from the old rect to the new rect,
/// rendering the content in a portal overlay that covers both positions.
#[derive(Clone)]
pub struct MorphBridge<V: View> {
    content: V,
    /// Unique key identifying this shared element across tree changes.
    element_key: String,
}

impl<V: View> MorphBridge<V> {
    /// Create a new MorphBridge with the given content and element key.
    pub fn new(content: V, element_key: impl Into<String>) -> Self {
        Self {
            content,
            element_key: element_key.into(),
        }
    }
}

impl<V: View> View for MorphBridge<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Register this element's current position with the renderer.
        // The renderer tracks old vs new positions and handles the animation.
        renderer.register_shared_element(&self.element_key, rect);
        self.content.render(renderer, rect);
    }
}

/// Linearly interpolate between two rectangles.
pub fn lerp_rect(a: &Rect, b: &Rect, t: f32) -> Rect {
    let t = t.clamp(0.0, 1.0);
    Rect {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
    }
}
