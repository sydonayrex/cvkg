use cvkg_core::mesh::Mesh;

/// Verify Mesh::aabb() returns correct center and half-extents.

#[test]
fn test_aabb_unit_cube() {
    let mut m = Mesh::default();
    m.vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];
    let (center, half) = m.aabb();
    assert!((center.x - 0.5).abs() < 1e-6);
    assert!((center.y - 0.5).abs() < 1e-6);
    assert!((center.z - 0.5).abs() < 1e-6);
    assert!((half.x - 0.5).abs() < 1e-6);
    assert!((half.y - 0.5).abs() < 1e-6);
    assert!((half.z - 0.5).abs() < 1e-6);
}

#[test]
fn test_aabb_single_vertex() {
    let mut m = Mesh::default();
    m.vertices = vec![[3.0, 4.0, 5.0]];
    let (center, half) = m.aabb();
    assert!((center.x - 3.0).abs() < 1e-6);
    assert!((half.x).abs() < 1e-6, "single vertex has zero half-extent");
}

#[test]
fn test_aabb_empty_mesh() {
    let m = Mesh::default();
    let (center, half) = m.aabb();
    assert!(
        center.x.is_nan() || half.x.abs() < 1e-6,
        "empty mesh should return zero or NaN AABB"
    );
}
