//! Kinematic character controller for 3D UI navigation.
//!
//! A kinematic character moves through the physics world without being affected
//! by forces. It uses shape casts and depenetration to slide along obstacles.
//! Ideal for player avatars in 3D UI spaces, first-person or third-person.

use std::collections::HashMap;

use glam::{Quat, Vec3};

use crate::{BodyId, Collider, PhysicsWorld, RigidBody, Shape, queries::ShapeCastHit3D};

/// Configuration for a kinematic character controller.
#[derive(Debug, Clone)]
pub struct CharacterConfig {
    /// Height of the character (for capsule shape).
    pub height: f32,
    /// Radius of the character capsule.
    pub radius: f32,
    /// Maximum slope angle (radians) the character can walk up.
    pub max_slope_angle: f32,
    /// Maximum step height the character can step over.
    pub max_step_height: f32,
    /// Skin width for depenetration (prevents character from touching walls).
    pub skin_width: f32,
    /// Maximum number of collision iterations per move.
    pub max_iterations: u32,
    /// Gravity to apply when not grounded.
    pub gravity: f32,
    /// Maximum fall speed.
    pub max_fall_speed: f32,
}

impl Default for CharacterConfig {
    fn default() -> Self {
        Self {
            height: 1.8,
            radius: 0.3,
            max_slope_angle: 45.0_f32.to_radians(),
            max_step_height: 0.3,
            skin_width: 0.05,
            max_iterations: 8,
            gravity: 20.0,
            max_fall_speed: 50.0,
        }
    }
}

/// A kinematic character controller that moves through the physics world.
#[derive(Debug)]
pub struct KinematicCharacter {
    /// Body ID of the internal collider.
    pub body_id: BodyId,
    /// Current position (center of capsule bottom).
    pub position: Vec3,
    /// Current velocity (manually managed).
    pub velocity: Vec3,
    /// Whether the character is on the ground.
    pub is_grounded: bool,
    /// Ground normal (valid when grounded).
    pub ground_normal: Vec3,
    /// Configuration.
    pub config: CharacterConfig,
    /// User data passed through to the collider.
    pub user_data: u64,
}

impl KinematicCharacter {
    /// Create a new kinematic character and register it with the physics world.
    pub fn new(world: &mut PhysicsWorld, position: Vec3, config: CharacterConfig) -> Self {
        let half_height = (config.height - config.radius * 2.0) * 0.5;
        let shape = Shape::capsule3d(config.radius, half_height);
        let body_id = world.add_body(RigidBody::new_3d(1.0, &shape));
        let collider = Collider::new_sensor(body_id, shape);
        world.add_collider(collider);

        Self {
            body_id,
            position,
            velocity: Vec3::ZERO,
            is_grounded: false,
            ground_normal: Vec3::Y,
            config,
            user_data: 0,
        }
    }

    /// Move the character by the given displacement, sliding along obstacles.
    pub fn move_character(&mut self, world: &mut PhysicsWorld, desired_displacement: Vec3) -> Vec3 {
        let mut remaining = desired_displacement;
        let mut total_move = Vec3::ZERO;

        for _ in 0..self.config.max_iterations {
            if remaining.length_squared() < 1e-8 {
                break;
            }

            let hit = self.shape_cast(world, remaining);

            if let Some(hit) = hit {
                let move_dist = hit.distance;
                let safe_dist = (move_dist - self.config.skin_width).max(0.0);

                if safe_dist > 1e-6 {
                    let move_dir = remaining.normalize();
                    let actual_move = move_dir * safe_dist;
                    self.position += actual_move;
                    total_move += actual_move;
                }

                let remaining_length = remaining.length();
                let leftover = remaining_length - safe_dist;

                if leftover > 1e-6 {
                    let move_dir = remaining.normalize();
                    let normal = hit.normal;
                    let along_normal = normal * move_dir.dot(normal);
                    let along_surface = move_dir - along_normal;
                    let surface_len = along_surface.length();

                    if surface_len > 1e-6 {
                        remaining = along_surface / surface_len * leftover;
                    } else {
                        break;
                    }
                } else {
                    break;
                }

                let slope_angle = hit.normal.angle_between(Vec3::Y);
                if slope_angle < self.config.max_slope_angle {
                    self.is_grounded = true;
                    self.ground_normal = hit.normal;
                }
            } else {
                self.position += remaining;
                total_move += remaining;
                break;
            }
        }

        if let Some(body) = world.body_mut(self.body_id) {
            body.position_3d = self.position;
        }

        total_move
    }

    /// Apply gravity to the character velocity.
    pub fn apply_gravity(&mut self, dt: f32) {
        if !self.is_grounded {
            self.velocity.y -= self.config.gravity * dt;
            if self.velocity.y.abs() > self.config.max_fall_speed {
                self.velocity.y = -self.config.max_fall_speed;
            }
        } else if self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
    }

    /// Teleport the character to a position without collision checks.
    pub fn teleport(&mut self, world: &mut PhysicsWorld, position: Vec3) {
        self.position = position;
        if let Some(body) = world.body_mut(self.body_id) {
            body.position_3d = self.position;
        }
    }

    fn shape_cast(&self, world: &PhysicsWorld, displacement: Vec3) -> Option<ShapeCastHit3D> {
        let half_height = (self.config.height - self.config.radius * 2.0) * 0.5;
        let shape = Shape::capsule3d(self.config.radius - self.config.skin_width, half_height);
        let max_dist = displacement.length();
        if max_dist < 1e-8 {
            return None;
        }

        // Build a body map for the query
        let mut body_map: HashMap<BodyId, &RigidBody> = HashMap::new();
        for (bid, &idx) in world.body_id_map() {
            if let Some(body) = world.bodies().get(idx) {
                body_map.insert(*bid, body);
            }
            if body_map.len() >= world.bodies().len() {
                break;
            }
        }

        let _body_id = self.body_id;
        crate::queries::shape_cast_3d(
            world.colliders(),
            &body_map,
            &shape,
            self.position,
            Quat::IDENTITY,
            displacement,
            max_dist,
            None,
        )
    }
}
