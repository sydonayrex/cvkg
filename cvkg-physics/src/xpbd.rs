//! XPBD (Extended Position-Based Dynamics) soft body and cloth simulation.
//!
//! This module implements position-based dynamics for deformable bodies
//! using the XPBD formulation, which provides stable, energy-conserving
//! simulation of cloth, soft bodies, and other deformable materials.
//!
//! ## Architecture
//!
//! ```text
//! SoftBodyWorld
//!     ├── particles: Vec<Particle>        // mass points with position, velocity
//!     ├── constraints: Vec<Constraint>    // distance, bending, tether, etc.
//!     ├── collision_constraints: Vec<>    // particle-triangle, particle-sphere
//!     └── solver: XpbdSolver             // Gauss-Seidel with compliance
//! ```
//!
//! ## XPBD Formulation
//!
//! Standard PBD solves position constraints directly. XPBD extends this with:
//! - **Compliance** (α = 1/k): inverse stiffness, allows soft constraints
//! - **Damping** (β): velocity-based damping within the constraint solve
//! - **Position correction**: Δx = -α * λ * ∇C - β * (v·∇C) * ∇C
//! - **Lagrange multiplier**: λ = -C / (|∇C|² * α + β * dt)
//!
//! This provides unconditional stability and energy conservation.

use glam::{Vec2, Vec3};

/// A particle in the soft body simulation.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Current position.
    pub position: Vec2,
    /// Previous position (for velocity estimation).
    pub prev_position: Vec2,
    /// Velocity (derived from position difference).
    pub velocity: Vec2,
    /// Accumulated external forces (gravity, wind, etc.).
    pub force: Vec2,
    /// Inverse mass (0 = fixed/kinematic).
    pub inv_mass: f32,
    /// User data index (for attaching to rigid bodies, etc.).
    pub user_data: Option<u32>,
}

impl Particle {
    /// Create a new particle at the given position with mass.
    pub fn new(position: Vec2, mass: f32) -> Self {
        let inv_mass = if mass > 0.0 && mass.is_finite() {
            1.0 / mass
        } else {
            0.0
        };
        Self {
            position,
            prev_position: position,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            inv_mass,
            user_data: None,
        }
    }

    /// Create a fixed particle (infinite mass).
    pub fn fixed(position: Vec2) -> Self {
        Self {
            position,
            prev_position: position,
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            inv_mass: 0.0,
            user_data: None,
        }
    }

    /// Estimated velocity from position difference.
    pub fn estimated_velocity(&self, dt: f32) -> Vec2 {
        if dt > 0.0 {
            (self.position - self.prev_position) / dt
        } else {
            Vec2::ZERO
        }
    }
}

/// 3D particle variant.
#[derive(Debug, Clone)]
pub struct Particle3D {
    pub position: Vec3,
    pub prev_position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
    pub inv_mass: f32,
    pub user_data: Option<u32>,
}

impl Particle3D {
    pub fn new(position: Vec3, mass: f32) -> Self {
        let inv_mass = if mass > 0.0 && mass.is_finite() {
            1.0 / mass
        } else {
            0.0
        };
        Self {
            position,
            prev_position: position,
            velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            inv_mass,
            user_data: None,
        }
    }

    pub fn fixed(position: Vec3) -> Self {
        Self {
            position,
            prev_position: position,
            velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            inv_mass: 0.0,
            user_data: None,
        }
    }

    pub fn estimated_velocity(&self, dt: f32) -> Vec3 {
        if dt > 0.0 {
            (self.position - self.prev_position) / dt
        } else {
            Vec3::ZERO
        }
    }
}

