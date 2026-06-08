//! Distance, pin, hinge, and angular limit constraints.

use glam::Vec2;

use crate::BodyId;

/// Constraint kinds for connecting bodies.
#[derive(Debug, Clone)]
pub enum ConstraintKind {
    /// Pin two bodies together at a world-space point (zero distance, fixed).
    Pin {
        /// World-space anchor point where the two bodies are pinned.
        anchor: Vec2,
    },
    /// Maintain a fixed distance between two points on two bodies.
    Distance {
        /// Local anchor on body A.
        local_anchor_a: Vec2,
        /// Local anchor on body B.
        local_anchor_b: Vec2,
        /// Desired distance.
        distance: f32,
        /// Compliance (0 = rigid, higher = softer). For XPBD.
        compliance: f32,
        /// Damping factor for compliance (0–1).
        damping: f32,
    },
    /// Hinge joint: bodies rotate relative to a shared anchor point.
    Hinge {
        /// Local anchor on body A.
        local_anchor_a: Vec2,
        /// Local anchor on body B.
        local_anchor_b: Vec2,
        /// Whether to enable angle limits.
        enable_limits: bool,
        /// Minimum angle (radians) relative to body A.
        min_angle: f32,
        /// Maximum angle (radians) relative to body A.
        max_angle: f32,
    },
    /// Angular limit: constrain the relative angle between two bodies.
    AngularLimit {
        /// Minimum angle of body B relative to body A.
        min_angle: f32,
        /// Maximum angle of body B relative to body A.
        max_angle: f32,
    },
    /// Spring constraint: bodies are connected with a damped spring.
    Spring {
        /// Local anchor on body A.
        local_anchor_a: Vec2,
        /// Local anchor on body B.
        local_anchor_b: Vec2,
        /// Rest length.
        rest_length: f32,
        /// Spring constant (stiffness).
        stiffness: f32,
        /// Damping coefficient.
        damping: f32,
    },
    /// Ball-and-socket joint in 3D: pins two bodies at a world-space point.
    BallSocket3D { anchor: glam::Vec3 },
    /// Hinge joint in 3D: bodies rotate around a shared axis.
    Hinge3D {
        anchor: glam::Vec3,
        axis: glam::Vec3,
    },
    /// Prismatic (slider) joint: bodies translate along a shared axis.
    Prismatic {
        /// Local anchor on body A.
        local_anchor_a: Vec2,
        /// Local anchor on body B.
        local_anchor_b: Vec2,
        /// Axis of translation in body A's local space.
        axis: Vec2,
        /// Minimum translation limit.
        min_limit: f32,
        /// Maximum translation limit.
        max_limit: f32,
        /// Whether to enable a motor along the axis.
        enable_motor: bool,
        /// Motor target speed (along axis).
        motor_speed: f32,
        /// Motor maximum force.
        motor_max_force: f32,
    },
    /// Motor constraint: drives relative velocity between two bodies.
    Motor {
        /// Body to drive.
        body: BodyId,
        /// Target linear velocity in world space.
        target_velocity: Vec2,
        /// Maximum force the motor can apply.
        max_force: f32,
        /// Damping coefficient (0 = no damping, 1 = critical damping).
        damping: f32,
    },
    /// Weld joint: locks all relative motion between two bodies.
    Weld {
        /// Local anchor on body A.
        local_anchor_a: Vec2,
        /// Local anchor on body B.
        local_anchor_b: Vec2,
    },
    /// 6-DOF joint: generic joint with per-axis linear and angular limits (3D).
    SixDof {
        /// Anchor point in world space.
        anchor: glam::Vec3,
        /// Primary axis (e.g., the axis of a gizmo handle).
        primary_axis: glam::Vec3,
        /// Linear limits along each axis (min_x, max_x, min_y, max_y, min_z, max_z).
        linear_limits: [f32; 6],
        /// Angular limits around each axis (min_x, max_x, min_y, max_y, min_z, max_z).
        angular_limits: [f32; 6],
        /// Whether each linear axis is locked (true = fixed, false = free/limited).
        linear_locked: [bool; 3],
        /// Whether each angular axis is locked.
        angular_locked: [bool; 3],
    },
}

/// A constraint connecting two bodies.
#[derive(Debug, Clone)]
pub struct Constraint {
    /// First body.
    pub body_a: BodyId,
    /// Second body.
    pub body_b: BodyId,
    /// The constraint kind.
    pub kind: ConstraintKind,
    /// Whether this constraint is active.
    pub enabled: bool,
    /// Optional threshold for breaking the constraint. If the strain exceeds this, it breaks.
    pub break_threshold: Option<f32>,
}

impl Constraint {
    /// Create a pin constraint at a world-space anchor.
    pub fn pin(body_a: BodyId, body_b: BodyId, anchor: Vec2) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Pin { anchor },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Add a breaking threshold to the constraint (can snap if strain exceeds this).
    pub fn with_breaking_threshold(mut self, threshold: f32) -> Self {
        self.break_threshold = Some(threshold);
        self
    }

