//! Gauss-Seidel impulse-based constraint solver.

use glam::Vec2;

use crate::BodyId;
use crate::RigidBody;
use crate::constraint::Constraint;
use crate::constraint::ConstraintKind;

/// Spring constraint parameters.
pub struct SpringConfig {
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Impulse-based constraint solver using Gauss-Seidel iteration.
///
/// Solves velocity-level constraints by computing corrective impulses
/// that are applied to bodies each iteration.
#[derive(Debug, Default)]
pub struct ImpulseSolver {
    /// Number of solver iterations per step. More = stiffer, more expensive.
    pub iterations: usize,
    /// Baumgarte factor for position drift correction (0.0–1.0).
    pub baumgarte: f32,
}

impl ImpulseSolver {
    /// Create a new solver.
    pub fn new() -> Self {
        Self {
            iterations: 8,
            baumgarte: 0.2,
        }
    }

    /// Set the number of iterations.
    pub fn with_iterations(mut self, n: usize) -> Self {
        self.iterations = n;
        self
    }

    /// Set the Baumgarte stabilization factor.
    pub fn with_baumgarte(mut self, b: f32) -> Self {
        self.baumgarte = b.clamp(0.0, 1.0);
        self
    }

    /// Solve all constraints, applying impulses to bodies. Returns pairs of newly broken constraints.
    pub fn solve(
        &self,
        constraints: &mut [Constraint],
        bodies: &mut [RigidBody],
        body_id_map: &std::collections::HashMap<BodyId, usize>,
        dt: f32,
    ) -> Vec<(BodyId, BodyId)> {
        let mut broken_pairs = Vec::new();

        for _ in 0..self.iterations {
            for constraint in constraints.iter_mut() {
                if !constraint.enabled {
                    continue;
                }
                let idx_a = match body_id_map.get(&constraint.body_a) {
                    Some(&i) => i,
                    None => continue,
                };
                let idx_b = match body_id_map.get(&constraint.body_b) {
                    Some(&i) => i,
                    None => continue,
                };

                // SAFETY: idx_a != idx_b guaranteed by collision pair construction
                if idx_a == idx_b {
                    continue;
                }

                let (left, right) = bodies.split_at_mut(idx_a.max(idx_b));
                let (body_a, body_b) = if idx_a < idx_b {
                    (&mut left[idx_a], &mut right[0])
                } else {
                    (&mut right[0], &mut left[idx_b])
                };

                let broke = self.solve_constraint(constraint, body_a, body_b, dt);
                if broke {
                    constraint.enabled = false;
                    broken_pairs.push((constraint.body_a, constraint.body_b));
                }
            }
        }
        broken_pairs
    }

    fn solve_constraint(
        &self,
        constraint: &Constraint,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        dt: f32,
    ) -> bool {
        match &constraint.kind {
            ConstraintKind::Distance {
                local_anchor_a,
                local_anchor_b,
                distance,
                ..
            } => self.solve_distance(
                body_a,
                body_b,
                local_anchor_a,
                local_anchor_b,
                *distance,
                constraint.break_threshold,
                dt,
            ),
            ConstraintKind::Pin { anchor } => {
                self.solve_pin(body_a, body_b, anchor, constraint.break_threshold, dt)
            }
            ConstraintKind::Spring {
                local_anchor_a,
                local_anchor_b,
                rest_length,
                stiffness,
                damping,
            } => {
                let config = SpringConfig {
                    rest_length: *rest_length,
                    stiffness: *stiffness,
                    damping: *damping,
                };
                self.solve_spring(
                    body_a,
                    body_b,
                    local_anchor_a,
                    local_anchor_b,
                    &config,
                    constraint.break_threshold,
                    dt,
                )
            }
            ConstraintKind::Hinge {
                local_anchor_a,
                local_anchor_b,
                ..
            } => self.solve_hinge(
                body_a,
                body_b,
                local_anchor_a,
                local_anchor_b,
                constraint.break_threshold,
                dt,
            ),
            ConstraintKind::AngularLimit {
                min_angle,
                max_angle,
            } => self.solve_angular_limit(
                body_a,
                body_b,
                *min_angle,
                *max_angle,
                constraint.break_threshold,
                dt,
            ),
            ConstraintKind::BallSocket3D { anchor } => {
                self.solve_ball_socket_3d(body_a, body_b, anchor, constraint.break_threshold, dt)
            }
            ConstraintKind::Hinge3D { anchor, axis } => {
                self.solve_hinge_3d(body_a, body_b, anchor, axis, constraint.break_threshold, dt)
            }
            ConstraintKind::Prismatic {
                local_anchor_a,
                local_anchor_b,
                axis,
                min_limit,
                max_limit,
                enable_motor,
                motor_speed,
                motor_max_force,
            } => self.solve_prismatic(
                body_a,
                body_b,
                local_anchor_a,
                local_anchor_b,
                axis,
                *min_limit,
                *max_limit,
                *enable_motor,
                *motor_speed,
                *motor_max_force,
                constraint.break_threshold,
                dt,
            ),
            ConstraintKind::Motor {
                body: _,
                target_velocity,
                max_force,
                damping,
            } => {
                // Motor applies force to drive a body toward target velocity
                self.solve_motor(body_a, body_b, *target_velocity, *max_force, *damping, dt)
            }
            ConstraintKind::Weld {
                local_anchor_a,
                local_anchor_b,
            } => self.solve_weld(
                body_a,
                body_b,
                local_anchor_a,
                local_anchor_b,
                constraint.break_threshold,
                dt,
            ),
            ConstraintKind::SixDof {
                anchor,
                primary_axis,
                linear_limits,
                angular_limits,
                linear_locked,
                angular_locked,
            } => self.solve_six_dof(
                body_a,
                body_b,
                anchor,
                primary_axis,
                *linear_limits,
                *angular_limits,
                *linear_locked,
                *angular_locked,
                constraint.break_threshold,
                dt,
            ),
        }
    }

