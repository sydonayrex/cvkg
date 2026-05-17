use glam::{Vec2, Vec3};

// --- TerrainErosion ---
// Hydraulic erosion simulation on a heightmap grid.
// Implements one cycle: rain → dissolution → sediment transport → deposition.

pub struct TerrainErosion {
    pub width: usize,
    pub height: usize,
    pub heightmap: Vec<f32>,
    pub sediment: Vec<f32>,
    pub water: Vec<f32>,
    pub hardness: Vec<f32>,
    // Simulation parameters
    pub rain_rate: f32,
    pub dissolution_rate: f32,
    pub sediment_capacity: f32,
    pub deposition_rate: f32,
    pub evaporation_rate: f32,
    pub min_slope: f32,
    pub cell_size: f32,
}

impl TerrainErosion {
    pub fn new(
        width: usize,
        height: usize,
        initial_heightmap: Vec<f32>,
        hardness: Vec<f32>,
        rain_rate: f32,
        dissolution_rate: f32,
        sediment_capacity: f32,
        deposition_rate: f32,
        evaporation_rate: f32,
        min_slope: f32,
        cell_size: f32,
    ) -> Self {
        let cell_count = width * height;
        assert_eq!(initial_heightmap.len(), cell_count);
        assert_eq!(hardness.len(), cell_count);
        TerrainErosion {
            width,
            height,
            heightmap: initial_heightmap,
            sediment: vec![0.0; cell_count],
            water: vec![0.0; cell_count],
            hardness,
            rain_rate,
            dissolution_rate,
            sediment_capacity,
            deposition_rate,
            evaporation_rate,
            min_slope,
            cell_size,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Run one complete erosion cycle: rain, dissolution, sediment transport, deposition.
    pub fn step(&mut self) {
        let w = self.width;
        let h = self.height;
        let n = w * h;

        // Phase 1: Rain
        for i in 0..n {
            self.water[i] += self.rain_rate;
        }

        // Phase 2: Dissolution
        for i in 0..n {
            let dissolved = self.water[i] * self.dissolution_rate * (1.0 - self.hardness[i]);
            self.heightmap[i] -= dissolved;
            self.sediment[i] += dissolved;
        }

        // Phase 3: Sediment transport & flow
        let mut new_heightmap = self.heightmap.clone();
        let mut new_water = vec![0.0f32; n];
        let mut new_sediment = self.sediment.clone();

        // Compute flow direction and transport for each cell
        for y in 0..h {
            for x in 0..w {
                let idx = self.idx(x, y);
                let current_height = self.heightmap[idx];
                let current_water = self.water[idx];
                let total_height = current_height + current_water;

                if current_water <= 0.0 || total_height <= 0.0 {
                    new_water[idx] += current_water;
                    continue;
                }

                // Find lowest neighbor
                let mut lowest_total = total_height;
                let mut total_drop = 0.0;

                let dirs: [(isize, isize); 8] = [
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ];

                for &(dx, dy) in &dirs {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && nx < w as isize && ny >= 0 && ny < h as isize {
                        let n_idx = self.idx(nx as usize, ny as usize);
                        let neighbor_total = self.heightmap[n_idx] + self.water[n_idx];
                        let drop = total_height - neighbor_total;
                        if drop > 0.0 {
                            lowest_total = lowest_total.min(neighbor_total);
                            total_drop += drop;
                        }
                    }
                }

                if total_drop > 0.0 && total_height > lowest_total + self.min_slope {
                    // Water flows downhill proportional to drop
                    let flow_ratio = ((total_height - lowest_total) / total_drop).min(1.0);
                    let water_to_move = current_water * flow_ratio * 0.5;
                    let sediment_to_move =
                        self.sediment[idx] * (water_to_move / current_water.max(1e-6));

                    if water_to_move > 0.0 {
                        new_water[idx] -= water_to_move;
                        new_sediment[idx] -= sediment_to_move;

                        // Distribute to lowest neighbors
                        for &(dx, dy) in &dirs {
                            let nx = x as isize + dx;
                            let ny = y as isize + dy;
                            if nx >= 0 && nx < w as isize && ny >= 0 && ny < h as isize {
                                let n_idx = self.idx(nx as usize, ny as usize);
                                let drop =
                                    total_height - (self.heightmap[n_idx] + self.water[n_idx]);
                                if drop > 0.0 {
                                    let frac = drop / total_drop;
                                    new_water[n_idx] += water_to_move * frac;
                                    new_sediment[n_idx] += sediment_to_move * frac;
                                }
                            }
                        }
                    }
                }

                // Check sediment capacity for additional erosion
                let velocity = ((total_height - lowest_total) / self.cell_size).max(0.0);
                let capacity = self.sediment_capacity * velocity * current_water;
                if new_sediment[idx] > capacity {
                    // Deposit excess
                    let excess = new_sediment[idx] - capacity;
                    let deposit = excess * self.deposition_rate;
                    new_sediment[idx] -= deposit;
                    new_heightmap[idx] += deposit;
                }
            }
        }

        self.heightmap = new_heightmap;
        self.water = new_water;
        self.sediment = new_sediment;

        // Phase 4: Evaporation
        for i in 0..n {
            self.water[i] *= 1.0 - self.evaporation_rate;
        }
    }

    /// Get height at specific grid cell
    pub fn height_at(&self, x: usize, y: usize) -> f32 {
        self.heightmap[self.idx(x, y)]
    }

    /// Get total heightmap data (reference)
    pub fn heightmap_data(&self) -> &[f32] {
        &self.heightmap
    }

    /// Reset water layer
    pub fn reset_water(&mut self) {
        self.water.fill(0.0);
    }
}

// --- MeshDeformer ---
// Blend shape / morph target system with RBF falloff support.

pub struct MorphTarget {
    pub name: String,
    pub deltas: Vec<Vec3>, // Per-vertex displacement
}

pub struct MeshDeformer {
    pub base_mesh: Vec<Vec3>,
    pub targets: Vec<MorphTarget>,
    // RBF parameters for localized deformation
    pub rbf_centers: Vec<Vec3>,
    pub rbf_weights: Vec<f32>,
    pub rbf_radius: f32,
    pub rbf_falloff_type: RbfFalloff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RbfFalloff {
    Gaussian,
    Linear,
    Smooth, // (1 - r^2)^2 for r < 1
}

impl MeshDeformer {
    pub fn new(base_mesh: Vec<Vec3>) -> Self {
        MeshDeformer {
            base_mesh,
            targets: Vec::new(),
            rbf_centers: Vec::new(),
            rbf_weights: Vec::new(),
            rbf_radius: 1.0,
            rbf_falloff_type: RbfFalloff::Gaussian,
        }
    }

    pub fn add_target(&mut self, name: &str, deltas: Vec<Vec3>) {
        assert_eq!(
            deltas.len(),
            self.base_mesh.len(),
            "Target vertex count must match base mesh"
        );
        self.targets.push(MorphTarget {
            name: name.to_string(),
            deltas,
        });
    }

    pub fn add_rbf_center(&mut self, center: Vec3, weight: f32) {
        self.rbf_centers.push(center);
        self.rbf_weights.push(weight);
    }

    pub fn set_rbf_radius(&mut self, radius: f32) {
        self.rbf_radius = radius.max(0.001);
    }

    fn rbf_weight_at_point(&self, point: Vec3, rbf_idx: usize) -> f32 {
        if self.rbf_centers.is_empty() {
            return 1.0;
        }

        let center = self.rbf_centers[rbf_idx];
        let r = (point - center).length() / self.rbf_radius;
        let w = match self.rbf_falloff_type {
            RbfFalloff::Gaussian => (-r * r * 0.5).exp(),
            RbfFalloff::Linear => (1.0 - r).max(0.0),
            RbfFalloff::Smooth => {
                if r >= 1.0 {
                    0.0
                } else {
                    let t = 1.0 - r * r;
                    t * t
                }
            }
        };
        w * self.rbf_weights[rbf_idx]
    }

    fn interpolated_rbf_factor(&self, point: Vec3) -> f32 {
        if self.rbf_centers.is_empty() {
            return 1.0;
        }

        let mut total_weight = 0.0f32;
        let mut weighted_sum = 0.0f32;
        for i in 0..self.rbf_centers.len() {
            let w = self.rbf_weight_at_point(point, i);
            total_weight += w;
            weighted_sum += w;
        }
        weighted_sum / total_weight.max(0.001)
    }

    /// Evaluate deformed vertices given per-target weights.
    /// weights[i] corresponds to self.targets[i].
    pub fn evaluate(&self, weights: &[f32]) -> Vec<Vec3> {
        assert_eq!(
            weights.len(),
            self.targets.len(),
            "Weight count must match target count"
        );

        let n = self.base_mesh.len();
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            let base = self.base_mesh[i];
            let rbf_factor = self.interpolated_rbf_factor(base);

            let mut displacement = Vec3::ZERO;
            for (ti, target) in self.targets.iter().enumerate() {
                displacement += target.deltas[i] * weights[ti] * rbf_factor;
            }
            result.push(base + displacement);
        }
        result
    }

    /// Evaluate with uniform weight across all targets
    pub fn evaluate_uniform(&self, weight: f32) -> Vec<Vec3> {
        let weights = vec![weight; self.targets.len()];
        self.evaluate(&weights)
    }
}

// --- VegetationWind ---
// Simulates instanced vegetation wind animation using Gerstner-like waves
// with per-instance phase offsets and gust noise.

pub struct WindParams {
    pub direction: Vec2,
    pub base_speed: f32,
    pub gust_frequency: f32,
    pub gust_amplitude: f32,
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
    pub turbulence_scale: f32,
    pub turbulence_speed: f32,
    pub height_scale: f32, // How much height affects displacement
}

impl Default for WindParams {
    fn default() -> Self {
        WindParams {
            direction: Vec2::new(1.0, 0.0),
            base_speed: 2.0,
            gust_frequency: 0.5,
            gust_amplitude: 1.5,
            wave_amplitude: 0.15,
            wave_frequency: 3.0,
            turbulence_scale: 0.3,
            turbulence_speed: 1.5,
            height_scale: 1.0,
        }
    }
}

pub struct WindInstance {
    pub position: Vec2,
    pub phase_offset: f32,
    pub stiffness: f32, // How resistant to wind (0 = floppy, 1 = rigid)
    pub height: f32,    // Affects lever arm for displacement
}

pub struct VegetationWind {
    pub instances: Vec<WindInstance>,
    pub params: WindParams,
    time: f32,
    // Precomputed noise offsets for each instance
    noise_offsets: Vec<Vec2>,
}

// Simple hash-based noise for gust simulation
fn hash_noise(x: f32, y: f32) -> f32 {
    let a = x * 127.1 + y * 311.7;
    let b = x * 269.5 + y * 183.3;
    let v = (a.sin() * 43758.5453 + b.sin() * 22578.1459).fract();
    v * 2.0 - 1.0
}

fn smooth_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;

    // Smoothstep interpolation
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let n00 = hash_noise(ix, iy);
    let n10 = hash_noise(ix + 1.0, iy);
    let n01 = hash_noise(ix, iy + 1.0);
    let n11 = hash_noise(ix + 1.0, iy + 1.0);

    let nx0 = n00 * (1.0 - sx) + n10 * sx;
    let nx1 = n01 * (1.0 - sx) + n11 * sx;

    nx0 * (1.0 - sy) + nx1 * sy
}

impl VegetationWind {
    pub fn new(params: WindParams, instances: Vec<WindInstance>) -> Self {
        let n = instances.len();
        let mut noise_offsets = Vec::with_capacity(n);
        for i in 0..n {
            noise_offsets.push(Vec2::new(
                (i as f32 * 17.31) % 100.0,
                (i as f32 * 31.73) % 100.0,
            ));
        }
        VegetationWind {
            instances,
            params,
            time: 0.0,
            noise_offsets,
        }
    }