/// Soft body constraint types for XPBD.
#[derive(Debug, Clone)]
pub enum SoftConstraint {
    /// Distance constraint between two particles (spring).
    Distance {
        particle_a: usize,
        particle_b: usize,
        rest_length: f32,
        compliance: f32, // α = 1/stiffness
        damping: f32,    // β for velocity damping
    },
    /// Bending constraint between three particles (angle).
    Bending {
        particle_a: usize,
        particle_b: usize, // center
        particle_c: usize,
        rest_angle: f32,
        compliance: f32,
        damping: f32,
    },
    /// Tether constraint: limits max distance between two particles.
    Tether {
        particle_a: usize,
        particle_b: usize,
        max_distance: f32,
        compliance: f32,
    },
    /// Volume/area constraint for closed soft bodies (pressure).
    Volume {
        particle_indices: Vec<usize>,
        rest_volume: f32,
        compliance: f32,
        pressure: f32,
    },
    /// Attachment constraint: particle follows a target position.
    Attachment {
        particle: usize,
        target: Vec2,
        compliance: f32,
        damping: f32,
    },
    /// Pin constraint: particle is fixed in space.
    Pin { particle: usize },
}

/// 3D soft body constraints.
#[derive(Debug, Clone)]
pub enum SoftConstraint3D {
    Distance {
        particle_a: usize,
        particle_b: usize,
        rest_length: f32,
        compliance: f32,
        damping: f32,
    },
    Bending {
        particle_a: usize,
        particle_b: usize,
        particle_c: usize,
        rest_angle: f32,
        compliance: f32,
        damping: f32,
    },
    Tether {
        particle_a: usize,
        particle_b: usize,
        max_distance: f32,
        compliance: f32,
    },
    Volume {
        particle_indices: Vec<usize>,
        rest_volume: f32,
        compliance: f32,
        pressure: f32,
    },
    Attachment {
        particle: usize,
        target: Vec3,
        compliance: f32,
        damping: f32,
    },
    Pin {
        particle: usize,
    },
    /// Triangle collision constraint (particle vs triangle).
    TriangleCollision {
        particle: usize,
        tri_a: usize,
        tri_b: usize,
        tri_c: usize,
        compliance: f32,
        friction: f32,
    },
}

/// XPBD solver configuration.
#[derive(Debug, Clone)]
pub struct XpbdSolverConfig {
    /// Number of solver iterations per substep.
    pub iterations: usize,
    /// Substeps per frame (more = more stable).
    pub substeps: u32,
    /// Global gravity.
    pub gravity: Vec2,
    /// Global damping factor (air resistance).
    pub damping: f32,
    /// Sleep threshold for particles.
    pub sleep_threshold: f32,
    /// Minimum substep duration to prevent division by zero.
    pub min_sub_dt: f32,
    /// Regularization epsilon for XPBD constraint solves.
    pub epsilon: f32,
}

impl Default for XpbdSolverConfig {
    fn default() -> Self {
        Self {
            iterations: 4,
            substeps: 4,
            gravity: Vec2::ZERO,
            damping: 0.01,
            sleep_threshold: 0.001,
            min_sub_dt: 1e-6,
            epsilon: 1e-6,
        }
    }
}

/// Main XPBD solver using Gauss-Seidel iteration.
#[derive(Debug, Default)]
pub struct XpbdSolver {
    config: XpbdSolverConfig,
}

impl XpbdSolver {
    pub fn new(config: XpbdSolverConfig) -> Self {
        Self { config }
    }

    /// Solve all constraints for one time step.
    pub fn solve(&self, particles: &mut [Particle], constraints: &mut [SoftConstraint], dt: f32) {
        let sub_dt = dt / self.config.substeps as f32;

        for _ in 0..self.config.substeps {
            // 1. Apply external forces and predict positions
            for p in particles.iter_mut() {
                if p.inv_mass > 0.0 {
                    p.velocity += (p.force * p.inv_mass + self.config.gravity) * sub_dt;
                    p.velocity *= 1.0 / (1.0 + sub_dt * self.config.damping);
                    p.prev_position = p.position;
                    p.position += p.velocity * sub_dt;
                    p.force = Vec2::ZERO;
                }
            }

            // 2. Solve constraints (Gauss-Seidel)
            for _ in 0..self.config.iterations {
                for constraint in constraints.iter_mut() {
                    self.solve_constraint(constraint, particles, sub_dt);
                }
            }

            // 3. Update velocities from position changes
            for p in particles.iter_mut() {
                if p.inv_mass > 0.0 {
                    p.velocity = (p.position - p.prev_position) / sub_dt;
                }
            }

            // 4. Sleep detection
            for p in particles.iter_mut() {
                if p.inv_mass > 0.0
                    && p.velocity.length_squared()
                        < self.config.sleep_threshold * self.config.sleep_threshold
                {
                    p.velocity = Vec2::ZERO;
                    p.prev_position = p.position;
                }
            }
        }
    }

