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

/// Body pair context for constraint solver methods.
struct SolverCtx<'a> {
    a: &'a mut RigidBody,
    b: &'a mut RigidBody,
    break_threshold: Option<f32>,
    dt: f32,
}

/// Anchor pair for constraints that use local anchors on both bodies.
struct AnchorPair<'a> {
    local_a: &'a Vec2,
    local_b: &'a Vec2,
}

/// Prismatic joint parameters.
struct PrismaticParams<'a> {
    local_a: &'a Vec2,
    local_b: &'a Vec2,
    axis: &'a Vec2,
    min_limit: f32,
    max_limit: f32,
    enable_motor: bool,
    motor_speed: f32,
    motor_max_force: f32,
}

/// 6-DOF joint parameters.
struct SixDofParams<'a> {
    anchor: &'a glam::Vec3,
    primary_axis: &'a glam::Vec3,
    linear_limits: [f32; 6],
    angular_limits: [f32; 6],
    linear_locked: [bool; 3],
    angular_locked: [bool; 3],
}

/// Impulse-based constraint solver using Gauss-Seidel iteration.
///
/// Solves velocity-level constraints by computing corrective impulses
/// that are applied to bodies each iteration.
#[derive(Debug, Default)]
pub struct ImpulseSolver {
    /// Number of solver iterations per step. More = stiffer, more expensive.
    pub iterations: usize,
    /// Baumgarte factor for position drift correction (0.0-1.0).
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

                let mut ctx = SolverCtx {
                    a: body_a,
                    b: body_b,
                    break_threshold: constraint.break_threshold,
                    dt,
                };

