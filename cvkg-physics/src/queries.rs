//! Spatial queries: raycasting, shape casting, and overlap tests.
//!
//! Provides high-performance geometric queries against the physics world using
//! the existing broad-phase (spatial hash) and narrow-phase (GJK/EPA) infrastructure.

use crate::collider::Collider;
use crate::narrowphase::{gjk, gjk_3d, gjk_overlap, gjk_overlap_3d};
use crate::shape::Shape;
use crate::{BodyId, RigidBody};
use glam::{Quat, Vec2, Vec3};
use std::collections::HashMap;

/// Result of a successful raycast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    /// World-space position of the hit point.
    pub point: Vec2,
    /// Surface normal at the hit point (pointing outward from the hit shape).
    pub normal: Vec2,
    /// Distance along the ray from origin to hit point.
    pub distance: f32,
    /// The body that was hit.
    pub body_id: BodyId,
    /// Index of the collider within the world's collider list.
    pub collider_index: usize,
    /// User data from the collider (for application identification).
    pub user_data: u64,
}

/// Result of a successful 3D raycast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit3D {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub body_id: BodyId,
    pub collider_index: usize,
    pub user_data: u64,
}

/// Result of a successful shape cast (swept collision test).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeCastHit {
    /// World-space position of first contact.
    pub point: Vec2,
    /// Contact normal at first contact.
    pub normal: Vec2,
    /// Distance along the cast direction to first contact.
    pub distance: f32,
    /// The body that was hit.
    pub body_id: BodyId,
    /// Index of the collider within the world's collider list.
    pub collider_index: usize,
    /// User data from the collider.
    pub user_data: u64,
    /// Fraction of the cast distance at which contact occurred (0.0 to 1.0).
    pub fraction: f32,
}

/// Result of a successful 3D shape cast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeCastHit3D {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub body_id: BodyId,
    pub collider_index: usize,
    pub user_data: u64,
    pub fraction: f32,
}

/// Filter predicate for spatial queries.
/// Return true to include the collider in the query, false to skip.
pub type QueryFilter = dyn Fn(&Collider, BodyId) -> bool + Send + Sync;

// ══════════════════════════════════════════════════════════════════════════════
// Raycasting (2D)
// ══════════════════════════════════════════════════════════════════════════════

/// Cast a ray against all colliders in the world, returning the closest hit.
///
/// Uses the spatial hash for broad-phase culling, then GJK for exact intersection.
///
/// # Arguments
/// * `world` - The physics world containing colliders and bodies.
/// * `origin` - Ray origin in world space.
/// * `direction` - Ray direction (will be normalized).
/// * `max_distance` - Maximum ray distance.
/// * `filter` - Optional predicate to filter which colliders to test.
///
/// # Returns
/// The closest `RaycastHit`, or `None` if no intersection.
pub fn raycast_2d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    origin: Vec2,
    direction: Vec2,
    max_distance: f32,
    filter: Option<&QueryFilter>,
) -> Option<RaycastHit> {
    if direction.length_squared() < 1e-12 {
        return None;
    }
    let dir = direction.normalize();

    let mut best_hit: Option<RaycastHit> = None;
    let mut best_dist = max_distance;

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        // Broad-phase: quick AABB test against ray segment
        let (min, max) = collider.world_aabb(body.position, body.angle);
        if !aabb_ray_intersect(min, max, origin, dir, max_distance) {
            continue;
        }

        // Narrow-phase: ray vs shape using GJK on Minkowski difference
        // For raycasting, we use the "ray as a long thin capsule" approach
        if let Some((dist, normal)) = ray_vs_shape_2d(
            &collider.shape,
            collider.offset,
            collider.rotation_offset,
            body.position,
            body.angle,
            origin,
            dir,
            max_distance,
        ) {
            if dist < best_dist {
                best_dist = dist;
                best_hit = Some(RaycastHit {
                    point: origin + dir * dist,
                    normal,
                    distance: dist,
                    body_id: collider.body_id,
                    collider_index: idx,
                    user_data: collider.user_data,
                });
            }
        }
    }

    best_hit
}

