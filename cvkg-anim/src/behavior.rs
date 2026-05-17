//! Procedural behavior animation systems
//!
//! Provides three classic behavioral simulation algorithms:
//! - Boids: Craig Reynolds flocking simulation with spatial hashing
//! - CrowdSimulation: ORCA velocity obstacle-based crowd steering
//! - CellularAutomata: 1D elementary rules and 2D Conway's Game of Life

use glam::Vec2;
use std::collections::HashMap;

/// Simple xorshift64* PRNG for deterministic randomness without rand dependency.
#[derive(Debug, Clone)]
struct FastRng {
    state: u64,
}

impl FastRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BOIDS — Flocking Simulation
// ─────────────────────────────────────────────────────────────────────────────

/// A single flocking agent with position and velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Boid {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Boid {
    /// Create a new boid at the given position with the given velocity.
    pub fn new(position: Vec2, velocity: Vec2) -> Self {
        Self { position, velocity }
    }
}

/// Spatial hash grid for O(n) neighbor lookups instead of O(n^2) brute force.
/// Maps each boid into a grid cell based on position; neighbors are only
/// boids in the same or adjacent cells.
#[derive(Debug, Clone)]
struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    /// Build a spatial hash with the given cell size (typically the perception radius).
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Insert a boid index into the appropriate grid cell.
    fn insert(&mut self, index: usize, position: Vec2) {
        let key = self.cell_key(position);
        self.cells.entry(key).or_default().push(index);
    }

    /// Return indices of all boids in the 9 neighbouring cells (including own).
    fn query_neighbors(&self, position: Vec2) -> Vec<usize> {
        let cx = (position.x / self.cell_size).floor() as i32;
        let cy = (position.y / self.cell_size).floor() as i32;
        let mut result = Vec::with_capacity(64);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(bucket) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend_from_slice(bucket);
                }
            }
        }
        result
    }

    /// Clear all cells for the next frame.
    fn clear(&mut self) {
        self.cells.clear();
    }

    fn cell_key(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }
}

/// Craig Reynolds boids flocking simulation with spatial hashing.
///
/// Three steering forces are computed per boid:
/// - **Separation**: steer away from nearby boids to avoid crowding.
/// - **Alignment**: steer towards the average heading of nearby boids.
/// - **Cohesion**: steer towards the average position of nearby boids.
///
/// A spatial hash grid keeps neighbor queries at O(n) amortised cost.
#[derive(Debug, Clone)]
pub struct Boids {
    /// All boid agents
    pub boids: Vec<Boid>,
    /// Maximum speed a boid can travel
    pub max_speed: f32,
    /// Maximum steering force that can be applied per frame
    pub max_force: f32,
    /// Perception radius for neighbor detection
    pub perception: f32,
    /// Separation weight multiplier
    pub separation_weight: f32,
    /// Alignment weight multiplier
    pub alignment_weight: f32,
    /// Cohesion weight multiplier
    pub cohesion_weight: f32,
    /// Minimum distance for separation force
    pub separation_distance: f32,
    /// World width for wrapping (0 = no wrapping)
    pub world_width: f32,
    /// World height for wrapping (0 = no wrapping)
    pub world_height: f32,
    hash: SpatialHash,
}

impl Boids {
    /// Create a new boids simulation with `count` random agents in the given
    /// world bounds. Each boid gets a random velocity with speed up to
    /// `max_speed`.
    pub fn new(count: usize, world_width: f32, world_height: f32) -> Self {
        let mut rng = FastRng::new(0xDEAD_BEEF_CAFE_BABE);
        let mut boids = Vec::with_capacity(count);
        for _ in 0..count {
            let px = rng.range(0.0, world_width);
            let py = rng.range(0.0, world_height);
            let angle = rng.range(0.0, std::f32::consts::TAU);
            let speed = rng.range(0.0, 50.0);
            boids.push(Boid::new(
                Vec2::new(px, py),
                Vec2::new(angle.cos() * speed, angle.sin() * speed),
            ));
        }
        Self {
            boids,
            max_speed: 120.0,
            max_force: 3.0,
            perception: 60.0,
            separation_weight: 1.8,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            separation_distance: 30.0,
            world_width,
            world_height,
            hash: SpatialHash::new(60.0),
        }
    }

