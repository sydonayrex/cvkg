//! Motion preset library for common animations.
//!
//! Provides pre-tuned animation configurations for fade, slide, scale, and bounce effects.

use cvkg_core::{Rect, Renderer};

/// Motion preset types for common transition animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPreset {
    /// Fade in: opacity 0→1, 300ms, ease-out
    FadeIn,
    /// Fade out: opacity 1→0, 200ms, ease-in
    FadeOut,
    /// Slide up: translateY 20→0, 350ms, spring(fluid)
    SlideUp,
    /// Slide down: translateY -20→0, 350ms, spring(fluid)
    SlideDown,
    /// Slide left: translateX 20→0, 350ms, spring(fluid)
    SlideLeft,
    /// Slide right: translateX -20→0, 350ms, spring(fluid)
    SlideRight,
    /// Scale in: scale 0.8→1.0, 300ms, spring(snappy)
    ScaleIn,
    /// Scale out: scale 1.0→0.8, 200ms, spring(snappy)
    ScaleOut,
    /// Bounce in: scale 0→1, 500ms, spring(bouncy)
    BounceIn,
    /// Rotate in: rotate -15°→0, 400ms, spring(snappy)
    RotateIn,
    /// Rotate out: rotate 0→-15°, 200ms, spring(snappy)
    RotateOut,
}

impl MotionPreset {
    /// Get the spring parameters for this motion preset.
    pub fn spring_params(&self) -> cvkg_anim::SpringParams {
        match self {
            Self::FadeIn | Self::FadeOut => cvkg_anim::SpringParams::fluid(),
            Self::SlideUp | Self::SlideDown | Self::SlideLeft | Self::SlideRight => {
                cvkg_anim::SpringParams::fluid()
            }
            Self::ScaleIn | Self::ScaleOut => cvkg_anim::SpringParams::snappy(),
            Self::BounceIn => cvkg_anim::SpringParams::bouncy(),
            Self::RotateIn | Self::RotateOut => cvkg_anim::SpringParams::snappy(),
        }
    }

    /// Get the target value for the animation (1.0 = completed, 0.0 = initial).
    pub fn target_value(&self) -> f32 {
        match self {
            Self::FadeOut | Self::SlideDown | Self::SlideRight | Self::ScaleOut | Self::RotateOut => 0.0,
            _ => 1.0,
        }
    }
}

/// Animated wrapper that applies a motion preset to any view.
#[derive(Clone)]
pub struct Motion<V> {
    /// The inner view to animate.
    pub view: V,
    /// The motion preset to apply.
    pub preset: MotionPreset,
    /// Current animation progress (0.0 to 1.0).
    pub progress: f32,
    /// Whether the animation is currently active.
    pub is_active: bool,
}

impl<V: Clone + cvkg_core::View> Motion<V> {
    /// Create a new Motion-wrapped view.
    pub fn new(view: V, preset: MotionPreset) -> Self {
        Self {
            view,
            preset,
            progress: if preset.target_value() == 1.0 { 0.0 } else { 1.0 },
            is_active: false,
        }
    }

    /// Set the animation progress manually.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Start the animation.
    pub fn play(mut self) -> Self {
        self.is_active = true;
        self
    }

    /// Stop the animation.
    pub fn stop(mut self) -> Self {
        self.is_active = false;
        self
    }
}

impl<V: cvkg_core::View> cvkg_core::View for Motion<V> {
    type Body = cvkg_core::Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let p = self.progress;
        let alpha = match self.preset {
            MotionPreset::FadeIn | MotionPreset::FadeOut => p,
            _ => 1.0,
        };

        let offset_x = match self.preset {
            MotionPreset::SlideLeft => (1.0 - p) * 20.0,
            MotionPreset::SlideRight => (1.0 - p) * (-20.0),
            _ => 0.0,
        };

        let offset_y = match self.preset {
            MotionPreset::SlideUp => (1.0 - p) * 20.0,
            MotionPreset::SlideDown => (1.0 - p) * (-20.0),
            _ => 0.0,
        };

        let scale = match self.preset {
            MotionPreset::ScaleIn => 0.8 + 0.2 * p,
            MotionPreset::ScaleOut => 1.0 - 0.2 * p,
            MotionPreset::BounceIn => {
                // Overshoot bounce effect
                let base = 0.5 + 0.5 * p;
                if p < 1.0 {
                    base + 0.1 * (1.0 - (2.0 * std::f32::consts::PI * p).cos())
                } else {
                    1.0
                }
            }
            _ => 1.0,
        };

        // Apply transform
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        
        let transform_rect = Rect {
            x: center_x + offset_x - rect.width * scale / 2.0,
            y: center_y + offset_y - rect.height * scale / 2.0,
            width: rect.width * scale,
            height: rect.height * scale,
        };

        renderer.push_vnode(transform_rect, "Motion");
        renderer.set_key(&format!("{:?}", self.preset));
        
        // Push opacity
        if alpha < 1.0 {
            renderer.push_opacity(alpha);
        }
        
        // Render inner view
        self.view.render(renderer, transform_rect);
        
        if alpha < 1.0 {
            renderer.pop_opacity();
        }
        renderer.pop_vnode();
    }
}

// === Convenience constructors ===

impl<V: Clone + cvkg_core::View> Motion<V> {
    /// Create a fade-in animation.
    pub fn fade_in(view: V) -> Self {
        Self::new(view, MotionPreset::FadeIn)
    }

    /// Create a fade-out animation.
    pub fn fade_out(view: V) -> Self {
        Self::new(view, MotionPreset::FadeOut)
    }

    /// Create a slide-up animation.
    pub fn slide_up(view: V) -> Self {
        Self::new(view, MotionPreset::SlideUp)
    }

    /// Create a slide-down animation.
    pub fn slide_down(view: V) -> Self {
        Self::new(view, MotionPreset::SlideDown)
    }

    /// Create a scale-in animation.
    pub fn scale_in(view: V) -> Self {
        Self::new(view, MotionPreset::ScaleIn)
    }

    /// Create a bounce-in animation.
    pub fn bounce_in(view: V) -> Self {
        Self::new(view, MotionPreset::BounceIn)
    }
}