                match &constraint.kind {
                    ConstraintKind::Distance {
                        local_anchor_a,
                        local_anchor_b,
                        distance,
                        ..
                    } => {
                        let anchors = AnchorPair {
                            local_a: local_anchor_a,
                            local_b: local_anchor_b,
                        };
                        if self.solve_distance(&mut ctx, &anchors, *distance) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::Pin { anchor } => {
                        if self.solve_pin(&mut ctx, anchor) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
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
                        let anchors = AnchorPair {
                            local_a: local_anchor_a,
                            local_b: local_anchor_b,
                        };
                        if self.solve_spring(&mut ctx, &anchors, &config) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::Hinge {
                        local_anchor_a,
                        local_anchor_b,
                        ..
                    } => {
                        let anchors = AnchorPair {
                            local_a: local_anchor_a,
                            local_b: local_anchor_b,
                        };
                        if self.solve_hinge(&mut ctx, &anchors) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::AngularLimit {
                        min_angle,
                        max_angle,
                    } => {
                        if self.solve_angular_limit(&mut ctx, *min_angle, *max_angle) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::BallSocket3D { anchor } => {
                        if self.solve_ball_socket_3d(&mut ctx, anchor) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::Hinge3D { anchor, axis } => {
                        if self.solve_hinge_3d(&mut ctx, anchor, axis) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
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
                    } => {
                        let params = PrismaticParams {
                            local_a: local_anchor_a,
                            local_b: local_anchor_b,
                            axis,
                            min_limit: *min_limit,
                            max_limit: *max_limit,
                            enable_motor: *enable_motor,
                            motor_speed: *motor_speed,
                            motor_max_force: *motor_max_force,
                        };
                        if self.solve_prismatic(&mut ctx, &params) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::Motor {
                        body: _,
                        target_velocity,
                        max_force,
                        damping,
                    } => {
                        if self.solve_motor(&mut ctx, *target_velocity, *max_force, *damping) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::Weld {
                        local_anchor_a,
                        local_anchor_b,
                    } => {
                        let anchors = AnchorPair {
                            local_a: local_anchor_a,
                            local_b: local_anchor_b,
                        };
                        if self.solve_weld(&mut ctx, &anchors) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                    ConstraintKind::SixDof {
                        anchor,
                        primary_axis,
                        linear_limits,
                        angular_limits,
                        linear_locked,
                        angular_locked,
                    } => {
                        let params = SixDofParams {
                            anchor,
                            primary_axis,
                            linear_limits: *linear_limits,
                            angular_limits: *angular_limits,
                            linear_locked: *linear_locked,
                            angular_locked: *angular_locked,
                        };
                        if self.solve_six_dof(&mut ctx, &params) {
                            constraint.enabled = false;
                            broken_pairs.push((constraint.body_a, constraint.body_b));
                        }
                    }
                }
            }
        }
        broken_pairs
    }

    fn solve_distance(
        &self,
        ctx: &mut SolverCtx<'_>,
        anchors: &AnchorPair<'_>,
        distance: f32,
    ) -> bool {
        let world_a = ctx.a.local_to_world(*anchors.local_a);
        let world_b = ctx.b.local_to_world(*anchors.local_b);
        let delta = world_b - world_a;
        let current_dist = delta.length();
        if current_dist < 1e-10 {
            return false;
        }

        let strain = (current_dist - distance).abs();
        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let correction = delta * ((current_dist - distance) / current_dist);
        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let imp = correction / total_inv_mass;
        if !ctx.a.is_static {
            ctx.a.position += imp * ctx.a.inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position -= imp * ctx.b.inv_mass;
        }
        false
    }

    fn solve_pin(&self, ctx: &mut SolverCtx<'_>, anchor: &Vec2) -> bool {
        let world_a = ctx.a.position;
        let world_b = ctx.b.position;
        let delta = *anchor - world_a;
        let delta_b = *anchor - world_b;
        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        if !ctx.a.is_static {
            ctx.a.position += delta * ctx.a.inv_mass / total_inv_mass * 0.8;
        }
        if !ctx.b.is_static {
            ctx.b.position += delta_b * ctx.b.inv_mass / total_inv_mass * 0.8;
        }
        false
    }

    fn solve_spring(
        &self,
        ctx: &mut SolverCtx<'_>,
        anchors: &AnchorPair<'_>,
        config: &SpringConfig,
    ) -> bool {
        let world_a = ctx.a.local_to_world(*anchors.local_a);
        let world_b = ctx.b.local_to_world(*anchors.local_b);
        let delta = world_b - world_a;
        let dist = delta.length();
        if dist < 1e-10 {
            return false;
        }

        let displacement = dist - config.rest_length;
        if let Some(thresh) = ctx.break_threshold
            && displacement.abs() > thresh
        {
            return true;
        }

        let dir = delta / dist;

        let spring_force = dir * (config.stiffness * displacement);
        let rel_vel = ctx.b.velocity - ctx.a.velocity;
        let damping_force = dir * (rel_vel.dot(dir) * config.damping);
        let total_force = (spring_force + damping_force) * ctx.dt;

        if !ctx.a.is_static {
            ctx.a.velocity += total_force * ctx.a.inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.velocity -= total_force * ctx.b.inv_mass;
        }
        false
    }

    fn solve_hinge(&self, ctx: &mut SolverCtx<'_>, anchors: &AnchorPair<'_>) -> bool {
        let world_a = ctx.a.local_to_world(*anchors.local_a);
        let world_b = ctx.b.local_to_world(*anchors.local_b);
        let delta = world_b - world_a;

        if let Some(thresh) = ctx.break_threshold
            && delta.length() > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let correction = delta / total_inv_mass * 0.8;
        if !ctx.a.is_static {
            ctx.a.position += correction * ctx.a.inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position -= correction * ctx.b.inv_mass;
        }
        false
    }

    fn solve_angular_limit(&self, ctx: &mut SolverCtx<'_>, min: f32, max: f32) -> bool {
        let relative_angle = ctx.b.angle - ctx.a.angle;
        let correction = if relative_angle < min {
            min - relative_angle
        } else if relative_angle > max {
            max - relative_angle
        } else {
            0.0
        };

        if let Some(thresh) = ctx.break_threshold
            && correction.abs() > thresh
        {
            return true;
        }

        if correction.abs() < 1e-8 {
            return false;
        }

        let total_inv_inertia = ctx.a.inv_inertia + ctx.b.inv_inertia;
        if total_inv_inertia < 1e-10 {
            return false;
        }

        let correction_angle = correction / total_inv_inertia * 0.5;
        if !ctx.a.is_static {
            ctx.a.angle -= correction_angle * ctx.a.inv_inertia;
            ctx.a.angular_velocity -= correction_angle * ctx.a.inv_inertia * 0.5;
        }
        if !ctx.b.is_static {
            ctx.b.angle += correction_angle * ctx.b.inv_inertia;
            ctx.b.angular_velocity += correction_angle * ctx.b.inv_inertia * 0.5;
        }
        false
    }

    /// Solve a 3D ball-and-socket constraint: keep both bodies at the anchor point.
    fn solve_ball_socket_3d(&self, ctx: &mut SolverCtx<'_>, anchor: &glam::Vec3) -> bool {
        let delta = *anchor - ctx.a.position_3d;
        let delta_b = *anchor - ctx.b.position_3d;

        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }
        let correction = (delta - delta_b) * 0.5;
        if !ctx.a.is_static {
            ctx.a.position_3d += correction * ctx.a.inv_mass / total_inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position_3d -= correction * ctx.b.inv_mass / total_inv_mass;
        }
        false
    }

    /// Solve a 3D hinge constraint: bodies rotate around a shared axis.
    fn solve_hinge_3d(
        &self,
        ctx: &mut SolverCtx<'_>,
        anchor: &glam::Vec3,
        axis: &glam::Vec3,
    ) -> bool {
        let delta = *anchor - ctx.a.position_3d;
        let delta_b = *anchor - ctx.b.position_3d;

        let strain = delta.length().max(delta_b.length());
        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }
        let correction = (delta - delta_b) * 0.5;
        if !ctx.a.is_static {
            ctx.a.position_3d += correction * ctx.a.inv_mass / total_inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position_3d -= correction * ctx.b.inv_mass / total_inv_mass;
        }

        let rel_ang_vel = ctx.a.angular_velocity_3d - ctx.b.angular_velocity_3d;
        let axis_component = axis * rel_ang_vel.dot(*axis);
        let perp_component = rel_ang_vel - axis_component;
        let total_inv_inertia = 1.0 / (ctx.a.inv_inertia_3d.x + ctx.b.inv_inertia_3d.x + 1e-10);
        let angular_impulse = perp_component * total_inv_inertia * 0.5;
        if !ctx.a.is_static {
            ctx.a.angular_velocity_3d -= angular_impulse;
        }
        if !ctx.b.is_static {
            ctx.b.angular_velocity_3d += angular_impulse;
        }
        false
    }

    /// Solve a prismatic (slider) joint: bodies translate along a shared axis.
    fn solve_prismatic(&self, ctx: &mut SolverCtx<'_>, params: &PrismaticParams<'_>) -> bool {
        let world_a = ctx.a.local_to_world(*params.local_a);
        let world_b = ctx.b.local_to_world(*params.local_b);
        let delta = world_b - world_a;

        let axis_n = *params.axis;
        let along_axis = axis_n * delta.dot(axis_n);
        let perp = delta - along_axis;

        let perp_dist = perp.length();

        if let Some(thresh) = ctx.break_threshold
            && perp_dist > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        if perp_dist > 1e-4 {
            let perp_dir = perp / perp_dist;
            let correction = perp_dir * (perp_dist * 0.5 / total_inv_mass);
            if !ctx.a.is_static {
                ctx.a.position += correction * ctx.a.inv_mass;
            }
            if !ctx.b.is_static {
                ctx.b.position -= correction * ctx.b.inv_mass;
            }
        }

        let along_dist = delta.dot(axis_n);
        if along_dist < params.min_limit - 1e-4 {
            let correction = axis_n * ((params.min_limit - along_dist) * 0.5 / total_inv_mass);
            if !ctx.a.is_static {
                ctx.a.position += correction * ctx.a.inv_mass;
            }
            if !ctx.b.is_static {
                ctx.b.position -= correction * ctx.b.inv_mass;
            }
        } else if along_dist > params.max_limit + 1e-4 {
            let correction = axis_n * ((params.max_limit - along_dist) * 0.5 / total_inv_mass);
            if !ctx.a.is_static {
                ctx.a.position += correction * ctx.a.inv_mass;
            }
            if !ctx.b.is_static {
                ctx.b.position -= correction * ctx.b.inv_mass;
            }
        }

        if params.enable_motor && params.motor_max_force > 0.0 {
            let rel_vel = ctx.b.velocity - ctx.a.velocity;
            let along_vel = rel_vel.dot(axis_n);
            let vel_error = params.motor_speed - along_vel;
            let motor_force = (vel_error * total_inv_mass / ctx.dt)
                .clamp(-params.motor_max_force, params.motor_max_force);
            let impulse = axis_n * motor_force * ctx.dt;

            if !ctx.a.is_static {
                ctx.a.velocity -= impulse * ctx.a.inv_mass;
            }
            if !ctx.b.is_static {
                ctx.b.velocity += impulse * ctx.b.inv_mass;
            }
        }

        let avg_ang = (ctx.a.angular_velocity + ctx.b.angular_velocity) * 0.5;
        if !ctx.a.is_static {
            ctx.a.angular_velocity = avg_ang;
        }
        if !ctx.b.is_static {
            ctx.b.angular_velocity = avg_ang;
        }

        false
    }

    /// Solve a motor constraint: drives a body toward a target velocity.
    fn solve_motor(
        &self,
        ctx: &mut SolverCtx<'_>,
        target_velocity: Vec2,
        max_force: f32,
        damping: f32,
    ) -> bool {
        if ctx.a.is_static || ctx.a.inv_mass < 1e-10 {
            return false;
        }

        let vel_error = target_velocity - ctx.a.velocity;
        let force_magnitude = vel_error.length() * ctx.a.mass / ctx.dt;

        if force_magnitude > max_force {
            let dir = vel_error.normalize();
            ctx.a.velocity += dir * (max_force * ctx.dt * ctx.a.inv_mass);
        } else {
            ctx.a.velocity += vel_error * (1.0 - damping);
        }

        false
    }

    /// Solve a weld joint: locks all relative translation and rotation.
    fn solve_weld(&self, ctx: &mut SolverCtx<'_>, anchors: &AnchorPair<'_>) -> bool {
        let world_a = ctx.a.local_to_world(*anchors.local_a);
        let world_b = ctx.b.local_to_world(*anchors.local_b);
        let delta = world_b - world_a;

        let strain = delta.length();
        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let correction = delta * (0.5 / total_inv_mass);
        if !ctx.a.is_static {
            ctx.a.position += correction * ctx.a.inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position -= correction * ctx.b.inv_mass;
        }

        let avg_ang = (ctx.a.angular_velocity + ctx.b.angular_velocity) * 0.5;
        if !ctx.a.is_static {
            ctx.a.angular_velocity = avg_ang;
        }
        if !ctx.b.is_static {
            ctx.b.angular_velocity = avg_ang;
        }

        false
    }

    /// Solve a 6-DOF joint: per-axis linear and angular limits in 3D.
    fn solve_six_dof(&self, ctx: &mut SolverCtx<'_>, params: &SixDofParams<'_>) -> bool {
        let delta = *params.anchor - ctx.a.position_3d;
        let delta_b = *params.anchor - ctx.b.position_3d;
        let strain = delta.length().max(delta_b.length());

        if let Some(thresh) = ctx.break_threshold
            && strain > thresh
        {
            return true;
        }

        let total_inv_mass = ctx.a.inv_mass + ctx.b.inv_mass;
        if total_inv_mass < 1e-10 {
            return false;
        }

        let rel_pos = ctx.b.position_3d - ctx.a.position_3d;
        let mut correction = glam::Vec3::ZERO;

        for i in 0..3 {
            let axis = match i {
                0 => glam::Vec3::X,
                1 => glam::Vec3::Y,
                _ => glam::Vec3::Z,
            };
            let proj = rel_pos.dot(axis);
            let lo = params.linear_limits[i * 2];
            let hi = params.linear_limits[i * 2 + 1];
            if params.linear_locked[i] && proj < lo {
                correction += axis * (lo - proj);
            } else if params.linear_locked[i] && proj > hi {
                correction += axis * (hi - proj);
            }
        }

        let pos_correction = correction * (0.5 / total_inv_mass);
        if !ctx.a.is_static {
            ctx.a.position_3d += pos_correction * ctx.a.inv_mass;
        }
        if !ctx.b.is_static {
            ctx.b.position_3d -= pos_correction * ctx.b.inv_mass;
        }

        // Angular limits around the primary axis
        if params.angular_locked[0] || params.angular_locked[1] || params.angular_locked[2] {
            let rel_ang = ctx.b.angular_velocity_3d - ctx.a.angular_velocity_3d;
            let mut ang_correction = glam::Vec3::ZERO;
            let axes = [glam::Vec3::X, glam::Vec3::Y, *params.primary_axis];
            for (i, axis) in axes.iter().enumerate() {
                let ang_proj = rel_ang.dot(*axis);
                let ang_lo = params.angular_limits[i * 2];
                let ang_hi = params.angular_limits[i * 2 + 1];
                if params.angular_locked[i] && ang_proj < ang_lo {
                    ang_correction += axis * (ang_lo - ang_proj);
                } else if params.angular_locked[i] && ang_proj > ang_hi {
                    ang_correction += axis * (ang_hi - ang_proj);
                }
            }
            let total_inv_inertia = ctx.a.inv_inertia_3d + ctx.b.inv_inertia_3d;
            if total_inv_inertia.length_squared() > 1e-10 {
                let ang_impulse = ang_correction * 0.5;
                if !ctx.a.is_static {
                    ctx.a.angular_velocity_3d += ang_impulse * ctx.a.inv_inertia_3d;
                }
                if !ctx.b.is_static {
                    ctx.b.angular_velocity_3d -= ang_impulse * ctx.b.inv_inertia_3d;
                }
            }
        }

        false
    }
}
