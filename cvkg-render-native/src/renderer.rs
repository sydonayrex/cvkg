use crate::asset_manager::NativeAssetManager;
use crate::audio::{RodioAudioEngine, VisualHapticEngine};
use crate::main_loop::{App, AppEvent};
use crate::window::{SafeAreaInsets, WindowManager};
use cvkg_core::{
    ColorTheme, DrawMaterial, FrameRenderer, Mesh, Rect, RenderIntensityMode, RenderStateSnapshot,
    Renderer, TelemetryData,
};
use std::sync::Arc;
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;
use winit::window::Window;

thread_local! {
    /// Thread-local raw pointer to the locked GpuRenderer for the duration of one render pass.
    ///
    /// # Safety
    /// This pointer is ONLY valid when a `MutexGuard<GpuRenderer>` is held on the same thread's
    /// call stack. It is set at the start of `begin_frame` and cleared at the end of `end_frame`.
    /// Accessing the pointer when no guard is held is undefined behavior.
    ///
    /// PRIVATE: Not `pub` to prevent external crates from creating dangling references.
    pub(crate) static GPU_FRAME_PTR: std::cell::Cell<*mut cvkg_render_gpu::GpuRenderer> =
        const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// Native renderer backend implementing the Renderer trait.
/// It wraps a shared GpuRenderer for high-performance GPU drawing.
/// During a render pass, GPU_FRAME_PTR is set so draw calls bypass the mutex.
pub struct NativeRenderer {
    pub(crate) gpu: Arc<std::sync::Mutex<cvkg_render_gpu::GpuRenderer>>,
    pub(crate) delta_time: f32,
    pub(crate) elapsed_time: f32,
    pub(crate) berserker_mode: RenderIntensityMode,
    pub(crate) rage: f32,
    pub(crate) window: Arc<Window>,
}

impl NativeRenderer {
    /// Returns a reference to the GPU renderer.
    /// If GPU_FRAME_PTR is set (we're inside a locked render pass) uses that directly.
    /// Otherwise falls back to acquiring the mutex (safe for calls outside the render pass).
    ///
    /// # Safety
    /// GPU_FRAME_PTR is only non-null when a MutexGuard is live on the same thread's call stack.
    #[inline(always)]
    fn gpu_ref(&mut self) -> impl std::ops::DerefMut<Target = cvkg_render_gpu::GpuRenderer> + '_ {
        GPU_FRAME_PTR.with(|ptr| {
            let raw = ptr.get();
            if !raw.is_null() {
                // SAFETY: Pointer is valid and the mutex guard is live above us on the call stack.
                GpuRef::Ptr(unsafe { &mut *raw })
            } else {
                GpuRef::Guard(self.gpu.lock().unwrap_or_else(|p| p.into_inner()))
            }
        })
    }

    /// Read-only variant for &self Renderer methods.
    /// Uses the same thread_local fast path; falls back to mutex for out-of-pass calls.
    ///
    /// # Safety
    /// GPU_FRAME_PTR is only non-null when a MutexGuard is live above us on the call stack.
    #[inline(always)]
    fn gpu_ref_shared(&self) -> impl std::ops::Deref<Target = cvkg_render_gpu::GpuRenderer> + '_ {
        GPU_FRAME_PTR.with(|ptr| {
            let raw = ptr.get();
            if !raw.is_null() {
                // SAFETY: Pointer is valid; the mutex guard is held for the render pass duration.
                // We only read via this path during &self methods, which is safe.
                GpuRefShared::Ptr(unsafe { &*raw })
            } else {
                GpuRefShared::Guard(self.gpu.lock().unwrap_or_else(|p| p.into_inner()))
            }
        })
    }

    /// Create a new NativeRenderer (internal use by App)
    pub(crate) fn new(
        window: Arc<Window>,
        gpu: Arc<std::sync::Mutex<cvkg_render_gpu::GpuRenderer>>,
        delta_time: f32,
        elapsed_time: f32,
        berserker_mode: RenderIntensityMode,
        rage: f32,
    ) -> Self {
        Self {
            gpu,
            delta_time,
            elapsed_time,
            berserker_mode,
            rage,
            window,
        }
    }

    /// Start the CVKG native application with the given view.
    /// `prewarm_assets` is a list of (name, raw_bytes) pairs uploaded to the GPU
    /// texture atlas on the first frame before any draw calls.
    pub fn run<V: cvkg_core::View + 'static>(
        view: V,
        prewarm_assets: Option<Vec<(String, Vec<u8>)>>,
    ) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();

        let event_loop = EventLoop::<AppEvent>::with_user_event()
            .with_any_thread(true)
            .build()
            .expect("failed to create winit event loop: platform initialization failed");
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut app = App {
            view,
            window_manager: WindowManager::new(),
            gpu: None,
            asset_manager: std::sync::Arc::new(NativeAssetManager::new()),
            proxy: event_loop.create_proxy(),
            start_time: std::time::Instant::now(),
            last_frame_time: std::time::Instant::now(),
            berserker_mode: RenderIntensityMode::Normal,
            rage: 0.0,
            state_detector: crate::window::WindowStateDetector::new(),
            frame_budget: cvkg_core::FrameBudgetTracker::default_120fps(),
            modifiers: winit::keyboard::ModifiersState::default(),
            audio_engine: None,
            haptic_engine: Arc::new(VisualHapticEngine::new()),
            pending_prewarm: prewarm_assets,
        };

        event_loop
            .run_app(&mut app)
            .expect("winit event loop terminated with error");
    }

    /// Convenience: run with a single background image loaded from a file path.
    /// The image is loaded from disk and pre-warmed on the first frame.
    /// `image_name` is the key used in `draw_image` / `draw_background_image`.
    pub fn run_with_background<V: cvkg_core::View + 'static>(
        view: V,
        image_name: &str,
        image_path: &str,
    ) {
        let image_data = std::fs::read(image_path)
            .unwrap_or_else(|e| panic!("Failed to load background image '{}': {}", image_path, e));
        let assets = vec![(image_name.to_string(), image_data)];
        Self::run(view, Some(assets));
    }
}

