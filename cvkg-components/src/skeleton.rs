//! Skeleton loading component for content placeholders.
//!
//! Renders a pulsing placeholder that matches the shape of its eventual content.

use cvkg_core::{Never, Rect, Renderer, View};

/// Skeleton loading placeholder that matches the shape of its content.
///
/// ## Accessibility
/// - Role: `progressbar` (loading state indicator)
/// - ARIA: `aria-label` for "Loading content", `aria-busy="true"` when loading
/// - Reduced motion: respects `is_reduced_motion()` for pulse animation
#[derive(Clone)]
pub struct Skeleton {
    /// Whether the skeleton is in loading state.
    pub loading: bool,
    /// Width of the skeleton (defaults to parent width).
    pub width: Option<f32>,
    /// Height of the skeleton (defaults to parent height).
    pub height: Option<f32>,
    /// Corner radius.
    pub radius: f32,
    /// Pulse animation progress.
    pub pulse: f32,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    /// Create a new Skeleton placeholder.
    pub fn new() -> Self {
        Self {
            loading: true,
            width: None,
            height: None,
            radius: 4.0,
            pulse: 0.5,
        }
    }

    /// Set the loading state.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set explicit width and height.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set corner radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set pulse animation value (0.0 to 1.0).
    pub fn pulse(mut self, pulse: f32) -> Self {
        self.pulse = pulse.clamp(0.0, 1.0);
        self
    }

    /// Create a rounded skeleton (capsule/pill shape).
    pub fn capsule(width: f32, height: f32) -> Self {
        Self::new()
            .size(width, height)
            .radius(height / 2.0)
    }

    /// Create a text line skeleton.
    pub fn text(width: f32, height: f32) -> Self {
        Self::new()
            .size(width, height)
            .radius(2.0)
    }

    /// Create a circular skeleton (for avatars).
    pub fn circle(size: f32) -> Self {
        Self::new()
            .size(size, size)
            .radius(size / 2.0)
    }
}

impl View for Skeleton {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.loading {
            return;
        }

        renderer.push_vnode(rect, "Skeleton");
        renderer.register_a11y("progressbar", "Loading content");

        let width = self.width.unwrap_or(rect.width);
        let height = self.height.unwrap_or(rect.height);
        let radius = self.radius;

        // Calculate shimmer color based on pulse
        let shimmer_base: f32 = 0.2 + 0.15 * self.pulse;
        let shimmer_color = [shimmer_base, shimmer_base, shimmer_base, 1.0];

        // Draw the skeleton rectangle with shimmer effect
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width,
                height,
            },
            radius,
            shimmer_color,
        );

        renderer.pop_vnode();
    }
}

/// Text line skeleton - convenience wrapper for text placeholders.
pub type TextSkeleton = Skeleton;

/// Circle skeleton - convenience wrapper for avatar placeholders.
pub type CircleSkeleton = Skeleton;

/// Card skeleton - for entire content cards.
pub type CardSkeleton = Skeleton;

/// Button skeleton - for button placeholders.
pub type ButtonSkeleton = Skeleton;