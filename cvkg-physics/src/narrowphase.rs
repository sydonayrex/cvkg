//! GJK/EPA narrow-phase collision detection for convex shapes.

use glam::Vec2;

use crate::RigidBody;
use crate::shape::Shape;

/// Contact point between two colliding shapes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    pub point: Vec2,
    pub normal: Vec2,
    pub depth: f32,
}

/// A contact manifold: all contacts between two colliding bodies.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactManifold {
    pub body_a: usize,
    pub body_b: usize,
    pub contacts: Vec<Contact>,
}

/// Minkowski difference support: support_A(dir) - support_B(-dir).
fn minkowski_support(
    shape_a: &Shape, pos_a: Vec2, angle_a: f32,
    shape_b: &Shape, pos_b: Vec2, angle_b: f32,
    dir: Vec2,
) -> Vec2 {
    let sa = world_support(shape_a, pos_a, angle_a, dir);
    let sb = world_support(shape_b, pos_b, angle_b, -dir);
    sa - sb
}

fn world_support(shape: &Shape, pos: Vec2, angle: f32, dir: Vec2) -> Vec2 {
    let (cos, sin) = (angle.cos(), angle.sin());
    let ld = Vec2::new(cos * dir.x + sin * dir.y, -sin * dir.x + cos * dir.y);
    let ls = shape.support(ld);
    let (cos, sin) = (angle.cos(), angle.sin());
    pos + Vec2::new(cos * ls.x - sin * ls.y, sin * ls.x + cos * ls.y)
}

/// GJK result with overlap status and termination simplex for EPA warm-start.
#[derive(Debug, Clone, Copy)]
pub struct GjkResult {
    pub overlapping: bool,
    pub simplex: [Vec2; 3],
    pub simplex_count: usize,
}

fn count_nonzero(s: &[Vec2; 3]) -> usize { s.iter().filter(|v| **v != Vec2::ZERO).count().max(1) }

/// GJK algorithm. Returns GjkResult with overlap status and simplex.
pub fn gjk(
    shape_a: &Shape, pos_a: Vec2, angle_a: f32,
    shape_b: &Shape, pos_b: Vec2, angle_b: f32,
) -> GjkResult {
    let mut dir = pos_b - pos_a;
    if dir.length_squared() < 1e-12 { dir = Vec2::X; }
    let mut s = [Vec2::ZERO; 3];
    s[0] = minkowski_support(shape_a, pos_a, angle_a, shape_b, pos_b, angle_b, dir);
    dir = -s[0];
    for _ in 0..32 {
        let p = minkowski_support(shape_a, pos_a, angle_a, shape_b, pos_b, angle_b, dir);
        if p.dot(dir) < 0.0 { return GjkResult { overlapping: false, simplex: s, simplex_count: count_nonzero(&s) }; }
        s[2] = s[1]; s[1] = s[0]; s[0] = p;
        let (nd, origin) = process_simplex(&mut s);
        if origin { return GjkResult { overlapping: true, simplex: s, simplex_count: count_nonzero(&s) }; }
        dir = nd;
    }
    GjkResult { overlapping: false, simplex: s, simplex_count: count_nonzero(&s) }
}

/// Backward-compatible overlap check.
pub fn gjk_overlap(
    a: &Shape, pa: Vec2, aa: f32, b: &Shape, pb: Vec2, ab: f32,
) -> bool { gjk(a, pa, aa, b, pb, ab).overlapping }

/// EPA algorithm. Creates initial triangle and expands toward origin.
pub fn epa(
    a: &Shape, pa: Vec2, aa: f32, b: &Shape, pb: Vec2, ab: f32,
) -> Option<Contact> {
    let gr = gjk(a, pa, aa, b, pb, ab);
    epa_with_simplex(a, pa, aa, b, pb, ab, &gr)
}

