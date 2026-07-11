//! Physics-driven reactivity for Taffy synchronization.
//!
//! Provides the `Spring` struct which ties Taffy target layout bounds
//! to actual drawn layout bounds via signals.

use crate::signals::Signal;
use cvkg_core::DirtyFlags;
use cvkg_core::Rect;
use std::sync::{Arc, RwLock};

/// A simple physics spring that interpolates a `Rect`.
pub struct Spring {
    /// The target layout bounds (usually written by Taffy during VDOM diffing)
    pub target: Signal<Rect>,
    /// The current visual bounds (read by the Renderer/GPU, mutated by the ticker)
    pub current: Signal<Rect>,
    /// Spring stiffness
    pub stiffness: f32,
    /// Spring damping
    pub damping: f32,

    velocity: Arc<RwLock<Rect>>,
}

impl Spring {
    /// Create a new Spring starting at the given bounds.
    pub fn new(initial: Rect, stiffness: f32, damping: f32) -> Self {
        Self {
            target: Signal::new(initial),
            current: Signal::new(initial),
            stiffness,
            damping,
            velocity: Arc::new(RwLock::new(Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            })),
        }
    }

    /// Step the physics simulation by `dt` seconds.
    ///
    /// No-ops once the spring has settled (target reached and velocity ~zero)
    /// so a resting animation does not mark the pipeline dirty every frame.
    pub fn tick(&self, dt: f32) {
        /// Sub-pixel / sub-pixel-per-second thresholds below which the spring
        /// is considered at rest.
        const REST_EPSILON: f32 = 0.01;

        let target = self.target.get();
        let current = self.current.get();
        let mut vel = self.velocity.write().unwrap();

        // At-rest early-out: if we're already on target and barely moving,
        // do nothing (no signal write => no dirty flag => pipeline stays clean).
        let displacement = (target.x - current.x).abs()
            + (target.y - current.y).abs()
            + (target.width - current.width).abs()
            + (target.height - current.height).abs();
        let speed = vel.x.abs() + vel.y.abs() + vel.width.abs() + vel.height.abs();
        if displacement < REST_EPSILON && speed < REST_EPSILON {
            // Snap exactly to target and zero velocity to avoid slow drift.
            if displacement > 0.0 {
                *vel = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                };
                self.current.set_with_flags(target, DirtyFlags::LAYOUT);
            }
            return;
        }

        // Calculate spring forces (Hooke's law + damping)
        let fx = (target.x - current.x) * self.stiffness - vel.x * self.damping;
        let fy = (target.y - current.y) * self.stiffness - vel.y * self.damping;
        let fw = (target.width - current.width) * self.stiffness - vel.width * self.damping;
        let fh = (target.height - current.height) * self.stiffness - vel.height * self.damping;

        // Update velocities
        vel.x += fx * dt;
        vel.y += fy * dt;
        vel.width += fw * dt;
        vel.height += fh * dt;

        // Update positions
        let mut next_bounds = current;
        next_bounds.x += vel.x * dt;
        next_bounds.y += vel.y * dt;
        next_bounds.width += vel.width * dt;
        next_bounds.height += vel.height * dt;

        // Mutate the signal, which synchronously fires effects.
        // A moving spring changes geometry, so LAYOUT (not the whole pipeline)
        // is the correct downstream annotation.
        self.current.set_with_flags(next_bounds, DirtyFlags::LAYOUT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // Phase 5 regression: a `Spring` animates a single node's LOCAL rect.
    // Per the migration plan, no descendant propagation is needed or present —
    // composition happens at read time via `VDom::world_rect`. This test pins
    // that `tick` drives `current` toward `target` and settles (no dirty churn
    // once at rest), proving the local-rect animation path is correct on its own.
    #[test]
    fn spring_animates_local_rect_and_settles() {
        let start = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let spring = Spring::new(start, 120.0, 14.0);

        // Move the target (e.g. a parent moved its child's local offset).
        spring.target.set(start); // ensure same start
        let target = Rect {
            x: 40.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        spring.target.set(target);

        // Before ticking, current == start.
        assert_relative_eq!(spring.current.get().x, 0.0);

        // Advance the simulation; current should approach the new local target.
        for _ in 0..240 {
            spring.tick(1.0 / 60.0);
        }

        let cur = spring.current.get();
        assert_relative_eq!(cur.x, target.x, epsilon = 0.5);
        assert_relative_eq!(cur.y, target.y, epsilon = 0.5);
        assert_relative_eq!(cur.width, target.width, epsilon = 1e-3);
        assert_relative_eq!(cur.height, target.height, epsilon = 1e-3);

        // One more tick should be a no-op (resting spring snaps & stops dirtying).
        let before = spring.current.get();
        spring.tick(1.0 / 60.0);
        let after = spring.current.get();
        assert_relative_eq!(before.x, after.x, epsilon = 1e-6);
    }

    #[test]
    fn spring_resting_is_noop() {
        let r = Rect {
            x: 10.0,
            y: 10.0,
            width: 30.0,
            height: 30.0,
        };
        let spring = Spring::new(r, 120.0, 14.0);
        // Already at rest at `r`; ticking must not move or dirty.
        spring.tick(1.0 / 60.0);
        let cur = spring.current.get();
        assert_relative_eq!(cur.x, 10.0);
        assert_relative_eq!(cur.y, 10.0);
    }
}
