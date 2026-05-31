//! PhysicsWorld: owns all bodies, runs simulation steps, triggers callbacks.

use std::collections::HashMap;

use glam::Vec2;
use glam::Vec3;

use crate::collider::Collider;
use crate::constraint::Constraint;
use crate::integration::{semi_implicit_euler, semi_implicit_euler_3d, update_sleep, wake};
use crate::narrowphase::collide;
use crate::solver::ImpulseSolver;
use crate::{BodyId, OnSleepCallback, RigidBody};

/// Physics world configuration.
#[derive(Debug, Clone)]
pub struct WorldConfig {
    /// Gravity vector (pixels/s²) for 2D simulation.
    pub gravity: Vec2,
    /// Gravity vector (pixels/s²) for 3D simulation.
    pub gravity_3d: Vec3,
    /// Number of substeps per simulation tick.
    pub substeps: u32,
    /// Sleep delay: number of steps below threshold before sleeping.
    pub sleep_delay: u32,
    /// Default linear damping.
    pub default_linear_damping: f32,
    /// Default angular damping.
    pub default_angular_damping: f32,
    /// Baumgarte factor for position correction.
    pub baumgarte: f32,
    /// Maximum velocity a body can travel per substep (pixels).
    /// Bodies exceeding this are clamped to prevent tunneling.
    pub max_velocity_per_substep: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, 500.0),
            gravity_3d: Vec3::new(0.0, -9.81, 0.0),
            substeps: 4,
            sleep_delay: 60,
            default_linear_damping: 0.01,
            default_angular_damping: 0.01,
            baumgarte: 0.2,
            max_velocity_per_substep: 200.0,
        }
    }
}

/// The physics world: owns all bodies, colliders, and constraints.
///
/// Example:
/// ```
/// use cvkg_physics::{PhysicsWorld, WorldConfig, RigidBody, Shape, Collider};
/// let mut world = PhysicsWorld::new(WorldConfig::default());
/// let body = world.add_body(RigidBody::new(1.0, &Shape::circle(16.0)));
/// world.add_collider(Collider::new(body, Shape::circle(16.0)));
/// world.step(1.0 / 60.0);
/// ```
pub struct PhysicsWorld {
    config: WorldConfig,
    bodies: Vec<RigidBody>,
    colliders: Vec<Collider>,
    constraints: Vec<Constraint>,
    body_id_map: HashMap<BodyId, usize>,
    next_body_id: u64,
    spatial_hash: crate::broadphase::SpatialHash,
    solver: ImpulseSolver,
    /// Callback fired when a body goes to sleep.
    pub on_sleep: Option<OnSleepCallback>,
}

/// The result of a simulation step, exposed for application consumption.
#[derive(Debug, Default)]
pub struct StepResult {
    /// Number of collision pairs detected this step.
    pub collision_pairs: u32,
    /// Bodies that went to sleep this step.
    pub slept_bodies: Vec<BodyId>,
    /// Bodies that woke up this step.
    pub woke_bodies: Vec<BodyId>,
}

impl PhysicsWorld {
    /// Create a new physics world with the given configuration.
    pub fn new(config: WorldConfig) -> Self {
        Self {
            config,
            bodies: Vec::new(),
            colliders: Vec::new(),
            constraints: Vec::new(),
            body_id_map: HashMap::new(),
            next_body_id: 1,
            spatial_hash: crate::broadphase::SpatialHash::new(),
            solver: ImpulseSolver::new().with_iterations(8).with_baumgarte(0.2),
            on_sleep: None,
        }
    }

    /// Add a rigid body and return its ID.
    pub fn add_body(&mut self, body: RigidBody) -> BodyId {
        let id = BodyId(self.next_body_id);
        self.next_body_id += 1;
        self.body_id_map.insert(id, self.bodies.len());
        self.bodies.push(body);
        id
    }