    /// Create instances from positions with auto-generated phase offsets
    pub fn from_positions(
        params: WindParams,
        positions: &[Vec2],
        stiffness: f32,
        height: f32,
    ) -> Self {
        let instances: Vec<WindInstance> = positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| WindInstance {
                position: pos,
                phase_offset: (i as f32 * 1.6180339887) % (std::f32::consts::TAU),
                stiffness: stiffness.clamp(0.0, 1.0),
                height: height,
            })
            .collect();
        Self::new(params, instances)
    }

    /// Compute gust factor at a given time and position
    fn gust_factor(&self, t: f32, position: Vec2, noise_offset: Vec2) -> f32 {
        let gust_phase = t * self.params.gust_frequency * std::f32::consts::TAU;
        let base_gust = gust_phase.sin();

        // Add spatial variation via noise
        let noise_val = smooth_noise(
            position.x * self.params.turbulence_scale
                + t * self.params.turbulence_speed
                + noise_offset.x,
            position.y * self.params.turbulence_scale
                + t * self.params.turbulence_speed * 0.7
                + noise_offset.y,
        );

        // Combine base gust with noise
        let gust = base_gust * 0.6 + noise_val * 0.4;
        gust * self.params.gust_amplitude
    }

    /// Compute Gerstner-like wave displacement for a single instance
    fn wave_displacement(&self, t: f32, instance: &WindInstance, noise_offset: Vec2) -> Vec3 {
        let dir = self.params.direction.normalize_or_zero();
        let gust = self.gust_factor(t, instance.position, noise_offset);

        // Effective wind strength
        let wind_strength = (self.params.base_speed + gust) * (1.0 - instance.stiffness * 0.7);

        // Gerstner-like wave: circular motion projected to displacement
        let phase = t * self.params.wave_frequency * std::f32::consts::TAU + instance.phase_offset;
        let height_factor = instance.height * self.params.height_scale;

        // Primary wave
        let wave_x = phase.sin() * self.params.wave_amplitude * height_factor;
        let wave_z = (phase * 2.0).sin() * self.params.wave_amplitude * 0.3 * height_factor;

        // Secondary harmonic for more organic motion
        let wave2_x = (phase * 1.7 + 0.5).sin() * self.params.wave_amplitude * 0.2 * height_factor;
        let wave2_z = (phase * 2.3 + 1.2).sin() * self.params.wave_amplitude * 0.1 * height_factor;

        // Combine waves, scale by wind strength and direction
        let total_x = (wave_x + wave2_x) * wind_strength;
        let total_z = (wave_z + wave2_z) * wind_strength;

        // Project onto wind direction and add vertical component
        Vec3::new(
            dir.x * total_x,
            (phase * 0.5).sin() * self.params.wave_amplitude * 0.1 * height_factor * wind_strength,
            dir.y * total_x + total_z,
        )
    }

