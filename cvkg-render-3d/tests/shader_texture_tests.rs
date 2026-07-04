/// PBR shader texture binding tests — verifies that the PBR fragment shader
/// declares the required texture bindings for albedo, normal, and ORM maps.

/// PBR shader must compile after texture bindings are added.
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

/// PBR shader must have albedo texture binding.
#[test]
fn test_mesh_pbr_has_albedo_binding() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    assert!(
        source.contains("t_albedo"),
        "PBR shader must declare t_albedo texture binding"
    );
    assert!(
        source.contains("texture_2d"),
        "PBR shader must use texture_2d type for albedo"
    );
}

/// PBR shader must have normal map texture binding.
#[test]
fn test_mesh_pbr_has_normal_binding() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    assert!(
        source.contains("t_normal"),
        "PBR shader must declare t_normal texture binding"
    );
}

/// PBR shader must have ORM (occlusion-roughness-metallic) texture binding.
#[test]
fn test_mesh_pbr_has_orm_binding() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    assert!(
        source.contains("t_orm"),
        "PBR shader must declare t_orm texture binding"
    );
}

/// PBR shader must have a material sampler.
#[test]
fn test_mesh_pbr_has_material_sampler() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    assert!(
        source.contains("s_material"),
        "PBR shader must declare s_material sampler binding"
    );
}
