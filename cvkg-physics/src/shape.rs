//! 2D collision shapes for rigid body physics.

use glam::Vec2;

/// Collision shape kinds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeKind {
    /// Circle defined by radius.
    Circle { radius: f32 },
    /// Axis-aligned bounding box defined by half-extents.
    Aabb { half_extents: Vec2 },
    /// Capsule: a rectangle with semicircular ends.
    Capsule {
        /// Radius of the semicircular ends.
        radius: f32,
        /// Half the length of the rectangular midsection (along local y-axis).
        half_height: f32,
    },
    /// Convex hull defined by vertices in counterclockwise order.
    ConvexHull { vertices: &'static [Vec2] },
}

/// A collision shape with computed mass properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    pub kind: ShapeKind,
}

impl Shape {
    /// Create a circle shape.
    pub fn circle(radius: f32) -> Self {
        Self {
            kind: ShapeKind::Circle { radius },
        }
    }

    /// Create an AABB shape from half-extents.
    pub fn aabb(half_extents: Vec2) -> Self {
        Self {
            kind: ShapeKind::Aabb { half_extents },
        }
    }

    /// Create a capsule shape.
    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            kind: ShapeKind::Capsule {
                radius,
                half_height,
            },
        }
    }

    /// Create a convex hull shape from static vertices (CCW order).
    pub fn convex_hull(vertices: &'static [Vec2]) -> Self {
        debug_assert!(vertices.len() >= 3, "ConvexHull needs at least 3 vertices");
        Self {
            kind: ShapeKind::ConvexHull { vertices },
        }
    }

    /// Compute the area of this shape.
    pub fn area(&self) -> f32 {
        match self.kind {
            ShapeKind::Circle { radius } => std::f32::consts::PI * radius * radius,
            ShapeKind::Aabb { half_extents } => 4.0 * half_extents.x * half_extents.y,
            ShapeKind::Capsule {
                radius,
                half_height,
            } => {
                let rect = 2.0 * half_height * 2.0 * radius;
                let circle = std::f32::consts::PI * radius * radius;
                rect + circle
            }
            ShapeKind::ConvexHull { vertices } => {
                // Shoelace formula
                let mut area = 0.0;
                for i in 0..vertices.len() {
                    let j = (i + 1) % vertices.len();
                    area += vertices[i].x * vertices[j].y;
                    area -= vertices[j].x * vertices[i].y;
                }
                area.abs() * 0.5
            }
        }
    }

    /// Compute the moment of inertia for a shape with given mass.
    /// Assumes uniform density.
    pub fn moment_of_inertia(&self, mass: f32) -> f32 {
        match self.kind {
            ShapeKind::Circle { radius } => 0.5 * mass * radius * radius,
            ShapeKind::Aabb { half_extents } => {
                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;
                mass * (w * w + h * h) / 12.0
            }
            ShapeKind::Capsule {
                radius,
                half_height,
            } => {
                // Approximate as rectangle + two half-circles
                let rect_mass = mass * (2.0 * half_height) / (2.0 * half_height + 2.0 * radius);
                let circle_mass = mass - rect_mass;
                let rect_i = rect_mass * (4.0 * half_height * 4.0 * half_height) / 12.0;
                let circle_i = circle_mass * 0.5 * radius * radius;
                rect_i + circle_i
            }
            ShapeKind::ConvexHull { vertices } => {
                // Use the general polygon formula
                let mut num = 0.0;
                let mut den = 0.0;
                for i in 0..vertices.len() {
                    let j = (i + 1) % vertices.len();
                    let cross = vertices[i].x * vertices[j].y - vertices[j].x * vertices[i].y;
                    num += cross
                        * (vertices[i].length_squared()
                            + vertices[i].dot(vertices[j])
                            + vertices[j].length_squared());
                    den += cross;
                }
                let total_area = self.area();
                let density = if total_area > 0.0 {
                    mass / total_area
                } else {
                    0.0
                };
                density * num / (6.0 * den.max(1e-10))
            }
        }
    }

    /// Get the support point in a given direction (for GJK).
    /// Returns the farthest point on the shape boundary in direction `dir`.
    pub fn support(&self, dir: Vec2) -> Vec2 {
        match self.kind {
            ShapeKind::Circle { radius } => {
                if dir.length_squared() < 1e-12 {
                    Vec2::new(radius, 0.0)
                } else {
                    dir.normalize() * radius
                }
            }
            ShapeKind::Aabb { half_extents } => Vec2::new(
                if dir.x >= 0.0 {
                    half_extents.x
                } else {
                    -half_extents.x
                },
                if dir.y >= 0.0 {
                    half_extents.y
                } else {
                    -half_extents.y
                },
            ),
            ShapeKind::Capsule {
                radius,
                half_height,
            } => {
                if dir.length_squared() < 1e-12 {
                    return Vec2::new(radius, 0.0);
                }
                let n = dir.normalize();
                // The capsule is: rectangle [-r, -h] to [r, h] plus semicircles at top/bottom
                let end_center = if n.y >= 0.0 {
                    Vec2::new(0.0, half_height)
                } else {
                    Vec2::new(0.0, -half_height)
                };
                end_center + n * radius
            }
            ShapeKind::ConvexHull { vertices } => {
                let mut best = vertices[0];
                let mut best_dot = best.dot(dir);
                for v in &vertices[1..] {
                    let d = v.dot(dir);
                    if d > best_dot {
                        best_dot = d;
                        best = *v;
                    }
                }
                best
            }
        }
    }

    /// Get the bounding radius (maximum distance from center to any point on shape).
    pub fn bounding_radius(&self) -> f32 {
        match self.kind {
            ShapeKind::Circle { radius } => radius,
            ShapeKind::Aabb { half_extents } => half_extents.length(),
            ShapeKind::Capsule {
                radius,
                half_height,
            } => half_height + radius,
            ShapeKind::ConvexHull { vertices } => {
                vertices.iter().map(|v| v.length()).fold(0.0, f32::max)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_area() {
        let s = Shape::circle(2.0);
        assert!((s.area() - std::f32::consts::PI * 4.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_area() {
        let s = Shape::aabb(Vec2::new(3.0, 4.0));
        assert!((s.area() - 48.0).abs() < 0.001);
    }

    #[test]
    fn test_circle_support() {
        let s = Shape::circle(5.0);
        let p = s.support(Vec2::new(1.0, 0.0));
        assert!((p.x - 5.0).abs() < 0.001);
        assert!(p.y.abs() < 0.001);
    }

    #[test]
    fn test_circle_inertia() {
        let s = Shape::circle(1.0);
        // I = 0.5 * m * r^2 = 0.5 for m=1, r=1
        assert!((s.moment_of_inertia(1.0) - 0.5).abs() < 0.001);
    }
}