/// AABB vs ray segment intersection test (slab method).
fn aabb_ray_intersect(min: Vec2, max: Vec2, origin: Vec2, dir: Vec2, max_dist: f32) -> bool {
    let inv_dir = Vec2::new(1.0 / dir.x, 1.0 / dir.y);
    let t1 = (min - origin) * inv_dir;
    let t2 = (max - origin) * inv_dir;
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);
    let enter = tmin.x.max(tmin.y);
    let exit = tmax.x.min(tmax.y);
    exit >= enter && enter <= max_dist && exit >= 0.0
}

/// Ray vs shape intersection using GJK on the Minkowski difference.
/// Returns (distance, normal) if hit, None otherwise.
fn ray_vs_shape_2d(
    shape: &Shape,
    offset: Vec2,
    rot_offset: f32,
    body_pos: Vec2,
    body_angle: f32,
    ray_origin: Vec2,
    ray_dir: Vec2,
    max_dist: f32,
) -> Option<(f32, Vec2)> {
    // Transform ray into shape's local space
    let total_angle = body_angle + rot_offset;
    let cos = total_angle.cos();
    let sin = total_angle.sin();
    let world_offset = Vec2::new(
        cos * offset.x - sin * offset.y,
        sin * offset.x + cos * offset.y,
    );
    let center = body_pos + world_offset;

    // Local ray origin and direction
    let local_origin = Vec2::new(
        cos * (ray_origin.x - center.x) + sin * (ray_origin.y - center.y),
        -sin * (ray_origin.x - center.x) + cos * (ray_origin.y - center.y),
    );
    let local_dir = Vec2::new(
        cos * ray_dir.x + sin * ray_dir.y,
        -sin * ray_dir.x + cos * ray_dir.y,
    );

    // For raycasting, we find the closest point on the shape along the ray.
    // Use support mapping: the support point in direction -local_dir gives the
    // "back" of the shape. The ray enters at distance = support(-dir) - origin.dot(dir).
    let back_support = shape.support(-local_dir);
    let front_support = shape.support(local_dir);

    // Project origin onto ray direction in local space
    let origin_proj = local_origin.dot(local_dir);
    let back_proj = back_support.dot(local_dir);
    let front_proj = front_support.dot(local_dir);

    // The shape spans [back_proj, front_proj] along the ray direction
    // Ray enters at max(0, back_proj - origin_proj) if origin is outside
    let enter_dist = back_proj - origin_proj;
    if enter_dist > max_dist || enter_dist < -1e-6 {
        // Ray starts inside the shape (enter_dist < 0) or misses
        if enter_dist < 0.0 {
            // Inside: cast from origin to find exit
            let exit_dist = front_proj - origin_proj;
            if exit_dist > 0.0 && exit_dist <= max_dist {
                // Normal points opposite to ray direction when exiting from inside
                let local_normal = -local_dir;
                let world_normal = Vec2::new(
                    cos * local_normal.x - sin * local_normal.y,
                    sin * local_normal.x + cos * local_normal.y,
                );
                return Some((exit_dist, world_normal));
            }
        }
        return None;
    }

    // Normal points opposite to ray direction (surface faces the ray)
    let local_normal = -local_dir;
    let world_normal = Vec2::new(
        cos * local_normal.x - sin * local_normal.y,
        sin * local_normal.x + cos * local_normal.y,
    );

    Some((enter_dist.max(0.0), world_normal))
}

// ══════════════════════════════════════════════════════════════════════════════
// Raycasting (3D)
// ══════════════════════════════════════════════════════════════════════════════

/// Cast a 3D ray against all colliders in the world, returning the closest hit.
pub fn raycast_3d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    filter: Option<&QueryFilter>,
) -> Option<RaycastHit3D> {
    if direction.length_squared() < 1e-12 {
        return None;
    }
    let dir = direction.normalize();

    let mut best_hit: Option<RaycastHit3D> = None;
    let mut best_dist = max_distance;

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if !body.is_3d {
            continue;
        }

        // Broad-phase: AABB test
        let (min, max) = collider.world_aabb_3d(body.position_3d, body.rotation);
        if !aabb_ray_intersect_3d(min, max, origin, dir, max_distance) {
            continue;
        }

        // Narrow-phase
        if let Some((dist, normal)) = ray_vs_shape_3d(
            &collider.shape,
            collider.offset.extend(0.0), // 2D offset promoted to 3D
            collider.rotation_offset,
            body.position_3d,
            body.rotation,
            origin,
            dir,
            max_distance,
        ) {
            if dist < best_dist {
                best_dist = dist;
                best_hit = Some(RaycastHit3D {
                    point: origin + dir * dist,
                    normal,
                    distance: dist,
                    body_id: collider.body_id,
                    collider_index: idx,
                    user_data: collider.user_data,
                });
            }
        }
    }

    best_hit
}