    fn solve_constraint(
        &self,
        constraint: &mut SoftConstraint,
        particles: &mut [Particle],
        dt: f32,
    ) {
        match constraint {
            SoftConstraint::Distance {
                particle_a,
                particle_b,
                rest_length,
                compliance,
                damping,
            } => {
                let [pa, pb] = Self::get_two_mut(particles, *particle_a, *particle_b);
                let delta = pb.position - pa.position;
                let dist = delta.length();
                if dist < 1e-10 {
                    return;
                }
                let dir = delta / dist;
                let c = dist - *rest_length;

                // Effective mass
                let w_sum = pa.inv_mass + pb.inv_mass;
                if w_sum < 1e-10 {
                    return;
                }

                // XPBD lambda = -C / (|∇C|² * α + β * dt + ε)
                // The epsilon term prevents division by zero and acts as a
                // regularization for near-rigid constraints
                let epsilon = self.config.epsilon;
                let alpha = *compliance;
                let beta = *damping;
                let grad_sq = w_sum;
                let denom = grad_sq * alpha + beta * dt + epsilon;
                let lambda = -c / denom;

                // Position correction: delta_x = lambda * dir * inv_mass
                // Clamp per-particle correction to prevent overshoot.
                // Each particle moves at most 40% of the current distance
                // per iteration, which guarantees convergence for
                // reasonable compliance values.
                let raw_correction_a = lambda * pa.inv_mass;
                let raw_correction_b = lambda * pb.inv_mass;
                let max_a = dist * 0.4;
                let max_b = dist * 0.4;
                let corr_a = raw_correction_a.clamp(-max_a, max_a);
                let corr_b = raw_correction_b.clamp(-max_b, max_b);
                pa.position -= dir * corr_a;
                pb.position += dir * corr_b;
            }
            SoftConstraint::Bending {
                particle_a,
                particle_b,
                particle_c,
                rest_angle,
                compliance,
                damping,
            } => {
                let Some([pa, pb, pc]) =
                    Self::get_three_mut(particles, *particle_a, *particle_b, *particle_c)
                else {
                    return;
                };
                let ab = pa.position - pb.position;
                let cb = pc.position - pb.position;
                let ab_len = ab.length();
                let cb_len = cb.length();
                if ab_len < 1e-10 || cb_len < 1e-10 {
                    return;
                }
                let cos_angle = (ab.dot(cb)) / (ab_len * cb_len).max(1e-10);
                let angle = cos_angle.clamp(-1.0, 1.0).acos();
                let c = angle - *rest_angle;
                if c.abs() < 1e-6 {
                    return;
                }

                // Gradient of angle constraint
                // Skip if particles are nearly colinear (sin_angle ~ 0)
                // to avoid numerical instability
                let sin_angle = (1.0 - cos_angle * cos_angle).max(0.0).sqrt();
                if sin_angle < 1e-3 {
                    return;
                }
                let inv_sin = 1.0 / sin_angle;

                let grad_a = (cb / cb_len - cos_angle * ab / ab_len) * inv_sin / ab_len;
                let grad_c = (ab / ab_len - cos_angle * cb / cb_len) * inv_sin / cb_len;
                let grad_b = -(grad_a + grad_c);

                let w_sum = pa.inv_mass * grad_a.length_squared()
                    + pb.inv_mass * grad_b.length_squared()
                    + pc.inv_mass * grad_c.length_squared();
                if w_sum < 1e-10 {
                    return;
                }

                let alpha = *compliance;
                let beta = *damping;
                let epsilon = 1e-4_f32;
                let lambda = -c / (w_sum * alpha + beta * dt + epsilon);

                pa.position += grad_a * (lambda * pa.inv_mass);
                pb.position += grad_b * (lambda * pb.inv_mass);
                pc.position += grad_c * (lambda * pc.inv_mass);
            }
            SoftConstraint::Tether {
                particle_a,
                particle_b,
                max_distance,
                compliance,
            } => {
                let [pa, pb] = Self::get_two_mut(particles, *particle_a, *particle_b);
                let delta = pb.position - pa.position;
                let dist = delta.length();
                if dist <= *max_distance {
                    return;
                }
                let dir = delta / dist;
                let c = dist - *max_distance;

                let w_sum = pa.inv_mass + pb.inv_mass;
                if w_sum < 1e-10 {
                    return;
                }

                let alpha = *compliance;
                let epsilon = 1e-4_f32;
                let lambda = -c / (w_sum * alpha + epsilon);

                let correction = dir * lambda;
                pa.position -= correction * pa.inv_mass;
                pb.position += correction * pb.inv_mass;
            }
            SoftConstraint::Volume {
                particle_indices,
                rest_volume,
                compliance,
                pressure,
            } => {
                if particle_indices.len() < 3 {
                    return;
                }
                // Compute signed area (2D volume)
                let mut area = 0.0;
                let n = particle_indices.len();
                for i in 0..n {
                    let a = particles[particle_indices[i]].position;
                    let b = particles[particle_indices[(i + 1) % n]].position;
                    area += a.x * b.y - a.y * b.x;
                }
                area *= 0.5;
                let c = area - *rest_volume;

                // Gradient: ∇C_i = 0.5 * (p_{i+1} - p_{i-1})^perp
                let mut grads = vec![Vec2::ZERO; n];
                for i in 0..n {
                    let prev = particles[particle_indices[(i + n - 1) % n]].position;
                    let next = particles[particle_indices[(i + 1) % n]].position;
                    let edge = next - prev;
                    grads[i] = Vec2::new(-edge.y, edge.x) * 0.5;
                }

                let w_sum: f32 = particle_indices
                    .iter()
                    .enumerate()
                    .map(|(i, &idx)| particles[idx].inv_mass * grads[i].length_squared())
                    .sum();
                if w_sum < 1e-10 {
                    return;
                }

                let alpha = *compliance;
                let epsilon = 1e-4_f32;
                let lambda = -c / (w_sum * alpha + epsilon) * *pressure;

                for (i, &idx) in particle_indices.iter().enumerate() {
                    particles[idx].position += grads[i] * (lambda * particles[idx].inv_mass);
                }
            }
            SoftConstraint::Attachment {
                particle,
                target,
                compliance,
                damping,
            } => {
                let p = &mut particles[*particle];
                if p.inv_mass < 1e-10 {
                    return;
                }
                let delta = *target - p.position;
                let c = delta.length();
                if c < 1e-10 {
                    return;
                }
                let dir = delta / c;

                let w = p.inv_mass;
                let alpha = *compliance;
                let beta = *damping;
                let epsilon = 1e-4_f32;
                let lambda = -c / (w * alpha + beta * dt + epsilon);

                p.position += dir * (lambda * w);
            }
            SoftConstraint::Pin { particle } => {
                // Pin constraint: force particle back to its previous position
                // (effectively infinite mass)
                let p = &mut particles[*particle];
                p.position = p.prev_position;
                p.velocity = Vec2::ZERO;
            }
        }
    }

