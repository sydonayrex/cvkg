//! Transitions and animated views for CVKG.
//!
//! Provides declarative transition effects (fade, slide, scale, slide-fade)
//! with configurable easing and duration, plus a staggered-list container
//! that staggers child animation start times.

use cvkg_core::{Never, Rect, Renderer, View};

// ══════════════════════════════════════════════════════════════════════════════
// Transition types
// ══════════════════════════════════════════════════════════════════════════════

/// Direction for slide-based transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

/// The transition effect to apply when a view enters or exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Fade the view in or out by modulating opacity.
    Fade,
    /// Slide the view in from the given direction.
    Slide(SlideDirection),
    /// Scale the view up or down from/to the given direction.
    Scale,
    /// Combined slide and fade.
    SlideFade(SlideDirection),
}

/// Easing curves for animation interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

// ══════════════════════════════════════════════════════════════════════════════
// Easing functions
// ══════════════════════════════════════════════════════════════════════════════

/// Linear interpolation: no easing.
pub fn linear(t: f32) -> f32 {
    t
}

/// Quadratic ease-in: starts slow, accelerates.
pub fn ease_in(t: f32) -> f32 {
    t * t
}

/// Quadratic ease-out: starts fast, decelerates.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Smoothstep ease-in-out: slow start and end, fast middle.
pub fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Evaluate the given easing function for a normalised time `t` in [0, 1].
pub fn eval_easing(easing: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => linear(t),
        Easing::EaseIn => ease_in(t),
        Easing::EaseOut => ease_out(t),
        Easing::EaseInOut => ease_in_out(t),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Animated<V>
// ══════════════════════════════════════════════════════════════════════════════

/// A view wrapper that applies a looping transition animation to its content.
///
/// The animation progress is derived from `renderer.elapsed_time()` modulo
/// `duration`, producing a continuously looping effect.
#[derive(Debug, Clone)]
pub struct Animated<V: View> {
    /// The child view to animate.
    pub content: V,
    /// The transition effect.
    pub transition: Transition,
    /// The easing curve.
    pub easing: Easing,
    /// Duration of one animation loop in seconds.
    pub duration: f32,
}

impl<V: View> Animated<V> {
    /// Wrap `content` with the given `transition` using default easing
    /// (EaseInOut) and duration (0.3 s).
    pub fn new(content: V, transition: Transition) -> Self {
        Self {
            content,
            transition,
            easing: Easing::EaseInOut,
            duration: 0.3,
        }
    }

    /// Set the easing curve.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set the animation loop duration in seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration.max(0.01);
        self
    }
}