    /// Update simulation by dt, returns per-instance Vec3 offsets
    pub fn update(&mut self, dt: f32) -> Vec<Vec3> {
        self.time += dt;
        let t = self.time;

        let mut offsets = Vec::with_capacity(self.instances.len());
        for (i, instance) in self.instances.iter().enumerate() {
            let offset = self.wave_displacement(t, instance, self.noise_offsets[i]);
            offsets.push(offset);
        }
        offsets
    }

    /// Get current simulation time
    pub fn current_time(&self) -> f32 {
        self.time
    }

    /// Reset simulation time
    pub fn reset_time(&mut self) {
        self.time = 0.0;
    }

    /// Update wind parameters at runtime
    pub fn set_params(&mut self, params: WindParams) {
        self.params = params;
    }

    /// Get a reference to instances
    pub fn instances(&self) -> &[WindInstance] {
        &self.instances
    }

    /// Get mutable reference to instances
    pub fn instances_mut(&mut self) -> &mut Vec<WindInstance> {
        &mut self.instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_erosion_creation() {
        let w = 4;
        let h = 4;
        let hm = vec![1.0; w * h];
        let hardness = vec![0.5; w * h];
        let mut erosion =
            TerrainErosion::new(w, h, hm, hardness, 0.01, 0.1, 1.0, 0.1, 0.05, 0.01, 1.0);
        assert_eq!(erosion.height_at(0, 0), 1.0);
        erosion.step();
        // After one step, rain has been added and some dissolution occurred
        assert!(erosion.heightmap.iter().any(|&h| h <= 1.0));
    }

    #[test]
    fn test_mesh_deformer_basic() {
        let base = vec![
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let mut deformer = MeshDeformer::new(base.clone());

        let deltas = vec![Vec3::Y, Vec3::Y, Vec3::Y];
        deformer.add_target("up", deltas);

        let result = deformer.evaluate(&[1.0]);
        assert_eq!(result[0], Vec3::Y);
        assert_eq!(result[1], Vec3::new(1.0, 1.0, 0.0));
        assert_eq!(result[2], Vec3::new(0.0, 2.0, 0.0));
    }

    #[test]
    fn test_mesh_deformer_rbf() {
        let base = vec![Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)];
        let mut deformer = MeshDeformer::new(base);

        let deltas = vec![Vec3::Y * 2.0, Vec3::Y * 2.0];
        deformer.add_target("up", deltas);
        deformer.set_rbf_radius(2.0);
        deformer.add_rbf_center(Vec3::ZERO, 1.0);

        let result = deformer.evaluate(&[1.0]);
        // First vertex at center should be fully affected
        assert!(result[0].y > 1.5);
        // Second vertex far from center should be barely affected
        assert!(result[1].y < 0.5);
    }

    #[test]
    fn test_vegetation_wind_update() {
        let params = WindParams::default();
        let positions = vec![Vec2::ZERO, Vec2::new(5.0, 3.0), Vec2::new(-2.0, 1.0)];
        let mut wind = VegetationWind::from_positions(params, &positions, 0.3, 2.0);

        let offsets = wind.update(0.016);
        assert_eq!(offsets.len(), 3);

        // Offsets should be non-zero after update
        assert!(offsets.iter().any(|o| o.length_squared() > 0.0));

        // Different instances should have different offsets (phase offsets differ)
        let offsets2 = wind.update(0.016);
        assert_ne!(offsets[0], offsets2[0]);
    }

    #[test]
    fn test_vegetation_wind_stiffness() {
        let params = WindParams::default();
        let positions = vec![Vec2::ZERO];

        let mut wind_floppy = VegetationWind::from_positions(params, &positions, 0.0, 1.0);

        let params2 = WindParams::default();
        let mut wind_rigid = VegetationWind::from_positions(params2, &positions, 1.0, 1.0);

        let off_floppy = wind_floppy.update(0.1);
        let off_rigid = wind_rigid.update(0.1);

        // Floppy should displace more than rigid
        assert!(off_floppy[0].length() >= off_rigid[0].length());
    }
}
