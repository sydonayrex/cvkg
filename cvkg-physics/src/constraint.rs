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
    BallSocket3D {
        anchor: glam::Vec3,
    },
    /// Hinge joint in 3D: bodies rotate around a shared axis.
    Hinge3D {
        anchor: glam::Vec3,
        axis: glam::Vec3,
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
}

impl Constraint {
    /// Create a pin constraint at a world-space anchor.
    pub fn pin(body_a: BodyId, body_b: BodyId, anchor: Vec2) -> Self {
        Self {
            body_a,
            body_b,
            kind: ConstraintKind::Pin { anchor },
            enabled: true,
        }
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
    }
}
