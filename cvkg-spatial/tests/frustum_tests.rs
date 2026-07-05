use cvkg_spatial::frustum::Frustum;
use glam::{Mat4, Vec3};

/// Verify frustum plane extraction from view-projection matrix.

#[test]
fn test_frustum_from_identity() {
    // Identity VP = no culling — everything passes.
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(frustum.intersects_aabb(Vec3::ZERO, Vec3::splat(100.0)));
}

#[test]
fn test_frustum_culls_object_behind_camera() {
    // Object at z=-100 is behind the default camera (looking down -Z).
    let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 1000.0);
    let frustum = Frustum::from_view_projection(&(proj * view));
    // Object at z=+100 (behind camera in RH) should be culled.
    assert!(!frustum.intersects_aabb(Vec3::new(0.0, 0.0, 100.0), Vec3::splat(1.0)));
}

#[test]
fn test_frustum_passes_object_in_front() {
    let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 1000.0);
    let frustum = Frustum::from_view_projection(&(proj * view));
    // Object at z=-10 (in front of camera) should pass.
    assert!(frustum.intersects_aabb(Vec3::new(0.0, 0.0, -10.0), Vec3::splat(1.0)));
}

#[test]
fn test_frustum_sphere_intersection() {
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(frustum.intersects_sphere(Vec3::ZERO, 100.0));
    // Sphere far outside should be culled.
    assert!(!frustum.intersects_sphere(Vec3::new(10000.0, 10000.0, 10000.0), 1.0));
}
