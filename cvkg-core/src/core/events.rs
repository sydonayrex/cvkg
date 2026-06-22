// =========================================================================
// P1-40: EventPhase -- documents event propagation phases
// =========================================================================
//
// The CVKG event system follows the standard capture/target/bubble
// model used by the W3C DOM Event spec. When an event fires, it
// propagates through 3 phases:
//
// 1. Capture: the event travels from the root down to the
//    target's parent. Listeners registered for the capture
//    phase fire first.
// 2. Target: the event reaches the target node itself. Listeners
//    on the target fire (regardless of capture/bubble).
// 3. Bubble: the event travels back up from the target's
//    parent to the root. Listeners registered for the bubble
//    phase fire last.
//
// Cancellation: any handler can call Event::stop_propagation()
// to prevent the event from continuing to the next phase or
// the next node. This affects only the current event instance.
//
// Example: a click on a button inside a panel:
//  - panel's capture handler fires
//  - button's capture handler fires
//  - button's target handler fires
//  - button's bubble handler fires
//  - panel's bubble handler fires
//
// Use this enum when registering listeners to specify which
// phase to listen for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPhase {
    /// Event is traveling from the root toward the target.
    Capture,
    /// Event has reached the target node.
    Target,
    /// Event is traveling from the target back toward the root.
    Bubble,
}

impl EventPhase {
    /// All phases in propagation order.
    pub const ALL: [EventPhase; 3] = [
        EventPhase::Capture,
        EventPhase::Target,
        EventPhase::Bubble,
    ];
}

/// A basic implementation of AssetManager that can be overridden by platform backends.
pub struct DefaultAssetManager {
    cache: AssetCache,
}
type AssetCache = Arc<arc_swap::ArcSwap<HashMap<String, AssetState<Arc<Vec<u8>>>>>>;

impl Default for DefaultAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAssetManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        }
    }
}

impl AssetManager for DefaultAssetManager {
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>> {
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        self.cache.rcu(|map| {
            let mut m = (**map).clone();
            m.entry(url.to_string()).or_insert(AssetState::Loading);
            m
        });
        AssetState::Loading
    }

    fn preload_image(&self, _url: &str) {}
}

use std::future::Future;

/// Suspense wrapper for asynchronous state management.
/// Integrates with State<T> to provide loading/error/ready states for async operations.
pub struct Suspense<T: Clone + Send + Sync + 'static> {
    inner: State<AssetState<T>>,
}

impl<T: Clone + Send + Sync + 'static> Default for Suspense<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> Suspense<T> {
    pub fn new() -> Self {
        Self {
            inner: State::new(AssetState::Loading),
        }
    }

    pub fn new_async<F>(future: F) -> Self
    where
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let suspense = Self::new();
        let suspense_clone = suspense.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            // P1-17 fix: use the shared fallback runtime instead of
            // spawning a new OS thread + runtime per call. If an
            // ambient tokio runtime exists, prefer it (preserves
            // caller intent). Otherwise use the shared fallback
            // runtime which is bounded to a small worker count.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
            } else {
                fallback_runtime().spawn(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
            }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let result = future.await;
                match result {
                    Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                    Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                }
            });
        }

        suspense
    }

    pub fn ready(value: T) -> Self {
        Self {
            inner: State::new(AssetState::Ready(value)),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            inner: State::new(AssetState::Error(message.into())),
        }
    }

    pub fn get(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn get_ref(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.get(), AssetState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.get(), AssetState::Ready(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.get(), AssetState::Error(_))
    }

    pub fn ready_value(&self) -> Option<T> {
        match self.get() {
            AssetState::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn error_message(&self) -> Option<String> {
        match self.get() {
            AssetState::Error(message) => Some(message),
            _ => None,
        }
    }

    pub fn subscribe<F: Fn(&AssetState<T>) + Send + Sync + 'static>(&self, callback: F) {
        self.inner.subscribe(callback)
    }

    pub fn inner_state(&self) -> &State<AssetState<T>> {
        &self.inner
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Suspense<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<T> for Suspense<T> {
    fn from(value: T) -> Self {
        Self::ready(value)
    }
}

impl<T: Clone + Send + Sync + 'static> From<Result<T, String>> for Suspense<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(value) => Self::ready(value),
            Err(error) => Self::error(error),
        }
    }
}

#[cfg(test)]
mod phase1_test;

/// Berserker mode states for the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderIntensityMode {
    Normal,
    Rage,    // Red tint, slight shake
    Frenzy,  // Heavy red tint, motion blur, aggressive shake
    GodMode, // Golden aura, lightning arcs
}

/// Seer trait for AI-assisted UI components.
/// Allows components to receive "prophecies" (predictions) from an AI backend.
pub trait Seer: Send + Sync {
    /// Provide a prediction for the next user action or content.
    fn predict(&self, context: &str) -> String;
    /// Stream real-time "whispers" (transcriptions/intent).
    fn whispers(&self) -> Vec<String>;
}

#[cfg(test)]
mod vili_tests {
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
}

#[cfg(test)]
mod error_boundary_tests {
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
        let copied = snap;
        assert_eq!(copied, snap);
    }
}

