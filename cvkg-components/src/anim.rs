//! Animation Combinators — Item 16: Animation System Integration
//!
//! Provides high-level animation combinators that connect SleipnirSolver (RK4 springs)
//! to component state changes. Enables declarative animations without manual state management.
//!
//! # OS-agnostic
//! Pure math (RK4 integration). Works on all platforms.
//!
//! # Usage
//! ```ignore
//! use cvkg_components::anim::{with_animation, Transition, StaggerConfig};
//!
//! // Animate a view with spring physics
//! my_view.modifier(with_animation(0x1234).translate_x(100.0).scale(1.2))
//!
//! // Use predefined transitions
//! my_view.transition(Transition::Spring(SleipnirParams::snappy()))
//! ```

use cvkg_core::{
    load_system_state, update_system_state, ModifiedView, Never, Rect, Renderer, View,
    ViewModifier,
};
use cvkg_anim::{SleipnirParams, SleipnirSolver};

/// Wraps a view with spring-animated transitions for position and scale.
///
/// When the target values change, all animated properties smoothly interpolate
/// using spring physics rather than linear easing.
pub struct AnimatedModifier {
    /// Unique hash for this animation's state in the system state map.
    pub state_hash: u64,
    /// Spring parameters controlling the animation feel.
    pub params: SleipnirParams,
    /// Target translation X. None = don't animate x.
    pub target_translate_x: Option<f32>,
    /// Target translation Y. None = don't animate y.
    pub target_translate_y: Option<f32>,
    /// Target scale. None = don't animate scale.
    pub target_scale: Option<f32>,
}

impl AnimatedModifier {
    /// Create a new animated modifier with the given state hash and spring params.
    pub fn new(state_hash: u64, params: SleipnirParams) -> Self {
        Self {
            state_hash,
            params,
            target_translate_x: None,
            target_translate_y: None,
            target_scale: None,
        }
    }

    /// Animate horizontal translation.
    pub fn translate_x(mut self, value: f32) -> Self {
        self.target_translate_x = Some(value);
        self
    }

    /// Animate vertical translation.
    pub fn translate_y(mut self, value: f32) -> Self {
        self.target_translate_y = Some(value);
        self
    }

    /// Animate uniform scale.
    pub fn scale(mut self, value: f32) -> Self {
        self.target_scale = Some(value.max(0.0));
        self
    }

    /// Animate all properties at once (convenience).
    pub fn all(mut self, tx: f32, ty: f32, scale: f32) -> Self {
        self.target_translate_x = Some(tx);
        self.target_translate_y = Some(ty);
        self.target_scale = Some(scale.max(0.0));
        self
    }
}

impl ViewModifier for AnimatedModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let dt = renderer.delta_time();

        // Initialize solvers if needed
        let needs_init = {
            let state = load_system_state();
            state.get_component_state::<SleipnirSolver>(self.state_hash).is_none()
        };

        if needs_init {
            update_system_state(|s| {
                let mut ns = s.clone();
                if let Some(target) = self.target_translate_x {
                    ns.set_component_state(
                        self.state_hash,
                        SleipnirSolver::new(self.params, target, 0.0),
                    );
                }
                if let Some(target) = self.target_translate_y {
                    ns.set_component_state(
                        self.state_hash + 1,
                        SleipnirSolver::new(self.params, target, 0.0),
                    );
                }
                if let Some(target) = self.target_scale {
                    ns.set_component_state(
                        self.state_hash + 2,
                        SleipnirSolver::new(self.params, target, 1.0),
                    );
                }
                ns
            });
        }

        // Tick solvers and collect values
        let (tx, ty, scale, is_moving) = update_system_state(|s| {
            let mut ns = s.clone();
            let mut tx = 0.0f32;
            let mut ty = 0.0f32;
            let mut scale = 1.0f32;
            let mut moving = false;

            for (i, target) in [
                self.target_translate_x,
                self.target_translate_y,
                self.target_scale,
            ]
            .iter()
            .enumerate()
            {
                if target.is_none() {
                    continue;
                }
                let hash = self.state_hash + i as u64;
                if let Some(solver) = ns.get_component_state::<SleipnirSolver>(hash) {
                    if let Ok(guard) = solver.read() {
                        let mut solver = *guard;
                        let val = solver.tick(dt);
                        if !solver.is_settled() {
                            moving = true;
                        }
                        if !solver.is_settled() {
                            moving = true;
                        }
                        match i {
                            0 => tx = val,
                            1 => ty = val,
                            2 => scale = val,
                            _ => {}
                        }
                        ns.set_component_state(hash, solver);
                    }
                }
            }
            (tx, ty, scale, moving, ns)
        });

        // Apply transform and render
        renderer.push_vnode(rect, "Animated");
        renderer.push_transform([tx, ty], [scale, scale], 0.0);
        view.render(renderer, rect);
        renderer.pop_transform();
        renderer.pop_vnode();

        // Request redraw if still animating
        if is_moving {
            renderer.request_redraw();
        }
    }
}