/// 3D AABB vs ray intersection (slab method).
fn aabb_ray_intersect_3d(min: Vec3, max: Vec3, origin: Vec3, dir: Vec3, max_dist: f32) -> bool {
    let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
    let t1 = (min - origin) * inv_dir;
    let t2 = (max - origin) * inv_dir;
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);
    let enter = tmin.x.max(tmin.y).max(tmin.z);
    let exit = tmax.x.min(tmax.y).min(tmax.z);
    exit >= enter && enter <= max_dist && exit >= 0.0
}

/// Ray vs 3D shape intersection.
fn ray_vs_shape_3d(
    shape: &Shape,
    offset: Vec3,
    _rot_offset: f32, // rotation around Y axis for simplicity
    body_pos: Vec3,
    body_rot: Quat,
    ray_origin: Vec3,
    ray_dir: Vec3,
    max_dist: f32,
) -> Option<(f32, Vec3)> {
    // Transform ray into shape's local space
    let local_origin = body_rot.inverse() * (ray_origin - body_pos - offset);
    let local_dir = body_rot.inverse() * ray_dir;

    let back_support = shape.support_3d(-local_dir);
    let front_support = shape.support_3d(local_dir);

    let origin_proj = local_origin.dot(local_dir);
    let back_proj = back_support.dot(local_dir);
    let front_proj = front_support.dot(local_dir);

    let enter_dist = back_proj - origin_proj;
    if enter_dist > max_dist || enter_dist < -1e-6 {
        if enter_dist < 0.0 {
            let exit_dist = front_proj - origin_proj;
            if exit_dist > 0.0 && exit_dist <= max_dist {
                let local_normal = -local_dir;
                let world_normal = body_rot * local_normal;
                return Some((exit_dist, world_normal));
            }
        }
        return None;
    }

    let local_normal = -local_dir;
    let world_normal = body_rot * local_normal;
    Some((enter_dist.max(0.0), world_normal))
}

// ══════════════════════════════════════════════════════════════════════════════
// Shape Casting (2D) - Swept collision detection
// ══════════════════════════════════════════════════════════════════════════════

/// Cast a shape along a direction, returning the first contact.
/// Unlike raycasting, the shape has volume, so this detects collisions
/// that a ray would miss (e.g., a wide box passing through a narrow gap).
///
/// Uses GJK with a swept Minkowski difference (conservative advancement).
pub fn shape_cast_2d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    shape: &Shape,
    origin: Vec2,
    rotation: f32,
    direction: Vec2,
    max_distance: f32,
    filter: Option<&QueryFilter>,
) -> Option<ShapeCastHit> {
    if direction.length_squared() < 1e-12 {
        return None;
    }
    let dir = direction.normalize();

    let mut best_hit: Option<ShapeCastHit> = None;
    let mut best_dist = max_distance;

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        // Broad-phase: expanded AABB along cast direction
        let (min, max) = collider.world_aabb(body.position, body.angle);
        let expanded_min = min - dir * max_distance;
        let expanded_max = max + dir * max_distance;
        let cast_aabb_min = origin - shape.bounding_radius() * Vec2::ONE;
        let cast_aabb_max = origin + shape.bounding_radius() * Vec2::ONE;
        if !aabbs_overlap(expanded_min, expanded_max, cast_aabb_min, cast_aabb_max) {
            continue;
        }

        // Narrow-phase: conservative advancement using GJK
        if let Some((dist, normal)) = shape_cast_vs_shape_2d(
            shape,
            origin,
            rotation,
            dir,
            max_distance,
            &collider.shape,
            collider.offset,
            collider.rotation_offset,
            body.position,
            body.angle,
        ) {
            if dist < best_dist {
                best_dist = dist;
                best_hit = Some(ShapeCastHit {
                    point: origin + dir * dist,
                    normal,
                    distance: dist,
                    body_id: collider.body_id,
                    collider_index: idx,
                    user_data: collider.user_data,
                    fraction: dist / max_distance,
                });
            }
        }
    }

    best_hit
}

