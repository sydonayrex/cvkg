//! Rigid body definition and properties.
//!
//! Supports both 2D and 3D simulation modes. When `is_3d` is false (default),
//! the 2D fields (position: Vec2, angle: f32, etc.) are used. When `is_3d` is
//! true, the 3D fields (position_3d: Vec3, rotation: Quat, etc.) take precedence.

use glam::{Quat, Vec2, Vec3};

use crate::shape::Shape;

/// Unique identifier for a rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(pub u64);

/// Rigid body state.
///
/// This struct carries both 2D and 3D fields. The `is_3d` flag selects which
/// set of fields the simulation pipeline uses. 2D fields are the default and
/// are always kept in sync when operating in 2D mode.
#[derive(Debug, Clone)]
pub struct RigidBody {
    // ── 2D fields (default mode) ──────────────────────────────────────────
    /// Position in world space (pixels). 2D mode: XY; 3D mode: read from position_3d.
    pub position: Vec2,
    /// Linear velocity (pixels/second). 2D mode.
    pub velocity: Vec2,
    /// Accumulated force for this frame (cleared after integration). 2D mode.
    pub force: Vec2,
    /// Orientation angle in radians (clockwise positive). 2D mode only.
    pub angle: f32,
    /// Angular velocity in radians/second. 2D mode only.
    pub angular_velocity: f32,
    /// Accumulated torque for this frame. 2D mode only.
    pub torque: f32,

    // ── 3D fields (used when is_3d == true) ──────────────────────────────
    /// Position in 3D world space. When `is_3d` is true this is authoritative.
    pub position_3d: Vec3,
    /// Linear velocity in 3D.
    pub velocity_3d: Vec3,
    /// Accumulated force in 3D.
    pub force_3d: Vec3,
    /// Orientation as a unit quaternion. Identity = no rotation.
    pub rotation: Quat,
    /// Angular velocity as a 3D vector (rad/s around each axis).
    pub angular_velocity_3d: Vec3,
    /// Accumulated torque as a 3D vector.
    pub torque_3d: Vec3,
    /// Inverse inertia as a 3D diagonal tensor (for principal axes).
    pub inv_inertia_3d: Vec3,

    // ── Shared fields ─────────────────────────────────────────────────────
    /// Mass in arbitrary units. 0 or infinite = static body.
    pub mass: f32,
    /// Inverse mass (1/mass). 0 for static bodies.
    pub inv_mass: f32,
    /// Moment of inertia (2D scalar or trace of 3D inertia tensor).
    pub inertia: f32,
    /// Inverse moment of inertia.
    pub inv_inertia: f32,
    /// Coefficient of restitution (bounciness, 0.0–1.0).
    pub restitution: f32,
    /// Coefficient of friction (0.0–1.0).
    pub friction: f32,
    /// Linear velocity damping (air resistance, 0.0 = none, 1.0 = full stop).
    pub linear_damping: f32,
    /// Angular velocity damping.
    pub angular_damping: f32,
    /// Gravity multiplier. 1.0 = normal gravity, 0.0 = ignore gravity.
    pub gravity_scale: f32,
    /// If true, this body is static (immovable).
    pub is_static: bool,
    /// If true, use 3D fields for simulation; otherwise use 2D fields.
    pub is_3d: bool,
    /// Sleep threshold: body goes to sleep when kinetic energy falls below this.
    pub sleep_threshold: f32,
    /// Whether this body is currently sleeping (skips integration).
    pub is_sleeping: bool,
    /// Number of simulation steps this body has been below sleep threshold.
    pub sleep_counter: u32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            // 2D defaults
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
            torque: 0.0,
            // 3D defaults
            position_3d: Vec3::ZERO,
            velocity_3d: Vec3::ZERO,
            force_3d: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            angular_velocity_3d: Vec3::ZERO,
            torque_3d: Vec3::ZERO,
            inv_inertia_3d: Vec3::ONE,
            // Shared defaults
            mass: 1.0,
            inv_mass: 1.0,
            inertia: 1.0,
            inv_inertia: 1.0,
            restitution: 0.5,
            friction: 0.3,
            linear_damping: 0.01,
            angular_damping: 0.01,
            gravity_scale: 1.0,
            is_static: false,
            is_3d: false,
            sleep_threshold: 0.005,
            is_sleeping: false,
            sleep_counter: 0,
        }
    }
}

