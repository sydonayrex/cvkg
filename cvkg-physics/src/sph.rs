//! SPH (Smoothed Particle Hydrodynamics) fluid simulation.
//!
//! Particle-based fluid with density/pressure solver, viscosity, and
//! surface tension. For ink splashes, liquid UI metaphors.

use std::collections::HashMap;

use glam::Vec3;

use crate::PhysicsWorld;

/// SPH fluid configuration.
#[derive(Debug, Clone)]
pub struct SphConfig {
    /// Rest density (kg/m^3 equivalent).
    pub rest_density: f32,
    /// Gas constant for pressure equation.
    pub gas_constant: f32,
    /// Viscosity coefficient.
    pub viscosity: f32,
    /// Surface tension coefficient.
    pub surface_tension: f32,
    /// Smoothing kernel radius.
    pub smoothing_radius: f32,
    /// Particle mass.
    pub particle_mass: f32,
    /// Gravity scale for fluid particles.
    pub gravity_scale: f32,
    /// Maximum particles.
    pub max_particles: usize,
}

impl Default for SphConfig {
    fn default() -> Self {
        Self {
            rest_density: 1000.0,
            gas_constant: 2000.0,
            viscosity: 250.0,
            surface_tension: 0.0728,
            smoothing_radius: 4.0,
            particle_mass: 1.0,
            gravity_scale: 1.0,
            max_particles: 4096,
        }
    }
}

/// A single SPH fluid particle.
#[derive(Debug, Clone)]
pub struct SphParticle {
    /// Position.
    pub position: Vec3,
    /// Velocity.
    pub velocity: Vec3,
    /// Acceleration (accumulated forces).
    pub acceleration: Vec3,
    /// Density.
    pub density: f32,
    /// Pressure.
    pub pressure: f32,
    /// Whether this particle is active.
    pub active: bool,
    /// User data.
    pub user_data: u64,
}

impl SphParticle {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            density: 0.0,
            pressure: 0.0,
            active: true,
            user_data: 0,
        }
    }
}

/// SPH fluid simulation world.
pub struct SphFluid {
    config: SphConfig,
    particles: Vec<SphParticle>,
    /// Spatial hash for neighbor lookup.
    spatial_hash: HashMap<(i32, i32, i32), Vec<usize>>,
    /// Cell size for spatial hash (should be >= smoothing_radius).
    cell_size: f32,
}

impl SphFluid {
    pub fn new(config: SphConfig) -> Self {
        let cell_size = config.smoothing_radius;
        Self {
            config,
            particles: Vec::new(),
            spatial_hash: HashMap::new(),
            cell_size,
        }
    }

    /// Add a particle at the given position.
    pub fn add_particle(&mut self, position: Vec3) -> Option<usize> {
        if self.particles.len() >= self.config.max_particles {
            return None;
        }
        let idx = self.particles.len();
        self.particles.push(SphParticle::new(position));
        Some(idx)
    }