/// Staggered animation config for list items.
#[derive(Clone, Debug)]
pub struct StaggerConfig {
    /// Delay in seconds between each item's animation start.
    pub delay_per_item: f32,
    /// Direction items animate from.
    pub direction: StaggerDirection,
    /// Spring parameters.
    pub params: SleipnirParams,
}

impl Default for StaggerConfig {
    fn default() -> Self {
        Self {
            delay_per_item: 0.05,
            direction: StaggerDirection::Bottom,
            params: SleipnirParams::snappy(),
        }
    }
}

/// Direction for staggered animations.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum StaggerDirection {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

/// Transition types for view appearance/disappearance.
#[derive(Clone, Debug)]
pub enum Transition {
    None,
    Fade { duration: f32 },
    Slide { duration: f32, direction: StaggerDirection },
    Scale { duration: f32 },
    Spring(SleipnirParams),
}

impl Transition {
    /// Convert to an AnimatedModifier for a given state hash.
    pub fn to_modifier(&self, state_hash: u64, appearing: bool) -> Option<AnimatedModifier> {
        match self {
            Transition::None => None,
            Transition::Slide { direction, .. } => {
                let (tx, ty) = match direction {
                    StaggerDirection::Left => (-100.0, 0.0),
                    StaggerDirection::Right => (100.0, 0.0),
                    StaggerDirection::Top => (0.0, -100.0),
                    StaggerDirection::Bottom => (0.0, 100.0),
                    StaggerDirection::Center => (0.0, 0.0),
                };
                let m = AnimatedModifier::new(state_hash, SleipnirParams::fluid())
                    .translate_x(if appearing { 0.0 } else { -tx })
                    .translate_y(if appearing { 0.0 } else { -ty });
                Some(m)
            }
            Transition::Scale { .. } => {
                let m = AnimatedModifier::new(state_hash, SleipnirParams::fluid())
                    .scale(if appearing { 1.0 } else { 0.8 });
                Some(m)
            }
            Transition::Spring(params) => {
                let m = AnimatedModifier::new(state_hash, *params)
                    .scale(if appearing { 1.0 } else { 0.9 });
                Some(m)
            }
            _ => None,
        }
    }
}

/// Convenience: create a snappy spring animation modifier.
pub fn with_animation(state_hash: u64) -> AnimatedModifier {
    AnimatedModifier::new(state_hash, SleipnirParams::snappy())
}

/// Convenience: create a bouncy spring animation modifier.
pub fn with_bouncy(state_hash: u64) -> AnimatedModifier {
    AnimatedModifier::new(state_hash, SleipnirParams::bouncy())
}

/// Convenience: create a fluid spring animation modifier.
pub fn with_fluid(state_hash: u64) -> AnimatedModifier {
    AnimatedModifier::new(state_hash, SleipnirParams::fluid())
}