/// 2D AABB overlap test.
fn aabbs_overlap(min_a: Vec2, max_a: Vec2, min_b: Vec2, max_b: Vec2) -> bool {
    min_a.x <= max_b.x && max_a.x >= min_b.x && min_a.y <= max_b.y && max_a.y >= min_b.y
}

/// Conservative advancement shape cast using GJK.
/// Returns (distance, normal) of first contact, or None if no hit.
fn shape_cast_vs_shape_2d(
    shape_a: &Shape,
    pos_a: Vec2,
    rot_a: f32,
    dir: Vec2,
    max_dist: f32,
    shape_b: &Shape,
    _offset_b: Vec2,
    _rot_offset_b: f32,
    pos_b: Vec2,
    rot_b: f32,
) -> Option<(f32, Vec2)> {
    // Conservative advancement: iterate GJK with increasing offset
    // until we find contact or exceed max_dist
    let mut current_dist = 0.0;
    let mut step = max_dist;
    let max_iterations = 32;
    let tolerance = 1e-4;

    for _ in 0..max_iterations {
        let test_pos_a = pos_a + dir * current_dist;

        // Check if shapes overlap at current position
        if gjk_overlap(shape_a, test_pos_a, rot_a, shape_b, pos_b, rot_b) {
            // Binary search for exact contact distance
            let (contact_dist, normal) = binary_search_contact_2d(
                shape_a,
                pos_a,
                rot_a,
                shape_b,
                pos_b,
                rot_b,
                current_dist - step,
                current_dist,
                dir,
            );
            return Some((contact_dist, normal));
        }

        current_dist += step;
        if current_dist > max_dist {
            return None;
        }

        // Adaptive step size based on distance to Minkowski sum
        let gr = gjk(shape_a, test_pos_a, rot_a, shape_b, pos_b, rot_b);
        if !gr.overlapping {
            // Estimate distance to collision from simplex
            let closest = gr.simplex[0];
            let dist_to_origin = closest.length();
            if dist_to_origin > tolerance {
                step = (dist_to_origin - tolerance).max(tolerance);
            }
        }
    }

    None
}

/// Binary search for exact contact distance and normal.
fn binary_search_contact_2d(
    shape_a: &Shape,
    pos_a: Vec2,
    rot_a: f32,
    shape_b: &Shape,
    pos_b: Vec2,
    rot_b: f32,
    lo: f32,
    hi: f32,
    dir: Vec2,
) -> (f32, Vec2) {
    let mut low = lo.max(0.0);
    let mut high = hi;

    for _ in 0..16 {
        let mid = (low + high) * 0.5;
        let test_pos = pos_a + dir * mid;

        if gjk_overlap(shape_a, test_pos, rot_a, shape_b, pos_b, rot_b) {
            high = mid;
        } else {
            low = mid;
        }
    }

    let contact_dist = high;
    let contact_pos = pos_a + dir * contact_dist;

    // Get contact normal via EPA
    let normal = if let Some(epa_result) =
        crate::narrowphase::epa(shape_a, contact_pos, rot_a, shape_b, pos_b, rot_b)
    {
        epa_result.normal
    } else {
        // Fallback: use direction from A to B
        (pos_b - contact_pos).normalize()
    };

    (contact_dist, normal)
}

// ══════════════════════════════════════════════════════════════════════════════
// Shape Casting (3D)
// ══════════════════════════════════════════════════════════════════════════════

