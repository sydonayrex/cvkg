//! Core-local descriptor for a 3D world-space physics body.
//!
//! Components configure this on `WorldSpaceConfig`; `cvkg-vdom` converts it to a
//! concrete `cvkg_physics::RigidBody3D` at the renderer boundary. Keeping the
//! descriptor in `cvkg-core` avoids a `cvkg-core <-> cvkg-physics` dependency
//! cycle while still letting the whole pipeline carry physics intent.

use crate::Transform3D;

/// Physics description for a world-space panel.
///
/// This is deliberately framework-agnostic: it captures the *intent* (mass,
/// initial position/velocity, gravity response, staticness) without binding to
/// any particular solver. The GPU/CPU renderer lower it into a real
/// `cvkg_physics::RigidBody3D` when building the simulation world.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBody {
    /// Mass in arbitrary units. `0.0` (or `is_static = true`) marks a static body.
    pub mass: f32,
    /// Initial world position. Authoritative when `is_3d` is true.
    pub position: glam::Vec3,
    /// Initial linear velocity.
    pub velocity: glam::Vec3,
    /// Multiplier on the world gravity applied to this body (1.0 = full).
    pub gravity_scale: f32,
    /// When true the body never moves (e.g. a fixed panel anchor).
    pub is_static: bool,
    /// Restitution coefficient [0, 1] used on collision response.
    pub restitution: f32,
    /// Linear damping per second.
    pub linear_damping: f32,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            position: glam::Vec3::ZERO,
            velocity: glam::Vec3::ZERO,
            gravity_scale: 1.0,
            is_static: false,
            restitution: 0.2,
            linear_damping: 0.1,
        }
    }
}

impl PhysicsBody {
    /// Build a body pinned at `transform`'s translation, defaulting to dynamic
    /// with full gravity. Convenient for panels that should drop/settle in 3D.
    pub fn at(transform: &Transform3D) -> Self {
        Self {
            position: transform.position,
            ..Default::default()
        }
    }
}
