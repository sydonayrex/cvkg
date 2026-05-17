use glam::{Quat, Vec3};
use std::f32::consts::PI;

// ─── FABRIK Inverse Kinematics ───────────────────────────────────────────────

/// A joint in an IK chain.
#[derive(Debug, Clone)]
pub struct IkJoint {
    pub position: Vec3,
    pub length: f32,
    /// Optional min/max angle limits (radians) relative to parent bone.
    pub min_angle: Option<f32>,
    pub max_angle: Option<f32>,
}

impl IkJoint {
    pub fn new(position: Vec3, length: f32) -> Self {
        Self {
            position,
            length,
            min_angle: None,
            max_angle: None,
        }
    }

    pub fn with_angle_limits(mut self, min: f32, max: f32) -> Self {
        self.min_angle = Some(min);
        self.max_angle = Some(max);
        self
    }
}

/// FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.
/// Solves a joint chain so the end effector reaches a target position.
#[derive(Debug, Clone)]
pub struct FabrikSolver {
    joints: Vec<IkJoint>,
    tolerance: f32,
    max_iterations: usize,
    /// Full base position (root is fixed here).
    base: Vec3,
}

impl FabrikSolver {
    pub fn new(joints: Vec<IkJoint>, base: Vec3) -> Self {
        Self {
            joints,
            tolerance: 0.01,
            max_iterations: 20,
            base,
        }
    }

    pub fn with_tolerance(mut self, t: f32) -> Self {
        self.tolerance = t;
        self
    }
    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    /// Solve IK for `target`. Returns the updated joint positions.
    pub fn solve(&self, target: Vec3) -> Vec<Vec3> {
        let n = self.joints.len();
        if n == 0 {
            return vec![];
        }

        // Build positions array: base + each joint tip
        let mut positions: Vec<Vec3> = Vec::with_capacity(n + 1);
        positions.push(self.base);
        let mut acc = self.base;
        for j in &self.joints {
            acc += Vec3::new(j.length, 0.0, 0.0); // initial direction along X
            positions.push(acc);
        }

        // Compute full chain reach
        let total_len: f32 = self.joints.iter().map(|j| j.length).sum();
        let target_dist = (target - self.base).length();

        // If target is unreachable, stretch toward it
        if target_dist >= total_len {
            for i in 0..n {
                let dir = (target - positions[i]).normalize_or_zero();
                positions[i + 1] = positions[i] + dir * self.joints[i].length;
            }
            return positions;
        }

        // FABRIK iterations
        for _ in 0..self.max_iterations {
            // Check convergence
            let dist = (positions[n] - target).length();
            if dist < self.tolerance {
                break;
            }

            // Forward reaching: set end effector to target
            positions[n] = target;

            for i in (1..=n).rev() {
                let dir = (positions[i - 1] - positions[i]).normalize_or_zero();
                positions[i - 1] = positions[i] + dir * self.joints[i - 1].length;
            }

            // Backward reaching: fix base
            positions[0] = self.base;

            for i in 0..n {
                let dir = (positions[i + 1] - positions[i]).normalize_or_zero();
                positions[i + 1] = positions[i] + dir * self.joints[i].length;
            }
        }

        positions
    }
}

// ─── CCD Inverse Kinematics ──────────────────────────────────────────────────

/// CCD (Cyclic Coordinate Descent) IK solver.
#[derive(Debug, Clone)]
pub struct CcdSolver {
    joints: Vec<IkJoint>,
    tolerance: f32,
    max_iterations: usize,
    base: Vec3,
}

impl CcdSolver {
    pub fn new(joints: Vec<IkJoint>, base: Vec3) -> Self {
        Self {
            joints,
            tolerance: 0.01,
            max_iterations: 20,
            base,
        }
    }

