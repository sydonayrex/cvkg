//! Physics world state snapshots for serialization, replay, and undo/redo.
//!
//! `PhysicsSnapshot` captures the complete simulation state at a point in time.
//! Use `PhysicsWorld::snapshot()` to capture and `PhysicsWorld::restore()` to
//! restore. Snapshots are deterministic: restoring and re-stepping produces
//! identical results when used with fixed timestep.

use std::collections::HashMap;

use glam::{Quat, Vec2, Vec3};

use crate::{BodyId, Constraint};

/// Complete snapshot of the physics world state.
///
/// Captures all body transforms, velocities, and constraint states.
/// Can be serialized to bytes for network sync or disk storage.
#[derive(Debug, Clone)]
pub struct PhysicsSnapshot {
    /// Per-body state indexed by BodyId.
    pub bodies: HashMap<BodyId, BodySnapshot>,
    /// All colliders at snapshot time.
    pub colliders: Vec<ColliderSnapshot>,
    /// All constraints at snapshot time.
    pub constraints: Vec<Constraint>,
    /// World config at snapshot time (gravity, substeps, etc.).
    pub config: crate::WorldConfig,
    /// Accumulated time in the fixed-timestep accumulator.
    pub accumulator: f32,
    /// Current simulation tick count.
    pub tick: u64,
}

/// Snapshot of a single rigid body.
#[derive(Debug, Clone)]
pub struct BodySnapshot {
    /// 2D position (valid when is_3d is false).
    pub position: Vec2,
    /// 3D position (valid when is_3d is true).
    pub position_3d: Vec3,
    /// 2D velocity.
    pub velocity: Vec2,
    /// 3D velocity.
    pub velocity_3d: Vec3,
    /// 2D angle (radians).
    pub angle: f32,
    /// 3D rotation quaternion.
    pub rotation: Quat,
    /// 2D angular velocity.
    pub angular_velocity: f32,
    /// 3D angular velocity.
    pub angular_velocity_3d: Vec3,
    /// Inverse mass.
    pub inv_mass: f32,
    /// Inverse inertia (2D scalar).
    pub inv_inertia: f32,
    /// Inverse inertia (3D vector).
    pub inv_inertia_3d: Vec3,
    /// Whether this is a 3D body.
    pub is_3d: bool,
    /// Whether the body is static.
    pub is_static: bool,
    /// Whether the body is sleeping.
    pub is_sleeping: bool,
    /// Linear damping.
    pub linear_damping: f32,
    /// Angular damping.
    pub angular_damping: f32,
    /// Restitution (bounciness).
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Gravity scale.
    pub gravity_scale: f32,
    /// Sleep threshold.
    pub sleep_threshold: f32,
    /// Collision category bitmask.
    pub category: u32,
    /// Collision filter bitmask.
    pub collides_with: u32,
}

/// Snapshot of a collider.
#[derive(Debug, Clone)]
pub struct ColliderSnapshot {
    /// The body this collider is attached to.
    pub body_id: BodyId,
    /// The shape kind.
    pub shape_kind: ShapeSnapshot,
    /// Local offset from body center.
    pub offset: Vec2,
    /// Local rotation offset (2D).
    pub rotation_offset: f32,
    /// Whether this is a sensor/trigger.
    pub is_sensor: bool,
    /// Collision category.
    pub category: u32,
    /// Collision filter mask.
    pub collides_with: u32,
}

/// Serializable shape description.
#[derive(Debug, Clone)]
pub enum ShapeSnapshot {
    Circle {
        radius: f32,
    },
    Aabb {
        half_extents: Vec2,
    },
    ConvexHull {
        vertices: Vec<Vec2>,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
    Sphere {
        radius: f32,
    },
    Box3D {
        half_extents: Vec3,
    },
    Capsule3D {
        half_height: f32,
        radius: f32,
    },
    Compound2D {
        children: Vec<(Vec2, Box<ShapeSnapshot>)>,
    },
    Compound3D {
        children: Vec<(Vec3, Box<ShapeSnapshot>)>,
    },
    Heightmap {
        heights: Vec<f32>,
        width: usize,
        depth: usize,
        world_size: Vec2,
    },
}

impl PhysicsSnapshot {
    /// Estimate the serialized size in bytes.
    pub fn estimated_size(&self) -> usize {
        let body_size = self.bodies.len() * 128;
        let collider_size = self.colliders.len() * 64;
        let constraint_size = self.constraints.len() * 48;
        64 + body_size + collider_size + constraint_size
    }

    /// Validate that the snapshot is internally consistent.
    pub fn validate(&self) -> Result<(), SnapshotError> {
        for collider in &self.colliders {
            if !self.bodies.contains_key(&collider.body_id) {
                return Err(SnapshotError::MissingBody(collider.body_id));
            }
        }
        for constraint in &self.constraints {
            if !self.bodies.contains_key(&constraint.body_a) {
                return Err(SnapshotError::MissingBody(constraint.body_a));
            }
            if !self.bodies.contains_key(&constraint.body_b) {
                return Err(SnapshotError::MissingBody(constraint.body_b));
            }
        }
        Ok(())
    }
}

/// Errors that can occur during snapshot operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    MissingBody(BodyId),
    Corrupted(String),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBody(id) => write!(f, "Snapshot references missing body {:?}", id),
            Self::Corrupted(msg) => write!(f, "Snapshot corrupted: {}", msg),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Convert a Shape to a ShapeSnapshot.
pub fn shape_to_snapshot(shape: &crate::Shape) -> ShapeSnapshot {
    use crate::ShapeKind;
    match &shape.kind {
        ShapeKind::Circle { radius } => ShapeSnapshot::Circle { radius: *radius },
        ShapeKind::Aabb { half_extents } => ShapeSnapshot::Aabb {
            half_extents: *half_extents,
        },
        ShapeKind::ConvexHull { vertices } => ShapeSnapshot::ConvexHull {
            vertices: vertices.to_vec(),
        },
        ShapeKind::Capsule {
            half_height,
            radius,
        } => ShapeSnapshot::Capsule {
            half_height: *half_height,
            radius: *radius,
        },
        ShapeKind::Sphere { radius } => ShapeSnapshot::Sphere { radius: *radius },
        ShapeKind::Box3D { half_extents } => ShapeSnapshot::Box3D {
            half_extents: *half_extents,
        },
        ShapeKind::Capsule3D {
            half_height,
            radius,
        } => ShapeSnapshot::Capsule3D {
            half_height: *half_height,
            radius: *radius,
        },
        ShapeKind::Compound2D { children } => ShapeSnapshot::Compound2D {
            children: children
                .iter()
                .map(|c| (c.offset, Box::new(shape_to_snapshot(&c.shape))))
                .collect(),
        },
        ShapeKind::Compound3D { children } => ShapeSnapshot::Compound3D {
            children: children
                .iter()
                .map(|c| (c.offset, Box::new(shape_to_snapshot(&c.shape))))
                .collect(),
        },
        ShapeKind::Heightmap(hm) => ShapeSnapshot::ConvexHull {
            // Heightmaps can't be perfectly serialized as a shape snapshot;
            // store as a convex hull approximation using corner vertices
            vertices: {
                let (min, max) = hm.aabb();
                vec![
                    glam::Vec2::new(min.x, min.z),
                    glam::Vec2::new(max.x, min.z),
                    glam::Vec2::new(max.x, max.z),
                    glam::Vec2::new(min.x, max.z),
                ]
            },
        },
    }
}
