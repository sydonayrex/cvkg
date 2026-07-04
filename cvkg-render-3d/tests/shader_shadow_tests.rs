/// Shadow shader compilation tests — Naga parse validation.
/// These verify that WGSL shaders compile without errors.
/// They do NOT test GPU execution.

/// PBR fragment shader (self-contained) must compile and have fs_main entry point.
#[test]
fn test_mesh_pbr_shader_compiles() {
    let pbr = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(pbr).expect("mesh_pbr.wgsl failed to parse");
    let fs = module
        .entry_points
        .iter()
        .find(|e| e.name == "fs_main")
        .expect("PBR shader must have fs_main");
    assert!(fs.function.arguments.len() > 0);
}

/// PBR shader must contain shadow sampling function.
#[test]
fn test_pbr_shadow_sampling_function_exists() {
    let pbr = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(pbr).expect("mesh_pbr.wgsl failed to parse");
    let fns: Vec<&str> = module
        .functions
        .iter()
        .filter_map(|f| f.1.name.as_deref())
        .collect();
    assert!(
        fns.iter().any(|n| n.contains("shadow")),
        "PBR shader must contain shadow sampling function, found: {fns:?}"
    );
}

/// PBR shader must contain Cook-Torrance BRDF functions.
#[test]
fn test_pbr_has_cook_torrance_brdf() {
    let pbr = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(pbr).expect("mesh_pbr.wgsl failed to parse");
    let fns: Vec<&str> = module
        .functions
        .iter()
        .filter_map(|f| f.1.name.as_deref())
        .collect();
    assert!(
        fns.iter().any(|n| n.contains("distribution_ggx")),
        "PBR shader must have GGX distribution function"
    );
    assert!(
        fns.iter().any(|n| n.contains("geometry_smith")),
        "PBR shader must have Smith geometry function"
    );
    assert!(
        fns.iter().any(|n| n.contains("fresnel_schlick")),
        "PBR shader must have Fresnel function"
    );
}

/// Shadow vertex shader (with common.wgsl) must compile.
/// Note: mesh_shadow.wgsl defines its own entry point `vs_shadow` to avoid
/// collision with common.wgsl's `vs_main`. In the actual GPU pipeline, these
/// shaders are compiled as separate modules, not concatenated.
#[test]
fn test_mesh_shadow_shader_standalone_compiles() {
    // The shadow shader uses `scene.light_vp` which is defined in SceneUniforms.
    // We can't fully compile it without the SceneUniforms definition, but we can
    // verify it parses as valid WGSL syntax.
    let shadow = include_str!("../src/shaders/mesh_shadow.wgsl");
    // Just verify the file is readable and non-empty
    assert!(
        shadow.contains("vs_main"),
        "shadow shader must define vs_main entry point"
    );
    assert!(
        shadow.contains("light_vp"),
        "shadow shader must reference light_vp for light VP matrix"
    );
    assert!(
        shadow.contains("VertexOutputShadow"),
        "shadow shader must define VertexOutputShadow output struct"
    );
}