    /// Remove a body by ID.
    ///
    /// Properly removes the body from the bodies vector using swap-remove
    /// for O(1) removal, updates the body_id_map for the swapped entry,
    /// and cleans up associated colliders and constraints.
    pub fn remove_body(&mut self, id: BodyId) {
        let idx = match self.body_id_map.remove(&id) {
            Some(idx) => idx,
            None => return,
        };

        // Remove associated colliders and constraints
        self.colliders.retain(|c| c.body_id != id);
        self.constraints
            .retain(|c| c.body_a != id && c.body_b != id);

        // Swap-remove the body from the vec for O(1) removal
        let last_idx = self.bodies.len() - 1;
        if idx != last_idx {
            // Move the last body into the removed slot
            self.bodies.swap_remove(idx);
            // Update the mapping for the moved body
            let moved_body_id = self
                .body_id_map
                .iter()
                .find_map(|(bid, &i)| if i == last_idx { Some(*bid) } else { None });
            if let Some(moved_id) = moved_body_id {
                self.body_id_map.insert(moved_id, idx);
            }
        } else {
            // Removing the last element, just pop
            self.bodies.pop();
        }
    }

    /// Get a reference to a body.
    pub fn body(&self, id: BodyId) -> Option<&RigidBody> {
        self.body_id_map.get(&id).map(|&i| &self.bodies[i])
    }

    /// Get a mutable reference to a body. Wakes it if sleeping.
    pub fn body_mut(&mut self, id: BodyId) -> Option<&mut RigidBody> {
        if let Some(&idx) = self.body_id_map.get(&id) {
            let body = &mut self.bodies[idx];
            if body.is_sleeping {
                wake(body);
            }
            Some(body)
        } else {
            None
        }
    }

    /// Add a collider to the world.
    pub fn add_collider(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }

    /// Add a constraint to the world.
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Get all body IDs.
    pub fn body_ids(&self) -> Vec<BodyId> {
        self.body_id_map.keys().copied().collect()
    }

    /// Get the number of bodies.
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Get the number of colliders.
    pub fn collider_count(&self) -> usize {
        self.colliders.len()
    }

    /// Get the number of constraints.
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    /// Run one simulation step.
    pub fn step(&mut self, dt: f32) -> StepResult {
        let mut result = StepResult::default();
        let sub_dt = dt / self.config.substeps as f32;

        for _ in 0..self.config.substeps {
            self.step_substep(sub_dt, &mut result);
        }

        result
    }