    /// Spawn particles in a sphere.
    pub fn spawn_sphere(&mut self, center: Vec3, radius: f32, count: usize) {
        for i in 0..count {
            if self.particles.len() >= self.config.max_particles {
                break;
            }
            // Fibonacci sphere distribution
            let phi = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt());
            let y = 1.0 - (i as f32 / count as f32) * 2.0;
            let r = (1.0 - y * y).sqrt();
            let theta = phi * i as f32;
            let offset = Vec3::new(r * theta.cos(), y, r * theta.sin()) * radius;
            self.particles.push(SphParticle::new(center + offset));
        }
    }

    /// Spawn particles in a box.
    pub fn spawn_box(&mut self, min: Vec3, max: Vec3, spacing: f32) {
        let mut pos = min;
        while pos.x <= max.x && self.particles.len() < self.config.max_particles {
            pos.y = min.y;
            while pos.y <= max.y && self.particles.len() < self.config.max_particles {
                pos.z = min.z;
                while pos.z <= max.z && self.particles.len() < self.config.max_particles {
                    self.particles.push(SphParticle::new(pos));
                    pos.z += spacing;
                }
                pos.y += spacing;
            }
            pos.x += spacing;
        }
    }

    /// Get the number of active particles.
    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }

    /// Step the simulation forward by dt.
    pub fn step(&mut self, dt: f32, world: &PhysicsWorld) {
        self.build_spatial_hash();
        self.compute_density_pressure();
        self.compute_forces(world);
        self.integrate(dt);
    }

    fn build_spatial_hash(&mut self) {
        self.spatial_hash.clear();
        let cs = self.cell_size;
        for (i, particle) in self.particles.iter().enumerate() {
            if !particle.active {
                continue;
            }
            let cx = (particle.position.x / cs).floor() as i32;
            let cy = (particle.position.y / cs).floor() as i32;
            let cz = (particle.position.z / cs).floor() as i32;
            self.spatial_hash.entry((cx, cy, cz)).or_default().push(i);
        }
    }

    fn compute_density_pressure(&mut self) {
        let h = self.config.smoothing_radius;
        let h2 = h * h;
        let mass = self.config.particle_mass;
        let rest_density = self.config.rest_density;
        let gas_constant = self.config.gas_constant;

        // Poly6 kernel normalization
        let poly6_coeff = 315.0 / (64.0 * std::f32::consts::PI * h * h * h * h * h * h * h * h * h);

        for i in 0..self.particles.len() {
            if !self.particles[i].active {
                continue;
            }
            let pos_i = self.particles[i].position;
            let mut density = 0.0;

            // Check neighboring cells
            let cx = (pos_i.x / self.cell_size).floor() as i32;
            let cy = (pos_i.y / self.cell_size).floor() as i32;
            let cz = (pos_i.z / self.cell_size).floor() as i32;

            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if let Some(neighbors) = self.spatial_hash.get(&(cx + dx, cy + dy, cz + dz))
                        {
                            for &j in neighbors {
                                let diff = pos_i - self.particles[j].position;
                                let r2 = diff.length_squared();
                                if r2 < h2 {
                                    let w = poly6_coeff * (h2 - r2) * (h2 - r2) * (h2 - r2);
                                    density += mass * w;
                                }
                            }
                        }
                    }
                }
            }

            self.particles[i].density = density.max(rest_density * 0.01);
            // Tait equation of state
            self.particles[i].pressure = gas_constant * (density - rest_density);
        }
    }

    fn compute_forces(&mut self, _world: &PhysicsWorld) {
        let h = self.config.smoothing_radius;
        let h2 = h * h;
        let mass = self.config.particle_mass;
        let viscosity = self.config.viscosity;
        let surface_tension = self.config.surface_tension;

        // Spiky kernel gradient normalization
        let spiky_coeff = -45.0 / (std::f32::consts::PI * h * h * h * h * h * h);

        // Viscosity kernel laplacian normalization
        let visc_coeff = 45.0 / (std::f32::consts::PI * h * h * h * h * h * h);

        let gravity = Vec3::new(0.0, -9.81 * self.config.gravity_scale, 0.0);

        for i in 0..self.particles.len() {
            if !self.particles[i].active {
                continue;
            }
            let pos_i = self.particles[i].position;
            let vel_i = self.particles[i].velocity;
            let density_i = self.particles[i].density;
            let pressure_i = self.particles[i].pressure;

            let mut pressure_force = Vec3::ZERO;
            let mut viscosity_force = Vec3::ZERO;
            let mut normal = Vec3::ZERO;

            let cx = (pos_i.x / self.cell_size).floor() as i32;
            let cy = (pos_i.y / self.cell_size).floor() as i32;
            let cz = (pos_i.z / self.cell_size).floor() as i32;

            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if let Some(neighbors) = self.spatial_hash.get(&(cx + dx, cy + dy, cz + dz))
                        {
                            for &j in neighbors {
                                if i == j {
                                    continue;
                                }
                                let diff = pos_i - self.particles[j].position;
                                let r2 = diff.length_squared();
                                if r2 < h2 && r2 > 1e-6 {
                                    let r = r2.sqrt();
                                    let dir = diff / r;

                                    let density_j = self.particles[j].density;
                                    let pressure_j = self.particles[j].pressure;

                                    // Pressure force (symmetric)
                                    let pressure_term =
                                        -mass * (pressure_i + pressure_j) / (2.0 * density_j);
                                    let spiky_grad = spiky_coeff * (h - r) * (h - r);
                                    pressure_force += dir * (pressure_term * spiky_grad);

                                    // Viscosity force
                                    let vel_diff = self.particles[j].velocity - vel_i;
                                    let visc_lap = visc_coeff * (h - r);
                                    viscosity_force +=
                                        vel_diff * (viscosity * mass * visc_lap / density_j);

                                    // Surface normal
                                    normal += dir * (mass / density_j * spiky_grad);
                                }
                            }
                        }
                    }
                }
            }

            // Surface tension
            let normal_len = normal.length();
            let surface_force = if normal_len > 1e-6 {
                -surface_tension * normal * normal_len
            } else {
                Vec3::ZERO
            };

            // Total acceleration
            self.particles[i].acceleration =
                gravity + pressure_force / density_i + viscosity_force + surface_force / density_i;
        }
    }

    fn integrate(&mut self, dt: f32) {
        for particle in &mut self.particles {
            if !particle.active {
                continue;
            }
            // Semi-implicit Euler
            particle.velocity += particle.acceleration * dt;
            particle.position += particle.velocity * dt;

            // Simple ground plane collision
            if particle.position.y < 0.0 {
                particle.position.y = 0.0;
                particle.velocity.y = -particle.velocity.y * 0.3;
            }
        }
    }

    /// Get all particles.
    pub fn particles(&self) -> &[SphParticle] {
        &self.particles
    }

    /// Get mutable access to all particles.
    pub fn particles_mut(&mut self) -> &mut Vec<SphParticle> {
        &mut self.particles
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        self.particles.clear();
        self.spatial_hash.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sph_creation() {
        let config = SphConfig::default();
        let mut fluid = SphFluid::new(config);
        assert_eq!(fluid.active_count(), 0);

        fluid.add_particle(Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(fluid.active_count(), 1);
    }

    #[test]
    fn test_sph_spawn_sphere() {
        let config = SphConfig::default();
        let mut fluid = SphFluid::new(config);
        fluid.spawn_sphere(Vec3::new(0.0, 5.0, 0.0), 2.0, 100);
        assert_eq!(fluid.active_count(), 100);
    }

    #[test]
    fn test_sph_spawn_box() {
        let config = SphConfig::default();
        let mut fluid = SphFluid::new(config);
        fluid.spawn_box(Vec3::ZERO, Vec3::new(4.0, 4.0, 4.0), 1.0);
        assert!(fluid.active_count() > 0);
    }

    #[test]
    fn test_sph_max_particles() {
        let mut config = SphConfig::default();
        config.max_particles = 10;
        let mut fluid = SphFluid::new(config);
        for _ in 0..20 {
            fluid.add_particle(Vec3::new(0.0, 1.0, 0.0));
        }
        assert_eq!(fluid.active_count(), 10);
    }

    #[test]
    fn test_sph_step() {
        let config = SphConfig::default();
        let mut fluid = SphFluid::new(config);
        fluid.add_particle(Vec3::new(0.0, 10.0, 0.0));

        let world = PhysicsWorld::new(crate::WorldConfig::default());
        fluid.step(1.0 / 60.0, &world);

        // Particle should have fallen due to gravity
        let p = &fluid.particles()[0];
        assert!(p.position.y < 10.0);
    }

    #[test]
    fn test_sph_density_computation() {
        let mut config = SphConfig::default();
        config.smoothing_radius = 10.0;
        let mut fluid = SphFluid::new(config);

        // Add two close particles
        fluid.add_particle(Vec3::new(0.0, 5.0, 0.0));
        fluid.add_particle(Vec3::new(0.1, 5.0, 0.0));

        let world = PhysicsWorld::new(crate::WorldConfig::default());
        fluid.step(1.0 / 60.0, &world);

        // Both should have non-zero density
        for p in fluid.particles() {
            assert!(p.density > 0.0);
        }
    }

    #[test]
    fn test_sph_clear() {
        let config = SphConfig::default();
        let mut fluid = SphFluid::new(config);
        fluid.spawn_sphere(Vec3::new(0.0, 5.0, 0.0), 2.0, 50);
        assert_eq!(fluid.active_count(), 50);

        fluid.clear();
        assert_eq!(fluid.active_count(), 0);
    }
}
