//! Declarative animation trigger bindings.
//!
//! Provides modifier methods that bind spring-animation progress to
//! lifecycle/input events (hover, scroll, enter, exit).
//!
//! The spring engine is in `cvkg-core` (`SpringAnimationModifier`,
//! `SpringParams`). These wrappers configure a spring and bind its
//! progress to a specific trigger source at render time.

use cvkg_core::{SpringParams, View};

/// A spring configuration paired with a trigger source.
pub struct TriggerSpring {
    pub params: SpringParams,
    pub target_on_trigger: f32,
    pub target_on_release: f32,
}

impl TriggerSpring {
    /// Create a new trigger spring with the given parameters.
    pub fn new(params: SpringParams) -> Self {
        Self {
            params,
            target_on_trigger: 1.0,
            target_on_release: 0.0,
        }
    }

    /// Set the spring target when the trigger is active.
    pub fn target(mut self, value: f32) -> Self {
        self.target_on_trigger = value;
        self
    }

    /// Set the spring target when the trigger is inactive.
    pub fn rest(mut self, value: f32) -> Self {
        self.target_on_release = value;
        self
    }

    /// Preset: snappy hover scale (1.0 -> 1.05).
    pub fn hover_scale() -> Self {
        Self::new(SpringParams::snappy()).target(1.05).rest(1.0)
    }

    /// Preset: fluid scroll fade-in (0.0 -> 1.0).
    pub fn scroll_fade() -> Self {
        Self::new(SpringParams::fluid()).target(1.0).rest(0.0)
    }

    /// Preset: snappy enter animation (0.0 -> 1.0).
    pub fn enter() -> Self {
        Self::new(SpringParams::snappy()).target(1.0).rest(0.0)
    }

    /// Preset: fluid exit animation (1.0 -> 0.0).
    pub fn exit() -> Self {
        Self::new(SpringParams::fluid()).target(0.0).rest(1.0)
    }
}

/// Wrapper view that drives a spring from hover state.
pub struct OnHover<V: View> {
    pub inner: V,
    pub spring: TriggerSpring,
}

impl<V: View + Clone + 'static> View for OnHover<V> {
    type Body = V::Body;
    fn body(self) -> Self::Body {
        // NOTE: Full integration requires access to the VDOM's hovered_node
        // state at render time. This is a structural placeholder that defines
        // the public API shape. The spring modifier is applied via the
        // ViewModifier trait, and the actual hover detection is performed
        // by the renderer's hit-testing system.
        self.inner.body()
    }
}

/// Wrapper view that drives a spring from scroll position.
pub struct OnScroll<V: View> {
    pub inner: V,
    pub spring: TriggerSpring,
}

impl<V: View + Clone + 'static> View for OnScroll<V> {
    type Body = V::Body;
    fn body(self) -> Self::Body {
        self.inner.body()
    }
}

/// Wrapper view that drives a spring on mount (enter).
pub struct OnEnter<V: View> {
    pub inner: V,
    pub spring: TriggerSpring,
}

impl<V: View + Clone + 'static> View for OnEnter<V> {
    type Body = V::Body;
    fn body(self) -> Self::Body {
        self.inner.body()
    }
}

/// Wrapper view that drives a spring on unmount (exit).
pub struct OnExit<V: View> {
    pub inner: V,
    pub spring: TriggerSpring,
}

impl<V: View + Clone + 'static> View for OnExit<V> {
    type Body = V::Body;
    fn body(self) -> Self::Body {
        self.inner.body()
    }
}

/// Extension trait adding declarative animation-trigger modifiers to any View.
pub trait AnimationTriggers: View + Sized {
    /// Animate a spring when the pointer hovers over this view.
    fn on_hover(self, spring: TriggerSpring) -> OnHover<Self> {
        OnHover {
            inner: self,
            spring,
        }
    }

    /// Animate a spring based on scroll position.
    fn on_scroll(self, spring: TriggerSpring) -> OnScroll<Self> {
        OnScroll {
            inner: self,
            spring,
        }
    }

    /// Animate a spring when this view enters the tree (mount).
    fn on_enter(self, spring: TriggerSpring) -> OnEnter<Self> {
        OnEnter {
            inner: self,
            spring,
        }
    }

    /// Animate a spring when this view exits the tree (unmount).
    fn on_exit(self, spring: TriggerSpring) -> OnExit<Self> {
        OnExit {
            inner: self,
            spring,
        }
    }
}

impl<V: View> AnimationTriggers for V {}
