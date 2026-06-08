//! Physics LOD (Level of Detail) and sleep island management.
//!
//! LOD reduces simulation cost for distant or unimportant bodies:
//! - Full: normal simulation
//! - Simplified: reduced substeps, simpler collision
//! - Frozen: body is kinematic, no simulation
//!
//! Sleep islands group connected bodies that share sleep/wake state.
//! When one body in an island moves, the entire island wakes.

use std::collections::{HashMap, HashSet, VecDeque};

use glam::Vec3;

use crate::{BodyId, PhysicsWorld};

/// LOD tier for a physics body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodTier {
    /// Full simulation quality.
    Full,
    /// Reduced quality: fewer substeps, simpler collision.
    Simplified,
    /// Frozen: body is kinematic, no simulation until woken.
    Frozen,
}

/// Configuration for physics LOD.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Distance threshold for Simplified tier (from camera/focus point).
    pub simplified_distance: f32,
    /// Distance threshold for Frozen tier.
    pub frozen_distance: f32,
    /// Substep multiplier for Simplified tier (1 = same, 2 = half, etc.).
    pub simplified_substep_divisor: u32,
    /// Whether to use spatial sleep islands.
    pub use_sleep_islands: bool,
    /// Minimum bodies in an island before sleeping is allowed.
    pub min_island_size: usize,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            simplified_distance: 500.0,
            frozen_distance: 1500.0,
            simplified_substep_divisor: 2,
            use_sleep_islands: true,
            min_island_size: 2,
        }
    }
}

/// Information about a body's LOD state.
#[derive(Debug, Clone)]
pub struct BodyLod {
    pub tier: LodTier,
    pub distance: f32,
}

/// A sleep island: a group of connected bodies that share sleep state.
#[derive(Debug)]
pub struct SleepIsland {
    /// Body IDs in this island.
    pub bodies: HashSet<BodyId>,
    /// Whether the entire island is sleeping.
    pub is_sleeping: bool,
    /// Combined kinetic energy of the island (for sleep threshold).
    pub total_kinetic_energy: f32,
}

/// LOD manager for physics world.
pub struct LodManager {
    config: LodConfig,
    body_lod: HashMap<BodyId, BodyLod>,
    islands: Vec<SleepIsland>,
    body_island_map: HashMap<BodyId, usize>,
}

impl LodManager {
    pub fn new(config: LodConfig) -> Self {
        Self {
            config,
            body_lod: HashMap::new(),
            islands: Vec::new(),
            body_island_map: HashMap::new(),
        }
    }

    /// Update LOD tiers for all bodies based on distance from a focus point.
    pub fn update_lod(&mut self, world: &PhysicsWorld, focus: Vec3) {
        self.body_lod.clear();

        for (body_id, &idx) in world.body_id_map() {
            if let Some(body) = world.bodies().get(idx) {
                let pos = if body.is_3d {
                    body.position_3d
                } else {
                    body.position.extend(0.0)
                };
                let distance = (pos - focus).length();
                let tier = if distance >= self.config.frozen_distance {
                    LodTier::Frozen
                } else if distance >= self.config.simplified_distance {
                    LodTier::Simplified
                } else {
                    LodTier::Full
                };
                self.body_lod.insert(*body_id, BodyLod { tier, distance });
            }
        }
    }

    /// Get the LOD tier for a body.
    pub fn tier(&self, body_id: BodyId) -> LodTier {
        self.body_lod
            .get(&body_id)
            .map(|l| l.tier)
            .unwrap_or(LodTier::Full)
    }

    /// Build sleep islands from the constraint graph.
    pub fn build_islands(&mut self, world: &PhysicsWorld) {
        if !self.config.use_sleep_islands {
            return;
        }

        self.islands.clear();
        self.body_island_map.clear();

        // Build adjacency from constraints
        let mut adjacency: HashMap<BodyId, Vec<BodyId>> = HashMap::new();
        for constraint in world.constraints() {
            if !constraint.enabled {
                continue;
            }
            adjacency
                .entry(constraint.body_a)
                .or_default()
                .push(constraint.body_b);
            adjacency
                .entry(constraint.body_b)
                .or_default()
                .push(constraint.body_a);
        }

        // BFS to find connected components
        let mut visited: HashSet<BodyId> = HashSet::new();

        for (body_id, _) in world.body_id_map() {
            if visited.contains(body_id) {
                continue;
            }

            let mut island_bodies: HashSet<BodyId> = HashSet::new();
            let mut queue: VecDeque<BodyId> = VecDeque::new();
            queue.push_back(*body_id);

            while let Some(current) = queue.pop_front() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current);
                island_bodies.insert(current);

                if let Some(neighbors) = adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            queue.push_back(*neighbor);
                        }
                    }
                }
            }

            if island_bodies.len() >= self.config.min_island_size {
                let island_idx = self.islands.len();
                for bid in &island_bodies {
                    self.body_island_map.insert(*bid, island_idx);
                }
                self.islands.push(SleepIsland {
                    bodies: island_bodies,
                    is_sleeping: false,
                    total_kinetic_energy: 0.0,
                });
            }
        }
    }

    /// Update sleep island states. Bodies in islands sleep/wake together.
    pub fn update_sleep_islands(&mut self, world: &mut PhysicsWorld) {
        for island in &mut self.islands {
            let mut total_ke = 0.0;
            let mut any_awake = false;

            for body_id in &island.bodies {
                if let Some(&idx) = world.body_id_map().get(body_id) {
                    if let Some(body) = world.bodies().get(idx) {
                        let ke = if body.is_3d {
                            0.5 * body.mass * body.velocity_3d.length_squared()
                        } else {
                            0.5 * body.mass * body.velocity.length_squared()
                        };
                        total_ke += ke;
                        if !body.is_sleeping {
                            any_awake = true;
                        }
                    }
                }
            }

            island.total_kinetic_energy = total_ke;
            island.is_sleeping = !any_awake;

            // Apply island sleep state to all bodies
            if island.is_sleeping {
                for body_id in &island.bodies {
                    if let Some(body) = world.body_mut(*body_id) {
                        if !body.is_sleeping {
                            body.is_sleeping = true;
                            body.velocity = glam::Vec2::ZERO;
                            body.velocity_3d = Vec3::ZERO;
                            body.angular_velocity = 0.0;
                            body.angular_velocity_3d = Vec3::ZERO;
                        }
                    }
                }
            }
        }
    }

    /// Wake an entire island when one body is activated.
    pub fn wake_island(&mut self, world: &mut PhysicsWorld, body_id: BodyId) {
        if let Some(&island_idx) = self.body_island_map.get(&body_id) {
            if let Some(island) = self.islands.get_mut(island_idx) {
                island.is_sleeping = false;
                for bid in &island.bodies {
                    if let Some(body) = world.body_mut(*bid) {
                        if body.is_sleeping {
                            body.is_sleeping = false;
                            body.sleep_counter = 0;
                        }
                    }
                }
            }
        }
    }

    /// Get the number of sleep islands.
    pub fn island_count(&self) -> usize {
        self.islands.len()
    }

    /// Get the number of sleeping islands.
    pub fn sleeping_islands(&self) -> usize {
        self.islands.iter().filter(|i| i.is_sleeping).count()
    }

    /// Get body LOD info.
    pub fn body_lod(&self) -> &HashMap<BodyId, BodyLod> {
        &self.body_lod
    }
}