    // Helper to get two mutable references
    fn get_two_mut(particles: &mut [Particle], a: usize, b: usize) -> [&mut Particle; 2] {
        assert_ne!(a, b);
        if a < b {
            let (left, right) = particles.split_at_mut(b);
            [&mut left[a], &mut right[0]]
        } else {
            let (left, right) = particles.split_at_mut(a);
            [&mut right[0], &mut left[b]]
        }
    }

    // Helper to get three mutable references safely
    // Uses checked index access + split_at_mut pattern for safe disjoint borrowing
    fn get_three_mut(
        particles: &mut [Particle],
        a: usize,
        b: usize,
        c: usize,
    ) -> Option<[&mut Particle; 3]> {
        if a == b || a == c || b == c {
            return None;
        }
        let len = particles.len();
        if a >= len || b >= len || c >= len {
            return None;
        }
        // Safe: we verify all indices are disjoint and in bounds above.
        // Use raw pointer only after validation -- this is the same safety
        // contract as get_disjoint_mut but works on all Rust versions.
        //
        // SAFETY: a, b, c are distinct and < len, so ptr.add(a), ptr.add(b),
        // ptr.add(c) are non-overlapping valid references.
        Some(unsafe {
            let ptr = particles.as_mut_ptr();
            [&mut *ptr.add(a), &mut *ptr.add(b), &mut *ptr.add(c)]
        })
    }
}

