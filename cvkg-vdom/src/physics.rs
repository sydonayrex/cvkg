//! Physics-driven reactivity for Taffy synchronization.
//!
//! Provides the `Spring` struct which ties Taffy target layout bounds
//! to actual drawn layout bounds via signals.

use crate::signals::Signal;
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
    pub fn tick(&self, dt: f32) {
        let target = self.target.get();
        let current = self.current.get();
        let mut vel = self.velocity.write().unwrap();

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

        // Mutate the signal, which synchronously fires effects
        self.current.set(next_bounds);
    }
}
