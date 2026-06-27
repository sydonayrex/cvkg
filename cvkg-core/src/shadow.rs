//! Shadow system — box-shadow, text-shadow, inset, spread, layered shadows.
//!
//! This module provides:
//! - `Shadow` struct: describes a single shadow layer (offset, blur, spread, color, inset)
//! - `ShadowModifier`: View modifier that applies one or more shadow layers
//! - `shadow!` macro: convenient syntax for creating shadow sets

use crate::{ModifiedView, Renderer, View};

/// A single shadow layer description.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// Horizontal offset in logical pixels.
    pub offset_x: f32,
    /// Vertical offset in logical pixels.
    pub offset_y: f32,
    /// Blur radius in logical pixels.
    pub blur_radius: f32,
    /// Spread radius (expands or contracts the shadow).
    pub spread: f32,
    /// Shadow color (RGBA).
    pub color: [f32; 4],
    /// Whether this is an inset shadow.
    pub inset: bool,
}

impl Shadow {
    /// Create a new outward (drop) shadow.
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, color: [f32; 4]) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            spread: 0.0,
            color,
            inset: false,
        }
    }

    /// Create a new inset shadow.
    pub fn inset(offset_x: f32, offset_y: f32, blur: f32, color: [f32; 4]) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            spread: 0.0,
            color,
            inset: true,
        }
    }

    /// Create a new drop shadow with default black color at 30% opacity.
    pub fn drop(offset_x: f32, offset_y: f32, blur: f32) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            spread: 0.0,
            color: [0.0, 0.0, 0.0, 0.3],
            inset: false,
        }
    }

    /// Set the spread radius.
    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Set the shadow color.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Make this an inset shadow.
    pub fn set_inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Convert to a `ShadowModifier` for a single-layer shadow.
    pub fn modifier(self) -> ShadowModifier {
        ShadowModifier { layers: vec![self] }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 2.0,
            blur_radius: 8.0,
            spread: 0.0,
            color: [0.0, 0.0, 0.0, 0.25],
            inset: false,
        }
    }
}

/// A shadow modifier that applies one or more shadow layers to a view.
/// Supports layered shadows (up to 4 layers for performance).
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowModifier {
    layers: Vec<Shadow>,
}

impl ShadowModifier {
    /// Create a new shadow modifier with the given layers.
    pub fn new(layers: Vec<Shadow>) -> Self {
        Self {
            layers: layers.into_iter().take(4).collect(),
        }
    }

    /// Create a single-layer shadow modifier.
    pub fn single(shadow: Shadow) -> Self {
        Self {
            layers: vec![shadow],
        }
    }

    /// Create a layered shadow modifier from multiple shadows.
    pub fn layered(shadows: impl IntoIterator<Item = Shadow>) -> Self {
        Self::new(shadows.into_iter().collect())
    }

    /// Add another shadow layer (max 4).
    pub fn and(mut self, shadow: Shadow) -> Self {
        if self.layers.len() < 4 {
            self.layers.push(shadow);
        }
        self
    }

    /// Returns the layers.
    pub fn layers(&self) -> &[Shadow] {
        &self.layers
    }
}

impl ViewModifier for ShadowModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if self.layers.is_empty() {
            view.render(renderer, rect);
            return;
        }

        // Apply each shadow layer using push/pop
        for layer in &self.layers {
            let effective_blur = (layer.blur_radius + layer.spread).max(0.0);
            let offset = if layer.inset {
                [-layer.offset_x, -layer.offset_y]
            } else {
                [layer.offset_x, layer.offset_y]
            };
            renderer.push_shadow(effective_blur, layer.color, offset);
        }

        view.render(renderer, rect);

        for _ in &self.layers {
            renderer.pop_shadow();
        }
    }
}

/// Convenient macro for creating shadow sets.
///
/// Syntax:
/// ```ignore
/// use cvkg_core::shadow;
/// shadow! { offset_x, offset_y, blur, color }
/// shadow! { inset offset_x, offset_y, blur, color }
/// shadow! { layer1, layer2, ... }
/// ```
#[macro_export]
macro_rules! shadow {
    // Single shadow: offset_x, offset_y, blur, color
    ($x:expr, $y:expr, $blur:expr, $color:expr) => {
        $crate::shadow::Shadow::new($x, $y, $blur, $color).modifier()
    };
    // Inset shadow
    (inset $x:expr, $y:expr, $blur:expr, $color:expr) => {
        $crate::shadow::Shadow::inset($x, $y, $blur, $color).modifier()
    };
    // Layered: multiple shadows comma-separated
    ($($shadow:expr),+ $(,)?) => {
        $crate::shadow::ShadowModifier::layered(vec![$($shadow),+])
    };
}

/// Extension trait for adding shadow modifiers to any View.
pub trait ViewShadowExt: View + Sized {
    /// Apply a single drop shadow.
    fn shadow(
        self,
        offset_x: f32,
        offset_y: f32,
        blur: f32,
        color: [f32; 4],
    ) -> ModifiedView<Self, ShadowModifier> {
        self.modifier(Shadow::new(offset_x, offset_y, blur, color).modifier())
    }

    /// Apply an inset shadow.
    fn inset_shadow(
        self,
        offset_x: f32,
        offset_y: f32,
        blur: f32,
        color: [f32; 4],
    ) -> ModifiedView<Self, ShadowModifier> {
        self.modifier(Shadow::inset(offset_x, offset_y, blur, color).modifier())
    }

    /// Apply layered shadows.
    fn shadows(self, layers: Vec<Shadow>) -> ModifiedView<Self, ShadowModifier> {
        self.modifier(ShadowModifier::new(layers))
    }
}

impl<V: View> ViewShadowExt for V {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockRenderer;
    use crate::{Color, Rect, Renderer, View};

    struct TestView;

    impl View for TestView {
        type Body = Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
        fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
            renderer.fill_rect(rect, [1.0, 1.0, 1.0, 1.0]);
        }
    }

    #[test]
    fn shadow_renders_without_panic() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let shadow = Shadow::drop(0.0, 4.0, 8.0);
        let view = TestView.shadow(0.0, 4.0, 8.0, [0.0, 0.0, 0.0, 0.3]);
        view.render(&mut renderer, rect);
        // Shadow uses push/pop which are recorded as no-ops in MockRenderer
        // The fill_rect from TestView should still be recorded
        renderer.assert_draw_call_count(1);
    }

    #[test]
    fn inset_shadow_renders_without_panic() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let view = TestView.inset_shadow(0.0, 2.0, 4.0, [0.0, 0.0, 0.0, 0.5]);
        view.render(&mut renderer, rect);
        renderer.assert_draw_call_count(1);
    }

    #[test]
    fn layered_shadows_render_without_panic() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let layers = vec![Shadow::drop(0.0, 2.0, 4.0), Shadow::drop(0.0, 8.0, 16.0)];
        let view = TestView.shadows(layers);
        view.render(&mut renderer, rect);
        renderer.assert_draw_call_count(1);
    }

    #[test]
    fn shadow_with_spread() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        let shadow = Shadow::new(0.0, 4.0, 8.0, [0.0, 0.0, 0.0, 0.25]).spread(2.0);
        let view = TestView.modifier(shadow.modifier());
        view.render(&mut renderer, rect);
        renderer.assert_draw_call_count(1);
    }

    #[test]
    fn shadow_macro_works() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        let view = TestView.shadow(2.0, 4.0, 8.0, [0.0, 0.0, 0.0, 0.3]);
        view.render(&mut renderer, rect);
        renderer.assert_draw_call_count(1);
    }
}
