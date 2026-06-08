//! PhysicsWorld: owns all bodies, runs simulation steps, triggers callbacks.

use std::collections::HashMap;

use glam::{Quat, Vec2, Vec3};

use crate::collider::Collider;
use crate::constraint::Constraint;
use crate::integration::{semi_implicit_euler, semi_implicit_euler_3d, update_sleep, wake};
use crate::narrowphase::{ContactManifold, collide};
use crate::solver::ImpulseSolver;
use crate::{BodyId, CcdResult3D, Contact, OnSleepCallback, RigidBody, gjk_ccd, gjk_ccd_3d};

/// Collision event types for callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventType {
    /// Two bodies started colliding this frame.
    Enter,
    /// Two bodies are still colliding (were colliding last frame too).
    Stay,
    /// Two bodies stopped colliding this frame.
    Exit,
}

/// A collision event between two bodies.
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// The type of collision event.
    pub event_type: CollisionEventType,
    /// First body involved in the collision.
    pub body_a: BodyId,
    /// Second body involved in the collision.
    pub body_b: BodyId,
    /// Contact manifold data (only for Enter and Stay events).
    pub manifold: Option<ContactManifold>,
}

/// Callback for collision events.
///
/// Use this to trigger sound effects, particle systems, haptic feedback, etc.
pub type CollisionCallback = Box<dyn Fn(&CollisionEvent) + Send + Sync>;