impl RigidBody {
    /// Create a new dynamic rigid body with the given mass and shape-derived inertia.
    pub fn new(mass: f32, shape: &Shape) -> Self {
        let inv_mass = if mass > 0.0 && mass.is_finite() {
            1.0 / mass
        } else {
            0.0
        };
        let inertia = if inv_mass > 0.0 {
            shape.moment_of_inertia(mass)
        } else {
            0.0
        };
        let inv_inertia = if inertia > 0.0 { 1.0 / inertia } else { 0.0 };

        Self {
            mass,
            inv_mass,
            inertia,
            inv_inertia,
            is_static: inv_mass == 0.0,
            ..Default::default()
        }
    }

    /// Create a new 3D dynamic rigid body.
    pub fn new_3d(mass: f32, shape: &Shape) -> Self {
        let mut body = Self::new(mass, shape);
        body.is_3d = true;
        // Compute 3D inverse inertia from the shape
        let inertia_3d = shape.moment_of_inertia_3d(mass);
        body.inv_inertia_3d = Vec3::new(
            if inertia_3d.x > 0.0 { 1.0 / inertia_3d.x } else { 0.0 },
            if inertia_3d.y > 0.0 { 1.0 / inertia_3d.y } else { 0.0 },
            if inertia_3d.z > 0.0 { 1.0 / inertia_3d.z } else { 0.0 },
        );
        body
    }

    /// Create a static (immovable) body.
    pub fn static_body() -> Self {
        Self {
            inv_mass: 0.0,
            inv_inertia: 0.0,
            is_static: true,
            ..Default::default()
        }
    }

    /// Create a static 3D body.
    pub fn static_body_3d() -> Self {
        let mut body = Self::static_body();
        body.is_3d = true;
        body.inv_inertia_3d = Vec3::ZERO;
        body
    }

    /// Apply a force at the center of mass.
    pub fn apply_force(&mut self, force: Vec2) {
        if !self.is_static {
            self.force += force;
        }
    }

    /// Apply a 3D force at the center of mass.
    pub fn apply_force_3d(&mut self, force: Vec3) {
        if !self.is_static {
            self.force_3d += force;
        }
    }

    /// Apply a force at a specific world-space point (generates torque).
    pub fn apply_force_at(&mut self, force: Vec2, world_point: Vec2) {
        if self.is_static {
            return;
        }
        self.force += force;
        let r = world_point - self.position;
        self.torque += r.x * force.y - r.y * force.x; // 2D cross product
    }

    /// Apply a 3D force at a specific world-space point (generates torque).
    pub fn apply_force_at_3d(&mut self, force: Vec3, world_point: Vec3) {
        if self.is_static {
            return;
        }
        self.force_3d += force;
        let r = world_point - self.position_3d;
        self.torque_3d += r.cross(force);
    }

