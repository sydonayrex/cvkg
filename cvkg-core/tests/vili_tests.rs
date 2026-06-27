    use super::*;

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

    #[test]
    fn test_magnetic_warp() {
        let renderer = DummyRenderer;
        let anchor = Rect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        // Pointer is near the anchor (distance < 120)
        let pointer = [125.0, 50.0];
        // distance from center (125, 125) is 75.
        // force = (1.0 - 75/120) * strength
        let warp = renderer.magnetic_warp(pointer, anchor, 1.0);
        // It should pull closer to (125, 125), so Y should be > 50
        assert!(warp[1] > 50.0);

        // Out of range pointer should remain unchanged
        let far_pointer = [500.0, 500.0];
        let far_warp = renderer.magnetic_warp(far_pointer, anchor, 1.0);
        assert_eq!(far_pointer, far_warp);
    }

    #[test]
    fn test_mani_glow() {
        let renderer = DummyRenderer;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let pointer_inside = [50.0, 50.0];
        let glow_max = renderer.mani_glow_intensity(pointer_inside, bounds, 120.0);
        assert_eq!(glow_max, 1.0);

        let pointer_edge = [50.0, -10.0];
        let glow_partial = renderer.mani_glow_intensity(pointer_edge, bounds, 120.0);
        assert!(glow_partial > 0.0 && glow_partial < 1.0);
    }

    #[test]
    fn test_fafnir_evolve() {
        let renderer = DummyRenderer;
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let pointer_inside = [50.0, 50.0];
        let scale = renderer.fafnir_evolve(pointer_inside, bounds, 1.2);
        assert_eq!(scale, 1.2); // Full scale when hovering center
    }

    #[test]
    fn test_undo_manager_basic() {
        let mut manager = UndoManager::new(3, 0.5);
        let val = std::sync::Arc::new(std::sync::Mutex::new(0));

        let v1 = val.clone();
        let v2 = val.clone();
        manager.push(
            "Add",
            move || *v1.lock().unwrap() -= 1,
            move || *v2.lock().unwrap() += 1,
        );

        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        let undo = manager.undo().unwrap();
        undo();
        assert_eq!(*val.lock().unwrap(), -1);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        let redo = manager.redo().unwrap();
        redo();
        assert_eq!(*val.lock().unwrap(), 0);
    }

    #[test]
    fn test_undo_manager_depth_limit() {
        let mut manager = UndoManager::new(2, 0.5);
        manager.push("1", || {}, || {});
        manager.push("2", || {}, || {});
        manager.push("3", || {}, || {});

        assert_eq!(manager.stack.len(), 2);
        assert_eq!(manager.position, 2);
    }

    #[test]
    fn test_undo_manager_coalescing() {
        let mut manager = UndoManager::new(10, 1.0);
        let count = std::sync::Arc::new(std::sync::Mutex::new(0));

        let c = count.clone();
        manager.push_coalesceable("Type", move || *c.lock().unwrap() -= 1, || {});

        let c = count.clone();
        manager.push_coalesceable("Type", move || *c.lock().unwrap() -= 1, || {});

        assert_eq!(manager.stack.len(), 1);

        let undo = manager.undo().unwrap();
        undo();
        assert_eq!(*count.lock().unwrap(), -2);
    }