    #[allow(clippy::collapsible_if)]
    fn step_substep(&mut self, dt: f32, result: &mut StepResult) {
        // 1. Integrate velocities and positions with velocity clamping
        for body in &mut self.bodies {
            if body.is_3d {
                semi_implicit_euler_3d(body, dt, self.config.gravity_3d);
            } else {
                semi_implicit_euler(body, dt, self.config.gravity);
            }
            // Clamp velocity to prevent tunneling (continuous collision prevention)
            let max_v = self.config.max_velocity_per_substep;
            if body.is_3d {
                if body.velocity_3d.length_squared() > max_v * max_v {
                    body.velocity_3d = body.velocity_3d.normalize() * max_v;
                }
                let ang_speed = body.angular_velocity_3d.length();
                if ang_speed > max_v {
                    body.angular_velocity_3d = body.angular_velocity_3d / ang_speed * max_v;
                }
            } else {
                if body.velocity.length_squared() > max_v * max_v {
                    body.velocity = body.velocity.normalize() * max_v;
                }
                if body.angular_velocity.abs() > max_v {
                    body.angular_velocity = body.angular_velocity.signum() * max_v;
                }
            }
        }

        // 2. Broad phase: find candidate collision pairs
        self.spatial_hash.clear();
        for (i, collider) in self.colliders.iter().enumerate() {
            let body = match self.body_id_map.get(&collider.body_id) {
                Some(&idx) => &self.bodies[idx],
                None => continue,
            };
            let (min, max) = collider.world_aabb(body.position, body.angle);
            self.spatial_hash.insert(BodyId(i as u64), min, max);
        }

        // 3. Narrow phase: generate contact manifolds
        let pairs = self.spatial_hash.candidate_pairs();
        let mut manifolds = Vec::new();

        for (id_a, id_b) in pairs {
            let idx_a = id_a.0 as usize;
            let idx_b = id_b.0 as usize;

            if idx_a >= self.colliders.len() || idx_b >= self.colliders.len() {
                continue;
            }

            let col_a = &self.colliders[idx_a];
            let col_b = &self.colliders[idx_b];

            // Category filter
            if !col_a.can_collide_with(col_b) {
                continue;
            }

            let body_a = match self.body_id_map.get(&col_a.body_id) {
                Some(&i) => &self.bodies[i],
                None => continue,
            };
            let body_b = match self.body_id_map.get(&col_b.body_id) {
                Some(&i) => &self.bodies[i],
                None => continue,
            };

            if body_a.is_sleeping && body_b.is_sleeping {
                continue;
            }

            if let Some(manifold) =
                collide(idx_a, &col_a.shape, body_a, idx_b, &col_b.shape, body_b)
            {
                manifolds.push(manifold);
                result.collision_pairs += 1;

                // Wake sleeping bodies
                if let Some(&i) = self.body_id_map.get(&col_a.body_id) {
                    if self.bodies[i].is_sleeping {
                        wake(&mut self.bodies[i]);
                    }
                }
                if let Some(&i) = self.body_id_map.get(&col_b.body_id) {
                    if self.bodies[i].is_sleeping {
                        wake(&mut self.bodies[i]);
                    }
                }
            }
        }

        // 4. Solve contacts (apply contact impulses)
        for manifold in &manifolds {
            let idx_a = match self.body_id_map.get(&BodyId(manifold.body_a as u64)) {
                Some(&i) => i,
                None => continue,
            };
            let idx_b = match self.body_id_map.get(&BodyId(manifold.body_b as u64)) {
                Some(&i) => i,
                None => continue,
            };
            self.resolve_contact(idx_a, idx_b, &manifold.contacts);
        }

        // 5. Solve constraints
        self.solver
            .solve(&self.constraints, &mut self.bodies, &self.body_id_map, dt);

        // 6. Update sleep states
        let mut slept = Vec::new();
        for (id, &idx) in &self.body_id_map {
            let body = &mut self.bodies[idx];
            if update_sleep(body, body.sleep_threshold, self.config.sleep_delay) {
                slept.push(*id);
            }
        }

        // Fire sleep callbacks
        if let Some(ref callback) = self.on_sleep {
            for id in &slept {
                callback(*id);
            }
        }
        result.slept_bodies = slept;
    }