/// EPA with pre-computed GJK result.
pub fn epa_with_simplex(
    a: &Shape, pa: Vec2, aa: f32, b: &Shape, pb: Vec2, ab: f32,
    gr: &GjkResult,
) -> Option<Contact> {
    if !gr.overlapping { return None; }
    let mut p: Vec<Vec2> = Vec::with_capacity(64);
    for i in 0..3 {
        let ang = i as f32 * std::f32::consts::TAU / 3.0;
        p.push(minkowski_support(a, pa, aa, b, pb, ab, Vec2::new(ang.cos(), ang.sin())));
    }
    let (mut cn, mut md) = (Vec2::ZERO, f32::MAX);
    for _ in 0..32 {
        let (mut ci, mut cd, mut cnorm) = (0, f32::MAX, Vec2::ZERO);
        for i in 0..p.len() {
            let j = (i + 1) % p.len();
            let e = p[j] - p[i];
            let n = Vec2::new(e.y, -e.x).normalize();
            let d = n.dot(p[i]);
            if d < cd { cd = d; ci = i; cnorm = n; }
        }
        if cd < 1e-12 { break; }
        let s = minkowski_support(a, pa, aa, b, pb, ab, cnorm);
        if (s.dot(cnorm) - cd).abs() < 1e-6 { cn = cnorm; md = cd; break; }
        p.insert((ci + 1) % p.len(), s);
        if p.len() > 64 { break; }
    }
    if md < f32::MAX { Some(Contact { point: pa + (pb - pa) * 0.5, normal: cn, depth: md }) } else { None }
}

/// Full narrow-phase collision test.
pub fn collide(
    ia: usize, sa: &Shape, ba: &RigidBody,
    ib: usize, sb: &Shape, bb: &RigidBody,
) -> Option<ContactManifold> {
    if gjk_overlap(sa, ba.position, ba.angle, sb, bb.position, bb.angle) {
        epa(sa, ba.position, ba.angle, sb, bb.position, bb.angle)
            .map(|c| ContactManifold { body_a: ia, body_b: ib, contacts: vec![c] })
    } else { None }
}

fn process_simplex(s: &mut [Vec2; 3]) -> (Vec2, bool) {
    let (a, ao) = (s[0], -s[0]);
    if s[2] != Vec2::ZERO {
        let (ab, ac) = (s[1] - a, s[2] - a);
        let abp = { let p = Vec2::new(-ab.y, ab.x); if p.dot(ac) > 0.0 { -p } else { p } };
        let acp = { let p = Vec2::new(-ac.y, ac.x); if p.dot(ab) > 0.0 { -p } else { p } };
        if abp.dot(ao) > 0.0 { s[2] = Vec2::ZERO; return (abp, false); }
        if acp.dot(ao) > 0.0 { s[1] = s[2]; s[2] = Vec2::ZERO; return (acp, false); }
        return (Vec2::ZERO, true);
    }
    let ab = s[1] - a;
    let abp = { let p = Vec2::new(-ab.y, ab.x); if p.dot(ao) > 0.0 { p } else { -p } };
    if abp.length_squared() < 1e-12 { (Vec2::new(-ab.y, ab.x), false) } else { (abp, false) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Shape;

    #[test]
    fn test_gjk_overlap() { assert!(gjk(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(5.0, 0.0), 0.0).overlapping); }
    #[test]
    fn test_gjk_separated() { assert!(!gjk(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(100.0, 0.0), 0.0).overlapping); }
    #[test]
    fn test_gjk_touching() { assert!(gjk(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(10.0, 0.0), 0.0).overlapping); }
    #[test]
    fn test_gjk_simplex() {
        let r = gjk(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(5.0, 0.0), 0.0);
        assert!(r.overlapping && r.simplex_count >= 3);
        for i in 0..r.simplex_count { assert!(r.simplex[i].is_finite()); }
    }
    #[test]
    fn test_epa_overlap() { assert!(epa(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(5.0, 0.0), 0.0).unwrap().depth > 0.0); }
    #[test]
    fn test_epa_warm_api() {
        let gr = gjk(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(5.0, 0.0), 0.0);
        let c = epa_with_simplex(&Shape::circle(5.0), Vec2::ZERO, 0.0, &Shape::circle(5.0), Vec2::new(5.0, 0.0), 0.0, &gr);
        assert!(c.unwrap().depth > 0.0);
    }
    #[test]
    fn test_gjk_circle_aabb() {
        assert!(gjk(&Shape::circle(3.0), Vec2::ZERO, 0.0, &Shape::aabb(Vec2::new(4.0, 4.0)), Vec2::new(5.0, 0.0), 0.0).overlapping);
    }
}