/// Returned by NativeRenderer::gpu_ref() — either a direct pointer ref or a mutex guard.
enum GpuRef<'a> {
    Ptr(&'a mut cvkg_render_gpu::GpuRenderer),
    Guard(std::sync::MutexGuard<'a, cvkg_render_gpu::GpuRenderer>),
}

impl<'a> std::ops::Deref for GpuRef<'a> {
    type Target = cvkg_render_gpu::GpuRenderer;
    fn deref(&self) -> &Self::Target {
        match self {
            GpuRef::Ptr(r) => r,
            GpuRef::Guard(g) => g,
        }
    }
}

impl<'a> std::ops::DerefMut for GpuRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            GpuRef::Ptr(r) => r,
            GpuRef::Guard(g) => &mut *g,
        }
    }
}

/// Read-only variant returned by NativeRenderer::gpu_ref_shared().
enum GpuRefShared<'a> {
    Ptr(&'a cvkg_render_gpu::GpuRenderer),
    Guard(std::sync::MutexGuard<'a, cvkg_render_gpu::GpuRenderer>),
}

impl<'a> std::ops::Deref for GpuRefShared<'a> {
    type Target = cvkg_render_gpu::GpuRenderer;
    fn deref(&self) -> &Self::Target {
        match self {
            GpuRefShared::Ptr(r) => r,
            GpuRefShared::Guard(g) => g,
        }
    }
}

impl cvkg_core::ElapsedTime for NativeRenderer {
    fn delta_time(&self) -> f32 {
        self.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        self.elapsed_time
    }
}

impl cvkg_core::RendererErrorHandler for NativeRenderer {}

impl cvkg_core::Renderer for NativeRenderer {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.gpu_ref().fill_rect(rect, color);
    }
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.gpu_ref().fill_rounded_rect(rect, radius, color);
    }
    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.gpu_ref().fill_ellipse(rect, color);
    }
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref().stroke_rect(rect, color, stroke_width);
    }
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref()
            .stroke_rounded_rect(rect, radius, color, stroke_width);
    }
    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref().stroke_ellipse(rect, color, stroke_width);
    }
    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.gpu_ref()
            .draw_line(x1, y1, x2, y2, color, stroke_width);
    }

    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        self.gpu_ref().fill_glass_rect(rect, radius, blur_radius);
    }

    fn fill_glass_rect_with_intensity(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        glass_intensity: f32,
    ) {
        self.gpu_ref()
            .fill_glass_rect_with_intensity(rect, radius, blur_radius, glass_intensity);
    }

    fn fill_glass_rect_with_pressure(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        pressure: f32,
    ) {
        self.gpu_ref()
            .fill_glass_rect_with_intensity(rect, radius, blur_radius, pressure);
    }

    fn fill_squircle(&mut self, rect: Rect, n: f32, color: [f32; 4]) {
        self.gpu_ref().fill_squircle(rect, n, color);
    }

    fn stroke_squircle(&mut self, rect: Rect, n: f32, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref().stroke_squircle(rect, n, color, stroke_width);
    }

    fn draw_focus_ring(
        &mut self,
        rect: Rect,
        radius: f32,
        offset: f32,
        width: f32,
        color: [f32; 4],
    ) {
        self.gpu_ref()
            .draw_focus_ring(rect, radius, offset, width, color);
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.gpu_ref()
            .draw_linear_gradient(rect, start_color, end_color, angle);
    }
    fn draw_radial_gradient(&mut self, rect: Rect, inner_color: [f32; 4], outer_color: [f32; 4]) {
        self.gpu_ref()
            .draw_radial_gradient(rect, inner_color, outer_color);
    }
    fn draw_texture(&mut self, texture_id: u32, rect: Rect) {
        self.gpu_ref().draw_texture(texture_id, rect);
    }
    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        self.gpu_ref().draw_image(image_name, rect);
    }
    fn load_image(&mut self, name: &str, data: &[u8]) {
        self.gpu_ref().load_image(name, data);
    }
    fn push_clip_rect(&mut self, rect: Rect) {
        self.gpu_ref().push_clip_rect(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.gpu_ref().pop_clip_rect();
    }
    fn push_opacity(&mut self, opacity: f32) {
        self.gpu_ref().push_opacity(opacity);
    }
    fn draw_3d_cube(&mut self, rect: Rect, color: [f32; 4], rotation: [f32; 3]) {
        self.gpu_ref().draw_3d_cube(rect, color, rotation);
    }
    fn render_scene_node_3d(
        &mut self,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
        color: [f32; 4],
        meshes: &[Mesh],
    ) {
        self.gpu_ref()
            .render_scene_node_3d(position, rotation, scale, color, meshes);
    }
    fn pop_opacity(&mut self) {
        self.gpu_ref().pop_opacity();
    }
    fn bifrost(&mut self, rect: Rect, blur: f32, saturation: f32, opacity: f32) {
        self.gpu_ref().bifrost(rect, blur, saturation, opacity);
    }
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.gpu_ref().push_mjolnir_slice(angle, offset);
    }
    fn pop_mjolnir_slice(&mut self) {
        self.gpu_ref().pop_mjolnir_slice();
    }
    fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu_ref().mjolnir_shatter(rect, pieces, force, color);
    }
    fn mjolnir_fluid_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu_ref()
            .mjolnir_fluid_shatter(rect, pieces, force, color);
    }
    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.gpu_ref().draw_mjolnir_bolt(from, to, color);
    }
    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.gpu_ref().gungnir(rect, color, radius, intensity);
    }
    fn mani_glow(&mut self, rect: Rect, color: [f32; 4], radius: f32) {
        self.gpu_ref().mani_glow(rect, color, radius);
    }
    fn register_handler(
        &mut self,
        event_type: &str,
        handler: Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.gpu_ref().register_handler(event_type, handler);
    }
    fn push_vnode(&mut self, rect: Rect, name: &'static str) {
        self.gpu_ref().push_vnode(rect, name);
    }
    fn pop_vnode(&mut self) {
        self.gpu_ref().pop_vnode();
    }
    fn set_z_index(&mut self, z: f32) {
        self.gpu_ref().set_z_index(z);
    }
    fn get_z_index(&self) -> f32 {
        self.gpu_ref_shared().get_z_index()
    }
    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        self.gpu_ref().register_shared_element(id, rect);
    }
    fn set_material(&mut self, material: DrawMaterial) {
        self.gpu_ref().set_material(material);
    }
    fn current_material(&self) -> DrawMaterial {
        self.gpu_ref_shared().current_material()
    }
    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        self.gpu_ref().serialize_svg(name)
    }
    fn apply_svg_filter(
        &mut self,
        name: &str,
        filter_id: &str,
        region: Rect,
    ) -> Result<String, String> {
        self.gpu_ref().apply_svg_filter(name, filter_id, region)
    }
    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.gpu_ref().push_shadow(radius, color, offset);
    }
    fn pop_shadow(&mut self) {
        self.gpu_ref().pop_shadow();
    }
    fn push_affine(&mut self, transform: [f32; 6]) {
        self.gpu_ref().push_affine(transform);
    }
    fn enter_portal(&mut self, z_index: i32) {
        log::warn!(
            "Portal rendering (enter_portal) not yet implemented in GPU backend; z_index={}",
            z_index
        );
    }
    fn exit_portal(&mut self) {
        log::warn!("Portal rendering (exit_portal) not yet implemented in GPU backend");
    }
    fn viewport_size(&self) -> Rect {
        let size = self.window.inner_size();
        let scale = self.window.scale_factor();
        let logical = size.to_logical::<f32>(scale);
        Rect::new(0.0, 0.0, logical.width, logical.height)
    }
    fn announce(&mut self, message: &str, priority: cvkg_core::AnnouncementPriority) {
        log::info!("Accessibility announcement [{:?}]: {}", priority, message);
    }
    fn load_svg(&mut self, name: &str, svg_data: &[u8]) {
        self.gpu_ref().load_svg(name, svg_data);
    }
    fn draw_svg(&mut self, name: &str, rect: Rect) {
        self.gpu_ref().draw_svg(name, rect, None, 0);
    }
    fn draw_svg_with_offset(&mut self, name: &str, rect: Rect, animation_time_offset: f32) {
        self.gpu_ref()
            .draw_svg_with_offset(name, rect, None, 0, animation_time_offset);
    }
    fn get_telemetry(&self) -> TelemetryData {
        self.gpu_ref_shared().telemetry.clone()
    }
    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        self.gpu_ref().prewarm_vram(assets);
    }

    fn text_scale_factor(&self) -> f32 {
        self.gpu_ref_shared().text_scale_factor()
    }

    fn is_over_budget(&self) -> bool {
        self.gpu_ref_shared().is_over_budget()
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.gpu_ref().draw_text(text, x, y, size, color);
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.gpu_ref().measure_text(text, size)
    }

    fn shape_rich_text(
        &mut self,
        spans: &[runic_text::TextSpan],
        max_width: Option<f32>,
        align: runic_text::TextAlign,
        overflow: runic_text::TextOverflow,
    ) -> Option<runic_text::ShapedText> {
        self.gpu_ref()
            .shape_rich_text(spans, max_width, align, overflow)
    }

    fn draw_shaped_text(&mut self, shaped: &runic_text::ShapedText, x: f32, y: f32) {
        self.gpu_ref().draw_shaped_text(shaped, x, y);
    }

    fn fill_glass_rect_with_tint(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        tint_color: [f32; 4],
        glass_intensity: f32,
    ) {
        self.gpu_ref().fill_glass_rect_with_tint(
            rect,
            radius,
            blur_radius,
            tint_color,
            glass_intensity,
        );
    }

    fn set_theme(&mut self, theme: ColorTheme) {
        self.gpu_ref().set_theme(theme);
    }

    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.gpu_ref().trigger_shatter_event(origin, force);
    }

    fn set_fireball_pos(&mut self, pos: [f32; 2]) {
        self.gpu_ref().set_fireball_pos(pos);
    }

    fn set_scene(&mut self, scene: &str) {
        self.gpu_ref().set_scene(scene);
    }

    fn set_scene_preset(&mut self, preset: u32) {
        self.gpu_ref().set_scene_preset(preset);
    }

    fn set_default_background_color(&mut self, color: [f32; 4]) {
        self.gpu_ref().set_default_background_color(color);
    }
    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        self.gpu_ref().push_transform(translation, scale, rotation);
    }
    fn pop_transform(&mut self) {
        self.gpu_ref().pop_transform();
    }

    fn set_berserker_mode(&mut self, state: RenderIntensityMode) {
        self.berserker_mode = state;

        if state == RenderIntensityMode::GodMode {
            log::info!("ENTERING GOD MODE: Activating Berserker Determinism (High Priority)");
            #[cfg(target_os = "linux")]
            unsafe {
                let ret = libc::setpriority(libc::PRIO_PROCESS, 0, -10);
                if ret != 0 {
                    log::warn!(
                        "GodMode: setpriority failed (errno: {}) — need CAP_SYS_NIO",
                        std::io::Error::last_os_error()
                    );
                }
            }
        } else {
            #[cfg(target_os = "linux")]
            unsafe {
                let _ = libc::setpriority(libc::PRIO_PROCESS, 0, 0);
            }
        }

        self.gpu_ref().set_berserker_mode(state);
    }

    fn set_rage(&mut self, rage: f32) {
        self.rage = rage;
        self.gpu_ref().set_rage(rage);
    }

    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        self.gpu_ref().memoize(id, data_hash, render_fn);
    }

    fn snapshot_render_state(&self) -> RenderStateSnapshot {
        self.gpu_ref_shared().snapshot_render_state()
    }

    fn restore_render_state(&mut self, snap: RenderStateSnapshot) {
        self.gpu_ref().restore_render_state(snap);
    }
    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn capture_png(&mut self) -> Vec<u8> {
        log::info!("CAPTURING_FRAME: Initiating GPU readback...");
        let gpu = self.gpu.lock().unwrap_or_else(|p| p.into_inner());
        pollster::block_on(gpu.capture_frame()).unwrap_or_else(|e| {
            log::error!("GPU frame capture failed: {}", e);
            Vec::new()
        })
    }

    fn print(&mut self) {
        log::info!("PRINT_BRIDGE: Spooling mission status to native printer...");
        log::debug!("[BRIDGE] PRINTER_READY // SPOOLING_DATA...");
    }
}
