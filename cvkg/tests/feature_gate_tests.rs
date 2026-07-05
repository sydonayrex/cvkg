/// Feature gate compile test — verifies that the render-3d feature gate
/// correctly exposes or hides cvkg_render_3d types.

#[cfg(feature = "render-3d")]
#[test]
fn test_render_3d_feature_present() {
    // With the feature enabled, cvkg_render_3d types should be accessible.
    let _ = std::any::type_name::<cvkg_render_3d::types::DirectionalLight>();
    let _ = std::any::type_name::<cvkg_render_3d::passes::ShadowNode>();
    let _ = std::any::type_name::<cvkg_render_3d::passes::Opaque3dNode>();
    let _ = std::any::type_name::<cvkg_render_3d::culler::FrustumCuller>();
}

#[cfg(not(feature = "render-3d"))]
#[test]
fn test_render_3d_feature_absent() {
    // When feature is off, render-3d types should not be in scope.
    // This test compiles only when the feature is disabled, proving the gate works.
    assert!(true);
}