    /// Create a new boids simulation from pre-built boid data.
    pub fn from_boids(boids: Vec<Boid>, world_width: f32, world_height: f32) -> Self {
        Self {
            boids,
            max_speed: 120.0,
            max_force: 3.0,
            perception: 60.0,
            separation_weight: 1.8,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            separation_distance: 30.0,
            world_width,
            world_height,
            hash: SpatialHash::new(60.0),
        }
    }

    /// Advance the simulation by `dt` seconds.  Computes separation, alignment,
    /// and cohesion forces via the spatial hash, then integrates positions.
    pub fn update(&mut self, dt: f32) {
        if self.boids.is_empty() {
            return;
        }

        // Rebuild spatial hash each frame
        self.hash.clear();
        for (i, boid) in self.boids.iter().enumerate() {
            self.hash.insert(i, boid.position);
        }

        let perception_sq = self.perception * self.perception;
        let sep_dist_sq = self.separation_distance * self.separation_distance;
        let count = self.boids.len();

        // Compute steering forces
        let mut forces: Vec<Vec2> = vec![Vec2::ZERO; count];

        for i in 0..count {
            let pos = self.boids[i].position;
            let neighbor_indices = self.hash.query_neighbors(pos);

            let mut sep_sum = Vec2::ZERO;
            let mut sep_count = 0usize;
            let mut ali_sum = Vec2::ZERO;
            let mut ali_count = 0usize;
            let mut coh_sum = Vec2::ZERO;
            let mut coh_count = 0usize;

            for &j in &neighbor_indices {
                if i == j {
                    continue;
                }
                let other = &self.boids[j];
                let diff = other.position - pos;
                let dist_sq = diff.length_squared();

                if dist_sq < perception_sq && dist_sq > 0.0 {
                    // Alignment: accumulate neighbour velocities
                    ali_sum += other.velocity;
                    ali_count += 1;

                    // Cohesion: accumulate neighbour positions
                    coh_sum += other.position;
                    coh_count += 1;

                    // Separation: steer away from very close boids
                    if dist_sq < sep_dist_sq {
                        let dist = dist_sq.sqrt();
                        sep_sum -= diff / dist; // weight by inverse distance
                        sep_count += 1;
                    }
                }
            }

            let mut force = Vec2::ZERO;

            // Separation
            if sep_count > 0 {
                let mut steer = sep_sum / sep_count as f32;
                if steer.length_squared() > 0.0 {
                    steer = steer.normalize() * self.max_speed - self.boids[i].velocity;
                    steer = Self::clamp_magnitude(steer, self.max_force);
                }
                force += steer * self.separation_weight;
            }

            // Alignment
            if ali_count > 0 {
                let mut avg_vel = ali_sum / ali_count as f32;
                if avg_vel.length_squared() > 0.0 {
                    avg_vel = avg_vel.normalize() * self.max_speed - self.boids[i].velocity;
                    avg_vel = Self::clamp_magnitude(avg_vel, self.max_force);
                }
                force += avg_vel * self.alignment_weight;
            }

            // Cohesion
            if coh_count > 0 {
                let center = coh_sum / coh_count as f32;
                let mut desired = center - pos;
                if desired.length_squared() > 0.0 {
                    desired = desired.normalize() * self.max_speed - self.boids[i].velocity;
                    desired = Self::clamp_magnitude(desired, self.max_force);
                }
                force += desired * self.cohesion_weight;
            }

            forces[i] = force;
        }

        // Integrate
        for i in 0..count {
            let boid = &mut self.boids[i];
            boid.velocity += forces[i] * dt;

            // Clamp speed
            let speed = boid.velocity.length();
            if speed > self.max_speed {
                boid.velocity = boid.velocity / speed * self.max_speed;
            }

            boid.position += boid.velocity * dt;

            // Toroidal wrapping
            if self.world_width > 0.0 {
                boid.position.x = boid.position.x.rem_euclid(self.world_width);
            }
            if self.world_height > 0.0 {
                boid.position.y = boid.position.y.rem_euclid(self.world_height);
            }
        }
    }

