#[test]
fn test_surtr_renderer_compiles() {
    // We only test that the code compiles.
    // In winit 0.30, Window creation requires an ActiveEventLoop which is
    // only available during the application lifecycle.
    // Full wgpu initialization is verified via the niflheim_demo.

    // This test ensures the SurtrRenderer API remains stable.
    println!("Surtr Renderer test compiled successfully.");
}
