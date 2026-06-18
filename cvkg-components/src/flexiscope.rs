//! FlexiScope -- Container Query Layout System.
//!
//! Allows components to respond to their own container width rather than
//! the viewport width. This enables truly responsive components that
//! adapt whether they are in a sidebar (300px) or main content (1200px).

use cvkg_core::{Never, Rect, Renderer, View};

/// A wrapper that selects different layout modes based on container width.
#[derive(Clone)]
pub struct FlexiScope<V: View, B: ContainerLayout> {
    /// A builder function that creates the content view given the layout mode.
    builder: std::sync::Arc<dyn Fn(B) -> V + Send + Sync>,
    /// Breakpoints: width thresholds and corresponding layout modes.
    breakpoints: Vec<ScopeThreshold<B>>,
}

/// A width threshold and corresponding layout mode.
#[derive(Debug, Clone)]
pub struct ScopeThreshold<B> {
    pub min_width: f32,
    pub mode: B,
}

/// Trait for layout modes that respond to container size.
pub trait ContainerLayout: Clone + PartialEq {
    fn select_mode(width: f32, breakpoints: &[ScopeThreshold<Self>]) -> Self
    where
        Self: Sized,
    {
        let mut selected = &breakpoints[0];
        for bp in breakpoints {
            if width >= bp.min_width {
                selected = bp;
            }
        }
        selected.mode.clone()
    }
}

impl<V: View, B: ContainerLayout + 'static> FlexiScope<V, B> {
    pub fn new(
        builder: impl Fn(B) -> V + Send + Sync + 'static,
        breakpoints: Vec<ScopeThreshold<B>>,
    ) -> Self {
        Self {
            builder: std::sync::Arc::new(builder),
            breakpoints,
        }
    }
}

impl<V: View, B: ContainerLayout + Send + Sync + 'static> View for FlexiScope<V, B> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.breakpoints.is_empty() {
            return;
        }
        let mode = B::select_mode(rect.width, &self.breakpoints);
        let view = (self.builder)(mode);
        view.render(renderer, rect);
    }
}

/// Compute a font size that scales linearly between two widths.
pub fn fluid_typography(
    container_width: f32,
    min_width: f32,
    max_width: f32,
    min_size: f32,
    max_size: f32,
) -> f32 {
    if container_width <= min_width {
        return min_size.max(8.0);
    }
    if container_width >= max_width {
        return max_size.min(96.0);
    }
    let t = (container_width - min_width) / (max_width - min_width);
    let size = min_size + (max_size - min_size) * t.clamp(0.0, 1.0);
    size.clamp(8.0, 96.0)
}