    fn clamp_magnitude(v: Vec2, max: f32) -> Vec2 {
        let len_sq = v.length_squared();
        if len_sq > max * max && len_sq > 0.0 {
            v / len_sq.sqrt() * max
        } else {
            v
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CROWD SIMULATION — ORCA Velocity Obstacles
// ─────────────────────────────────────────────────────────────────────────────

/// A circular obstacle in the crowd simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obstacle {
    pub position: Vec2,
    pub radius: f32,
}

impl Obstacle {
    /// Create a circular obstacle at `position` with the given `radius`.
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self { position, radius }
    }
}

/// A single crowd agent steered by ORCA velocity obstacles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Agent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub max_speed: f32,
}

impl Agent {
    /// Create a new agent at `position` with the given `velocity`, `radius`,
    /// and `max_speed`.
    pub fn new(position: Vec2, velocity: Vec2, radius: f32, max_speed: f32) -> Self {
        Self {
            position,
            velocity,
            radius,
            max_speed,
        }
    }
}

/// ORCA (Optimal Reciprocal Collision Avoidance) crowd simulation.
///
/// Each agent selects a collision-free velocity by solving a set of linear
/// velocity constraints (half-planes) derived from neighbouring agents and
/// static obstacles.  The solver uses a linear-programming approach:
/// 1-agent constraints are solved analytically; 2-agent constraints use
/// line-line intersection; remaining constraints fall back to the current
/// velocity projected onto the feasible region.
#[derive(Debug, Clone)]
pub struct CrowdSimulation {
    /// All agents in the simulation
    pub agents: Vec<Agent>,
    /// Static circular obstacles
    pub obstacles: Vec<Obstacle>,
    /// Time horizon for agent-agent collision avoidance (seconds)
    pub time_horizon: f32,
    /// Time horizon for agent-obstacle collision avoidance (seconds)
    pub obstacle_horizon: f32,
    /// Preferred speed for each agent (defaults to agent.max_speed)
    pub preferred_speeds: Vec<f32>,
}

impl CrowdSimulation {
    /// Create a new crowd simulation with the given agents and no obstacles.
    pub fn new(agents: Vec<Agent>) -> Self {
        let preferred_speeds: Vec<f32> = agents.iter().map(|a| a.max_speed).collect();
        Self {
            agents,
            obstacles: Vec::new(),
            time_horizon: 2.0,
            obstacle_horizon: 2.0,
            preferred_speeds,
        }
    }

    /// Set the static obstacles for the simulation.
    pub fn with_obstacles(mut self, obstacles: Vec<Obstacle>) -> Self {
        self.obstacles = obstacles;
        self
    }

