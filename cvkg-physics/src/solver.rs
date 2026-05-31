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

    /// Solve all constraints, applying impulses to bodies.
    pub fn solve(
        &self,
        constraints: &[Constraint],
        bodies: &mut [RigidBody],
        body_id_map: &std::collections::HashMap<BodyId, usize>,
        dt: f32,
    ) {
        for _ in 0..self.iterations {
            for constraint in constraints {
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

                self.solve_constraint(constraint, body_a, body_b, dt);
            }
        }
    }

    fn solve_constraint(
        &self,
        constraint: &Constraint,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
        dt: f32,
    ) {
        match &constraint.kind {
            ConstraintKind::Distance {
                local_anchor_a,
                local_anchor_b,
                distance,
                ..
            } => {
                self.solve_distance(
                    body_a,
                    body_b,
                    local_anchor_a,
                    local_anchor_b,
                    *distance,
                    dt,
                );
            }
            ConstraintKind::Pin { anchor } => {
                self.solve_pin(body_a, body_b, anchor, dt);
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
                self.solve_spring(body_a, body_b, local_anchor_a, local_anchor_b, &config, dt);
            }
            ConstraintKind::Hinge {
                local_anchor_a,
                local_anchor_b,
                ..
            } => {
                self.solve_hinge(body_a, body_b, local_anchor_a, local_anchor_b, dt);
            }
            ConstraintKind::AngularLimit {
                min_angle,
                max_angle,
            } => {
                self.solve_angular_limit(body_a, body_b, *min_angle, *max_angle, dt);
            }
        }
    }

    fn solve_distance(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        distance: f32,
        _dt: f32,
    ) {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;
        let current_dist = delta.length();
        if current_dist < 1e-10 {
            return;
        }

        let correction = delta * ((current_dist - distance) / current_dist);
        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return;
        }

        let imp = correction / total_inv_mass;
        if !a.is_static {
            a.position += imp * a.inv_mass;
        }
        if !b.is_static {
            b.position -= imp * b.inv_mass;
        }
    }

    fn solve_pin(&self, a: &mut RigidBody, b: &mut RigidBody, anchor: &Vec2, _dt: f32) {
        // Pin: both body anchor points must coincide at `anchor`
        let world_a = a.position; // For pin, we use body center
        let world_b = b.position;
        let delta = *anchor - world_a;
        let delta_b = *anchor - world_b;
        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return;
        }

        if !a.is_static {
            a.position += delta * a.inv_mass / total_inv_mass * 0.8;
        }
        if !b.is_static {
            b.position += delta_b * b.inv_mass / total_inv_mass * 0.8;
        }
    }

    fn solve_spring(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        config: &SpringConfig,
        dt: f32,
    ) {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;
        let dist = delta.length();
        if dist < 1e-10 {
            return;
        }

        let dir = delta / dist;
        let displacement = dist - config.rest_length;

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
    }

    fn solve_hinge(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        local_a: &Vec2,
        local_b: &Vec2,
        _dt: f32,
    ) {
        let world_a = a.local_to_world(*local_a);
        let world_b = b.local_to_world(*local_b);
        let delta = world_b - world_a;
        let total_inv_mass = a.inv_mass + b.inv_mass;
        if total_inv_mass < 1e-10 {
            return;
        }

        let correction = delta / total_inv_mass * 0.8;
        if !a.is_static {
            a.position += correction * a.inv_mass;
        }
        if !b.is_static {
            b.position -= correction * b.inv_mass;
        }
    }

    fn solve_angular_limit(
        &self,
        a: &mut RigidBody,
        b: &mut RigidBody,
        min: f32,
        max: f32,
        _dt: f32,
    ) {
        let relative_angle = b.angle - a.angle;
        let correction = if relative_angle < min {
            min - relative_angle
        } else if relative_angle > max {
            max - relative_angle
        } else {
            0.0
        };

        if correction.abs() < 1e-8 {
            return;
        }

        let total_inv_inertia = a.inv_inertia + b.inv_inertia;
        if total_inv_inertia < 1e-10 {
            return;
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
    }
}