    fn resolve_contact(
        &mut self,
        idx_a: usize,
        idx_b: usize,
        contacts: &[crate::narrowphase::Contact],
    ) {
        if contacts.is_empty() {
            return;
        }

        // Use the average contact normal and depth
        let normal =
            contacts.iter().fold(Vec2::ZERO, |acc, c| acc + c.normal) / contacts.len() as f32;
        let depth = contacts.iter().map(|c| c.depth).fold(0.0, f32::max);

        // Read all scalar values before mutable borrows
        let inv_mass_a = self.bodies[idx_a].inv_mass;
        let inv_mass_b = self.bodies[idx_b].inv_mass;
        let total_inv_mass = inv_mass_a + inv_mass_b;
        if total_inv_mass < 1e-10 || normal.length_squared() < 1e-12 {
            return;
        }

        let is_static_a = self.bodies[idx_a].is_static;
        let is_static_b = self.bodies[idx_b].is_static;
        let restitution_a = self.bodies[idx_a].restitution;
        let restitution_b = self.bodies[idx_b].restitution;
        let friction_a = self.bodies[idx_a].friction;
        let friction_b = self.bodies[idx_b].friction;
        let baumgarte = self.config.baumgarte;

        // Position correction (Baumgarte stabilization)
        let correction = normal * (depth / total_inv_mass * baumgarte);

        if !is_static_a {
            self.bodies[idx_a].position += correction * inv_mass_a;
        }
        if !is_static_b {
            self.bodies[idx_b].position -= correction * inv_mass_b;
        }

        // Velocity correction (impulse-based bounce)
        let vel_a = self.bodies[idx_a].velocity;
        let vel_b = self.bodies[idx_b].velocity;
        let rel_vel = vel_b - vel_a;
        let vel_along_normal = rel_vel.dot(normal);

        // Only resolve if bodies are approaching
        if vel_along_normal > 0.0 {
            return;
        }

        let restitution = restitution_a.min(restitution_b);
        let impulse_scalar = -(1.0 + restitution) * vel_along_normal / total_inv_mass;
        let impulse = normal * impulse_scalar;

        if !is_static_a {
            self.bodies[idx_a].velocity -= impulse * inv_mass_a;
        }
        if !is_static_b {
            self.bodies[idx_b].velocity += impulse * inv_mass_b;
        }

        // Friction impulse
        let tangent = rel_vel - normal * rel_vel.dot(normal);
        let tangent_len = tangent.length();
        if tangent_len > 1e-10 {
            let tangent_dir = tangent / tangent_len;
            let friction_impulse = tangent_dir * (-rel_vel.dot(tangent_dir) / total_inv_mass);
            let friction_mag = friction_impulse
                .length()
                .min(impulse_scalar.abs() * (friction_a + friction_b) * 0.5);

            if !is_static_a {
                self.bodies[idx_a].velocity -= tangent_dir * friction_mag * inv_mass_a;
            }
            if !is_static_b {
                self.bodies[idx_b].velocity += tangent_dir * friction_mag * inv_mass_b;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Shape;

    #[test]
    fn test_create_world() {
        let world = PhysicsWorld::new(WorldConfig::default());
        assert_eq!(world.body_count(), 0);
    }

    #[test]
    fn test_add_body() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(16.0);
        let id = world.add_body(RigidBody::new(1.0, &shape));
        assert_eq!(world.body_count(), 1);

        let body = world.body(id).unwrap();
        assert!(!body.is_static);
        assert_eq!(body.mass, 1.0);
    }

    #[test]
    fn test_remove_body() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(16.0);
        let id1 = world.add_body(RigidBody::new(1.0, &shape));
        let id2 = world.add_body(RigidBody::new(2.0, &shape));
        let id3 = world.add_body(RigidBody::new(3.0, &shape));
        assert_eq!(world.body_count(), 3);

        // Remove middle body
        world.remove_body(id2);
        assert_eq!(world.body_count(), 2);
        assert!(world.body(id2).is_none());
        assert!(world.body(id1).is_some());
        assert!(world.body(id3).is_some());

        // Remove first body
        world.remove_body(id1);
        assert_eq!(world.body_count(), 1);
        assert!(world.body(id1).is_none());
        assert!(world.body(id3).is_some());

        // Remove last body
        world.remove_body(id3);
        assert_eq!(world.body_count(), 0);
    }

    #[test]
    fn test_remove_body_updates_indices() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(10.0);
        let id1 = world.add_body(RigidBody::new(1.0, &shape));
        let id2 = world.add_body(RigidBody::new(1.0, &shape));
        let id3 = world.add_body(RigidBody::new(1.0, &shape));

        // Remove middle body -- the last body should be swapped into its place
        world.remove_body(id2);
        assert_eq!(world.body_count(), 2);

        // Both remaining bodies should still be accessible
        let b1 = world.body(id1).unwrap();
        let b3 = world.body(id3).unwrap();
        assert_eq!(b1.mass, 1.0);
        assert_eq!(b3.mass, 1.0);

        // Verify the bodies can be mutated through the world
        let b1_mut = world.body_mut(id1).unwrap();
        b1_mut.apply_impulse(Vec2::new(10.0, 0.0));
        assert!((b1_mut.velocity.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_falling_body() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(16.0);
        let body_id = world.add_body(RigidBody::new(1.0, &shape));

        // Step for 1 second
        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }

        let body = world.body(body_id).unwrap();
        assert!(body.position.y > 0.0); // Should have fallen
        assert!(body.velocity.y > 0.0); // Moving downward
    }

    #[test]
    fn test_static_body_immovable() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let id = world.add_body(RigidBody::static_body());

        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }

        let body = world.body(id).unwrap();
        assert_eq!(body.position, Vec2::ZERO);
        assert_eq!(body.velocity, Vec2::ZERO);
    }

    #[test]
    fn test_apply_impulse() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(16.0);
        let id = world.add_body(RigidBody::new(1.0, &shape));

        if let Some(body) = world.body_mut(id) {
            body.apply_impulse(Vec2::new(100.0, -200.0));
        }

        let body = world.body(id).unwrap();
        assert!((body.velocity.x - 100.0).abs() < 0.001);
        assert!((body.velocity.y - (-200.0)).abs() < 0.001);
    }
}