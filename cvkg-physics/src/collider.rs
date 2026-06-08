//! Collider: binds a shape to a rigid body with offset and rotation.

use glam::{Quat, Vec2, Vec3};

use crate::BodyId;
use crate::shape::Shape;

/// A collider attaches a shape to a body and provides world-space
/// collision geometry queries.
#[derive(Debug, Clone)]
pub struct Collider {
    /// The body this collider is attached to.
    pub body_id: BodyId,
    /// The collision shape.
    pub shape: Shape,
    /// Local offset from the body's center of mass.
    pub offset: Vec2,
    /// Local rotation offset (radians).
    pub rotation_offset: f32,
    /// Collision category bitmask for filtering.
    pub category: u32,
    /// Bitmask of categories this collider can collide with.
    pub collides_with: u32,
    /// Optional user data pointer (for application callbacks).
    pub user_data: u64,
    /// If true, this collider is a trigger/sensor — it generates collision
    /// events but does not produce contact forces (no physical response).
    /// Use for: pickup zones, kill zones, proximity detection, area triggers.
    pub is_sensor: bool,
}

impl Collider {
    /// Create a new collider with the given shape, attached to `body_id` at the body's center.
    pub fn new(body_id: BodyId, shape: Shape) -> Self {
        Self {
            body_id,
            shape,
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            category: 0x0001,
            collides_with: 0xFFFF,
            user_data: 0,
            is_sensor: false,
        }
    }

    /// Create a new trigger/sensor collider.
    /// Triggers generate collision events but do not produce physical response.
    pub fn new_sensor(body_id: BodyId, shape: Shape) -> Self {
        let mut c = Self::new(body_id, shape);
        c.is_sensor = true;
        c
    }

    /// Set the local offset from the body center.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Set the local rotation offset.
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation_offset = rotation;
        self
    }

    /// Set the collision category bitmask.
    pub fn with_category(mut self, category: u32) -> Self {
        self.category = category;
        self
    }

    /// Set which categories this collider can collide with.
    pub fn with_collides_with(mut self, mask: u32) -> Self {
        self.collides_with = mask;
        self
    }

    /// Set whether this collider is a sensor/trigger.
    pub fn with_sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    /// Check if this collider should collide with another based on category masks.
    pub fn can_collide_with(&self, other: &Collider) -> bool {
        (self.category & other.collides_with) != 0 && (other.category & self.collides_with) != 0
    }

    /// Get the world-space AABB for broad-phase culling.
    /// Requires the body's current position and angle.
    pub fn world_aabb(&self, body_position: Vec2, body_angle: f32) -> (Vec2, Vec2) {
        let total_angle = body_angle + self.rotation_offset;
        let cos = total_angle.cos();
        let sin = total_angle.sin();

        // Transform offset to world space
        let world_offset = Vec2::new(
            cos * self.offset.x - sin * self.offset.y,
            sin * self.offset.x + cos * self.offset.y,
        );
        let center = body_position + world_offset;
        let r = self.shape.bounding_radius();
        let half = Vec2::new(r, r);
        (center - half, center + half)
    }

    /// Get the support point in world space for a given direction.
    pub fn world_support(&self, dir: Vec2, body_position: Vec2, body_angle: f32) -> Vec2 {
        let total_angle = body_angle + self.rotation_offset;
        // Rotate direction into local space
        let cos = total_angle.cos();
        let sin = total_angle.sin();
        let local_dir = Vec2::new(cos * dir.x + sin * dir.y, -sin * dir.x + cos * dir.y);
        let local_support = self.shape.support(local_dir);
        // Transform support point to world space
        let cos = total_angle.cos();
        let sin = total_angle.sin();
        let rotated = Vec2::new(
            cos * local_support.x - sin * local_support.y,
            sin * local_support.x + cos * local_support.y,
        );
        body_position + self.offset + rotated
    }

    /// Get the world-space AABB for 3D broad-phase culling.
    /// Requires the body's current 3D position and rotation.
    pub fn world_aabb_3d(&self, body_position: Vec3, body_rotation: Quat) -> (Vec3, Vec3) {
        let world_offset = body_rotation * self.offset.extend(0.0);
        let center = body_position + world_offset;
        let r = self.shape.bounding_radius();
        let half = Vec3::new(r, r, r);
        (center - half, center + half)
    }
}