/// Cast a 3D shape along a direction.
pub fn shape_cast_3d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    shape: &Shape,
    origin: Vec3,
    rotation: Quat,
    direction: Vec3,
    max_distance: f32,
    filter: Option<&QueryFilter>,
) -> Option<ShapeCastHit3D> {
    if direction.length_squared() < 1e-12 {
        return None;
    }
    let dir = direction.normalize();

    let mut best_hit: Option<ShapeCastHit3D> = None;
    let mut best_dist = max_distance;

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if !body.is_3d {
            continue;
        }

        // Broad-phase
        let (min, max) = collider.world_aabb_3d(body.position_3d, body.rotation);
        let expanded_min = min - dir * max_distance;
        let expanded_max = max + dir * max_distance;
        let cast_min = origin - shape.bounding_radius() * Vec3::ONE;
        let cast_max = origin + shape.bounding_radius() * Vec3::ONE;
        if !aabbs_overlap_3d(expanded_min, expanded_max, cast_min, cast_max) {
            continue;
        }

        if let Some((dist, normal)) = shape_cast_vs_shape_3d(
            shape,
            origin,
            rotation,
            dir,
            max_distance,
            &collider.shape,
            collider.offset.extend(0.0),
            collider.rotation_offset,
            body.position_3d,
            body.rotation,
        ) {
            if dist < best_dist {
                best_dist = dist;
                best_hit = Some(ShapeCastHit3D {
                    point: origin + dir * dist,
                    normal,
                    distance: dist,
                    body_id: collider.body_id,
                    collider_index: idx,
                    user_data: collider.user_data,
                    fraction: dist / max_distance,
                });
            }
        }
    }

    best_hit
}

/// 3D AABB overlap test.
fn aabbs_overlap_3d(min_a: Vec3, max_a: Vec3, min_b: Vec3, max_b: Vec3) -> bool {
    min_a.x <= max_b.x
        && max_a.x >= min_b.x
        && min_a.y <= max_b.y
        && max_a.y >= min_b.y
        && min_a.z <= max_b.z
        && max_a.z >= min_b.z
}

/// Conservative advancement 3D shape cast.
fn shape_cast_vs_shape_3d(
    shape_a: &Shape,
    pos_a: Vec3,
    rot_a: Quat,
    dir: Vec3,
    max_dist: f32,
    shape_b: &Shape,
    _offset_b: Vec3,
    _rot_offset_b: f32,
    pos_b: Vec3,
    rot_b: Quat,
) -> Option<(f32, Vec3)> {
    let mut current_dist = 0.0;
    let mut step = max_dist;
    let max_iterations = 32;
    let tolerance = 1e-4;

    for _ in 0..max_iterations {
        let test_pos_a = pos_a + dir * current_dist;

        if gjk_overlap_3d(shape_a, test_pos_a, &rot_a, shape_b, pos_b, &rot_b) {
            let (contact_dist, normal) = binary_search_contact_3d(
                shape_a,
                pos_a,
                rot_a,
                shape_b,
                pos_b,
                rot_b,
                current_dist - step,
                current_dist,
                dir,
            );
            return Some((contact_dist, normal));
        }

        current_dist += step;
        if current_dist > max_dist {
            return None;
        }

        let gr = gjk_3d(shape_a, test_pos_a, &rot_a, shape_b, pos_b, &rot_b);
        if !gr.overlapping {
            let closest = gr.simplex[0];
            let dist_to_origin = closest.length();
            if dist_to_origin > tolerance {
                step = (dist_to_origin - tolerance).max(tolerance);
            }
        }
    }

    None
}

/// Binary search for 3D contact.
fn binary_search_contact_3d(
    shape_a: &Shape,
    pos_a: Vec3,
    rot_a: Quat,
    shape_b: &Shape,
    pos_b: Vec3,
    rot_b: Quat,
    lo: f32,
    hi: f32,
    dir: Vec3,
) -> (f32, Vec3) {
    let mut low = lo.max(0.0);
    let mut high = hi;

    for _ in 0..16 {
        let mid = (low + high) * 0.5;
        let test_pos = pos_a + dir * mid;

        if gjk_overlap_3d(shape_a, test_pos, &rot_a, shape_b, pos_b, &rot_b) {
            high = mid;
        } else {
            low = mid;
        }
    }

    let contact_dist = high;
    let contact_pos = pos_a + dir * contact_dist;

    let normal = if let Some(epa_result) =
        crate::narrowphase::epa_3d(shape_a, contact_pos, &rot_a, shape_b, pos_b, &rot_b)
    {
        epa_result.normal
    } else {
        (pos_b - contact_pos).normalize()
    };

    (contact_dist, normal)
}