    pub fn with_tolerance(mut self, t: f32) -> Self {
        self.tolerance = t;
        self
    }
    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    pub fn solve(&self, target: Vec3) -> Vec<Vec3> {
        let n = self.joints.len();
        if n == 0 {
            return vec![];
        }

        let mut positions: Vec<Vec3> = Vec::with_capacity(n + 1);
        positions.push(self.base);
        let mut acc = self.base;
        for j in &self.joints {
            acc += Vec3::new(j.length, 0.0, 0.0);
            positions.push(acc);
        }

        let total_len: f32 = self.joints.iter().map(|j| j.length).sum();
        if (target - self.base).length() >= total_len {
            for i in 0..n {
                let dir = (target - positions[i]).normalize_or_zero();
                positions[i + 1] = positions[i] + dir * self.joints[i].length;
            }
            return positions;
        }

        for _ in 0..self.max_iterations {
            if (positions[n] - target).length() < self.tolerance {
                break;
            }

            for i in (1..=n).rev() {
                let joint_pos = positions[i - 1];
                let to_end = positions[n] - joint_pos;
                let to_target = target - joint_pos;

                if to_end.length_squared() < 1e-10 || to_target.length_squared() < 1e-10 {
                    continue;
                }

                let cross = to_end.cross(to_target);
                let dot = to_end.dot(to_target);
                let angle = cross.length().atan2(dot);

                if angle.abs() < 1e-6 {
                    continue;
                }

                let axis = if cross.length_squared() > 1e-10 {
                    cross.normalize()
                } else {
                    // Vectors are parallel, pick arbitrary perpendicular
                    let perp = if to_end.x.abs() < 0.9 {
                        Vec3::X.cross(to_end).normalize()
                    } else {
                        Vec3::Y.cross(to_end).normalize()
                    };
                    perp
                };

                let rot = Quat::from_axis_angle(axis, angle);

                // Rotate end effector and all joints above this one
                for k in i..=n {
                    let rel = positions[k] - joint_pos;
                    positions[k] = joint_pos + rot * rel;
                }
            }
        }

        positions
    }
}

// ─── Procedural Locomotion ───────────────────────────────────────────────────

/// Foot IK target for procedural locomotion.
#[derive(Debug, Clone)]
pub struct FootTarget {
    pub position: Vec3,
    pub grounded: bool,
    pub ground_normal: Vec3,
}

/// Procedural locomotion system with footplanting and hip sway.
#[derive(Debug, Clone)]
pub struct ProceduralLocomotion {
    /// Current hip position.
    pub hip_position: Vec3,
    /// Hip height above ground.
    pub hip_height: f32,
    /// Step length.
    pub step_length: f32,
    /// Hip sway amplitude.
    pub sway_amount: f32,
    /// Hip sway speed.
    pub sway_speed: f32,
    /// Left foot offset from hip.
    pub left_foot_offset: Vec3,
    /// Right foot offset from hip.
    pub right_foot_offset: Vec3,
    /// Current left foot target.
    pub left_foot: FootTarget,
    /// Current right foot target.
    pub right_foot: FootTarget,
    /// Internal time accumulator.
    time: f32,
    /// Previous hip Y for ground detection.
    prev_hip_y: f32,
    /// Ground level.
    ground_y: f32,
    /// Foot lift height during step.
    pub foot_lift_height: f32,
    /// Foot step threshold (distance before triggering new step).
    pub step_threshold: f32,
    /// Left foot current world position.
    left_foot_world: Vec3,
    /// Right foot current world position.
    right_foot_world: Vec3,
    /// Is left foot stepping.
    left_stepping: bool,
    /// Is right foot stepping.
    right_stepping: bool,
    /// Step progress 0..1.
    step_progress: f32,
    /// Which foot is stepping (true = left).
    left_is_stepping: bool,
    /// Step start position.
    step_start: Vec3,
    /// Step end position.
    step_end: Vec3,
}

impl ProceduralLocomotion {
    pub fn new(hip_position: Vec3, hip_height: f32) -> Self {
        Self {
            hip_position,
            hip_height,
            step_length: 0.6,
            sway_amount: 0.05,
            sway_speed: 3.0,
            left_foot_offset: Vec3::new(0.0, 0.0, -0.15),
            right_foot_offset: Vec3::new(0.0, 0.0, 0.15),
            left_foot: FootTarget {
                position: Vec3::ZERO,
                grounded: true,
                ground_normal: Vec3::Y,
            },
            right_foot: FootTarget {
                position: Vec3::ZERO,
                grounded: true,
                ground_normal: Vec3::Y,
            },
            time: 0.0,
            prev_hip_y: hip_position.y,
            ground_y: 0.0,
            foot_lift_height: 0.15,
            step_threshold: 0.35,
            left_foot_world: hip_position + Vec3::new(0.0, 0.0, -0.15),
            right_foot_world: hip_position + Vec3::new(0.0, 0.0, 0.15),
            left_stepping: false,
            right_stepping: false,
            step_progress: 0.0,
            left_is_stepping: false,
            step_start: Vec3::ZERO,
            step_end: Vec3::ZERO,
        }
    }

