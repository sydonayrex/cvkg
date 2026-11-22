//! Intercrate smoke coverage for infrastructure crates that previously had
//! only co-located unit tests. Each crate is exercised next to `cvkg-core`
//! types so cross-crate API drift is caught at compile time, and the calls
//! that have a stable API are checked at runtime.

use cvkg_core::Rect;

#[test]
fn infrastructure_crates_interoperate() {
    // Accessibility: tree construction and mutation API is reachable.
    let _tree = cvkg_accessibility::AccessibilityTree::new();

    // Animation: solver can be built from shared spring params.
    let _solver = cvkg_anim::SpringSolver::new(cvkg_anim::SpringParams::snappy(), 1.0, 0.0);

    // CLI, export raster, materials, physics, render-native and SVG filters
    // are type-level interop checks: their public types can be named and
    // sized from this crate.
    assert!(std::mem::size_of::<cvkg_cli::DevToolsDashboard>() > 0);
    assert!(std::mem::size_of::<cvkg_export_raster::CapturedFrame>() > 0);
    assert!(std::mem::size_of::<cvkg_materials::GlassMaterial>() > 0);
    assert!(std::mem::size_of::<cvkg_physics::RigidBody>() > 0);
    assert!(std::mem::size_of::<cvkg_render_native::TranslationContract>() > 0);
    assert!(std::mem::size_of::<cvkg_svg_filters::FilterGraph>() > 0);

    // Compositor: routing produces command buckets without a GPU.
    let mut engine = cvkg_compositor::CompositorEngine::new();
    let _buckets = engine.flatten_and_route();

    // Game HUD: health bar construction uses core colors.
    let _bar = cvkg_game_hud::HealthBar::new(50.0, 100.0);

    // Icons: default registry is populated.
    let icons = cvkg_icons::IconRegistry::with_defaults();
    assert!(icons.len() > 0, "default icon registry must not be empty");

    // Reflection: empty registry has no types.
    let registry = cvkg_reflect::ReflectRegistry::new();
    assert_eq!(registry.type_names().count(), 0);

    // Software renderer: a core Rect draws into its framebuffer.
    let mut renderer =
        cvkg_render_software::SoftwareRenderer::with_color(4, 4, [0.0, 0.0, 0.0, 1.0]);
    cvkg_core::Renderer::fill_rect(
        &mut renderer,
        Rect::new(0.0, 0.0, 4.0, 4.0),
        [1.0, 0.0, 0.0, 1.0],
    );
    assert_ne!(renderer.framebuffer().pixels()[0], 0);

    // Scheduler: production frame driver can start a frame.
    let mut scheduler = cvkg_scheduler::FrameScheduler::new();
    scheduler.begin_frame();

    // Telemetry: collector is constructible.
    let telemetry = cvkg_telemetry::Telemetry::new();
    assert_eq!(telemetry.event_count(), 0);
}
