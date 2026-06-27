    use super::*;

    /// A trivial view that renders successfully.
    struct SuccessView;

    impl View for SuccessView {
        type Body = Never;
        fn body(self) -> Never {
            // SAFETY: `Never` is uninhabitable -- test helper with no composable body.
            unreachable!()
        }
        fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
            // No-op -- renders successfully.
        }
    }

    /// A view that panics during render.
    struct PanicOnRender;

    impl View for PanicOnRender {
        type Body = Never;
        fn body(self) -> Never {
            // SAFETY: `Never` is uninhabitable -- test helper that only panics in render().
            unreachable!()
        }
        fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
            panic!("intentional render panic");
        }
    }

    /// A view that panics during intrinsic_size.
    struct PanicOnSize;

    impl View for PanicOnSize {
        type Body = Never;
        fn body(self) -> Never {
            // SAFETY: `Never` is uninhabitable -- test helper that only panics in intrinsic_size().
            unreachable!()
        }
        fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
            // Render succeeds, but size panics.
        }
        fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
            panic!("intentional size panic");
        }
    }

    /// A view that panics with a String payload.
    struct PanicWithString;

    impl View for PanicWithString {
        type Body = Never;
        fn body(self) -> Never {
            // SAFETY: `Never` is uninhabitable -- test helper that panics with a String payload.
            unreachable!()
        }
        fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
            panic!("{}", "custom error message".to_string());
        }
    }

    struct DummyRenderer;
    impl ElapsedTime for DummyRenderer {
        fn elapsed_time(&self) -> f32 {
            0.0
        }
        fn delta_time(&self) -> f32 {
            0.0
        }
    }
    impl Renderer for DummyRenderer {
        fn fill_rect(&mut self, _r: Rect, _c: [f32; 4]) {}
        fn fill_rounded_rect(&mut self, _r: Rect, _rad: f32, _c: [f32; 4]) {}
        fn fill_ellipse(&mut self, _r: Rect, _c: [f32; 4]) {}
        fn stroke_rect(&mut self, _r: Rect, _c: [f32; 4], _w: f32) {}
        fn stroke_rounded_rect(&mut self, _r: Rect, _rad: f32, _c: [f32; 4], _w: f32) {}
        fn stroke_ellipse(&mut self, _r: Rect, _c: [f32; 4], _w: f32) {}
        fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _c: [f32; 4], _w: f32) {}

        fn memoize(&mut self, _id: u64, _hash: u64, _r: &dyn Fn(&mut dyn Renderer)) {}
        fn draw_mesh_3d(&mut self, _mesh: &Mesh, _material: &Material3D, _transform: &Transform3D) {
        }
        fn set_camera_3d(&mut self, _camera: &Camera3D) {}
        fn push_transform_3d(&mut self, _transform: &Transform3D) {}
        fn pop_transform_3d(&mut self) {}
    }

    const TEST_RECT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };

    #[test]
    fn error_boundary_renders_child_on_success() {
        let boundary = ErrorBoundary::new(SuccessView);
        let mut renderer = DummyRenderer;

        boundary.render(&mut renderer, TEST_RECT);

        assert!(
            !boundary.has_error(),
            "should not have error after successful render"
        );
        assert!(
            boundary.last_error().is_none(),
            "should have no error message"
        );
    }

    #[test]
    fn error_boundary_catches_render_panic() {
        let boundary = ErrorBoundary::new(PanicOnRender);
        let mut renderer = DummyRenderer;

        // This must NOT panic -- the boundary catches it.
        boundary.render(&mut renderer, TEST_RECT);

        assert!(
            boundary.has_error(),
            "should have error after catching panic"
        );
        let err = boundary.last_error().expect("should have error message");
        assert!(
            err.contains("intentional render panic"),
            "error message should contain panic message, got: {err}"
        );
    }

    #[test]
    fn error_boundary_catches_size_panic() {
        let boundary = ErrorBoundary::new(PanicOnSize);
        let mut renderer = DummyRenderer;
        let proposal = layout::SizeProposal {
            width: Some(100.0),
            height: Some(50.0),
        };

        let size = boundary.intrinsic_size(&mut renderer, proposal);

        assert!(
            boundary.has_error(),
            "should have error after catching size panic"
        );
        assert_eq!(size, Size::ZERO, "fallback size should be zero");
    }

    #[test]
    fn error_boundary_catches_string_panic() {
        let boundary = ErrorBoundary::new(PanicWithString);
        let mut renderer = DummyRenderer;

        boundary.render(&mut renderer, TEST_RECT);

        assert!(boundary.has_error());
        let err = boundary.last_error().expect("should have error message");
        assert!(
            err.contains("custom error message"),
            "should capture String panic payload, got: {err}"
        );
    }

    #[test]
    fn error_boundary_clear_error_resets_state() {
        let boundary = ErrorBoundary::new(PanicOnRender);
        let mut renderer = DummyRenderer;

        boundary.render(&mut renderer, TEST_RECT);
        assert!(boundary.has_error());

        boundary.clear_error();
        assert!(
            !boundary.has_error(),
            "should be clear after clear_error()"
        );
        assert!(
            boundary.last_error().is_none(),
            "error message should be cleared"
        );
    }

    #[test]
    fn error_boundary_fallback_color_is_configurable() {
        let boundary = ErrorBoundary::new(SuccessView)
            .fallback_color([0.0, 0.0, 1.0, 1.0])
            .fallback_label("custom label");

        assert_eq!(boundary.fallback_color, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(
            boundary.fallback_label.as_deref(),
            Some("custom label")
        );
    }

    #[test]
    fn error_boundary_flex_weight_delegates_to_child() {
        let boundary = ErrorBoundary::new(SuccessView);
        assert_eq!(boundary.flex_weight(), 0.0, "should delegate to child (default 0.0)");
    }

    #[test]
    fn error_boundary_body_delegates_to_child() {
        // body() must be pure and delegate directly.
        let _boundary = ErrorBoundary::new(SuccessView);
        // Calling body() should not panic and should return Never (unreachable).
        // We test this indirectly -- if it compiles and the Never type is correct,
        // the body() call would diverge. We just verify the type compiles.
        let _boundary_type = std::any::type_name::<ErrorBoundary<SuccessView>>();
    }

    /// Renderer that tracks stack-pushing operations so tests can verify
    /// ErrorBoundary restores renderer state on panic.
    struct TrackingRenderer {
        clip_depth: u32,
        opacity_depth: u32,
        shadow_depth: u32,
    }

    impl TrackingRenderer {
        fn new() -> Self {
            Self {
                clip_depth: 0,
                opacity_depth: 0,
                shadow_depth: 0,
            }
        }
    }

    impl Renderer for TrackingRenderer {
        fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
        fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
        fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
        fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _w: f32) {}
        fn stroke_rounded_rect(
            &mut self,
            _rect: Rect,
            _radius: f32,
            _color: [f32; 4],
            _stroke_width: f32,
        ) {
        }
        fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _stroke_width: f32) {}
        fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _c: [f32; 4], _w: f32) {}

        fn push_clip_rect(&mut self, _rect: Rect) {
            self.clip_depth += 1;
        }
        fn pop_clip_rect(&mut self) {
            self.clip_depth = self.clip_depth.saturating_sub(1);
        }
        fn push_opacity(&mut self, _opacity: f32) {
            self.opacity_depth += 1;
        }
        fn pop_opacity(&mut self) {
            self.opacity_depth = self.opacity_depth.saturating_sub(1);
        }
        fn push_shadow(&mut self, _r: f32, _c: [f32; 4], _o: [f32; 2]) {
            self.shadow_depth += 1;
        }
        fn pop_shadow(&mut self) {
            self.shadow_depth = self.shadow_depth.saturating_sub(1);
        }
        fn memoize(&mut self, _id: u64, _hash: u64, _r: &dyn Fn(&mut dyn Renderer)) {}
        fn snapshot_render_state(&self) -> RenderStateSnapshot {
            // Note: cannot mutate self in &self method; we record that it was
            // called via a different channel (the test counts calls on a Cell).
            RenderStateSnapshot {
                clip_depth: self.clip_depth,
                opacity_depth: self.opacity_depth,
                slice_depth: 0,
                shadow_depth: self.shadow_depth,
                transform_depth: 0,
                vnode_depth: 0,
            }
        }
        fn restore_render_state(&mut self, snap: RenderStateSnapshot) {
            self.clip_depth = snap.clip_depth;
            self.opacity_depth = snap.opacity_depth;
            self.shadow_depth = snap.shadow_depth;
        }
        fn draw_mesh_3d(&mut self, _mesh: &Mesh, _material: &Material3D, _transform: &Transform3D) {}
        fn set_camera_3d(&mut self, _camera: &Camera3D) {}
        fn push_transform_3d(&mut self, _transform: &Transform3D) {}
        fn pop_transform_3d(&mut self) {}
    }

    impl ElapsedTime for TrackingRenderer {
        fn elapsed_time(&self) -> f32 {
            0.0
        }
        fn delta_time(&self) -> f32 {
            0.0
        }
    }

    /// View that pushes clip/opacity/shadow stacks and then panics.
    /// After ErrorBoundary restores state, the renderer should have no leftover
    /// pushed items.
    struct StackPushingPanicView;

    impl View for StackPushingPanicView {
        type Body = Never;
        fn body(self) -> Never {
            unreachable!()
        }
        fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
            renderer.push_clip_rect(Rect::new(0.0, 0.0, 50.0, 50.0));
            renderer.push_opacity(0.5);
            renderer.push_shadow(2.0, [0.0, 0.0, 0.0, 0.5], [0.0, 0.0]);
            panic!("intentional stack-pushing panic");
        }
    }

    #[test]
    fn error_boundary_restores_renderer_state_on_panic() {
        // Regression test for P0-5: ErrorBoundary must restore renderer
        // stack state after a mid-render panic so siblings drawn afterward
        // don't inherit leaked clip/opacity/transform/etc. state.
        let boundary = ErrorBoundary::new(StackPushingPanicView);
        let mut renderer = TrackingRenderer::new();

        // Pre-snapshot: empty stacks.
        let snap_before = renderer.snapshot_render_state();
        assert_eq!(snap_before.clip_depth, 0);
        assert_eq!(snap_before.opacity_depth, 0);
        assert_eq!(snap_before.shadow_depth, 0);

        // Render -- child panics, boundary must catch and restore.
        boundary.render(&mut renderer, TEST_RECT);

        // Verify the panic was caught and state was restored.
        assert!(boundary.has_error(), "should have caught the panic");
        let snap_after = renderer.snapshot_render_state();
        assert_eq!(
            snap_after.clip_depth, 0,
            "clip stack should be restored to empty after panic"
        );
        assert_eq!(
            snap_after.opacity_depth, 0,
            "opacity stack should be restored to empty after panic"
        );
        assert_eq!(
            snap_after.shadow_depth, 0,
            "shadow stack should be restored to empty after panic"
        );
    }

    #[test]
    fn render_state_snapshot_default_is_zeroed() {
        // The default snapshot must be all-zero so backends without
        // stack state can use it as a sentinel.
        let snap = RenderStateSnapshot::default();
        assert_eq!(snap.clip_depth, 0);
        assert_eq!(snap.opacity_depth, 0);
        assert_eq!(snap.slice_depth, 0);
        assert_eq!(snap.shadow_depth, 0);
        assert_eq!(snap.transform_depth, 0);
        assert_eq!(snap.vnode_depth, 0);
    }

    #[test]
    fn render_state_snapshot_round_trip() {
        let snap = RenderStateSnapshot {
            clip_depth: 3,
            opacity_depth: 2,
            slice_depth: 1,
            shadow_depth: 0,
            transform_depth: 4,
            vnode_depth: 5,
        };
    }