    /// Create a distance constraint.
    pub fn distance(
        body_a: BodyId,
        body_b: BodyId,
        local_a: Vec2,
        local_b: Vec2,
        dist: f32,
    ) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Distance {
                local_anchor_a: local_a,
                local_anchor_b: local_b,
                distance: dist,
                compliance: 0.0,
                damping: 0.0,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Create a hinge joint.
    pub fn hinge(body_a: BodyId, body_b: BodyId, local_a: Vec2, local_b: Vec2) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Hinge {
                local_anchor_a: local_a,
                local_anchor_b: local_b,
                enable_limits: false,
                min_angle: 0.0,
                max_angle: 0.0,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Create a spring constraint.
    pub fn spring(
        body_a: BodyId,
        body_b: BodyId,
        local_a: Vec2,
        local_b: Vec2,
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    ) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Spring {
                local_anchor_a: local_a,
                local_anchor_b: local_b,
                rest_length,
                stiffness,
                damping,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Create an angular limit constraint.
    pub fn angular_limit(body_a: BodyId, body_b: BodyId, min: f32, max: f32) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::AngularLimit {
                min_angle: min,
                max_angle: max,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Create a prismatic (slider) joint.
    pub fn prismatic(
        body_a: BodyId,
        body_b: BodyId,
        local_a: Vec2,
        local_b: Vec2,
        axis: Vec2,
    ) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Prismatic {
                local_anchor_a: local_a,
                local_anchor_b: local_b,
                axis,
                min_limit: 0.0,
                max_limit: f32::INFINITY,
                enable_motor: false,
                motor_speed: 0.0,
                motor_max_force: 0.0,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Configure prismatic joint with limits and motor.
    pub fn with_prismatic_limits(mut self, min: f32, max: f32) -> Self {
        if let ConstraintKind::Prismatic {
            min_limit,
            max_limit,
            ..
        } = &mut self.kind
        {
            *min_limit = min;
            *max_limit = max;
        }
        self
    }

    /// Enable motor on prismatic joint.
    pub fn with_prismatic_motor(mut self, speed: f32, max_force: f32) -> Self {
        if let ConstraintKind::Prismatic {
            enable_motor,
            motor_speed,
            motor_max_force,
            ..
        } = &mut self.kind
        {
            *enable_motor = true;
            *motor_speed = speed;
            *motor_max_force = max_force;
        }
        self
    }

    /// Create a motor constraint that drives a body toward a target velocity.
    pub fn motor(body: BodyId, target_velocity: Vec2, max_force: f32) -> Self {
        Self {
            body_a: body,
            body_b: body,
            kind: ConstraintKind::Motor {
                body,
                target_velocity,
                max_force,
                damping: 0.5,
            },
            enabled: true,
            break_threshold: None,
        }
    }

    /// Create a weld joint that locks all relative motion.
    pub fn weld(body_a: BodyId, body_b: BodyId, local_a: Vec2, local_b: Vec2) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Weld {
                local_anchor_a: local_a,
                local_anchor_b: local_b,
            },
            enabled: true,
            break_threshold: None,
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3D Constraints
// ══════════════════════════════════════════════════════════════════════════

/// Ball-and-socket joint in 3D: pins two bodies together at a world-space point.
/// Allows free rotation but prevents translation away from the anchor.
pub fn ball_socket_constraint(
    body_a: BodyId,
    body_b: BodyId,
    anchor: glam::Vec3,
) -> super::Constraint {
    super::Constraint {
        body_a,
        body_b,
        kind: ConstraintKind::BallSocket3D { anchor },
        enabled: true,
        break_threshold: None,
    }
}

/// Hinge joint in 3D: bodies rotate around a shared axis passing through an anchor point.
pub fn hinge_constraint_3d(
    body_a: BodyId,
    body_b: BodyId,
    anchor: glam::Vec3,
    axis: glam::Vec3,
) -> super::Constraint {
    super::Constraint {
        body_a,
        body_b,
        kind: ConstraintKind::Hinge3D { anchor, axis },
        enabled: true,
        break_threshold: None,
    }
}

/// 6-DOF joint: generic joint with per-axis linear and angular limits (3D).
/// Useful for gizmo manipulation, robotic arms, and configurable 3D constraints.
pub fn six_dof_constraint(
    body_a: BodyId,
    body_b: BodyId,
    anchor: glam::Vec3,
    primary_axis: glam::Vec3,
) -> super::Constraint {
    super::Constraint {
        body_a,
        body_b,
        kind: ConstraintKind::SixDof {
            anchor,
            primary_axis,
            linear_limits: [f32::NEG_INFINITY; 6],
            angular_limits: [f32::NEG_INFINITY; 6],
            linear_locked: [false; 3],
            angular_locked: [false; 3],
        },
        enabled: true,
        break_threshold: None,
    }
}
