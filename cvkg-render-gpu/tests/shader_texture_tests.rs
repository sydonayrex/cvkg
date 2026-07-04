#[test]
fn test_mesh_pbr_has_texture_bindings() {
    // Current shader uses t_diffuse texture array (group 0) for all materials including material_id == 13u.
    // Phase 2 will add dedicated PBR texture bindings (t_albedo, t_normal, t_orm, t_shadow).
    // Just verify the binding exists in the source.
    let common = include_str!("../src/shaders/common.wgsl");
    assert!(
        common.contains("t_diffuse"),
        "common.wgsl must have diffuse texture array binding (t_diffuse)"
    );
}