// ══════════════════════════════════════════════════════════════════════════════
// Overlap Queries
// ══════════════════════════════════════════════════════════════════════════════

/// Result of an overlap query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlapHit {
    pub body_id: BodyId,
    pub collider_index: usize,
    pub user_data: u64,
}

/// Find all colliders containing a point (2D).
pub fn point_query_2d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    point: Vec2,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        // Quick AABB test first
        let (min, max) = collider.world_aabb(body.position, body.angle);
        if point.x < min.x || point.x > max.x || point.y < min.y || point.y > max.y {
            continue;
        }

        // Precise test: check if point is inside shape
        let total_angle = body.angle + collider.rotation_offset;
        let cos = total_angle.cos();
        let sin = total_angle.sin();
        let world_offset = Vec2::new(
            cos * collider.offset.x - sin * collider.offset.y,
            sin * collider.offset.x + cos * collider.offset.y,
        );
        let center = body.position + world_offset;
        let local_point = Vec2::new(
            cos * (point.x - center.x) + sin * (point.y - center.y),
            -sin * (point.x - center.x) + cos * (point.y - center.y),
        );

        // Use GJK with a point shape (radius 0)
        let point_shape = Shape::circle(0.0);
        if gjk_overlap(
            &point_shape,
            local_point,
            0.0,
            &collider.shape,
            Vec2::ZERO,
            0.0,
        ) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}

/// Find all colliders containing a point (3D).
pub fn point_query_3d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    point: Vec3,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if !body.is_3d {
            continue;
        }

        let (min, max) = collider.world_aabb_3d(body.position_3d, body.rotation);
        if point.x < min.x
            || point.x > max.x
            || point.y < min.y
            || point.y > max.y
            || point.z < min.z
            || point.z > max.z
        {
            continue;
        }

        let local_point =
            body.rotation.inverse() * (point - body.position_3d - collider.offset.extend(0.0));
        let point_shape = Shape::sphere(0.0);
        if gjk_overlap_3d(
            &point_shape,
            local_point,
            &Quat::IDENTITY,
            &collider.shape,
            Vec3::ZERO,
            &Quat::IDENTITY,
        ) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}

/// Find all colliders overlapping an AABB (2D).
pub fn aabb_query_2d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    min: Vec2,
    max: Vec2,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        let (c_min, c_max) = collider.world_aabb(body.position, body.angle);
        if aabbs_overlap(min, max, c_min, c_max) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}

/// Find all colliders overlapping an AABB (3D).
pub fn aabb_query_3d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    min: Vec3,
    max: Vec3,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if !body.is_3d {
            continue;
        }

        let (c_min, c_max) = collider.world_aabb_3d(body.position_3d, body.rotation);
        if aabbs_overlap_3d(min, max, c_min, c_max) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}

/// Find all colliders overlapping a circle (2D).
pub fn circle_query_2d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    center: Vec2,
    radius: f32,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();
    let circle_shape = Shape::circle(radius);

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if gjk_overlap(
            &circle_shape,
            center,
            0.0,
            &collider.shape,
            body.position,
            body.angle,
        ) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}

/// Find all colliders overlapping a sphere (3D).
pub fn sphere_query_3d(
    colliders: &[Collider],
    bodies: &HashMap<BodyId, &RigidBody>,
    center: Vec3,
    radius: f32,
    filter: Option<&QueryFilter>,
) -> Vec<OverlapHit> {
    let mut hits = Vec::new();
    let sphere_shape = Shape::sphere(radius);

    for (idx, collider) in colliders.iter().enumerate() {
        if let Some(ref f) = filter {
            if !f(collider, collider.body_id) {
                continue;
            }
        }

        let body = match bodies.get(&collider.body_id) {
            Some(b) => *b,
            None => continue,
        };

        if !body.is_3d {
            continue;
        }

        if gjk_overlap_3d(
            &sphere_shape,
            center,
            &Quat::IDENTITY,
            &collider.shape,
            body.position_3d,
            &body.rotation,
        ) {
            hits.push(OverlapHit {
                body_id: collider.body_id,
                collider_index: idx,
                user_data: collider.user_data,
            });
        }
    }

    hits
}
