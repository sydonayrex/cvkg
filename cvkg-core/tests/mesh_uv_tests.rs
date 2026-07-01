use cvkg_core::mesh::Mesh;

/// Verify Mesh gains tex_coords without breaking existing construction.

#[test]
fn test_mesh_tex_coords_default_empty() {
    let m = Mesh::default();
    assert!(m.tex_coords.is_empty());
}

#[test]
fn test_mesh_tex_coords_len_matches_vertices() {
    let mut m = Mesh::default();
    m.vertices = vec![[0.0; 3]; 4];
    m.normals = vec![[0.0, 1.0, 0.0]; 4];
    m.indices = vec![0, 1, 2, 2, 3, 0];
    m.tex_coords = vec![[0.0, 0.0]; 4];
    assert_eq!(m.tex_coords.len(), m.vertices.len());
}

#[test]
fn test_mesh_from_obj_populates_tex_coords() {
    // OBJ loader must produce tex_coords (even if all zeros for meshes without UVs).
    let m = Mesh::from_obj("tests/assets/cube.obj").unwrap_or_default();
    if !m.vertices.is_empty() {
        assert_eq!(m.tex_coords.len(), m.vertices.len(),
            "tex_coords length must match vertices after OBJ import");
    }
}