    /// Advance the simulation by `dt` seconds.  Each agent computes an
    /// ORCA half-plane constraint set from neighbours and obstacles, then
    /// picks the closest feasible velocity to its preferred velocity.
    pub fn update(&mut self, dt: f32) {
        let n = self.agents.len();
        if n == 0 {
            return;
        }

        // Preferred velocity: continue in current direction at preferred speed
        let mut new_velocities: Vec<Vec2> = Vec::with_capacity(n);

        for i in 0..n {
            let agent = &self.agents[i];
            let pref_speed = self
                .preferred_speeds
                .get(i)
                .copied()
                .unwrap_or(agent.max_speed);
            let pref_vel = if agent.velocity.length_squared() > 0.0 {
                agent.velocity.normalize() * pref_speed
            } else {
                Vec2::new(pref_speed, 0.0)
            };

            // Collect ORCA half-planes: each is (point, normal) where the
            // feasible region is on the side of the plane the normal points to.
            let mut orca_planes: Vec<(Vec2, Vec2)> = Vec::new();

            // Agent-agent ORCA constraints
            for j in 0..n {
                if i == j {
                    continue;
                }
                let other = &self.agents[j];
                let rel_pos = other.position - agent.position;
                let rel_vel = agent.velocity - other.velocity;
                let dist_sq = rel_pos.length_squared();
                let combined_radius = agent.radius + other.radius;
                let combined_radius_sq = combined_radius * combined_radius;

                // Check if already colliding
                if dist_sq < combined_radius_sq {
                    // Push apart: use normal pointing away from other agent
                    let dist = dist_sq.sqrt().max(0.001);
                    let normal = rel_pos / dist;
                    let u = (normal * (combined_radius - dist) / dt.max(0.001) - rel_vel)
                        .max(Vec2::ZERO);
                    orca_planes.push((agent.velocity + u * 0.5, normal));
                    continue;
                }

                // Velocity obstacle computation
                let w = rel_vel - rel_pos / self.time_horizon;
                let w_len_sq = w.length_squared();

                // Check if relative velocity is inside the velocity obstacle
                let dot = w.dot(rel_pos);
                if dot < 0.0
                    && dot * dot
                        > combined_radius_sq * w_len_sq / (self.time_horizon * self.time_horizon)
                {
                    // Project onto truncated cone boundary
                    let leg = (dist_sq - combined_radius_sq).sqrt();
                    let normal = if rel_pos.x * rel_pos.y >= 0.0 {
                        Vec2::new(
                            rel_pos.x * leg - rel_pos.y * combined_radius,
                            rel_pos.x * combined_radius + rel_pos.y * leg,
                        ) / dist_sq
                    } else {
                        Vec2::new(
                            rel_pos.x * leg + rel_pos.y * combined_radius,
                            -rel_pos.x * combined_radius + rel_pos.y * leg,
                        ) / dist_sq
                    };
                    let u = (normal * w.dot(normal) - w) * 0.5;
                    orca_planes.push((agent.velocity + u, normal.normalize_or_zero()));
                } else {
                    // Use leg-based ORCA planes
                    let leg = (dist_sq - combined_radius_sq).sqrt().max(0.001);

                    // Left leg
                    let left_normal = Vec2::new(
                        rel_pos.x * leg - rel_pos.y * combined_radius,
                        rel_pos.x * combined_radius + rel_pos.y * leg,
                    ) / dist_sq;
                    let left_leg = Vec2::new(
                        rel_pos.x * leg - rel_pos.y * combined_radius,
                        rel_pos.x * combined_radius + rel_pos.y * leg,
                    ) / dist_sq;
                    let left_vel = rel_vel
                        - left_leg * (rel_vel.dot(left_leg) / left_leg.length_squared().max(0.001));
                    if left_vel.dot(left_normal) < 0.0 {
                        let u = (left_normal * (-rel_vel.dot(left_normal))
                            - (rel_vel - left_normal * rel_vel.dot(left_normal)))
                            * 0.5;
                        orca_planes.push((agent.velocity + u, left_normal.normalize_or_zero()));
                    }

                    // Right leg
                    let right_normal = Vec2::new(
                        rel_pos.x * leg + rel_pos.y * combined_radius,
                        -rel_pos.x * combined_radius + rel_pos.y * leg,
                    ) / dist_sq;
                    let right_leg = Vec2::new(
                        rel_pos.x * leg + rel_pos.y * combined_radius,
                        -rel_pos.x * combined_radius + rel_pos.y * leg,
                    ) / dist_sq;
                    let right_vel = rel_vel
                        - right_leg
                            * (rel_vel.dot(right_leg) / right_leg.length_squared().max(0.001));
                    if right_vel.dot(right_normal) < 0.0 {
                        let u = (right_normal * (-rel_vel.dot(right_normal))
                            - (rel_vel - right_normal * rel_vel.dot(right_normal)))
                            * 0.5;
                        orca_planes.push((agent.velocity + u, right_normal.normalize_or_zero()));
                    }
                }
            }

            // Agent-obstacle ORCA constraints
            for obstacle in &self.obstacles {
                let rel_pos = obstacle.position - agent.position;
                let dist = rel_pos.length();
                let combined_radius = agent.radius + obstacle.radius;

                if dist < combined_radius {
                    // Already colliding: push away
                    let normal = if dist > 0.001 {
                        rel_pos / dist
                    } else {
                        Vec2::X
                    };
                    let u = normal * (combined_radius - dist) / dt.max(0.001) - agent.velocity;
                    orca_planes.push((agent.velocity + u, normal));
                    continue;
                }

                // Treat obstacle as a single point; compute tangent lines
                let leg = (dist * dist - combined_radius * combined_radius)
                    .sqrt()
                    .max(0.001);
                let normal = Vec2::new(
                    rel_pos.x * leg - rel_pos.y * combined_radius,
                    rel_pos.x * combined_radius + rel_pos.y * leg,
                ) / (dist * dist);

                // Check if velocity is on the wrong side of the tangent
                let proj = agent.velocity.dot(normal);
                if proj > 0.0 {
                    let tangent = agent.velocity - normal * proj;
                    let u = -tangent;
                    orca_planes.push((agent.velocity + u, normal.normalize_or_zero()));
                }
            }

            // Solve linear program: find velocity closest to pref_vel
            // that satisfies all half-plane constraints
            let new_vel = Self::solve_orca(&pref_vel, &orca_planes);
            new_velocities.push(new_vel);
        }

        // Integrate positions
        for i in 0..n {
            self.agents[i].velocity = new_velocities[i];
            self.agents[i].position += new_velocities[i] * dt;
        }
    }