    fn solve_distance(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        distance: f32,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;
        let current_dist = delta.length();
        if current_dist < 1e-10 {
            return false;
        }

        let strain = (current_dist - distance).abs();
        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        let correction = delta * ((current_dist - distance) / current_dist);
        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let imp = correction / total_inv_mass;
        if !a.is_static {
            a.position += imp * a.inv_mass;
        }
        if !b.is_static {
            b.position -= imp * b.inv_mass;
        }
        false
    }

    fn solve_pin(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        anchor: &Vec2,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        // Pin: both body anchor points must coincide at `anchor`
        let world_a = a.position; // For pin, we use body center
        let world_b = b.position;
        let delta = *anchor - world_a;
        let delta_b = *anchor - world_b;
        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        if !a.is_static {
            a.position += delta * a.inv_mass / total_inv_mass * 0.8;
        }
        if !b.is_static {
            b.position += delta_b * b.inv_mass / total_inv_mass * 0.8;
        }
        false
    }

    fn solve_spring(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        config: &SpringConfig,
        break_threshold: Option<f32>,
        dt: f32,
    ) -> bool {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;
        let dist = delta.length();
        if dist < 1e-10 {
            return false;
        }

        let displacement = dist - config.rest_length;
        if let Some(thresh) = break_threshold {
            if displacement.abs() > thresh {
                return true;
            }
        }

        let dir = delta / dist;

        // Spring force (Hooke's law)
        let spring_force = dir * (config.stiffness * displacement);

        // Damping force
        let rel_vel = b.velocity - a.velocity;
        let damping_force = dir * (rel_vel.dot(dir) * config.damping);

        let total_force = (spring_force + damping_force) * dt;

        if !a.is_static {
            a.velocity += total_force * a.inv_mass;
        }
        if !b.is_static {
            b.velocity -= total_force * b.inv_mass;
        }
        false
    }

    fn solve_hinge(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;

        if let Some(thresh) = break_threshold {
            if delta.length() > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let correction = delta / total_inv_mass * 0.8;
        if !a.is_static {
            a.position += correction * a.inv_mass;
        }
        if !b.is_static {
            b.position -= correction * b.inv_mass;
        }
        false
    }

    fn solve_angular_limit(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        min: f32,
        max: f32,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        let relative_angle = b.angle - a.angle;
        let correction = if relative_angle < min {
            min - relative_angle
        } else if relative_angle > max {
            max - relative_angle
        } else {
            0.0
        };

        if let Some(thresh) = break_threshold {
            if correction.abs() > thresh {
                return true;
            }
        }

        if correction.abs() < 1e-8 {
            return false;
        }

        let total_inv_inertia = a.inv_inertia + b.inv_inertia;
        if total_inv_inertia < 1e-10 {
            return false;
        }

        let correction_angle = correction / total_inv_inertia * 0.5;
        if !a.is_static {
            a.angle -= correction_angle * a.inv_inertia;
            a.angular_velocity -= correction_angle * a.inv_inertia * 0.5;
        }
        if !b.is_static {
            b.angle += correction_angle * b.inv_inertia;
            b.angular_velocity += correction_angle * b.inv_inertia * 0.5;
        }
        false
    }

    /// Solve a 3D ball-and-socket constraint: keep both bodies at the anchor point.
    fn solve_ball_socket_3d(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        anchor: &glam::Vec3,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        let delta = *anchor - a.position_3d;
        let delta_b = *anchor - b.position_3d;

        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        // Simple position correction: move both bodies toward the anchor
        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }
        let correction = (delta - delta_b) * 0.5;
        if !a.is_static {
            a.position_3d += correction * a.inv_mass / total_inv_mass;
        }
        if !b.is_static {
            b.position_3d -= correction * b.inv_mass / total_inv_mass;
        }
        false
    }

