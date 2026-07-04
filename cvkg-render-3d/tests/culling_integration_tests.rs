/// Culling integration tests — verifies that the FrustumCuller skips
/// invisible meshes and passes visible ones. Uses mock data — no GPU required.
use cvkg_render_3d::culler::FrustumCuller;
use cvkg_spatial::frustum::Frustum;
use glam::{Mat4, Vec3};

/// A minimal mesh-like struct for testing culling.
struct TestMesh {
    center: Vec3,
    half_extents: Vec3,
}

/// Cull a list of meshes against a frustum, returning indices of visible ones.
fn cull_meshes(frustum: &Frustum, meshes: &[TestMesh]) -> Vec<usize> {
    meshes
        .iter()
        .enumerate()
        .filter(|(_, m)| frustum.intersects_aabb(m.center, m.half_extents))
        .map(|(i, _)| i)
        .collect()
}

/// Build a standard RH perspective camera frustum (looking down -Z).
fn perspective_frustum() -> Frustum {
    let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 1000.0);
    Frustum::from_view_projection(&(proj * view))
}

#[test]
fn test_culling_loop_skips_outside_meshes() {
    let frustum = perspective_frustum();
    let mesh_center = Vec3::new(10000.0, 10000.0, 10000.0); // far outside
    let half = Vec3::splat(1.0);
    assert!(
        !frustum.intersects_aabb(mesh_center, half),
        "mesh outside frustum must be culled"
    );
}

#[test]
fn test_culling_loop_passes_inside_meshes() {
    let frustum = perspective_frustum();
    let mesh_center = Vec3::new(0.0, 0.0, -5.0); // in front of camera
    let half = Vec3::splat(1.0);
    assert!(
        frustum.intersects_aabb(mesh_center, half),
        "mesh inside frustum must pass"
    );
}

#[test]
fn test_frustum_culler_filters_batch() {
    let frustum = perspective_frustum();
    let culler = FrustumCuller::new(frustum);

    let meshes = vec![
        TestMesh {
            center: Vec3::new(0.0, 0.0, -5.0),
            half_extents: Vec3::splat(1.0),
        }, // visible
        TestMesh {
            center: Vec3::new(10000.0, 10000.0, 10000.0),
            half_extents: Vec3::splat(1.0),
        }, // culled
        TestMesh {
            center: Vec3::new(-3.0, 0.0, -10.0),
            half_extents: Vec3::splat(0.5),
        }, // visible
    ];

    let visible: Vec<usize> = meshes
        .iter()
        .enumerate()
        .filter(|(_, m)| culler.is_visible(m.center, m.half_extents))
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        visible,
        vec![0, 2],
        "only indices 0 and 2 should be visible"
    );
}

#[test]
fn test_cull_all_outside() {
    let frustum = perspective_frustum();
    let meshes = vec![
        TestMesh {
            center: Vec3::new(10000.0, 0.0, 0.0),
            half_extents: Vec3::splat(1.0),
        },
        TestMesh {
            center: Vec3::new(-10000.0, 0.0, 0.0),
            half_extents: Vec3::splat(1.0),
        },
        TestMesh {
            center: Vec3::new(0.0, 10000.0, 0.0),
            half_extents: Vec3::splat(1.0),
        },
    ];

    let visible = cull_meshes(&frustum, &meshes);
    assert!(visible.is_empty(), "all meshes outside must be culled");
}

#[test]
fn test_cull_none_outside() {
    let frustum = perspective_frustum();
    let meshes = vec![
        TestMesh {
            center: Vec3::new(0.0, 0.0, -5.0),
            half_extents: Vec3::splat(1.0),
        },
        TestMesh {
            center: Vec3::new(1.0, -1.0, -10.0),
            half_extents: Vec3::splat(0.5),
        },
    ];

    let visible = cull_meshes(&frustum, &meshes);
    assert_eq!(visible.len(), 2, "all meshes inside must pass");
}