impl<V: View> View for Animated<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Animated does not have a body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Compute looping progress from elapsed time.
        let elapsed = renderer.elapsed_time();
        let duration = self.duration.max(0.01);
        let raw = (elapsed % duration) / duration;
        let t = eval_easing(self.easing, raw);

        match self.transition {
            Transition::Fade => {
                renderer.push_opacity(t);
                self.content.render(renderer, rect);
                renderer.pop_opacity();
            }
            Transition::Slide(direction) => {
                let (tx, ty) = match direction {
                    SlideDirection::Up => (0.0, -rect.height * (1.0 - t)),
                    SlideDirection::Down => (0.0, rect.height * (1.0 - t)),
                    SlideDirection::Left => (-rect.width * (1.0 - t), 0.0),
                    SlideDirection::Right => (rect.width * (1.0 - t), 0.0),
                };
                renderer.push_transform([tx, ty], [1.0, 1.0], 0.0);
                self.content.render(renderer, rect);
                renderer.pop_transform();
            }
            Transition::Scale => {
                let s = t;
                renderer.push_transform([0.0, 0.0], [s, s], 0.0);
                self.content.render(renderer, rect);
                renderer.pop_transform();
            }
            Transition::SlideFade(direction) => {
                let (tx, ty) = match direction {
                    SlideDirection::Up => (0.0, -rect.height * (1.0 - t)),
                    SlideDirection::Down => (0.0, rect.height * (1.0 - t)),
                    SlideDirection::Left => (-rect.width * (1.0 - t), 0.0),
                    SlideDirection::Right => (rect.width * (1.0 - t), 0.0),
                };
                renderer.push_opacity(t);
                renderer.push_transform([tx, ty], [1.0, 1.0], 0.0);
                self.content.render(renderer, rect);
                renderer.pop_transform();
                renderer.pop_opacity();
            }
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// StaggeredList<V>
// ══════════════════════════════════════════════════════════════════════════════

/// A container that wraps each child in an `Animated` view with an
/// increasing delay so that children appear one after another.
///
/// Each child is delayed by `stagger_delay * index` seconds relative to the
/// previous child.  The base transition, easing, and duration are shared.
#[derive(Debug, Clone)]
pub struct StaggeredList<V: View> {
    /// The child views.
    pub children: Vec<V>,
    /// The transition applied to every child.
    pub transition: Transition,
    /// The easing curve.
    pub easing: Easing,
    /// Duration of each child's animation loop in seconds.
    pub duration: f32,
    /// Additional delay per child index in seconds.
    pub stagger_delay: f32,
}

impl<V: View> StaggeredList<V> {
    /// Create a new staggered list with the given children and transition.
    /// Uses default easing (EaseInOut), duration (0.3 s), and stagger delay (0.05 s).
    pub fn new(children: Vec<V>, transition: Transition) -> Self {
        Self {
            children,
            transition,
            easing: Easing::EaseInOut,
            duration: 0.3,
            stagger_delay: 0.05,
        }
    }

    /// Set the easing curve.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set the per-child animation duration in seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration.max(0.01);
        self
    }

    /// Set the stagger delay between children in seconds.
    pub fn stagger_delay(mut self, delay: f32) -> Self {
        self.stagger_delay = delay.max(0.0);
        self
    }
}

impl<V: View> View for StaggeredList<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("StaggeredList does not have a body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let count = self.children.len().max(1);
        let child_height = rect.height / count as f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_rect = Rect {
                x: rect.x,
                y: rect.y + i as f32 * child_height,
                width: rect.width,
                height: child_height,
            };

            // Compute per-child progress with stagger offset.
            let elapsed = renderer.elapsed_time();
            let duration = self.duration.max(0.01);
            let offset = i as f32 * self.stagger_delay;
            let raw = ((elapsed + offset) % duration) / duration;
            let t = eval_easing(self.easing, raw);

            match self.transition {
                Transition::Fade => {
                    renderer.push_opacity(t);
                    child.render(renderer, child_rect);
                    renderer.pop_opacity();
                }
                Transition::Slide(direction) => {
                    let (tx, ty) = match direction {
                        SlideDirection::Up => (0.0, -child_rect.height * (1.0 - t)),
                        SlideDirection::Down => (0.0, child_rect.height * (1.0 - t)),
                        SlideDirection::Left => (-child_rect.width * (1.0 - t), 0.0),
                        SlideDirection::Right => (child_rect.width * (1.0 - t), 0.0),
                    };
                    renderer.push_transform([tx, ty], [1.0, 1.0], 0.0);
                    child.render(renderer, child_rect);
                    renderer.pop_transform();
                }
                Transition::Scale => {
                    let s = t;
                    renderer.push_transform([0.0, 0.0], [s, s], 0.0);
                    child.render(renderer, child_rect);
                    renderer.pop_transform();
                }
                Transition::SlideFade(direction) => {
                    let (tx, ty) = match direction {
                        SlideDirection::Up => (0.0, -child_rect.height * (1.0 - t)),
                        SlideDirection::Down => (0.0, child_rect.height * (1.0 - t)),
                        SlideDirection::Left => (-child_rect.width * (1.0 - t), 0.0),
                        SlideDirection::Right => (child_rect.width * (1.0 - t), 0.0),
                    };
                    renderer.push_opacity(t);
                    renderer.push_transform([tx, ty], [1.0, 1.0], 0.0);
                    child.render(renderer, child_rect);
                    renderer.pop_transform();
                    renderer.pop_opacity();
                }
            }
        }
    }
}