    /// Solve a 3D hinge constraint: bodies rotate around a shared axis.
    fn solve_hinge_3d(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        anchor: &glam::Vec3,
        axis: &glam::Vec3,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        // Position correction: keep anchor points aligned
        let delta = *anchor - a.position_3d;
        let delta_b = *anchor - b.position_3d;

        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }
        let correction = (delta - delta_b) * 0.5;
        if !a.is_static {
            a.position_3d += correction * a.inv_mass / total_inv_mass;
        }
        if !b.is_static {
            b.position_3d -= correction * b.inv_mass / total_inv_mass;
        }

        // Rotation correction: align the axis
        // Simplified: just damp angular velocity around the hinge axis
        let rel_ang_vel = a.angular_velocity_3d - b.angular_velocity_3d;
        let axis_component = axis * rel_ang_vel.dot(*axis);
        let perp_component = rel_ang_vel - axis_component;
        // Remove perpendicular angular velocity (keep rotation around axis only)
        let total_inv_inertia = 1.0 / (a.inv_inertia_3d.x + b.inv_inertia_3d.x + 1e-10);
        let angular_impulse = perp_component * total_inv_inertia * 0.5;
        if !a.is_static {
            a.angular_velocity_3d -= angular_impulse;
        }
        if !b.is_static {
            b.angular_velocity_3d += angular_impulse;
        }
        false
    }