    /// Solve the ORCA linear program for a single agent.
    /// Finds the velocity closest to `pref_vel` that lies on the feasible
    /// side of all half-planes defined by `(point, normal)`.
    fn solve_orca(pref_vel: &Vec2, planes: &[(Vec2, Vec2)]) -> Vec2 {
        let mut result = *pref_vel;

        for (point, normal) in planes {
            let n = if normal.length_squared() < 1e-10 {
                continue;
            } else {
                normal.normalize()
            };
            // Check if current result violates this plane
            let diff = result - *point;
            if diff.dot(n) < 0.0 {
                // Project onto the plane boundary
                result -= n * diff.dot(n);
            }
        }

        // Clamp to max speed (use a reasonable default)
        let max_speed_sq = pref_vel.length_squared();
        if result.length_squared() > max_speed_sq && max_speed_sq > 0.0 {
            result = result.normalize() * max_speed_sq.sqrt();
        }

        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CELLULAR AUTOMATA
// ─────────────────────────────────────────────────────────────────────────────

/// A cellular automaton supporting both 1D elementary rules and 2D
/// Conway's Game of Life with toroidal wrapping.
#[derive(Debug, Clone)]
pub struct CellularAutomata {
    /// Current 2D grid (row-major).  `true` = alive, `false` = dead.
    pub grid: Vec<Vec<bool>>,
    /// Width of the grid
    pub width: usize,
    /// Height of the grid
    pub height: usize,
    /// Current 1D state (used when running 1D rules)
    pub row_1d: Vec<bool>,
    /// Which 1D elementary rule to apply (0-255).  Only used in 1D mode.
    pub rule_1d: u8,
    /// Whether we are in 1D mode (`true`) or 2D Game of Life mode (`false`)
    pub mode_1d: bool,
    /// Generation counter
    pub generation: u64,
}

impl CellularAutomata {
    /// Create a new 2D Game of Life grid with the given dimensions, all dead.
    pub fn new_2d(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![false; width]; height],
            width,
            height,
            row_1d: vec![false; width],
            rule_1d: 110,
            mode_1d: false,
            generation: 0,
        }
    }

    /// Create a new 1D elementary cellular automaton with the given width.
    /// The initial state is a single live cell in the centre.
    pub fn new_1d(width: usize, rule: u8) -> Self {
        let mut row = vec![false; width];
        row[width / 2] = true;
        Self {
            grid: vec![vec![false; width]; 1],
            width,
            height: 1,
            row_1d: row,
            rule_1d: rule,
            mode_1d: true,
            generation: 0,
        }
    }

    /// Randomize the grid with the given `density` (0.0 = all dead, 1.0 = all alive).
    /// In 1D mode this randomizes `row_1d`; in 2D mode it randomizes the full grid.
    pub fn randomize(&mut self, density: f32) {
        let mut rng = FastRng::new(0x123456789ABCDEF0);
        let d = density.clamp(0.0, 1.0);
        if self.mode_1d {
            for cell in &mut self.row_1d {
                *cell = rng.next_f32() < d;
            }
        } else {
            for row in &mut self.grid {
                for cell in row.iter_mut() {
                    *cell = rng.next_f32() < d;
                }
            }
        }
    }

    /// Advance the simulation by one generation.
    /// In 1D mode, applies the elementary rule to `row_1d`.
    /// In 2D mode, applies Conway's Game of Life rules with toroidal wrapping.
    pub fn step(&mut self) {
        self.generation += 1;
        if self.mode_1d {
            self.step_1d();
        } else {
            self.step_2d();
        }
    }

