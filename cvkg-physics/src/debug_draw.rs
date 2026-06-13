//! Physics debug visualization API.
//!
//! Provides draw commands for contact normals, AABBs, joint anchors,
//! velocity vectors, and spatial hash grids. The Renderer trait
//! implementations consume these commands to render debug overlays.

use glam::{Vec2, Vec3};

/// A single debug draw command.
#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    /// Draw a line segment in world space.
    Line {
        /// Start point (world space).
        start: Vec2,
        /// End point (world space).
        end: Vec2,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a line segment in 3D world space.
    Line3D {
        /// Start point (world space).
        start: Vec3,
        /// End point (world space).
        end: Vec3,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw an axis-aligned bounding box.
    Aabb {
        /// Minimum corner.
        min: Vec2,
        /// Maximum corner.
        max: Vec2,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a 3D axis-aligned bounding box.
    Aabb3D {
        /// Minimum corner.
        min: Vec3,
        /// Maximum corner.
        max: Vec3,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a circle (outline).
    Circle {
        /// Center point.
        center: Vec2,
        /// Radius.
        radius: f32,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a point/cross marker.
    Point {
        /// Position.
        pos: Vec2,
        /// Size in pixels.
        size: f32,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a 3D point/cross marker.
    Point3D {
        /// Position.
        pos: Vec3,
        /// Size in world units.
        size: f32,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a vector arrow (line with direction indicator).
    Arrow {
        /// Origin.
        origin: Vec2,
        /// Direction and magnitude.
        direction: Vec2,
        /// RGBA color.
        color: [f32; 4],
    },
    /// Draw a text label at a world position.
    Text {
        /// Position.
        pos: Vec2,
        /// Label text.
        text: String,
        /// RGBA color.
        color: [f32; 4],
    },
}

impl DebugDrawCommand {
    /// Create a line command with white color.
    pub fn line(start: Vec2, end: Vec2) -> Self {
        Self::Line {
            start,
            end,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create a 3D line command with white color.
    pub fn line_3d(start: Vec3, end: Vec3) -> Self {
        Self::Line3D {
            start,
            end,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create an AABB command with white color.
    pub fn aabb(min: Vec2, max: Vec2) -> Self {
        Self::Aabb {
            min,
            max,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create a circle command with white color.
    pub fn circle(center: Vec2, radius: f32) -> Self {
        Self::Circle {
            center,
            radius,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create a point command with white color.
    pub fn point(pos: Vec2, size: f32) -> Self {
        Self::Point {
            pos,
            size,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create an arrow command with white color.
    pub fn arrow(origin: Vec2, direction: Vec2) -> Self {
        Self::Arrow {
            origin,
            direction,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set the color of any command.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        match &mut self {
            Self::Line { color: c, .. } => *c = color,
            Self::Line3D { color: c, .. } => *c = color,
            Self::Aabb { color: c, .. } => *c = color,
            Self::Aabb3D { color: c, .. } => *c = color,
            Self::Circle { color: c, .. } => *c = color,
            Self::Point { color: c, .. } => *c = color,
            Self::Point3D { color: c, .. } => *c = color,
            Self::Arrow { color: c, .. } => *c = color,
            Self::Text { color: c, .. } => *c = color,
        }
        self
    }
}

/// Preset colors for debug visualization.
pub mod colors {
    pub const RED: [f32; 4] = [1.0, 0.2, 0.2, 1.0];
    pub const GREEN: [f32; 4] = [0.2, 1.0, 0.2, 1.0];
    pub const BLUE: [f32; 4] = [0.2, 0.4, 1.0, 1.0];
    pub const YELLOW: [f32; 4] = [1.0, 1.0, 0.2, 1.0];
    pub const CYAN: [f32; 4] = [0.2, 1.0, 1.0, 1.0];
    pub const MAGENTA: [f32; 4] = [1.0, 0.2, 1.0, 1.0];
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    pub const ORANGE: [f32; 4] = [1.0, 0.6, 0.1, 1.0];

    /// Contact normal color.
    pub const CONTACT_NORMAL: [f32; 4] = CYAN;
    /// Contact point color.
    pub const CONTACT_POINT: [f32; 4] = RED;
    /// AABB / bounding volume color.
    pub const BOUNDING_BOX: [f32; 4] = GREEN;
    /// Joint anchor color.
    pub const JOINT_ANCHOR: [f32; 4] = YELLOW;
    /// Joint limit color.
    pub const JOINT_LIMIT: [f32; 4] = ORANGE;
    /// Velocity vector color.
    pub const VELOCITY: [f32; 4] = BLUE;
    /// Sleeping body color.
    pub const SLEEPING: [f32; 4] = [0.5, 0.5, 0.5, 0.5];
    /// Spatial hash grid color.
    pub const SPATIAL_HASH: [f32; 4] = [0.3, 0.3, 0.3, 0.3];
}

/// Collect debug draw commands for contact manifolds.
pub fn draw_contacts(contacts: &[crate::Contact], out: &mut Vec<DebugDrawCommand>) {
    use colors::*;
    for contact in contacts {
        // Contact point
        out.push(DebugDrawCommand::point(contact.point, 3.0).with_color(CONTACT_POINT));
        // Contact normal (short line)
        let normal_end = contact.point + contact.normal * 15.0;
        out.push(DebugDrawCommand::line(contact.point, normal_end).with_color(CONTACT_NORMAL));
    }
}

/// Collect debug draw commands for body AABBs.
pub fn draw_body_aabbs(
    bodies: &[crate::RigidBody],
    colliders: &[crate::Collider],
    body_id_map: &std::collections::HashMap<crate::BodyId, usize>,
    out: &mut Vec<DebugDrawCommand>,
) {
    use colors::*;
    for (body_id, &idx) in body_id_map {
        if let Some(body) = bodies.get(idx) {
            if body.is_sleeping {
                continue;
            }
            let is_3d = body.is_3d;
            // Find colliders for this body
            for collider in colliders {
                if collider.body_id != *body_id {
                    continue;
                }
                let radius = collider.shape.bounding_radius();
                if is_3d {
                    let pos = body.position_3d;
                    out.push(DebugDrawCommand::Aabb3D {
                        min: pos - Vec3::splat(radius),
                        max: pos + Vec3::splat(radius),
                        color: BOUNDING_BOX,
                    });
                } else {
                    let pos = body.position;
                    out.push(
                        DebugDrawCommand::aabb(
                            pos - Vec2::splat(radius),
                            pos + Vec2::splat(radius),
                        )
                        .with_color(BOUNDING_BOX),
                    );
                }
            }
        }
    }
}

/// Collect debug draw commands for joint anchors and limits.
pub fn draw_constraints(
    constraints: &[crate::Constraint],
    bodies: &[crate::RigidBody],
    body_id_map: &std::collections::HashMap<crate::BodyId, usize>,
    out: &mut Vec<DebugDrawCommand>,
) {
    use colors::*;
    for constraint in constraints {
        if !constraint.enabled {
            continue;
        }
        let pos_a = body_id_map
            .get(&constraint.body_a)
            .and_then(|&idx| bodies.get(idx))
            .map(|b| {
                if b.is_3d {
                    b.position_3d.truncate()
                } else {
                    b.position
                }
            })
            .unwrap_or(Vec2::ZERO);
        let pos_b = body_id_map
            .get(&constraint.body_b)
            .and_then(|&idx| bodies.get(idx))
            .map(|b| {
                if b.is_3d {
                    b.position_3d.truncate()
                } else {
                    b.position
                }
            })
            .unwrap_or(Vec2::ZERO);

        match &constraint.kind {
            crate::constraint::ConstraintKind::Pin { anchor } => {
                out.push(DebugDrawCommand::point(*anchor, 4.0).with_color(JOINT_ANCHOR));
                out.push(DebugDrawCommand::line(pos_a, *anchor).with_color(JOINT_LIMIT));
                out.push(DebugDrawCommand::line(pos_b, *anchor).with_color(JOINT_LIMIT));
            }
            crate::constraint::ConstraintKind::Hinge {
                local_anchor_a: _,
                local_anchor_b: _,
                ..
            } => {
                out.push(DebugDrawCommand::line(pos_a, pos_b).with_color(JOINT_LIMIT));
                out.push(DebugDrawCommand::point(pos_a, 4.0).with_color(JOINT_ANCHOR));
            }
            crate::constraint::ConstraintKind::Distance {
                local_anchor_a: _,
                local_anchor_b: _,
                distance,
                ..
            } => {
                out.push(DebugDrawCommand::line(pos_a, pos_b).with_color(JOINT_LIMIT));
                let mid = (pos_a + pos_b) * 0.5;
                out.push(DebugDrawCommand::Text {
                    pos: mid,
                    text: format!("{:.1}", distance),
                    color: JOINT_LIMIT,
                });
            }
            crate::constraint::ConstraintKind::Spring { .. } => {
                out.push(DebugDrawCommand::line(pos_a, pos_b).with_color(ORANGE));
                let mid = (pos_a + pos_b) * 0.5;
                out.push(DebugDrawCommand::point(mid, 3.0).with_color(ORANGE));
            }
            crate::constraint::ConstraintKind::Prismatic {
                axis,
                min_limit,
                max_limit,
                ..
            } => {
                let normalized = *axis;
                let end_a = pos_a + normalized * min_limit.max(-100.0);
                let end_b = pos_b + normalized * max_limit.min(100.0);
                out.push(DebugDrawCommand::line(end_a, end_b).with_color(JOINT_LIMIT));
                out.push(DebugDrawCommand::point(pos_a, 4.0).with_color(JOINT_ANCHOR));
                let mid = (pos_a + pos_b) * 0.5;
                out.push(DebugDrawCommand::Text {
                    pos: mid,
                    text: format!("{:.0}..{:.0}", min_limit, max_limit),
                    color: JOINT_LIMIT,
                });
            }
            _ => {
                // Generic: draw a line between constrained bodies
                out.push(DebugDrawCommand::line(pos_a, pos_b).with_color(JOINT_LIMIT));
            }
        }
    }
}

/// Collect debug draw commands for velocity vectors.
pub fn draw_velocities(
    bodies: &[crate::RigidBody],
    body_id_map: &std::collections::HashMap<crate::BodyId, usize>,
    scale: f32,
    out: &mut Vec<DebugDrawCommand>,
) {
    for &idx in body_id_map.values() {
        if let Some(body) = bodies.get(idx) {
            if body.is_sleeping {
                continue;
            }
            if body.is_3d {
                let pos = body.position_3d.truncate();
                let vel = body.velocity_3d.truncate() * scale;
                if vel.length_squared() > 0.01 {
                    out.push(DebugDrawCommand::arrow(pos, vel).with_color(colors::VELOCITY));
                }
            } else if body.velocity.length_squared() > 0.01 {
                out.push(
                    DebugDrawCommand::arrow(body.position, body.velocity * scale)
                        .with_color(colors::VELOCITY),
                );
            }
        }
    }
}

/// Collect debug draw commands for the spatial hash grid.
pub fn draw_spatial_hash(
    _spatial_hash: &crate::SpatialHash,
    cell_size: f32,
    out: &mut Vec<DebugDrawCommand>,
) {
    // This is a simplified grid visualization.
    // The spatial hash doesn't expose its internal grid, so we draw a
    // screen-space grid overlay instead.
    let grid_range = 20;
    for i in -grid_range..=grid_range {
        let x = i as f32 * cell_size;
        out.push(
            DebugDrawCommand::line(
                Vec2::new(x, -grid_range as f32 * cell_size),
                Vec2::new(x, grid_range as f32 * cell_size),
            )
            .with_color(colors::SPATIAL_HASH),
        );
        let y = i as f32 * cell_size;
        out.push(
            DebugDrawCommand::line(
                Vec2::new(-grid_range as f32 * cell_size, y),
                Vec2::new(grid_range as f32 * cell_size, y),
            )
            .with_color(colors::SPATIAL_HASH),
        );
    }
}
