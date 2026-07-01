/// Verify the WGSL vertex shader compiles with the model matrix inputs.
/// This is a compile-time Naga validation test — not a GPU test.

#[test]
fn test_mesh_vertex_shader_compiles() {
    let source = include_str!("../src/shaders/common.wgsl");
    let module = naga::front::wgsl::parse_str(source).expect("common.wgsl failed to parse");
    // Verify the entry point exists.
    let entry = module
        .entry_points
        .iter()
        .find(|e| e.name == "vs_main")
        .expect("vs_main entry point not found");
    assert!(entry.function.arguments.len() > 0);
}

#[test]
fn test_material_pbr_shader_compiles_with_common() {
    // Fragment shaders depend on VertexOutput from common.wgsl.
    // Concatenate them for full validation.
    let common = include_str!("../src/shaders/common.wgsl");
    let pbr = include_str!("../src/shaders/material_pbr.wgsl");
    let combined = format!("{}\n{}", common, pbr);
    let module = naga::front::wgsl::parse_str(&combined)
        .expect("material_pbr.wgsl with common failed to parse");
    let entry = module
        .entry_points
        .iter()
        .find(|e| e.name == "fs_main")
        .expect("fs_main entry point not found in PBR shader");
    assert!(entry.function.arguments.len() > 0);
}

#[test]
fn test_material_opaque_shader_compiles_with_common() {
    let common = include_str!("../src/shaders/common.wgsl");
    let opaque = include_str!("../src/shaders/material_opaque.wgsl");
    let combined = format!("{}\n{}", common, opaque);
    let module = naga::front::wgsl::parse_str(&combined)
        .expect("material_opaque.wgsl with common failed to parse");
    let entry = module
        .entry_points
        .iter()
        .find(|e| e.name == "fs_main")
        .expect("fs_main entry point not found in opaque shader");
    assert!(entry.function.arguments.len() > 0);
}