    /// Solve a prismatic (slider) joint: bodies translate along a shared axis.
    fn solve_prismatic(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        axis: &Vec2,
        min_limit: f32,
        max_limit: f32,
        enable_motor: bool,
        motor_speed: f32,
        motor_max_force: f32,
        break_threshold: Option<f32>,
        dt: f32,
    ) -> bool {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;

        // Project delta onto the constrained axis and perpendicular plane
        let axis_n = *axis; // should be normalized by caller
        let along_axis = axis_n * delta.dot(axis_n);
        let perp = delta - along_axis;

        // Perpendicular correction: remove any deviation from the axis line
        let perp_dist = perp.length();

        // Check break threshold based on perpendicular strain
        if let Some(thresh) = break_threshold {
            if perp_dist > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        // Position correction for perpendicular deviation
        if perp_dist > 1e-4 {
            let perp_dir = perp / perp_dist;
            let correction = perp_dir * (perp_dist * 0.5 / total_inv_mass);
            if !a.is_static {
                a.position += correction * a.inv_mass;
            }
            if !b.is_static {
                b.position -= correction * b.inv_mass;
            }
        }

        // Axis limit enforcement
        let along_dist = delta.dot(axis_n);
        if along_dist < min_limit - 1e-4 {
            let correction = axis_n * ((min_limit - along_dist) * 0.5 / total_inv_mass);
            if !a.is_static {
                a.position += correction * a.inv_mass;
            }
            if !b.is_static {
                b.position -= correction * b.inv_mass;
            }
        } else if along_dist > max_limit + 1e-4 {
            let correction = axis_n * ((max_limit - along_dist) * 0.5 / total_inv_mass);
            if !a.is_static {
                a.position += correction * a.inv_mass;
            }
            if !b.is_static {
                b.position -= correction * b.inv_mass;
            }
        }

        // Motor: apply force along the axis to drive toward target speed
        if enable_motor && motor_max_force > 0.0 {
            let rel_vel = b.velocity - a.velocity;
            let along_vel = rel_vel.dot(axis_n);
            let vel_error = motor_speed - along_vel;
            let motor_force =
                (vel_error * total_inv_mass / dt).clamp(-motor_max_force, motor_max_force);
            let impulse = axis_n * motor_force * dt;

            if !a.is_static {
                a.velocity -= impulse * a.inv_mass;
            }
            if !b.is_static {
                b.velocity += impulse * b.inv_mass;
            }
        }

        // Lock relative rotation (simplified: both bodies get same angular velocity)
        let avg_ang = (a.angular_velocity + b.angular_velocity) * 0.5;
        if !a.is_static {
            a.angular_velocity = avg_ang;
        }
        if !b.is_static {
            b.angular_velocity = avg_ang;
        }

        false
    }

    /// Solve a motor constraint: drives a body toward a target velocity.
    fn solve_motor(
        &self,
        a: &mut RigidBody,
        _b: &mut RigidBody,
        target_velocity: Vec2,
        max_force: f32,
        damping: f32,
        dt: f32,
    ) -> bool {
        if a.is_static || a.inv_mass < 1e-10 {
            return false;
        }

        let vel_error = target_velocity - a.velocity;
        let force_magnitude = vel_error.length() * a.mass / dt;

        if force_magnitude > max_force {
            let dir = vel_error.normalize();
            a.velocity += dir * (max_force * dt * a.inv_mass);
        } else {
            // Apply with damping
            a.velocity += vel_error * (1.0 - damping);
        }

        false
    }

    /// Solve a weld joint: locks all relative translation and rotation.
    fn solve_weld(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;

        let strain = delta.length();
        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        // Position correction: force anchor points to coincide
        let correction = delta * (0.5 / total_inv_mass);
        if !a.is_static {
            a.position += correction * a.inv_mass;
        }
        if !b.is_static {
            b.position -= correction * b.inv_mass;
        }

        // Angular correction: lock relative rotation
        let avg_ang = (a.angular_velocity + b.angular_velocity) * 0.5;
        if !a.is_static {
            a.angular_velocity = avg_ang;
        }
        if !b.is_static {
            b.angular_velocity = avg_ang;
        }

        false
    }

    /// Solve a 6-DOF joint: per-axis linear and angular limits in 3D.
    fn solve_six_dof(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        anchor: &glam::Vec3,
        _primary_axis: &glam::Vec3,
        linear_limits: [f32; 6],
        angular_limits: [f32; 6],
        linear_locked: [bool; 3],
        angular_locked: [bool; 3],
        break_threshold: Option<f32>,
        _dt: f32,
    ) -> bool {
        // Position correction: anchor points must coincide
        let delta = *anchor - a.position_3d;
        let delta_b = *anchor - b.position_3d;
        let strain = delta.length().max(delta_b.length());

        if let Some(thresh) = break_threshold {
            if strain > thresh {
                return true;
            }
        }

        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        // Linear correction: enforce per-axis limits
        let rel_pos = b.position_3d - a.position_3d;
        let mut correction = glam::Vec3::ZERO;

        for i in 0..3 {
            let axis = match i {
                0 => glam::Vec3::X,
                1 => glam::Vec3::Y,
                _ => glam::Vec3::Z,
            };
            let along = rel_pos.dot(axis);

            if linear_locked[i] {
                // Fully locked: remove all displacement along this axis
                correction += axis * along;
            } else {
                // Limited: clamp to min/max
                let min = linear_limits[i * 2];
                let max = linear_limits[i * 2 + 1];
                if along < min {
                    correction += axis * (min - along);
                } else if along > max {
                    correction += axis * (max - along);
                }
            }
        }

        if correction.length_squared() > 1e-10 {
            let total_inv = total_inv_mass;
            if !a.is_static {
                a.position_3d += correction * (a.inv_mass / total_inv) * 0.5;
            }
            if !b.is_static {
                b.position_3d -= correction * (b.inv_mass / total_inv) * 0.5;
            }
        }

        // Angular correction: enforce per-axis rotation limits
        // Simplified: decompose relative angular velocity and damp constrained axes
        let rel_ang = b.angular_velocity_3d - a.angular_velocity_3d;
        let mut ang_correction = glam::Vec3::ZERO;

        for i in 0..3 {
            let axis = match i {
                0 => glam::Vec3::X,
                1 => glam::Vec3::Y,
                _ => glam::Vec3::Z,
            };
            let along = rel_ang.dot(axis);

            if angular_locked[i] {
                // Fully locked: remove all angular velocity along this axis
                ang_correction += axis * along;
            } else {
                // Limited: damp angular velocity outside limits
                let min = angular_limits[i * 2];
                let max = angular_limits[i * 2 + 1];
                if along < min {
                    ang_correction += axis * (min - along);
                } else if along > max {
                    ang_correction += axis * (max - along);
                }
            }
        }

        if ang_correction.length_squared() > 1e-10 {
            let total_inv_inertia = 1.0 / (a.inv_inertia_3d.x + b.inv_inertia_3d.x + 1e-10);
            let angular_impulse = ang_correction * total_inv_inertia * 0.5;
            if !a.is_static {
                a.angular_velocity_3d -= angular_impulse;
            }
            if !b.is_static {
                b.angular_velocity_3d += angular_impulse;
            }
        }

        false
    }
}