/// Cloth grid layout parameters.
pub struct ClothGrid {
    pub origin: Vec2,
    pub width: usize,
    pub height: usize,
    pub spacing: f32,
}

/// Cloth material parameters.
pub struct ClothMaterial {
    pub mass: f32,
    pub structural_stiffness: f32,
    pub bending_stiffness: f32,
}

/// Soft body world: owns particles and constraints, runs simulation.
#[derive(Debug, Default)]
pub struct SoftBodyWorld {
    particles: Vec<Particle>,
    constraints: Vec<SoftConstraint>,
    solver: XpbdSolver,
    #[allow(dead_code)]
    config: XpbdSolverConfig,
}

impl SoftBodyWorld {
    pub fn new(config: XpbdSolverConfig) -> Self {
        let solver = XpbdSolver::new(config.clone());
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            solver,
            config,
        }
    }

    /// Add a particle, return its index.
    pub fn add_particle(&mut self, particle: Particle) -> usize {
        self.particles.push(particle);
        self.particles.len() - 1
    }

    /// Add a constraint.
    pub fn add_constraint(&mut self, constraint: SoftConstraint) {
        self.constraints.push(constraint);
    }

    /// Get a particle by index.
    pub fn particle(&self, index: usize) -> Option<&Particle> {
        self.particles.get(index)
    }

    /// Get a mutable particle by index.
    pub fn particle_mut(&mut self, index: usize) -> Option<&mut Particle> {
        self.particles.get_mut(index)
    }

    /// Get all particles.
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Get all constraints.
    pub fn constraints(&self) -> &[SoftConstraint] {
        &self.constraints
    }

    /// Step the simulation.
    pub fn step(&mut self, dt: f32) {
        self.solver
            .solve(&mut self.particles, &mut self.constraints, dt);
    }

    /// Create a cloth grid with distance and bending constraints.
    pub fn create_cloth(&mut self, grid: &ClothGrid, material: &ClothMaterial) -> Vec<usize> {
        let mut indices = Vec::with_capacity(grid.width * grid.height);

        // Create particles
        for y in 0..grid.height {
            for x in 0..grid.width {
                let pos = grid.origin + Vec2::new(x as f32 * grid.spacing, y as f32 * grid.spacing);
                let p = if y == 0 {
                    Particle::fixed(pos)
                } else {
                    Particle::new(pos, material.mass)
                };
                indices.push(self.add_particle(p));
            }
        }

        let idx = |x: usize, y: usize| -> usize { indices[y * grid.width + x] };

        // Structural constraints (horizontal + vertical)
        let compliance = 1.0 / material.structural_stiffness;
        for y in 0..grid.height {
            for x in 0..grid.width {
                if x + 1 < grid.width {
                    self.add_constraint(SoftConstraint::Distance {
                        particle_a: idx(x, y),
                        particle_b: idx(x + 1, y),
                        rest_length: grid.spacing,
                        compliance,
                        damping: 0.01,
                    });
                }
                if y + 1 < grid.height {
                    self.add_constraint(SoftConstraint::Distance {
                        particle_a: idx(x, y),
                        particle_b: idx(x, y + 1),
                        rest_length: grid.spacing,
                        compliance,
                        damping: 0.01,
                    });
                }
            }
        }

        // Bending constraints (diagonals for shear resistance)
        let bend_compliance = 1.0 / material.bending_stiffness;
        for y in 0..grid.height - 1 {
            for x in 0..grid.width - 1 {
                self.add_constraint(SoftConstraint::Distance {
                    particle_a: idx(x, y),
                    particle_b: idx(x + 1, y + 1),
                    rest_length: grid.spacing * 2.0_f32.sqrt(),
                    compliance: bend_compliance,
                    damping: 0.005,
                });
                self.add_constraint(SoftConstraint::Distance {
                    particle_a: idx(x + 1, y),
                    particle_b: idx(x, y + 1),
                    rest_length: grid.spacing * 2.0_f32.sqrt(),
                    compliance: bend_compliance,
                    damping: 0.005,
                });
            }
        }

        // Bending constraints (angle constraints for out-of-plane bending)
        for y in 1..grid.height - 1 {
            for x in 0..grid.width {
                self.add_constraint(SoftConstraint::Bending {
                    particle_a: idx(x, y - 1),
                    particle_b: idx(x, y),
                    particle_c: idx(x, y + 1),
                    rest_angle: std::f32::consts::PI,
                    compliance: bend_compliance,
                    damping: 0.005,
                });
            }
        }
        for y in 0..grid.height {
            for x in 1..grid.width - 1 {
                self.add_constraint(SoftConstraint::Bending {
                    particle_a: idx(x - 1, y),
                    particle_b: idx(x, y),
                    particle_c: idx(x + 1, y),
                    rest_angle: std::f32::consts::PI,
                    compliance: bend_compliance,
                    damping: 0.005,
                });
            }
        }

        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_particle_creation() {
        let p = Particle::new(Vec2::new(100.0, 100.0), 1.0);
        assert_eq!(p.position, Vec2::new(100.0, 100.0));
        assert_eq!(p.inv_mass, 1.0);

        let fixed = Particle::fixed(Vec2::ZERO);
        assert_eq!(fixed.inv_mass, 0.0);
    }

    #[test]
    fn test_distance_constraint_solve() {
        let mut world = SoftBodyWorld::new(XpbdSolverConfig {
            iterations: 32,
            substeps: 12,
            ..Default::default()
        });
        let a = world.add_particle(Particle::new(Vec2::new(0.0, 0.0), 1.0));
        let b = world.add_particle(Particle::new(Vec2::new(20.0, 0.0), 1.0));

        world.add_constraint(SoftConstraint::Distance {
            particle_a: a,
            particle_b: b,
            rest_length: 10.0,
            compliance: 0.001,
            damping: 0.05,
        });

        // Step multiple times to converge
        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }

        let pa = world.particle(a).unwrap();
        let pb = world.particle(b).unwrap();
        let dist = (pb.position - pa.position).length();
        assert!((dist - 10.0).abs() < 0.5, "dist = {}", dist);
    }

    #[test]
    fn test_cloth_creation() {
        let mut world = SoftBodyWorld::new(XpbdSolverConfig::default());
        let grid = ClothGrid {
            origin: Vec2::ZERO,
            width: 5,
            height: 5,
            spacing: 10.0,
        };
        let material = ClothMaterial {
            mass: 1.0,
            structural_stiffness: 1000.0,
            bending_stiffness: 100.0,
        };
        let indices = world.create_cloth(&grid, &material);

        assert_eq!(indices.len(), 25);
        assert!(!world.constraints().is_empty());

        // Top row should be fixed
        for idx in indices.iter().take(5) {
            let p = world.particle(*idx).unwrap();
            assert_eq!(p.inv_mass, 0.0);
        }
    }

    #[test]
    fn test_soft_body_step() {
        let mut world = SoftBodyWorld::new(XpbdSolverConfig {
            gravity: Vec2::new(0.0, 500.0),
            ..Default::default()
        });

        let a = world.add_particle(Particle::new(Vec2::new(0.0, 10.0), 1.0));
        let b = world.add_particle(Particle::fixed(Vec2::new(0.0, 20.0)));

        world.add_constraint(SoftConstraint::Distance {
            particle_a: a,
            particle_b: b,
            rest_length: 10.0,
            compliance: 0.01,
            damping: 0.01,
        });

        let start_y = world.particle(a).unwrap().position.y;

        // Step many times to let gravity pull the particle down
        for _ in 0..30 {
            world.step(1.0 / 60.0);
        }

        let end_y = world.particle(a).unwrap().position.y;
        // Particle should have moved in the direction of gravity (+Y)
        assert!(
            end_y > start_y,
            "Particle should fall: start_y={}, end_y={}",
            start_y,
            end_y
        );
    }
}
