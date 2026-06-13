use crate::signals::Signal;
use cvkg_core::{Rect, Renderer, Size, SizeProposal, View};

/// A View that explicitly delegates its transform/bounds to a Reactive Signal.
/// This acts as a boundary: the content inside will NOT be re-laid out by Taffy
/// when the signal changes, instead the signal directly updates the GPU SceneNode.
#[derive(Clone)]
pub struct AnimatedBox<V: View> {
    pub content: V,
    pub bounds_signal: Signal<Rect>,
}

impl<V: View> AnimatedBox<V> {
    pub fn new(content: V, bounds_signal: Signal<Rect>) -> Self {
        Self {
            content,
            bounds_signal,
        }
    }
}

impl<V: View + Clone + 'static> View for AnimatedBox<V> {
    type Body = Self;

    fn body(self) -> Self::Body {
        self
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        // We report our size based on the signal's current bounds if we have them,
        // or we delegate to the content.
        let bounds = self.bounds_signal.get();
        if bounds.width > 0.0 && bounds.height > 0.0 {
            Size::new(bounds.width, bounds.height)
        } else {
            self.content.intrinsic_size(renderer, proposal)
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // The VNodeRenderer intercepts AnimatedBox via dynamic dispatch or downcasting
        // to wire up the signal subscription, but as a fallback, we just render the content.
        self.content.render(renderer, rect);
    }
}
