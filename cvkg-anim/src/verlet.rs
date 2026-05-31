/// A single particle in the Verlet integration simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerletParticle {
    /// Current position of the particle in 2D space.
    pub position: [f32; 2],
    /// Previous position of the particle (used to infer velocity vectors).
    pub prev_position: [f32; 2],
    /// Accumulated external force/acceleration (e.g., gravity).
    pub acceleration: [f32; 2],
    /// Whether the particle is pinned in place.
    pub pinned: bool,
}

impl VerletParticle {
    /// Creates a new free particle.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y],
            prev_position: [x, y],
            acceleration: [0.0, 0.0],
            pinned: false,
        }
    }

    /// Creates a pinned particle that does not move under physics.
    pub fn pinned(x: f32, y: f32) -> Self {
        let mut p = Self::new(x, y);
        p.pinned = true;
        p
    }
}

/// A distance constraint between two Verlet particles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceConstraint {
    /// Index of the first particle in the solver array.
    pub p1_idx: usize,
    /// Index of the second particle in the solver array.
    pub p2_idx: usize,
    /// Target distance in pixels.
    pub target_distance: f32,
    /// Elastic stiffness (0.0 to 1.0, where 1.0 is rigid).
    pub stiffness: f32,
}

impl DistanceConstraint {
    /// Creates a new distance constraint.
    pub fn new(p1_idx: usize, p2_idx: usize, target_distance: f32, stiffness: f32) -> Self {
        Self {
            p1_idx,
            p2_idx,
            target_distance,
            stiffness: stiffness.clamp(0.0, 1.0),
        }
    }
}

/// A Verlet integration and Position-Based Dynamics constraint solver.
///
/// # Contract
/// Simulates distance-constrained networks (like ropes, chains, or fabrics) using
/// Verlet integration and iterative relaxation projections. Extremely stable and robust.
#[derive(Debug, Clone, Default)]
pub struct VerletSolver {
    pub particles: Vec<VerletParticle>,
    pub constraints: Vec<DistanceConstraint>,
    /// Gravity vector applied to all free particles.
    pub gravity: [f32; 2],
}

impl VerletSolver {
    /// Creates a new solver with gravity.
    pub fn new(gx: f32, gy: f32) -> Self {
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            gravity: [gx, gy],
        }
    }

    /// Adds a particle and returns its index.
    pub fn add_particle(&mut self, particle: VerletParticle) -> usize {
        self.particles.push(particle);
        self.particles.len() - 1
    }

    /// Adds a distance constraint.
    pub fn add_constraint(&mut self, constraint: DistanceConstraint) {
        self.constraints.push(constraint);
    }

    /// Advance the Verlet simulation.
    pub fn tick(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        // 1. Verlet Integration step
        for p in &mut self.particles {
            if p.pinned {
                continue;
            }

            let temp_x = p.position[0];
            let temp_y = p.position[1];

            // Verlet formula: x_new = x_curr + (x_curr - x_prev) + acc * dt * dt
            let vx = temp_x - p.prev_position[0];
            let vy = temp_y - p.prev_position[1];

            let ax = p.acceleration[0] + self.gravity[0];
            let ay = p.acceleration[1] + self.gravity[1];

            p.position[0] = temp_x + vx + ax * dt * dt;
            p.position[1] = temp_y + vy + ay * dt * dt;

            p.prev_position[0] = temp_x;
            p.prev_position[1] = temp_y;

            // Reset acceleration
            p.acceleration = [0.0, 0.0];
        }

        // 2. Iterative constraint projection
        const ITERATIONS: usize = 4;
        for _ in 0..ITERATIONS {
            for c in &self.constraints {
                let p1 = self.particles[c.p1_idx];
                let p2 = self.particles[c.p2_idx];

                let dx = p2.position[0] - p1.position[0];
                let dy = p2.position[1] - p1.position[1];
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 1e-4 {
                    continue;
                }

                let diff = c.target_distance - dist;
                // Position correction proportional to inverse masses (pinned has infinite mass)
                let percent = (diff / dist) * c.stiffness * 0.5;
                let offset_x = dx * percent;
                let offset_y = dy * percent;

                // Adjust positions
                match (p1.pinned, p2.pinned) {
                    (false, false) => {
                        self.particles[c.p1_idx].position[0] -= offset_x;
                        self.particles[c.p1_idx].position[1] -= offset_y;
                        self.particles[c.p2_idx].position[0] += offset_x;
                        self.particles[c.p2_idx].position[1] += offset_y;
                    }
                    (true, false) => {
                        self.particles[c.p2_idx].position[0] += offset_x * 2.0;
                        self.particles[c.p2_idx].position[1] += offset_y * 2.0;
                    }
                    (false, true) => {
                        self.particles[c.p1_idx].position[0] -= offset_x * 2.0;
                        self.particles[c.p1_idx].position[1] -= offset_y * 2.0;
                    }
                    (true, true) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verlet_rope_simulation() {
        let mut solver = VerletSolver::new(0.0, 10.0); // 10.0 gravity downward

        // Pinned top, hanging particle below
        let top = solver.add_particle(VerletParticle::pinned(0.0, 0.0));
        let bottom = solver.add_particle(VerletParticle::new(0.0, 10.0));

        // Distance constraint of 10.0 units
        solver.add_constraint(DistanceConstraint::new(top, bottom, 10.0, 1.0));

        // Tick simulation
        for _ in 0..10 {
            solver.tick(0.1);
        }

        // Bottom particle position should be exactly at y=10 due to constraint projection
        let bp = solver.particles[bottom].position;
        assert!((bp[0] - 0.0).abs() < 0.01);
        assert!(
            (bp[1] - 10.0).abs() < 0.1,
            "Expected y position near 10, got {}",
            bp[1]
        );
    }
}