    /// Update locomotion. `velocity` is the desired movement direction/speed.
    pub fn update(&mut self, dt: f32, velocity: Vec3) {
        self.time += dt;

        // Move hip
        self.hip_position += velocity * dt;
        self.hip_position.y = self.ground_y + self.hip_height;

        // Hip sway
        let sway = (self.time * self.sway_speed).sin() * self.sway_amount;
        self.hip_position.z += sway * dt;

        // Compute desired foot positions relative to hip
        let left_desired = self.hip_position + self.left_foot_offset;
        let right_desired = self.hip_position + self.right_foot_offset;

        // Check if feet need to step
        let left_dist = (self.left_foot_world - left_desired).length();
        let right_dist = (self.right_foot_world - right_desired).length();

        if !self.left_stepping && !self.right_stepping {
            if left_dist > self.step_threshold {
                self.start_step(true, left_desired);
            } else if right_dist > self.step_threshold {
                self.start_step(false, right_desired);
            }
        }

        // Animate stepping foot
        if self.left_stepping || self.right_stepping {
            self.step_progress += dt * 3.0; // step speed
            if self.step_progress >= 1.0 {
                self.step_progress = 1.0;
                if self.left_is_stepping {
                    self.left_foot_world = self.step_end;
                    self.left_foot.position = self.step_end;
                    self.left_foot.grounded = true;
                    self.left_stepping = false;
                } else {
                    self.right_foot_world = self.step_end;
                    self.right_foot.position = self.step_end;
                    self.right_foot.grounded = true;
                    self.right_stepping = false;
                }
                self.step_progress = 0.0;
            } else {
                let t = self.step_progress;
                // Smooth step curve
                let smooth = t * t * (3.0 - 2.0 * t);
                let mut pos = self.step_start.lerp(self.step_end, smooth);
                // Lift foot
                let lift = (t * PI).sin() * self.foot_lift_height;
                pos.y += lift;

                if self.left_is_stepping {
                    self.left_foot_world = pos;
                    self.left_foot.position = pos;
                    self.left_foot.grounded = false;
                } else {
                    self.right_foot_world = pos;
                    self.right_foot.position = pos;
                    self.right_foot.grounded = false;
                }
            }
        }

        self.prev_hip_y = self.hip_position.y;
    }

    fn start_step(&mut self, left: bool, target: Vec3) {
        if left {
            self.left_stepping = true;
            self.left_is_stepping = true;
            self.step_start = self.left_foot_world;
        } else {
            self.right_stepping = true;
            self.left_is_stepping = false;
            self.step_start = self.right_foot_world;
        }
        self.step_end = target;
        self.step_progress = 0.0;
    }
}

// ─── Ragdoll Blender ─────────────────────────────────────────────────────────

/// Per-bone blend weight for ragdoll transition.
#[derive(Debug, Clone)]
pub struct BoneWeight {
    pub bone_index: usize,
    pub weight: f32, // 0 = animated, 1 = physics
}

/// Blends between keyframed animation and physics ragdoll.
#[derive(Debug, Clone)]
pub struct RagdollBlender {
    /// Current blend weight 0..1 (0 = fully animated, 1 = fully ragdoll).
    pub blend_weight: f32,
    /// Target blend weight.
    target_weight: f32,
    /// Blend speed (units per second).
    pub blend_speed: f32,
    /// Per-bone weights.
    bone_weights: Vec<BoneWeight>,
    /// Animated bone positions (from keyframes).
    animated_positions: Vec<Vec3>,
    /// Physics bone positions (from ragdoll sim).
    physics_positions: Vec<Vec3>,
    /// Output blended positions.
    output_positions: Vec<Vec3>,
}

impl RagdollBlender {
    pub fn new(bone_count: usize) -> Self {
        Self {
            blend_weight: 0.0,
            target_weight: 0.0,
            blend_speed: 2.0,
            bone_weights: (0..bone_count)
                .map(|i| BoneWeight {
                    bone_index: i,
                    weight: 0.0,
                })
                .collect(),
            animated_positions: vec![Vec3::ZERO; bone_count],
            physics_positions: vec![Vec3::ZERO; bone_count],
            output_positions: vec![Vec3::ZERO; bone_count],
        }
    }

    /// Set per-bone weight. weight=0 means fully animated, weight=1 means fully physics.
    pub fn set_bone_weight(&mut self, bone: usize, weight: f32) {
        if let Some(bw) = self.bone_weights.iter_mut().find(|b| b.bone_index == bone) {
            bw.weight = weight.clamp(0.0, 1.0);
        }
    }

    /// Set the target blend weight and update per-bone weights uniformly.
    pub fn blend(&mut self, target: f32) {
        self.target_weight = target.clamp(0.0, 1.0);
        for bw in &mut self.bone_weights {
            bw.weight = self.target_weight;
        }
    }

    /// Set animated bone positions from keyframe data.
    pub fn set_animated(&mut self, positions: &[Vec3]) {
        let n = positions.len().min(self.animated_positions.len());
        self.animated_positions[..n].copy_from_slice(&positions[..n]);
    }