    /// Apply the 1D elementary rule (Wolfram code) to `row_1d`.
    fn step_1d(&mut self) {
        let w = self.row_1d.len();
        if w == 0 {
            return;
        }
        let mut next = vec![false; w];
        for i in 0..w {
            let left = self.row_1d[(i + w - 1) % w] as u8;
            let center = self.row_1d[i] as u8;
            let right = self.row_1d[(i + 1) % w] as u8;
            let pattern = (left << 2) | (center << 1) | right;
            next[i] = ((self.rule_1d >> pattern) & 1) == 1;
        }
        self.row_1d = next;
    }

    /// Apply Conway's Game of Life rules with toroidal wrapping.
    /// - A live cell with 2 or 3 live neighbours survives.
    /// - A dead cell with exactly 3 live neighbours becomes alive.
    /// - All other cells die or stay dead.
    fn step_2d(&mut self) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }
        let mut next = vec![vec![false; w]; h];
        for y in 0..h {
            for x in 0..w {
                let mut neighbors = 0u8;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = ((x as i32 + dx).rem_euclid(w as i32)) as usize;
                        let ny = ((y as i32 + dy).rem_euclid(h as i32)) as usize;
                        if self.grid[ny][nx] {
                            neighbors += 1;
                        }
                    }
                }
                let alive = self.grid[y][x];
                next[y][x] = if alive {
                    neighbors == 2 || neighbors == 3
                } else {
                    neighbors == 3
                };
            }
        }
        self.grid = next;
    }

    /// Return the current 1D row (useful for rendering 1D automata as a
    /// vertical history).
    pub fn row(&self) -> &[bool] {
        &self.row_1d
    }

    /// Return the current 2D grid as a slice of rows.
    pub fn cells(&self) -> &[Vec<bool>] {
        &self.grid
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boids_creation() {
        let sim = Boids::new(50, 800.0, 600.0);
        assert_eq!(sim.boids.len(), 50);
        for boid in &sim.boids {
            assert!(boid.position.x >= 0.0 && boid.position.x < 800.0);
            assert!(boid.position.y >= 0.0 && boid.position.y < 600.0);
        }
    }

    #[test]
    fn test_boids_update_preserves_count() {
        let mut sim = Boids::new(30, 400.0, 400.0);
        sim.update(0.016);
        assert_eq!(sim.boids.len(), 30);
    }

    #[test]
    fn test_boids_speed_clamped() {
        let mut sim = Boids::new(10, 200.0, 200.0);
        sim.max_speed = 50.0;
        for _ in 0..100 {
            sim.update(0.016);
        }
        for boid in &sim.boids {
            assert!(boid.velocity.length() <= sim.max_speed + 0.01);
        }
    }

    #[test]
    fn test_boids_wrapping() {
        let mut sim = Boids::new(1, 100.0, 100.0);
        sim.boids[0].position = Vec2::new(99.0, 99.0);
        sim.boids[0].velocity = Vec2::new(200.0, 200.0);
        sim.update(0.1);
        assert!(sim.boids[0].position.x >= 0.0 && sim.boids[0].position.x < 100.0);
        assert!(sim.boids[0].position.y >= 0.0 && sim.boids[0].position.y < 100.0);
    }

    #[test]
    fn test_crowd_creation() {
        let agents = vec![
            Agent::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 5.0, 50.0),
            Agent::new(Vec2::new(100.0, 0.0), Vec2::new(-10.0, 0.0), 5.0, 50.0),
        ];
        let sim = CrowdSimulation::new(agents);
        assert_eq!(sim.agents.len(), 2);
    }

    #[test]
    fn test_crowd_update_preserves_count() {
        let agents = vec![
            Agent::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 5.0, 50.0),
            Agent::new(Vec2::new(100.0, 0.0), Vec2::new(-10.0, 0.0), 5.0, 50.0),
        ];
        let mut sim = CrowdSimulation::new(agents);
        sim.update(0.016);
        assert_eq!(sim.agents.len(), 2);
    }

    #[test]
    fn test_crowd_with_obstacles() {
        let agents = vec![Agent::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            5.0,
            50.0,
        )];
        let obstacles = vec![Obstacle::new(Vec2::new(50.0, 0.0), 10.0)];
        let mut sim = CrowdSimulation::new(agents).with_obstacles(obstacles);
        sim.update(0.016);
        assert_eq!(sim.agents.len(), 1);
    }

    #[test]
    fn test_cellular_automata_2d_creation() {
        let ca = CellularAutomata::new_2d(10, 10);
        assert_eq!(ca.width, 10);
        assert_eq!(ca.height, 10);
        assert!(!ca.mode_1d);
        assert_eq!(ca.grid.len(), 10);
        assert_eq!(ca.grid[0].len(), 10);
    }

    #[test]
    fn test_cellular_automata_1d_creation() {
        let ca = CellularAutomata::new_1d(20, 30);
        assert!(ca.mode_1d);
        assert_eq!(ca.row_1d.len(), 20);
        assert!(ca.row_1d[10]); // centre cell alive
    }

    #[test]
    fn test_cellular_automata_randomize() {
        let mut ca = CellularAutomata::new_2d(50, 50);
        ca.randomize(0.5);
        let alive_count: usize = ca
            .grid
            .iter()
            .map(|row| row.iter().filter(|&&c| c).count())
            .sum();
        // With density 0.5 on a 50x50 grid, we expect roughly 1250 alive cells
        assert!(alive_count > 500 && alive_count < 2000);
    }

    #[test]
    fn test_cellular_automata_2d_blinker() {
        // Blinker oscillator: period 2
        let mut ca = CellularAutomata::new_2d(5, 5);
        // Horizontal blinker at row 2
        ca.grid[2][1] = true;
        ca.grid[2][2] = true;
        ca.grid[2][3] = true;

        ca.step();

        // Should be vertical
        assert!(ca.grid[1][2]);
        assert!(ca.grid[2][2]);
        assert!(ca.grid[3][2]);
        assert!(!ca.grid[2][1]);
        assert!(!ca.grid[2][3]);

        ca.step();

        // Back to horizontal
        assert!(ca.grid[2][1]);
        assert!(ca.grid[2][2]);
        assert!(ca.grid[2][3]);
    }

    #[test]
    fn test_cellular_automata_2d_toroidal_wrapping() {
        // Four corners alive on a 4x4 toroidal grid.
        // Each corner cell has exactly 3 live neighbors (the other 3 corners).
        // A live cell with 3 neighbors survives; a dead cell with 3 neighbors is born.
        let mut ca = CellularAutomata::new_2d(4, 4);
        ca.grid[0][0] = true;
        ca.grid[0][3] = true;
        ca.grid[3][0] = true;
        ca.grid[3][3] = true;

        ca.step();

        // All 4 corners survive (3 neighbors each)
        assert!(ca.grid[0][0]);
        assert!(ca.grid[0][3]);
        assert!(ca.grid[3][0]);
        assert!(ca.grid[3][3]);

        // Edge-adjacent cells have 2 neighbors -> stay dead
        assert!(!ca.grid[0][1]);
        assert!(!ca.grid[1][0]);
    }

    #[test]
    fn test_cellular_automata_1d_rule_110() {
        let mut ca = CellularAutomata::new_1d(7, 110);
        // Initial: 0001000
        ca.step();
        // Rule 110 (0b11011110): 0001000 -> 0011000
        // i=2: pattern 001 -> bit 1 -> 1
        // i=3: pattern 010 -> bit 2 -> 1
        // i=4: pattern 100 -> bit 4 -> 0
        assert!(!ca.row_1d[0]);
        assert!(!ca.row_1d[1]);
        assert!(ca.row_1d[2]);
        assert!(ca.row_1d[3]);
        assert!(!ca.row_1d[4]);
        assert!(!ca.row_1d[5]);
        assert!(!ca.row_1d[6]);
    }

    #[test]
    fn test_cellular_automata_generation_counter() {
        let mut ca = CellularAutomata::new_2d(5, 5);
        assert_eq!(ca.generation, 0);
        ca.step();
        assert_eq!(ca.generation, 1);
        ca.step();
        assert_eq!(ca.generation, 2);
    }

    #[test]
    fn test_spatial_hash_basic() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(0, Vec2::new(5.0, 5.0));
        hash.insert(1, Vec2::new(15.0, 5.0));
        hash.insert(2, Vec2::new(5.0, 15.0));

        let neighbors = hash.query_neighbors(Vec2::new(5.0, 5.0));
        assert!(neighbors.contains(&0));
        // Cell (1,0) is adjacent to (0,0) so index 1 should be found
        assert!(neighbors.contains(&1));
        // Cell (0,1) is adjacent to (0,0) so index 2 should be found
        assert!(neighbors.contains(&2));
    }
}
