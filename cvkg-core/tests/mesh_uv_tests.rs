/// Mesh tex_coords backward compatibility tests.
/// Verifies that adding tex_coords to Mesh doesn't break existing construction.
use cvkg_core::mesh::Mesh;

/// Default mesh has zero vertices, so tex_coords should also be empty.
#[test]
fn test_mesh_tex_coords_default_empty() {
    let m = Mesh::default();
    assert!(m.tex_coords.is_empty());
}

/// tex_coords length must match vertices length after manual construction.
#[test]
fn test_mesh_tex_coords_len_matches_vertices() {
    let mut m = Mesh::default();
    m.vertices = vec![[0.0; 3]; 4];
    m.normals = vec![[0.0, 1.0, 0.0]; 4];
    m.indices = vec![0, 1, 2, 2, 3, 0];
    m.tex_coords = vec![[0.0, 0.0]; 4];
    assert_eq!(m.tex_coords.len(), m.vertices.len());
}

/// from_stl must produce tex_coords (all zeros since STL has no UVs).
#[test]
fn test_mesh_from_stl_tex_coords_empty() {
    // Minimal valid STL (single triangle).
    let stl = b"solid test\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid test\n";
    let m = Mesh::from_stl(stl).unwrap();
    assert_eq!(m.tex_coords.len(), m.vertices.len());
    assert!(m.tex_coords.iter().all(|uv| uv == &[0.0, 0.0]));
}

/// Constructed mesh can use struct update syntax without tex_coords field.
#[test]
fn test_mesh_struct_update_syntax() {
    let m = Mesh {
        vertices: vec![[0.0; 3]; 3],
        normals: vec![[0.0; 3]; 3],
        ..Default::default()
    };
    // tex_coords defaults to empty vec via Default
    assert!(m.tex_coords.is_empty());
}

/// tex_coords with actual UV values round-trips through Default + assignment.
#[test]
fn test_mesh_tex_coords_values_preserved() {
    let mut m = Mesh::default();
    m.vertices = vec![[0.0; 3]; 2];
    m.tex_coords = vec![[0.5, 0.5], [1.0, 1.0]];
    assert_eq!(m.tex_coords[0], [0.5, 0.5]);
    assert_eq!(m.tex_coords[1], [1.0, 1.0]);
}
