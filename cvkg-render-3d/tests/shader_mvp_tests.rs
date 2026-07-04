/// Vertex MVP shader tests — verifies that the 3D vertex and PBR shaders
/// are well-formed. mesh_vertex.wgsl depends on common.wgsl types so we use
/// content checks; mesh_pbr.wgsl is self-contained and compiles via naga.

/// mesh_vertex.wgsl must define the vs_main_3d entry point.
#[test]
fn test_mesh_vertex_shader_has_vs_main() {
    let source = include_str!("../src/shaders/mesh_vertex.wgsl");
    assert!(
        source.contains("vs_main_3d"),
        "vertex shader must define vs_main_3d entry point"
    );
}

/// mesh_vertex.wgsl must use the model matrix from instance data.
#[test]
fn test_mesh_vertex_shader_uses_model_matrix() {
    let source = include_str!("../src/shaders/mesh_vertex.wgsl");
    assert!(
        source.contains("model_row0"),
        "vertex shader must reference model_row0 for 3D instance data"
    );
    assert!(
        source.contains("model_row1"),
        "vertex shader must reference model_row1 for 3D instance data"
    );
    assert!(
        source.contains("model_row2"),
        "vertex shader must reference model_row2 for 3D instance data"
    );
}

/// mesh_vertex.wgsl must compute clip_position via view_proj * world_pos.
#[test]
fn test_mesh_vertex_shader_computes_clip_position() {
    let source = include_str!("../src/shaders/mesh_vertex.wgsl");
    assert!(
        source.contains("clip_position"),
        "vertex shader must output clip_position"
    );
    assert!(
        source.contains("world_pos"),
        "vertex shader must compute world_pos"
    );
}

/// mesh_pbr.wgsl is self-contained and must compile via naga.
#[test]
fn test_mesh_pbr_shader_compiles() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(source).expect("mesh_pbr.wgsl failed to parse");
    let fs = module
        .entry_points
        .iter()
        .find(|e| e.name == "fs_main")
        .expect("PBR shader must have fs_main");
    assert!(fs.function.arguments.len() > 0);
}
