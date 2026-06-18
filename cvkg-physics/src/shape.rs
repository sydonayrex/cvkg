//! 2D and 3D collision shapes for rigid body physics.
//!
//! 2D shapes: Circle, Aabb, Capsule, ConvexHull.
//! 3D shapes: Sphere, Box3D, Capsule3D.

use glam::{Quat, Vec2, Vec3};

/// Collision shape kinds -- 2D and 3D.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeKind {
    // ── 2D shapes ─────────────────────────────────────────────────────────
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

    // ── 3D shapes ─────────────────────────────────────────────────────────
    /// Sphere defined by radius.
    Sphere { radius: f32 },
    /// Axis-aligned box defined by half-extents in 3D.
    Box3D { half_extents: Vec3 },
    /// Capsule in 3D: cylinder with hemispherical caps.
    Capsule3D {
        /// Radius of the hemispherical caps and cylinder.
        radius: f32,
        /// Half the length of the cylindrical midsection (along local y-axis).
        half_height: f32,
    },

    // ── Compound shapes ────────────────────────────────────────────────────
    /// 2D compound shape: multiple child shapes with local transforms.
    Compound2D {
        children: &'static [CompoundChild2D],
    },
    /// 3D compound shape: multiple child shapes with local transforms.
    Compound3D {
        children: &'static [CompoundChild3D],
    },
    /// Heightmap terrain shape (stored as Box due to Vec data).
    Heightmap(Box<crate::heightmap::HeightmapShape>),
}

/// A child shape in a 2D compound shape with its local transform.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundChild2D {
    /// The child shape.
    pub shape: Shape,
    /// Local offset from compound center.
    pub offset: Vec2,
    /// Local rotation (radians).
    pub rotation: f32,
}

/// A child shape in a 3D compound shape with its local transform.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundChild3D {
    /// The child shape.
    pub shape: Shape,
    /// Local offset from compound center.
    pub offset: Vec3,
    /// Local rotation as quaternion.
    pub rotation: Quat,
}

/// A collision shape with computed mass properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub kind: ShapeKind,
}

/// Heightmap shape stored separately since it contains Vec data.
pub use crate::heightmap::HeightmapShape;

impl Shape {
    // ── 2D constructors ───────────────────────────────────────────────────

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

    // ── 3D constructors ───────────────────────────────────────────────────

    /// Create a sphere shape.
    pub fn sphere(radius: f32) -> Self {
        Self {
            kind: ShapeKind::Sphere { radius },
        }
    }

    /// Create a 3D box shape from half-extents.
    pub fn box3d(half_extents: Vec3) -> Self {
        Self {
            kind: ShapeKind::Box3D { half_extents },
        }
    }

    /// Create a 3D capsule shape.
    pub fn capsule3d(radius: f32, half_height: f32) -> Self {
        Self {
            kind: ShapeKind::Capsule3D {
                radius,
                half_height,
            },
        }
    }

    /// Create a 2D compound shape from child shapes with local transforms.
    pub fn compound_2d(children: &'static [CompoundChild2D]) -> Self {
        Self {
            kind: ShapeKind::Compound2D { children },
        }
    }

    /// Create a 3D compound shape from child shapes with local transforms.
    pub fn compound_3d(children: &'static [CompoundChild3D]) -> Self {
        Self {
            kind: ShapeKind::Compound3D { children },
        }
    }

    /// Create a heightmap terrain shape.
    pub fn heightmap(
        heights: Vec<f32>,
        width: usize,
        depth: usize,
        world_size: glam::Vec2,
    ) -> Self {
        Self {
            kind: ShapeKind::Heightmap(Box::new(crate::heightmap::HeightmapShape::new(
                heights, width, depth, world_size,
            ))),
        }
    }

    /// Create a flat heightmap terrain at y=0.
    pub fn heightmap_flat(width: usize, depth: usize, world_size: glam::Vec2) -> Self {
        Self {
            kind: ShapeKind::Heightmap(Box::new(crate::heightmap::HeightmapShape::flat(
                width, depth, world_size,
            ))),
        }
    }

    // ── Area / volume ─────────────────────────────────────────────────────