/// Callback fired when a constraint breaks due to exceeding its strain threshold.
pub type ConstraintBrokenCallback = Box<dyn Fn(BodyId, BodyId) + Send + Sync>;

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
    /// Fixed timestep for deterministic simulation (seconds).
    /// If Some(dt), the accumulator pattern is used: dt is accumulated and
    /// simulation steps at fixed intervals. Remainder is interpolated.
    /// If None, variable timestep is used (legacy behavior).
    pub fixed_timestep: Option<f32>,
    /// Maximum number of fixed steps per frame to prevent spiral of death.
    pub max_steps_per_frame: u32,
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
            fixed_timestep: None,
            max_steps_per_frame: 8,
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
    /// Callback fired for collision events (enter, stay, exit).
    pub on_collision: Option<CollisionCallback>,
    /// Callback fired when a constraint breaks.
    pub on_constraint_broken: Option<ConstraintBrokenCallback>,
    /// Previous frame's contact pairs for collision event detection.
    /// Key: (BodyId, BodyId) sorted, Value: ContactManifold
    prev_contacts: HashMap<(BodyId, BodyId), ContactManifold>,
    /// Scene bridge for syncing physics transforms to the scene graph.
    /// When present, each step automatically syncs 3D body transforms.
    pub scene_bridge: crate::scene_bridge::SceneBridge,
    /// Accumulated time for fixed-timestep simulation.
    accumulator: f32,
    /// Current simulation tick count.
    tick: u64,
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
            on_collision: None,
            on_constraint_broken: None,
            prev_contacts: HashMap::new(),
            scene_bridge: crate::scene_bridge::SceneBridge::new(),
            accumulator: 0.0,
            tick: 0,
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

        if let Some(fixed_dt) = self.config.fixed_timestep {
            // Fixed timestep with accumulator pattern
            self.accumulator += dt;
            let max_steps = self.config.max_steps_per_frame;
            let mut steps_taken = 0;

            while self.accumulator >= fixed_dt && steps_taken < max_steps {
                let sub_dt = fixed_dt / self.config.substeps as f32;
                for _ in 0..self.config.substeps {
                    self.step_substep(sub_dt, &mut result);
                }
                self.accumulator -= fixed_dt;
                self.tick += 1;
                steps_taken += 1;
            }

            // Clamp accumulator to prevent unbounded growth
            if self.accumulator > fixed_dt * max_steps as f32 {
                self.accumulator = 0.0;
            }
        } else {
            // Variable timestep (legacy)
            let sub_dt = dt / self.config.substeps as f32;
            for _ in 0..self.config.substeps {
                self.step_substep(sub_dt, &mut result);
            }
            self.tick += 1;
        }

        result
    }

    /// Get the current interpolation alpha for fixed-timestep rendering.
    /// Returns a value in [0, 1) representing the fraction of time between
    /// the most recent and next fixed step. Use this to interpolate body
    /// transforms for smooth rendering.
    pub fn interpolation_alpha(&self) -> f32 {
        match self.config.fixed_timestep {
            Some(fixed_dt) if fixed_dt > 0.0 => (self.accumulator / fixed_dt).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    /// Get the current simulation tick count.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Get a reference to all colliders.
    pub fn colliders(&self) -> &[Collider] {
        &self.colliders
    }

    /// Get a reference to all bodies.
    pub fn bodies(&self) -> &[RigidBody] {
        &self.bodies
    }

    /// Get a reference to the body ID map.
    pub fn body_id_map(&self) -> &HashMap<BodyId, usize> {
        &self.body_id_map
    }

    /// Get a reference to all constraints.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Capture a complete snapshot of the physics world state.
    ///
    /// Use `restore()` to return to this state later. Snapshots support
    /// undo/redo, save/load, and deterministic replay.
    ///
    /// # Example
    /// ```
    /// use cvkg_physics::{PhysicsWorld, WorldConfig};
    /// let mut world = PhysicsWorld::new(WorldConfig::default());
    /// let saved = world.snapshot();
    /// // ... modify world ...
    /// world.restore(&saved).unwrap();
    /// ```
    pub fn snapshot(&self) -> crate::snapshot::PhysicsSnapshot {
        use crate::snapshot::{BodySnapshot, ColliderSnapshot, PhysicsSnapshot};

        let mut bodies = std::collections::HashMap::new();
        for (body_id, &idx) in &self.body_id_map {
            if let Some(body) = self.bodies.get(idx) {
                bodies.insert(
                    *body_id,
                    BodySnapshot {
                        position: body.position,
                        position_3d: body.position_3d,
                        velocity: body.velocity,
                        velocity_3d: body.velocity_3d,
                        angle: body.angle,
                        rotation: body.rotation,
                        angular_velocity: body.angular_velocity,
                        angular_velocity_3d: body.angular_velocity_3d,
                        inv_mass: body.inv_mass,
                        inv_inertia: body.inv_inertia,
                        inv_inertia_3d: body.inv_inertia_3d,
                        is_3d: body.is_3d,
                        is_static: body.is_static,
                        is_sleeping: body.is_sleeping,
                        linear_damping: body.linear_damping,
                        angular_damping: body.angular_damping,
                        restitution: body.restitution,
                        friction: body.friction,
                        gravity_scale: body.gravity_scale,
                        sleep_threshold: body.sleep_threshold,
                        category: 0,
                        collides_with: 0,
                    },
                );
            }
        }

        let colliders: Vec<_> = self
            .colliders
            .iter()
            .map(|c| ColliderSnapshot {
                body_id: c.body_id,
                shape_kind: crate::snapshot::shape_to_snapshot(&c.shape),
                offset: c.offset,
                rotation_offset: c.rotation_offset,
                is_sensor: c.is_sensor,
                category: c.category,
                collides_with: c.collides_with,
            })
            .collect();

        PhysicsSnapshot {
            bodies,
            colliders,
            constraints: self.constraints.clone(),
            config: self.config.clone(),
            accumulator: self.accumulator,
            tick: self.tick,
        }
    }

    /// Restore the physics world to a previously captured state.
    ///
    /// Rebuilds all bodies, colliders, and constraints from the snapshot.
    /// The body ID map is reconstructed to match the snapshot exactly.
    ///
    /// Returns an error if the snapshot references inconsistent data.
    pub fn restore(
        &mut self,
        snap: &crate::snapshot::PhysicsSnapshot,
    ) -> Result<(), crate::snapshot::SnapshotError> {
        snap.validate()?;

        self.bodies.clear();
        self.body_id_map.clear();
        self.colliders.clear();
        self.constraints.clear();

        // Rebuild bodies
        let mut sorted_ids: Vec<_> = snap.bodies.keys().collect();
        sorted_ids.sort_by_key(|id| id.0);

        for body_id in sorted_ids {
            let body_snap = &snap.bodies[body_id];
            let mut body = if body_snap.is_3d {
                crate::RigidBody::new_3d(
                    1.0 / body_snap.inv_mass.max(1e-10),
                    &crate::Shape::sphere(1.0),
                )
            } else {
                crate::RigidBody::new(
                    1.0 / body_snap.inv_mass.max(1e-10),
                    &crate::Shape::circle(1.0),
                )
            };

            body.position = body_snap.position;
            body.position_3d = body_snap.position_3d;
            body.velocity = body_snap.velocity;
            body.velocity_3d = body_snap.velocity_3d;
            body.angle = body_snap.angle;
            body.rotation = body_snap.rotation;
            body.angular_velocity = body_snap.angular_velocity;
            body.angular_velocity_3d = body_snap.angular_velocity_3d;
            body.inv_mass = body_snap.inv_mass;
            body.inv_inertia = body_snap.inv_inertia;
            body.inv_inertia_3d = body_snap.inv_inertia_3d;
            body.is_3d = body_snap.is_3d;
            body.is_static = body_snap.is_static;
            body.is_sleeping = body_snap.is_sleeping;
            body.linear_damping = body_snap.linear_damping;
            body.angular_damping = body_snap.angular_damping;
            body.restitution = body_snap.restitution;
            body.friction = body_snap.friction;
            body.gravity_scale = body_snap.gravity_scale;
            body.sleep_threshold = body_snap.sleep_threshold;

            let idx = self.bodies.len();
            self.bodies.push(body);
            self.body_id_map.insert(*body_id, idx);
        }

        // Rebuild colliders (shapes restored as circles/spheres -- full shape
        // reconstruction would require dynamic lifetime management)
        for col_snap in &snap.colliders {
            let shape = match &col_snap.shape_kind {
                crate::snapshot::ShapeSnapshot::Circle { radius } => crate::Shape::circle(*radius),
                crate::snapshot::ShapeSnapshot::Aabb { half_extents } => {
                    crate::Shape::aabb(*half_extents)
                }
                crate::snapshot::ShapeSnapshot::Capsule {
                    half_height,
                    radius,
                } => crate::Shape::capsule(*radius, *half_height),
                crate::snapshot::ShapeSnapshot::Sphere { radius } => crate::Shape::sphere(*radius),
                crate::snapshot::ShapeSnapshot::Box3D { half_extents } => {
                    crate::Shape::box3d(*half_extents)
                }
                crate::snapshot::ShapeSnapshot::Capsule3D {
                    half_height,
                    radius,
                } => crate::Shape::capsule3d(*radius, *half_height),
                crate::snapshot::ShapeSnapshot::ConvexHull { vertices } => {
                    // ConvexHull requires &'static [Vec2]; restore as AABB approximation
                    // since we can't reconstruct the static lifetime
                    if vertices.len() >= 2 {
                        let min = vertices
                            .iter()
                            .copied()
                            .reduce(|a, b| a.min(b))
                            .unwrap_or(glam::Vec2::ZERO);
                        let max = vertices
                            .iter()
                            .copied()
                            .reduce(|a, b| a.max(b))
                            .unwrap_or(glam::Vec2::ZERO);
                        let half_extents = (max - min) * 0.5;
                        crate::Shape::aabb(half_extents)
                    } else {
                        crate::Shape::circle(16.0)
                    }
                }
                crate::snapshot::ShapeSnapshot::Compound2D { children } => {
                    // Compound shapes require &'static children; restore as AABB
                    // encompassing all child offsets
                    if !children.is_empty() {
                        let max_dist = children
                            .iter()
                            .map(|(offset, _)| offset.length())
                            .fold(0.0f32, f32::max);
                        crate::Shape::circle(max_dist + 16.0)
                    } else {
                        crate::Shape::circle(16.0)
                    }
                }
                crate::snapshot::ShapeSnapshot::Compound3D { .. } => crate::Shape::sphere(16.0),
                crate::snapshot::ShapeSnapshot::Heightmap {
                    heights,
                    width,
                    depth,
                    world_size,
                    ..
                } => crate::Shape::heightmap(heights.clone(), *width, *depth, *world_size),
            };
            let collider = crate::Collider {
                body_id: col_snap.body_id,
                shape,
                offset: col_snap.offset,
                rotation_offset: col_snap.rotation_offset,
                is_sensor: col_snap.is_sensor,
                category: col_snap.category,
                collides_with: col_snap.collides_with,
                user_data: 0,
            };
            self.colliders.push(collider);
        }

        // Rebuild constraints
        self.constraints = snap.constraints.clone();

        // Restore config and timing
        self.config = snap.config.clone();
        self.accumulator = snap.accumulator;
        self.tick = snap.tick;

        // Reset collision tracking
        self.prev_contacts.clear();

        Ok(())
    }

    /// Collect 3D transforms from all bodies for scene graph sync.
    /// Returns a map of BodyId → (position, rotation) for all bodies with is_3d=true.
    pub fn collect_3d_transforms(
        &self,
    ) -> std::collections::HashMap<BodyId, crate::scene_bridge::Body3DTransform> {
        let mut transforms = std::collections::HashMap::new();
        for (body_id, &idx) in &self.body_id_map {
            if let Some(body) = self.bodies.get(idx) {
                if body.is_3d {
                    transforms.insert(
                        *body_id,
                        crate::scene_bridge::Body3DTransform {
                            position: body.position_3d,
                            rotation: body.rotation,
                        },
                    );
                }
            }
        }
        transforms
    }

    /// Process collision events by comparing current frame contacts with previous frame.
    /// Fires callbacks for Enter, Stay, and Exit events.
    fn process_collision_events(&mut self, current_manifolds: &[ContactManifold]) {
        // Build current contacts map with sorted body IDs as keys
        let mut current_contacts: HashMap<(BodyId, BodyId), ContactManifold> = HashMap::new();
        for manifold in current_manifolds {
            let body_a_id = self.colliders[manifold.body_a].body_id;
            let body_b_id = self.colliders[manifold.body_b].body_id;
            let key = if body_a_id.0 < body_b_id.0 {
                (body_a_id, body_b_id)
            } else {
                (body_b_id, body_a_id)
            };
            current_contacts.insert(key, manifold.clone());
        }

        // Detect Enter and Stay events
        for (key, manifold) in &current_contacts {
            let event_type = if self.prev_contacts.contains_key(key) {
                CollisionEventType::Stay
            } else {
                CollisionEventType::Enter
            };

            if let Some(ref callback) = self.on_collision {
                let event = CollisionEvent {
                    event_type,
                    body_a: key.0,
                    body_b: key.1,
                    manifold: Some(manifold.clone()),
                };
                callback(&event);
            }
        }

        // Detect Exit events (in prev but not in current)
        for key in self.prev_contacts.keys() {
            if !current_contacts.contains_key(key) {
                if let Some(ref callback) = self.on_collision {
                    let event = CollisionEvent {
                        event_type: CollisionEventType::Exit,
                        body_a: key.0,
                        body_b: key.1,
                        manifold: None,
                    };
                    callback(&event);
                }
            }
        }

        // Update prev_contacts for next frame
        self.prev_contacts = current_contacts;
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

        // 2b. CCD pass: detect collisions for fast-moving bodies
        // Run CCD for bodies with velocity exceeding a threshold
        let ccd_velocity_threshold = self.config.max_velocity_per_substep * 0.5;
        let mut ccd_manifolds = Vec::new();

        for (i, collider_a) in self.colliders.iter().enumerate() {
            let body_a = match self.body_id_map.get(&collider_a.body_id) {
                Some(&idx) => &self.bodies[idx],
                None => continue,
            };

            // Skip if body is not fast enough for CCD
            let speed_a = if body_a.is_3d {
                body_a.velocity_3d.length()
            } else {
                body_a.velocity.length()
            };
            if speed_a < ccd_velocity_threshold {
                continue;
            }

            // Check against all other colliders
            for (j, collider_b) in self.colliders.iter().enumerate() {
                if i >= j {
                    continue;
                } // Avoid duplicate checks

                if !collider_a.can_collide_with(collider_b) {
                    continue;
                }

                let body_b = match self.body_id_map.get(&collider_b.body_id) {
                    Some(&idx) => &self.bodies[idx],
                    None => continue,
                };

                if body_a.is_sleeping && body_b.is_sleeping {
                    continue;
                }

                // Run CCD
                let vel_a = if body_a.is_3d {
                    body_a.velocity_3d
                } else {
                    body_a.velocity.extend(0.0)
                };
                let vel_b = if body_b.is_3d {
                    body_b.velocity_3d
                } else {
                    body_b.velocity.extend(0.0)
                };
                let pos_a = if body_a.is_3d {
                    body_a.position_3d
                } else {
                    body_a.position.extend(0.0)
                };
                let pos_b = if body_b.is_3d {
                    body_b.position_3d
                } else {
                    body_b.position.extend(0.0)
                };
                let rot_a = if body_a.is_3d {
                    body_a.rotation
                } else {
                    Quat::from_rotation_z(body_a.angle)
                };
                let rot_b = if body_b.is_3d {
                    body_b.rotation
                } else {
                    Quat::from_rotation_z(body_b.angle)
                };

                let radius_a = collider_a.shape.bounding_radius();
                let radius_b = collider_b.shape.bounding_radius();

                if let Some(ccd_result) = if body_a.is_3d || body_b.is_3d {
                    gjk_ccd_3d(
                        &collider_a.shape,
                        pos_a,
                        &rot_a,
                        vel_a,
                        &collider_b.shape,
                        pos_b,
                        &rot_b,
                        vel_b,
                        radius_a,
                        radius_b,
                    )
                } else {
                    gjk_ccd(
                        &collider_a.shape,
                        pos_a.truncate(),
                        body_a.angle,
                        vel_a.truncate(),
                        &collider_b.shape,
                        pos_b.truncate(),
                        body_b.angle,
                        vel_b.truncate(),
                        radius_a,
                        radius_b,
                    )
                    .map(|(toi, point, normal)| CcdResult3D {
                        toi,
                        point: point.extend(0.0),
                        normal: normal.extend(0.0),
                    })
                } {
                    // Convert CCD result to manifold
                    let manifold = ContactManifold {
                        body_a: i,
                        body_b: j,
                        contacts: vec![Contact {
                            point: ccd_result.point.truncate(),
                            normal: ccd_result.normal.truncate(),
                            depth: 0.0, // CCD contacts have zero penetration at TOI
                        }],
                    };
                    ccd_manifolds.push(manifold);

                    // Wake sleeping bodies
                    if let Some(&idx_a) = self.body_id_map.get(&collider_a.body_id) {
                        if self.bodies[idx_a].is_sleeping {
                            // Note: we can't mutate here, will wake in contact resolution
                        }
                    }
                    if let Some(&idx_b) = self.body_id_map.get(&collider_b.body_id) {
                        if self.bodies[idx_b].is_sleeping {
                            // Note: we can't mutate here
                        }
                    }
                }
            }
        }

        // 3. Narrow phase: generate contact manifolds
        let pairs = self.spatial_hash.candidate_pairs();
        let mut manifolds = Vec::new();
        let mut sensor_manifolds = Vec::new();

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

            let is_sensor_pair = col_a.is_sensor || col_b.is_sensor;

            if let Some(manifold) =
                collide(idx_a, &col_a.shape, body_a, idx_b, &col_b.shape, body_b)
            {
                if is_sensor_pair {
                    sensor_manifolds.push(manifold);
                } else {
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
        }

        // Process collision events for BOTH regular and sensor collisions
        let mut all_manifolds = manifolds.clone();
        all_manifolds.extend(sensor_manifolds.iter().cloned());
        self.process_collision_events(&all_manifolds);

        // 4. Solve contacts (apply contact impulses) - ONLY for non-sensor collisions
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
        let broken = self.solver.solve(
            &mut self.constraints,
            &mut self.bodies,
            &self.body_id_map,
            dt,
        );

        if !broken.is_empty() {
            if let Some(ref cb) = self.on_constraint_broken {
                for (body_a, body_b) in broken {
                    cb(body_a, body_b);
                }
            }
        }

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

    #[test]
    fn test_breakable_constraint() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let shape = Shape::circle(16.0);
        let id1 = world.add_body(RigidBody::static_body());
        let mut b2 = RigidBody::new(1.0, &shape);
        b2.position = Vec2::new(10.0, 0.0);
        let id2 = world.add_body(b2);

        // Connect them with a distance constraint (target dist 10)
        // Strain is |current_dist - 10|
        let constraint = Constraint::distance(id1, id2, Vec2::ZERO, Vec2::ZERO, 10.0)
            .with_breaking_threshold(2.0);
        world.add_constraint(constraint);

        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        let broke = Arc::new(AtomicBool::new(false));
        let broke_clone = broke.clone();
        world.on_constraint_broken = Some(Box::new(move |_, _| {
            broke_clone.store(true, Ordering::SeqCst);
        }));

        // Apply a small impulse, shouldn't break
        if let Some(body) = world.body_mut(id2) {
            body.apply_impulse(Vec2::new(1.0, 0.0));
        }
        world.step(1.0 / 60.0);
        assert!(!broke.load(Ordering::SeqCst));
        assert!(world.constraints[0].enabled);

        // Apply a massive impulse that forces displacement beyond 2.0
        if let Some(body) = world.body_mut(id2) {
            body.apply_impulse(Vec2::new(1000.0, 0.0));
        }
        world.step(1.0 / 60.0);

        // The constraint should have broken
        assert!(broke.load(Ordering::SeqCst));
        assert!(!world.constraints[0].enabled);
    }
}