    /// Set physics bone positions from ragdoll simulation.
    pub fn set_physics(&mut self, positions: &[Vec3]) {
        let n = positions.len().min(self.physics_positions.len());
        self.physics_positions[..n].copy_from_slice(&positions[..n]);
    }

    /// Update blend and compute output positions. Call each frame.
    pub fn update(&mut self, dt: f32) -> &[Vec3] {
        // Smoothly approach target
        let diff = self.target_weight - self.blend_weight;
        if diff.abs() > 0.001 {
            self.blend_weight += diff * (self.blend_speed * dt).min(1.0);
        } else {
            self.blend_weight = self.target_weight;
        }

        // Blend per-bone
        for (i, bw) in self.bone_weights.iter().enumerate() {
            let w = self.blend_weight * bw.weight;
            self.output_positions[i] =
                self.animated_positions[i].lerp(self.physics_positions[i], w);
        }

        &self.output_positions
    }

    pub fn output(&self) -> &[Vec3] {
        &self.output_positions
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fabrik_reaches_target() {
        let joints = vec![
            IkJoint::new(Vec3::ZERO, 1.0),
            IkJoint::new(Vec3::new(1.0, 0.0, 0.0), 1.0),
        ];
        let solver = FabrikSolver::new(joints, Vec3::ZERO);
        let positions = solver.solve(Vec3::new(1.5, 0.5, 0.0));
        let last = *positions.last().unwrap();
        let dist = (last - Vec3::new(1.5, 0.5, 0.0)).length();
        assert!(dist < 0.1, "FABRIK didn't reach target, dist={}", dist);
    }

    #[test]
    fn test_fabrik_unreachable() {
        let joints = vec![IkJoint::new(Vec3::ZERO, 0.5)];
        let solver = FabrikSolver::new(joints, Vec3::ZERO);
        let positions = solver.solve(Vec3::new(5.0, 0.0, 0.0));
        // Should stretch toward target
        let last = *positions.last().unwrap();
        assert!(last.x > 0.0);
    }

    #[test]
    fn test_ccd_reaches_target() {
        let joints = vec![
            IkJoint::new(Vec3::ZERO, 1.0),
            IkJoint::new(Vec3::new(1.0, 0.0, 0.0), 1.0),
        ];
        let solver = CcdSolver::new(joints, Vec3::ZERO);
        let positions = solver.solve(Vec3::new(1.5, 0.5, 0.0));
        let last = *positions.last().unwrap();
        let dist = (last - Vec3::new(1.5, 0.5, 0.0)).length();
        assert!(dist < 0.5, "CCD didn't approach target, dist={}", dist);
    }

    #[test]
    fn test_locomotion_creates_foot_targets() {
        let mut loco = ProceduralLocomotion::new(Vec3::new(0.0, 1.0, 0.0), 1.0);
        loco.update(0.016, Vec3::new(1.0, 0.0, 0.0));
        // After enough time, a step should trigger
        for _ in 0..200 {
            loco.update(0.016, Vec3::new(2.0, 0.0, 0.0));
        }
        assert!(loco.left_foot.grounded || loco.right_foot.grounded);
    }

    #[test]
    fn test_ragdoll_blend() {
        let mut blender = RagdollBlender::new(3);
        blender.set_animated(&[Vec3::ZERO, Vec3::Y, Vec3::Y * 2.0]);
        blender.set_physics(&[Vec3::X, Vec3::X + Vec3::Y, Vec3::X + Vec3::Y * 2.0]);
        blender.blend(1.0);

        // Step many frames to converge (blend_speed=2.0, dt=0.016)
        for _ in 0..300 {
            blender.update(0.016);
        }

        let out = blender.output();
        // Should be close to physics positions
        for i in 0..3 {
            let dist = (out[i] - blender.physics_positions[i]).length();
            assert!(
                dist < 0.05,
                "Bone {} not blended to physics: dist={}",
                i,
                dist
            );
        }
    }

    #[test]
    fn test_ragdoll_bone_weights() {
        let mut blender = RagdollBlender::new(4);
        blender.set_bone_weight(0, 0.0); // fully animated
        blender.set_bone_weight(1, 0.5); // half
        blender.set_bone_weight(2, 1.0); // fully physics
        blender.set_bone_weight(3, 0.25);

        assert_eq!(blender.bone_weights[0].weight, 0.0);
        assert_eq!(blender.bone_weights[1].weight, 0.5);
        assert_eq!(blender.bone_weights[2].weight, 1.0);
        assert_eq!(blender.bone_weights[3].weight, 0.25);
    }
}