    /// Compute the area of this shape (2D) or surface area (3D sphere/box).
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
            ShapeKind::Sphere { radius } => 4.0 * std::f32::consts::PI * radius * radius,
            ShapeKind::Box3D { half_extents } => {
                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;
                let d = 2.0 * half_extents.z;
                2.0 * (w * h + h * d + w * d)
            }
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => {
                // Cylinder surface + sphere surface
                let cyl = 2.0 * std::f32::consts::PI * radius * 2.0 * half_height;
                let sphere = 4.0 * std::f32::consts::PI * radius * radius;
                cyl + sphere
            }
            ShapeKind::Compound2D { children } => children.iter().map(|c| c.shape.area()).sum(),
            ShapeKind::Compound3D { children } => children.iter().map(|c| c.shape.area()).sum(),
            ShapeKind::Heightmap(ref hm) => {
                // Approximate area as the XZ plane footprint
                hm.world_size.x * hm.world_size.y
            }
        }
    }

    // ── Moment of inertia ─────────────────────────────────────────────────

    /// Compute the moment of inertia for a shape with given mass.
    /// For 2D shapes, returns the scalar inertia about the Z axis.
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
            // 3D shapes: return the scalar inertia about the principal axis
            // (for use as a fallback; prefer moment_of_inertia_3d for 3D sim)
            ShapeKind::Sphere { radius } => 0.4 * mass * radius * radius,
            ShapeKind::Box3D { half_extents } => {
                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;
                let d = 2.0 * half_extents.z;
                mass * (w * w + h * h + d * d) / 12.0
            }
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => {
                // Approximate: cylinder + sphere
                let cyl_mass =
                    mass * (2.0 * half_height) / (2.0 * half_height + 4.0 / 3.0 * radius);
                let sphere_mass = mass - cyl_mass;
                let cyl_i = cyl_mass
                    * (3.0 * radius * radius + 4.0 * half_height * 4.0 * half_height)
                    / 12.0;
                let sphere_i = sphere_mass * 0.4 * radius * radius;
                cyl_i + sphere_i
            }
            ShapeKind::Compound2D { children } => {
                // Sum of child inertias (assuming mass distributed proportionally to area)
                let total_area: f32 = children.iter().map(|c| c.shape.area()).sum();
                children
                    .iter()
                    .map(|c| {
                        let child_area = c.shape.area();
                        let child_mass = if total_area > 0.0 {
                            mass * child_area / total_area
                        } else {
                            0.0
                        };
                        c.shape.moment_of_inertia(child_mass)
                    })
                    .sum()
            }
            ShapeKind::Compound3D { children } => {
                // Sum of child inertias (assuming mass distributed proportionally to volume/surface)
                let total_area: f32 = children.iter().map(|c| c.shape.area()).sum();
                children
                    .iter()
                    .map(|c| {
                        let child_area = c.shape.area();
                        let child_mass = if total_area > 0.0 {
                            mass * child_area / total_area
                        } else {
                            0.0
                        };
                        c.shape.moment_of_inertia(child_mass)
                    })
                    .sum()
            }
            ShapeKind::Heightmap(ref hm) => {
                // Approximate: thin plate at average height
                let w = hm.world_size.x;
                let d = hm.world_size.y;
                mass * (w * w + d * d) / 12.0
            }
        }
    }

    /// Compute the 3D moment of inertia tensor (diagonal) for a shape with given mass.
    /// Returns the principal moments (Ixx, Iyy, Izz) assuming uniform density.
    pub fn moment_of_inertia_3d(&self, mass: f32) -> Vec3 {
        match self.kind {
            ShapeKind::Sphere { radius } => {
                let i = 0.4 * mass * radius * radius;
                Vec3::new(i, i, i)
            }
            ShapeKind::Box3D { half_extents } => {
                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;
                let d = 2.0 * half_extents.z;
                Vec3::new(
                    mass * (h * h + d * d) / 12.0,
                    mass * (w * w + d * d) / 12.0,
                    mass * (w * w + h * h) / 12.0,
                )
            }
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => {
                // Approximate as cylinder + sphere
                let total_len = 2.0 * half_height + 4.0 / 3.0 * radius;
                let cyl_mass = mass * (2.0 * half_height) / total_len;
                let sphere_mass = mass - cyl_mass;
                // Cylinder along Y axis
                let cyl_ix =
                    cyl_mass * (3.0 * radius * radius + 4.0 * half_height * half_height) / 12.0;
                let cyl_iy = cyl_mass * 0.5 * radius * radius;
                // Sphere
                let sph_i = sphere_mass * 0.4 * radius * radius;
                Vec3::new(cyl_ix + sph_i, cyl_iy + sph_i, cyl_ix + sph_i)
            }
            // 2D shapes promoted to 3D: treat as flat in Z
            ShapeKind::Circle { radius } => {
                let i = 0.5 * mass * radius * radius;
                Vec3::new(i, i, 2.0 * i)
            }
            ShapeKind::Aabb { half_extents } => {
                let w = 2.0 * half_extents.x;
                let h = 2.0 * half_extents.y;
                Vec3::new(
                    mass * h * h / 12.0,
                    mass * w * w / 12.0,
                    mass * (w * w + h * h) / 12.0,
                )
            }
            ShapeKind::Capsule {
                radius,
                half_height,
            } => {
                let i = self.moment_of_inertia(mass);
                let rect_mass = mass * (2.0 * half_height) / (2.0 * half_height + 2.0 * radius);
                let circle_mass = mass - rect_mass;
                let iy = rect_mass * (radius * radius) / 3.0 + circle_mass * 0.25 * radius * radius;
                let ix = (i - iy).max(0.0);
                Vec3::new(ix, iy, i)
            }
            ShapeKind::ConvexHull { .. } => {
                // Fallback: use scalar inertia for all axes
                let i = self.moment_of_inertia(mass);
                Vec3::new(i, i, i)
            }
            ShapeKind::Compound2D { children } => {
                // Sum of child 3D inertias (assuming mass distributed proportionally to area)
                let total_area: f32 = children.iter().map(|c| c.shape.area()).sum();
                children
                    .iter()
                    .map(|c| {
                        let child_area = c.shape.area();
                        let child_mass = if total_area > 0.0 {
                            mass * child_area / total_area
                        } else {
                            0.0
                        };
                        c.shape.moment_of_inertia_3d(child_mass)
                    })
                    .sum()
            }
            ShapeKind::Compound3D { children } => {
                // Sum of child 3D inertias (assuming mass distributed proportionally to area)
                let total_area: f32 = children.iter().map(|c| c.shape.area()).sum();
                children
                    .iter()
                    .map(|c| {
                        let child_area = c.shape.area();
                        let child_mass = if total_area > 0.0 {
                            mass * child_area / total_area
                        } else {
                            0.0
                        };
                        c.shape.moment_of_inertia_3d(child_mass)
                    })
                    .sum()
            }
            ShapeKind::Heightmap(ref hm) => {
                // Approximate as a flat box with average height
                let w = hm.world_size.x;
                let d = hm.world_size.y;
                let h = (hm.max_height - hm.min_height).max(1.0);
                let ixx = mass * (h * h + d * d) / 12.0;
                let iyy = mass * (w * w + d * d) / 12.0;
                let izz = mass * (w * w + h * h) / 12.0;
                Vec3::new(ixx, iyy, izz)
            }
        }
    }

    // ── Support functions ─────────────────────────────────────────────────

    /// Get the 2D support point in a given direction (for 2D GJK).
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
            // 3D shapes projected to 2D (for mixed-mode use)
            ShapeKind::Sphere { radius } => {
                if dir.length_squared() < 1e-12 {
                    Vec2::new(radius, 0.0)
                } else {
                    dir.normalize() * radius
                }
            }
            ShapeKind::Box3D { half_extents } => Vec2::new(
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
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => {
                if dir.length_squared() < 1e-12 {
                    return Vec2::new(radius, 0.0);
                }
                let n = dir.normalize();
                let end_center = if n.y >= 0.0 {
                    Vec2::new(0.0, half_height)
                } else {
                    Vec2::new(0.0, -half_height)
                };
                end_center + n * radius
            }
            ShapeKind::Compound2D { children } => {
                let mut best = Vec2::ZERO;
                let mut best_dot = -f32::INFINITY;
                for child in children {
                    // Transform direction into child's local space
                    let cos = child.rotation.cos();
                    let sin = child.rotation.sin();
                    let local_dir =
                        Vec2::new(cos * dir.x + sin * dir.y, -sin * dir.x + cos * dir.y);
                    let local_support = child.shape.support(local_dir);
                    // Transform support point to world space
                    let rotated = Vec2::new(
                        cos * local_support.x - sin * local_support.y,
                        sin * local_support.x + cos * local_support.y,
                    );
                    let world_support = child.offset + rotated;
                    let dot = world_support.dot(dir);
                    if dot > best_dot {
                        best_dot = dot;
                        best = world_support;
                    }
                }
                best
            }
            ShapeKind::Compound3D { .. } => Vec2::ZERO,
            ShapeKind::Heightmap(ref hm) => {
                // 2D support for heightmap: return point at max Y
                let r = hm.bounding_radius();
                Vec2::new(r, hm.max_height)
            }
        }
    }

    /// Get the 3D support point in a given direction (for 3D GJK).
    pub fn support_3d(&self, dir: Vec3) -> Vec3 {
        match self.kind {
            ShapeKind::Sphere { radius } => {
                if dir.length_squared() < 1e-12 {
                    Vec3::new(radius, 0.0, 0.0)
                } else {
                    dir.normalize() * radius
                }
            }
            ShapeKind::Box3D { half_extents } => Vec3::new(
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
                if dir.z >= 0.0 {
                    half_extents.z
                } else {
                    -half_extents.z
                },
            ),
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => {
                if dir.length_squared() < 1e-12 {
                    return Vec3::new(radius, 0.0, 0.0);
                }
                let n = dir.normalize();
                let end_center = if n.y >= 0.0 {
                    Vec3::new(0.0, half_height, 0.0)
                } else {
                    Vec3::new(0.0, -half_height, 0.0)
                };
                end_center + n * radius
            }
            // 2D shapes promoted to 3D (flat in Z)
            ShapeKind::Circle { radius } => {
                let d2 = Vec2::new(dir.x, dir.y);
                if d2.length_squared() < 1e-12 {
                    Vec3::new(radius, 0.0, 0.0)
                } else {
                    let n = d2.normalize() * radius;
                    Vec3::new(n.x, n.y, 0.0)
                }
            }
            ShapeKind::Aabb { half_extents } => Vec3::new(
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
                0.0,
            ),
            ShapeKind::Capsule {
                radius,
                half_height,
            } => {
                let d2 = Vec2::new(dir.x, dir.y);
                if d2.length_squared() < 1e-12 {
                    return Vec3::new(radius, 0.0, 0.0);
                }
                let n = d2.normalize();
                let end_center = if n.y >= 0.0 {
                    Vec3::new(0.0, half_height, 0.0)
                } else {
                    Vec3::new(0.0, -half_height, 0.0)
                };
                end_center + Vec3::new(n.x, n.y, 0.0) * radius
            }
            ShapeKind::ConvexHull { vertices } => {
                let mut best = vertices[0];
                let d2 = Vec2::new(dir.x, dir.y);
                let mut best_dot = best.dot(d2);
                for v in &vertices[1..] {
                    let d = v.dot(d2);
                    if d > best_dot {
                        best_dot = d;
                        best = *v;
                    }
                }
                Vec3::new(best.x, best.y, 0.0)
            }
            ShapeKind::Compound2D { children } => {
                let mut best = Vec3::ZERO;
                let mut best_dot = -f32::INFINITY;
                for child in children {
                    // Transform direction into child's local space (2D rotation in XY plane)
                    let cos = child.rotation.cos();
                    let sin = child.rotation.sin();
                    let local_dir =
                        Vec2::new(cos * dir.x + sin * dir.y, -sin * dir.x + cos * dir.y);
                    let local_support = child.shape.support(local_dir);
                    // Transform support point to world space (XY plane)
                    let rotated = Vec2::new(
                        cos * local_support.x - sin * local_support.y,
                        sin * local_support.x + cos * local_support.y,
                    );
                    let world_support =
                        Vec3::new(child.offset.x + rotated.x, child.offset.y + rotated.y, 0.0);
                    let dot = world_support.dot(dir);
                    if dot > best_dot {
                        best_dot = dot;
                        best = world_support;
                    }
                }
                best
            }
            ShapeKind::Compound3D { children } => {
                let mut best = Vec3::ZERO;
                let mut best_dot = -f32::INFINITY;
                for child in children {
                    // Transform direction into child's local space
                    let local_dir = child.rotation.inverse() * dir;
                    let local_support = child.shape.support_3d(local_dir);
                    // Transform support point to world space
                    let world_support = child.offset + child.rotation * local_support;
                    let dot = world_support.dot(dir);
                    if dot > best_dot {
                        best_dot = dot;
                        best = world_support;
                    }
                }
                best
            }
            ShapeKind::Heightmap(ref hm) => {
                // Heightmap support: sample height at the x,z of the direction
                // and return a point on the surface in the direction of the query
                let r = hm.bounding_radius();
                if dir.length_squared() < 1e-12 {
                    Vec3::new(r, hm.max_height, 0.0)
                } else {
                    let n = dir.normalize();
                    // Project direction onto XZ plane for heightmap sampling
                    let xz_len = (n.x * n.x + n.z * n.z).sqrt();
                    if xz_len > 1e-6 {
                        // Sample height at a point in the direction
                        let sample_dist = r.min(1.0);
                        let sx = n.x / xz_len * sample_dist;
                        let sz = n.z / xz_len * sample_dist;
                        let sy = hm.sample_height(sx, sz).unwrap_or(0.0);
                        Vec3::new(sx, sy + n.y * r, sz)
                    } else {
                        // Vertical direction: return highest or lowest point
                        if n.y >= 0.0 {
                            Vec3::new(0.0, hm.max_height, 0.0)
                        } else {
                            Vec3::new(0.0, hm.min_height, 0.0)
                        }
                    }
                }
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
            ShapeKind::Sphere { radius } => radius,
            ShapeKind::Box3D { half_extents } => half_extents.length(),
            ShapeKind::Capsule3D {
                radius,
                half_height,
            } => half_height + radius,
            ShapeKind::Compound2D { children } => children
                .iter()
                .map(|c| c.offset.length() + c.shape.bounding_radius())
                .fold(0.0, f32::max),
            ShapeKind::Compound3D { children } => children
                .iter()
                .map(|c| c.offset.length() + c.shape.bounding_radius())
                .fold(0.0, f32::max),
            ShapeKind::Heightmap(ref hm) => hm.bounding_radius(),
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

    // ── 3D shape tests ───────────────────────────────────────────────────

    #[test]
    fn test_sphere_creation() {
        let s = Shape::sphere(2.0);
        assert!(matches!(s.kind, ShapeKind::Sphere { radius: 2.0 }));
    }

    #[test]
    fn test_box3d_creation() {
        let s = Shape::box3d(Vec3::new(1.0, 2.0, 3.0));
        assert!(
            matches!(s.kind, ShapeKind::Box3D { half_extents } if half_extents == Vec3::new(1.0, 2.0, 3.0))
        );
    }

    #[test]
    fn test_capsule3d_creation() {
        let s = Shape::capsule3d(1.0, 3.0);
        assert!(matches!(
            s.kind,
            ShapeKind::Capsule3D {
                radius: 1.0,
                half_height: 3.0
            }
        ));
    }

    #[test]
    fn test_sphere_support_3d() {
        let s = Shape::sphere(5.0);
        let p = s.support_3d(Vec3::new(1.0, 0.0, 0.0));
        assert!((p.x - 5.0).abs() < 0.001);
        assert!(p.y.abs() < 0.001);
        assert!(p.z.abs() < 0.001);
    }

    #[test]
    fn test_box3d_support_3d() {
        let s = Shape::box3d(Vec3::new(2.0, 3.0, 4.0));
        let p = s.support_3d(Vec3::new(-1.0, 1.0, -1.0));
        assert!((p.x - (-2.0)).abs() < 0.001);
        assert!((p.y - 3.0).abs() < 0.001);
        assert!((p.z - (-4.0)).abs() < 0.001);
    }

    #[test]
    fn test_capsule3d_support_3d() {
        let s = Shape::capsule3d(1.0, 3.0);
        // Pointing up: should hit top cap center + radius
        let p = s.support_3d(Vec3::new(0.0, 1.0, 0.0));
        assert!((p.y - 4.0).abs() < 0.001); // half_height + radius
        // Pointing down
        let p = s.support_3d(Vec3::new(0.0, -1.0, 0.0));
        assert!((p.y - (-4.0)).abs() < 0.001);
    }

    #[test]
    fn test_sphere_inertia_3d() {
        let s = Shape::sphere(1.0);
        let i = s.moment_of_inertia_3d(1.0);
        // I = 0.4 * m * r^2 = 0.4
        assert!((i.x - 0.4).abs() < 0.001);
        assert!((i.y - 0.4).abs() < 0.001);
        assert!((i.z - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_box3d_inertia_3d() {
        let s = Shape::box3d(Vec3::new(1.0, 1.0, 1.0));
        let i = s.moment_of_inertia_3d(1.0);
        // Ixx = m*(h^2 + d^2)/12 = 1*(4+4)/12 = 2/3
        assert!((i.x - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_sphere_bounding_radius() {
        let s = Shape::sphere(3.0);
        assert!((s.bounding_radius() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_box3d_bounding_radius() {
        let s = Shape::box3d(Vec3::new(1.0, 2.0, 3.0));
        assert!((s.bounding_radius() - Vec3::new(1.0, 2.0, 3.0).length()).abs() < 0.001);
    }
}