    /// Apply an impulse (instantaneous velocity change).
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if !self.is_static {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Apply a 3D impulse.
    pub fn apply_impulse_3d(&mut self, impulse: Vec3) {
        if !self.is_static {
            self.velocity_3d += impulse * self.inv_mass;
        }
    }

    /// Apply an angular impulse.
    pub fn apply_angular_impulse(&mut self, impulse: f32) {
        if !self.is_static {
            self.angular_velocity += impulse * self.inv_inertia;
        }
    }

    /// Apply a 3D angular impulse.
    pub fn apply_angular_impulse_3d(&mut self, impulse: Vec3) {
        if !self.is_static {
            self.angular_velocity_3d += impulse * self.inv_inertia_3d;
        }
    }

    /// Get the kinetic energy of this body (used for sleep detection).
    pub fn kinetic_energy(&self) -> f32 {
        if self.is_3d {
            let linear = self.velocity_3d.length_squared() * self.mass * 0.5;
            let angular = self.angular_velocity_3d.length_squared() * self.inertia * 0.5;
            linear + angular
        } else {
            let linear = self.velocity.length_squared() * self.mass * 0.5;
            let angular = self.angular_velocity * self.angular_velocity * self.inertia * 0.5;
            linear + angular
        }
    }

    /// Get the world-space position of a local point.
    pub fn local_to_world(&self, local: Vec2) -> Vec2 {
        let cos = self.angle.cos();
        let sin = self.angle.sin();
        let rotated = Vec2::new(cos * local.x - sin * local.y, sin * local.x + cos * local.y);
        self.position + rotated
    }

    /// Get the 3D world-space position of a local 3D point.
    pub fn local_to_world_3d(&self, local: Vec3) -> Vec3 {
        self.position_3d + self.rotation * local
    }

    /// Get the local-space position of a world point.
    pub fn world_to_local(&self, world: Vec2) -> Vec2 {
        let delta = world - self.position;
        let cos = self.angle.cos();
        let sin = self.angle.sin();
        Vec2::new(
            cos * delta.x + sin * delta.y,
            -sin * delta.x + cos *delta.y,
        )
    }

    /// Get the local-space 3D position of a 3D world point.
    pub fn world_to_local_3d(&self, world: Vec3) -> Vec3 {
        self.rotation.inverse() * (world - self.position_3d)
    }

    /// Get the rotation matrix as (cos, sin) pair.
    pub fn rotation_cs(&self) -> (f32, f32) {
        (self.angle.cos(), self.angle.sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Shape;

    #[test]
    fn test_dynamic_body() {
        let shape = Shape::circle(1.0);
        let body = RigidBody::new(2.0, &shape);
        assert!(!body.is_static);
        assert_eq!(body.inv_mass, 0.5);
    }

    #[test]
    fn test_static_body() {
        let body = RigidBody::static_body();
        assert!(body.is_static);
        assert_eq!(body.inv_mass, 0.0);
    }

    #[test]
    fn test_apply_impulse() {
        let shape = Shape::circle(1.0);
        let mut body = RigidBody::new(1.0, &shape);
        body.apply_impulse(Vec2::new(10.0, 0.0));
        assert!((body.velocity.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_static_ignores_force() {
        let mut body = RigidBody::static_body();
        body.apply_force(Vec2::new(100.0, 100.0));
        assert_eq!(body.force, Vec2::ZERO);
    }

    #[test]
    fn test_local_to_world() {
        let shape = Shape::circle(1.0);
        let mut body = RigidBody::new(1.0, &shape);
        body.position = Vec2::new(5.0, 5.0);
        body.angle = 0.0;
        let world = body.local_to_world(Vec2::new(1.0, 0.0));
        assert!((world.x - 6.0).abs() < 0.001);
        assert!((world.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_kinetic_energy() {
        let shape = Shape::circle(1.0);
        let mut body = RigidBody::new(2.0, &shape);
        body.velocity = Vec2::new(3.0, 4.0);
        // KE = 0.5 * 2 * 25 = 25
        assert!((body.kinetic_energy() - 25.0).abs() < 0.1);
    }

    // ── 3D body tests ────────────────────────────────────────────────────

    #[test]
    fn test_3d_body_creation() {
        let shape = Shape::sphere(1.0);
        let body = RigidBody::new_3d(2.0, &shape);
        assert!(body.is_3d);
        assert!(!body.is_static);
        assert_eq!(body.inv_mass, 0.5);
        assert_eq!(body.position_3d, Vec3::ZERO);
        assert_eq!(body.rotation, Quat::IDENTITY);
    }

    #[test]
    fn test_3d_static_body() {
        let body = RigidBody::static_body_3d();
        assert!(body.is_3d);
        assert!(body.is_static);
        assert_eq!(body.inv_inertia_3d, Vec3::ZERO);
    }

    #[test]
    fn test_3d_apply_force() {
        let shape = Shape::sphere(1.0);
        let mut body = RigidBody::new_3d(1.0, &shape);
        body.apply_force_3d(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(body.force_3d, Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_3d_apply_impulse() {
        let shape = Shape::sphere(1.0);
        let mut body = RigidBody::new_3d(1.0, &shape);
        body.apply_impulse_3d(Vec3::new(10.0, 0.0, 0.0));
        assert!((body.velocity_3d.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_3d_local_to_world() {
        let shape = Shape::sphere(1.0);
        let mut body = RigidBody::new_3d(1.0, &shape);
        body.position_3d = Vec3::new(5.0, 5.0, 5.0);
        let world = body.local_to_world_3d(Vec3::new(1.0, 0.0, 0.0));
        assert!((world.x - 6.0).abs() < 0.001);
        assert!((world.y - 5.0).abs() < 0.001);
        assert!((world.z - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_3d_kinetic_energy() {
        let shape = Shape::sphere(1.0);
        let mut body = RigidBody::new_3d(2.0, &shape);
        body.velocity_3d = Vec3::new(3.0, 4.0, 0.0);
        // KE = 0.5 * 2 * 25 = 25
        assert!((body.kinetic_energy() - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_3d_apply_force_at() {
        let shape = Shape::sphere(1.0);
        let mut body = RigidBody::new_3d(1.0, &shape);
        // Force at offset point should generate torque
        body.apply_force_at_3d(Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        // r = (1,0,0), F = (0,1,0) => torque = r x F = (0,0,1)
        assert!((body.torque_3d.z - 1.0).abs() < 0.001);
    }
}
