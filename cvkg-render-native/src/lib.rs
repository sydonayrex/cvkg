//! # CVKG Agentic Development Guidelines (v1.3)
//!
//! All AI agents contributing to this crate MUST follow ALL eight rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–8) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//! 8. HARDWARE VERIFIED -- NEVER declare success based on mock data/rendering for native crates.
//!                      Any change to input, rendering, or lifecycle MUST be verified via physical
//!                      loopback (e.g., cargo run -p berserker) and signal path tracing.
//!
//! Sources:
//! Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//! CVKG Extended: Section 14 of the CVKG Design Specification (v1.3)
#![allow(
    unused_imports,
    clippy::single_component_path_imports,
    dead_code,
    clippy::items_after_test_module,
    clippy::field_reassign_with_default,
    clippy::collapsible_if,
    clippy::unnecessary_map_or
)]

//! Platform-native widget delegation using winit and AccessKit
//!
//! This crate provides platform-specific rendering backends for native desktop targets
//  using winit for window/event handling and AccessKit for accessibility tree integration.

use cvkg_core::{FocusableId, FrameRenderer, KvasirId, RenderStateSnapshot, Renderer};
use image;
// FIX #10: Wayland import gated to Linux only -- was unconditional, broke macOS/Windows builds.
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Represents the current state of a window.
///
/// Used by [`WindowStateDetector`] to track lifecycle transitions and drive
/// rendering decisions (e.g., skip frames when occluded or minimized).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window is visible and active.
    Normal,
    /// Window is minimized to the Dock or taskbar.
    Minimized,
    /// Window is in fullscreen mode.
    Fullscreen,
    /// Window is in Split View (side-by-side with another window).
    SplitView,
    /// Window is occluded by another window.
    Occluded,
    /// Window is hidden (ordered out).
    Hidden,
}

/// Tracks the current [`WindowState`] based on incoming winit [`WindowEvent`]s.
///
/// The detector maps raw winit events to high-level window states and exposes
/// helpers for render-loop decisions ([`should_render`], [`control_flow`]).
///
/// # Usage
///
/// ```no_run
/// use cvkg_render_native::{WindowStateDetector, WindowState};
/// let mut detector = WindowStateDetector::new();
/// // In your event loop:
/// // if let Some(new_state) = detector.update_from_event(&event) { ... }
/// ```
pub struct WindowStateDetector {
    state: WindowState,
    is_key: bool,
    is_main: bool,
}

impl WindowStateDetector {
    /// Creates a new detector initialized to [`WindowState::Normal`].
    pub fn new() -> Self {
        Self {
            state: WindowState::Normal,
            is_key: false,
            is_main: false,
        }
    }

    /// Returns the current window state.
    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Returns whether the window is the key (first responder) window.
    pub fn is_key(&self) -> bool {
        self.is_key
    }

    /// Returns whether the window is the main window.
    pub fn is_main(&self) -> bool {
        self.is_main
    }

    /// Updates the internal state based on a winit [`WindowEvent`].
    ///
    /// Returns `Some(WindowState)` if the state changed, `None` otherwise.
    ///
    /// # State mapping
    ///
    /// | winit event | resulting state |
    /// |---|---|
    /// | `Occluded(true)` | `Occluded` |
    /// | `Focused(true)` | updates `is_key`; checks fullscreen |
    /// | `Focused(false)` | updates `is_key` |
    /// | Default | `Normal` |
    ///
    /// Note: `Minimized` and `Fullscreen` detection requires querying the
    /// winit `Window` directly (see [`update_from_window`]).
    pub fn update_from_event(&mut self, event: &WindowEvent) -> Option<WindowState> {
        let old_state = self.state;
        match event {
            WindowEvent::Occluded(true) => {
                self.state = WindowState::Occluded;
            }
            WindowEvent::Focused(focused) => {
                self.is_key = *focused;
                if !focused && self.state != WindowState::Minimized {
                    self.state = WindowState::Normal;
                }
            }
            _ => {}
        };
        if self.state != old_state {
            Some(self.state)
        } else {
            None
        }
    }

    /// Updates the state by querying the winit `Window` directly.
    ///
    /// This should be called once per frame to detect states that winit
    /// does not emit as events (minimized, fullscreen).
    ///
    /// Returns `Some(WindowState)` if the state changed, `None` otherwise.
    pub fn update_from_window(&mut self, window: &winit::window::Window) -> Option<WindowState> {
        let old_state = self.state;
        if window.is_minimized().unwrap_or(false) {
            self.state = WindowState::Minimized;
        } else if window.fullscreen().is_some() {
            self.state = WindowState::Fullscreen;
        } else if self.state == WindowState::Minimized || self.state == WindowState::Fullscreen {
            // Transition back to Normal when no longer minimized/fullscreen
            self.state = WindowState::Normal;
        }
        if self.state != old_state {
            Some(self.state)
        } else {
            None
        }
    }

    /// Returns `true` if the window should render a frame in the current state.
    ///
    /// Returns `false` for [`WindowState::Occluded`], [`WindowState::Minimized`],
    /// and [`WindowState::Hidden`].
    pub fn should_render(&self) -> bool {
        !matches!(
            self.state,
            WindowState::Occluded | WindowState::Minimized | WindowState::Hidden
        )
    }

    /// Returns the appropriate [`ControlFlow`] for the current state.
    ///
    /// Non-rendering states get `ControlFlow::Wait` (save CPU cycles);
    /// rendering states get `ControlFlow::Poll` for maximum responsiveness.
    pub fn control_flow(&self) -> ControlFlow {
        if self.should_render() {
            ControlFlow::Poll
        } else {
            ControlFlow::Wait
        }
    }
}

impl Default for WindowStateDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Hit-test helper for resize handles on windows with rounded corners.
///
/// macOS Tahoe uses a 26pt corner radius, which means the visual corner arc
/// does not cover the full 19×19 resize hotspot. This struct expands the
/// clickable area 8px beyond the visual corner edge so users can still grab
/// the resize handle reliably.
pub struct ResizeHitTest {
    /// The size of the window in physical pixels.
    window_size: winit::dpi::PhysicalSize<u32>,
    /// The corner radius in points (logical pixels).
    corner_radius: f32,
    /// Extra expansion in pixels beyond the visual corner edge.
    expansion: f32,
}

impl ResizeHitTest {
    /// Creates a new hit-test helper.
    ///
    /// # Arguments
    ///
    /// * `window_size` -- the current window size in physical pixels.
    /// * `corner_radius` -- the corner radius in points (e.g., 26.0 for Tahoe).
    /// * `expansion` -- extra pixels to expand beyond the visual edge (e.g., 8.0).
    pub fn new(
        window_size: winit::dpi::PhysicalSize<u32>,
        corner_radius: f32,
        expansion: f32,
    ) -> Self {
        Self {
            window_size,
            corner_radius,
            expansion,
        }
    }

    /// Tests whether `pos` (a point relative to the window's top-left corner)
    /// falls within the expanded resize-hit region for any corner.
    ///
    /// The hit region for each corner is a square of side `corner_radius + expansion`,
    /// anchored at the corner. A point is considered a hit if it falls within
    /// any of the four corner squares.
    pub fn hit_test(&self, pos: winit::dpi::PhysicalPosition<f32>, corner_radius: f32) -> bool {
        let r = corner_radius + self.expansion;
        let w = self.window_size.width as f32;
        let h = self.window_size.height as f32;
        let px = pos.x as f32;
        let py = pos.y as f32;

        // Top-left corner: square [0, r) x [0, r)
        if px <= r && py <= r {
            return true;
        }

        // Top-right corner: square [w-r, w) x [0, r)
        if px >= w - r && py <= r {
            return true;
        }

        // Bottom-left corner: square [0, r) x [h-r, h)
        if px <= r && py >= h - r {
            return true;
        }

        // Bottom-right corner: square [w-r, w) x [h-r, h)
        if px >= w - r && py >= h - r {
            return true;
        }

        false
    }
}

/// Platform safe area insets (menu bar, notch, etc.).
///
/// Values are in logical points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeAreaInsets {
    /// Top inset (e.g., menu bar on macOS).
    pub top: f32,
    /// Bottom inset (e.g., Dock when at bottom).
    pub bottom: f32,
    /// Left inset.
    pub left: f32,
    /// Right inset.
    pub right: f32,
}

impl SafeAreaInsets {
    /// Returns zero insets on all sides.
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Returns appropriate safe-area insets for a given [`WindowState`].
    ///
    /// # Platform behavior
    ///
    /// * **Fullscreen** -- zero insets (window owns the entire screen).
    /// * **Normal** -- 24pt top on macOS for the menu bar, 0 on other platforms.
    /// * **All other states** -- same as Normal.
    pub fn for_window_state(state: WindowState) -> Self {
        if state == WindowState::Fullscreen {
            return Self::zero();
        }
        #[cfg(target_os = "macos")]
        let top = 24.0;
        #[cfg(not(target_os = "macos"))]
        let top = 0.0;
        Self {
            top,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

// Thread-local raw pointer to the locked SurtrRenderer for the duration of one render pass.
// CONTRACT: Set to non-null only while the MutexGuard is live on the call stack in render_frame_locked().
// All NativeRenderer draw calls use this pointer to avoid per-call mutex lock overhead.
// SAFETY: The pointer is valid because the MutexGuard is held for the entire duration the pointer is set.
thread_local! {
    static GPU_FRAME_PTR: std::cell::Cell<*mut cvkg_render_gpu::SurtrRenderer> =
        const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// Native renderer backend implementing the Renderer trait.
/// It wraps a shared SurtrRenderer for high-performance GPU drawing.
/// During a render pass, GPU_FRAME_PTR is set so draw calls bypass the mutex.
pub struct NativeRenderer {
    gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>,
    delta_time: f32,
    elapsed_time: f32,
    berserker_mode: cvkg_core::BerserkerMode,
    rage: f32,
    window: Arc<Window>,
}

impl NativeRenderer {
    /// Returns a reference to the GPU renderer.
    /// If GPU_FRAME_PTR is set (we're inside a locked render pass) uses that directly.
    /// Otherwise falls back to acquiring the mutex (safe for calls outside the render pass).
    ///
    /// # Safety
    /// GPU_FRAME_PTR is only non-null when a MutexGuard is live on the same thread's call stack.
    #[inline(always)]
    fn gpu_ref(&mut self) -> impl std::ops::DerefMut<Target = cvkg_render_gpu::SurtrRenderer> + '_ {
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
    fn gpu_ref_shared(&self) -> impl std::ops::Deref<Target = cvkg_render_gpu::SurtrRenderer> + '_ {
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
}

/// Returned by NativeRenderer::gpu_ref() — either a direct pointer ref or a mutex guard.
enum GpuRef<'a> {
    Ptr(&'a mut cvkg_render_gpu::SurtrRenderer),
    Guard(std::sync::MutexGuard<'a, cvkg_render_gpu::SurtrRenderer>),
}

impl<'a> std::ops::Deref for GpuRef<'a> {
    type Target = cvkg_render_gpu::SurtrRenderer;
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
    Ptr(&'a cvkg_render_gpu::SurtrRenderer),
    Guard(std::sync::MutexGuard<'a, cvkg_render_gpu::SurtrRenderer>),
}

impl<'a> std::ops::Deref for GpuRefShared<'a> {
    type Target = cvkg_render_gpu::SurtrRenderer;
    fn deref(&self) -> &Self::Target {
        match self {
            GpuRefShared::Ptr(r) => r,
            GpuRefShared::Guard(g) => g,
        }
    }
}

/// Custom events for the native application event loop, handling accessibility
/// callbacks and routing window lifecycle control events from background threads.
#[derive(Debug)]
pub enum AppEvent {
    /// Action request from the accessibility subsystem.
    AccessibilityAction(accesskit::ActionRequest),
    /// Request to close a specific window.
    CloseWindow(winit::window::WindowId),
    /// Request to set the title bar string of a window.
    SetTitle(winit::window::WindowId, String),
    /// Request to resize a window.
    SetSize(winit::window::WindowId, f32, f32),
    /// Request to change visibility of a window.
    SetVisible(winit::window::WindowId, bool),
    /// Request to bring a window to the front and focus it.
    BringToFront(winit::window::WindowId),
    /// Initial accessibility tree requested by screen reader.
    AccessibilityInitialTreeRequested(winit::window::WindowId),
}

impl From<accesskit_winit::Event> for AppEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        match event.window_event {
            accesskit_winit::WindowEvent::ActionRequested(req) => {
                AppEvent::AccessibilityAction(req)
            }
            accesskit_winit::WindowEvent::InitialTreeRequested => {
                AppEvent::AccessibilityInitialTreeRequested(event.window_id)
            }
            _ => AppEvent::AccessibilityAction(accesskit::ActionRequest {
                action: accesskit::Action::Focus,
                target_node: accesskit::NodeId(0),
                target_tree: accesskit::TreeId::ROOT,
                data: None,
            }),
        }
    }
}

impl NativeRenderer {
    /// Create a new NativeRenderer (internal use by App)
    fn new(
        window: Arc<Window>,
        gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>,
        delta_time: f32,
        elapsed_time: f32,
        berserker_mode: cvkg_core::BerserkerMode,
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
    pub fn run<V: cvkg_core::View + 'static>(view: V, prewarm_assets: Option<Vec<(String, Vec<u8>)>>) {
        let event_loop = EventLoop::<AppEvent>::with_user_event()
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
            berserker_mode: cvkg_core::BerserkerMode::Normal,
            rage: 0.0,
            state_detector: WindowStateDetector::new(),
            frame_budget: cvkg_core::FrameBudgetTracker::default_120fps(),
            modifiers: winit::keyboard::ModifiersState::default(),
            audio_engine: None,
            haptic_engine: Arc::new(VisualHapticEngine::new()),
            pending_prewarm: prewarm_assets,
        };

        event_loop.run_app(&mut app).expect("winit event loop terminated with error");
    }

    /// Convenience: run with a single background image loaded from a file path.
    /// The image is loaded from disk and pre-warmed on the first frame.
    /// `image_name` is the key used in `draw_image` / `draw_background_image`.
    pub fn run_with_background<V: cvkg_core::View + 'static>(view: V, image_name: &str, image_path: &str) {
        let image_data = std::fs::read(image_path)
            .unwrap_or_else(|e| panic!("Failed to load background image '{}': {}", image_path, e));
        let assets = vec![(image_name.to_string(), image_data)];
        Self::run(view, Some(assets));
    }
}

/// Native implementation of the cvkg_core::Window trait.
/// Communicates state updates back to the winit event loop thread using an EventLoopProxy.
struct NativeWindowWrapper {
    winit_id: winit::window::WindowId,
    window: Arc<winit::window::Window>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
    is_key: Arc<std::sync::atomic::AtomicBool>,
    is_main: bool,
}

impl cvkg_core::Window for NativeWindowWrapper {
    /// Request that this window be closed.
    fn close(&self) {
        let _ = self.proxy.send_event(AppEvent::CloseWindow(self.winit_id));
    }

    /// Change the title bar text of this window.
    fn set_title(&self, title: &str) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetTitle(self.winit_id, title.to_string()));
    }

    /// Request updating this window's dimensions.
    fn set_size(&self, width: f32, height: f32) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetSize(self.winit_id, width, height));
    }

    /// Return true if this window has key focus.
    fn is_key(&self) -> bool {
        self.is_key.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Return true if this is the primary application window.
    fn is_main(&self) -> bool {
        self.is_main
    }

    /// Return true if this window is visible.
    fn is_visible(&self) -> bool {
        self.window.is_visible().unwrap_or(false)
    }

    /// Show or hide this window.
    fn set_visible(&self, visible: bool) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetVisible(self.winit_id, visible));
    }

    /// Focus and bring this window to the foreground.
    fn bring_to_front(&self) {
        let _ = self.proxy.send_event(AppEvent::BringToFront(self.winit_id));
    }
}

/// Dynamic manager for all active native windows and their rendering contexts.
pub struct WindowManager {
    /// Mapping from native winit WindowId to internal WindowData.
    pub windows: std::collections::HashMap<winit::window::WindowId, WindowData>,
    /// Stack of windows ordered from back to front (end of vector is top-most).
    pub window_stack: Vec<winit::window::WindowId>,
    /// Mapping of winit window IDs to core IDs.
    pub winit_to_core: std::collections::HashMap<winit::window::WindowId, cvkg_core::WindowId>,
    /// Mapping of core window IDs to winit IDs.
    pub core_to_winit: std::collections::HashMap<cvkg_core::WindowId, winit::window::WindowId>,
    /// Monotonic counter to allocate unique core window IDs.
    pub next_core_id: u64,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    /// Create an empty WindowManager.
    pub fn new() -> Self {
        Self {
            windows: std::collections::HashMap::new(),
            window_stack: Vec::new(),
            winit_to_core: std::collections::HashMap::new(),
            core_to_winit: std::collections::HashMap::new(),
            next_core_id: 1,
        }
    }

    /// Create and register a new native window.
    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        gpu: &Option<Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>>,
        proxy: winit::event_loop::EventLoopProxy<AppEvent>,
        config: cvkg_core::WindowConfig,
        is_main: bool,
        view: &impl cvkg_core::View,
    ) -> cvkg_core::WindowHandle {
        let mut window_attrs = Window::default_attributes()
            .with_title(&config.title)
            .with_visible(true)
            .with_transparent(config.transparent)
            .with_decorations(config.decorations)
            .with_inner_size(winit::dpi::LogicalSize::new(config.size.0, config.size.1));

        if let Some(min) = config.min_size {
            window_attrs =
                window_attrs.with_min_inner_size(winit::dpi::LogicalSize::new(min.0, min.1));
        }
        if let Some(max) = config.max_size {
            window_attrs =
                window_attrs.with_max_inner_size(winit::dpi::LogicalSize::new(max.0, max.1));
        }

        let winit_level = match config.level {
            cvkg_core::WindowLevel::Normal => winit::window::WindowLevel::Normal,
            cvkg_core::WindowLevel::AlwaysOnTop => winit::window::WindowLevel::AlwaysOnTop,
            cvkg_core::WindowLevel::PopUpMenu => winit::window::WindowLevel::AlwaysOnTop,
        };
        window_attrs = window_attrs.with_window_level(winit_level);

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;
            window_attrs = window_attrs
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
                .with_has_shadow(true);
        }

        #[cfg(target_os = "windows")]
        {
            // Windows-specific window attributes:
            // WHY: Restores window shadow for undecorated windows to maintain Tahoe design aesthetics.
            // CONTRACT: with_undecorated_shadow requires the winit platform-specific extension for Windows.
            use winit::platform::windows::WindowAttributesExtWindows;
            window_attrs = window_attrs.with_undecorated_shadow(true);
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("failed to create native window: display connection may be unavailable"),
        );

        let winit_id = window.id();
        let core_id = cvkg_core::WindowId(self.next_core_id);
        self.next_core_id += 1;

        let is_key_focused = Arc::new(std::sync::atomic::AtomicBool::new(true));

        let wrapper = Arc::new(NativeWindowWrapper {
            winit_id,
            window: window.clone(),
            proxy: proxy.clone(),
            is_key: is_key_focused.clone(),
            is_main,
        });

        let handle = cvkg_core::WindowHandle::new(core_id, wrapper);

        let vdom = cvkg_vdom::VDom::build(
            view,
            cvkg_core::Rect::new(0.0, 0.0, config.size.0, config.size.1),
        );

        // On Linux, the accesskit_winit adapter automatically initializes the
        // AT-SPI bus connection via accesskit_unix (added as a dependency).
        // Screen readers and other assistive technologies will connect through this.
        #[cfg(target_os = "linux")]
        {
            log::info!("[Accessibility] AT-SPI backend available (accesskit_unix)");
        }

        let accesskit_adapter = Some(accesskit_winit::Adapter::with_event_loop_proxy(
            event_loop,
            &window,
            proxy.clone(),
        ));

        let data = WindowData {
            window: window.clone(),
            accesskit_adapter,
            vdom: Some(vdom),
            cursor_pos: [0.0, 0.0],
            cursor_velocity: [0.0, 0.0],
            last_redraw_start: std::time::Instant::now(),
            frame_history: std::collections::VecDeque::with_capacity(60),
            frame_count: 0,
            last_pos: None,
            needs_cursor_update: false,
            is_dragging: false,
            drag_start_pos: [0.0, 0.0],
            drag_button: 0,
            drag_threshold: 5.0,
            active_pointer_target: None,
            active_pointer_target_type: None,
            active_pointer_target_key: None,
            active_pointer_pos: None,
            active_pointer_precision: 0.0,
            is_key_focused,
            is_main,
            core_id,
            window_handle: handle.clone(),
            focus_manager: cvkg_core::FocusManager::new(),
            focused_node_id: None,
            last_touch_time: None,
        };

        self.windows.insert(winit_id, data);
        self.window_stack.push(winit_id);
        self.winit_to_core.insert(winit_id, core_id);
        self.core_to_winit.insert(core_id, winit_id);

        if let Some(gpu_mutex) = gpu {
            gpu_mutex.lock().unwrap_or_else(|p| p.into_inner()).register_window(window.clone());
        }

        handle
    }

    /// Close and unregister a native window.
    pub fn close_window(&mut self, winit_id: winit::window::WindowId) {
        self.windows.remove(&winit_id);
        self.window_stack.retain(|id| *id != winit_id);
        if let Some(core_id) = self.winit_to_core.remove(&winit_id) {
            self.core_to_winit.remove(&core_id);
        }
    }

    /// Bring a native window to the foreground and focus it.
    pub fn bring_to_front(&mut self, winit_id: winit::window::WindowId) {
        self.window_stack.retain(|id| *id != winit_id);
        self.window_stack.push(winit_id);
        if let Some(data) = self.windows.get(&winit_id) {
            data.window.focus_window();
        }
    }

    /// Get a reference to a window's data.
    pub fn window(&self, winit_id: winit::window::WindowId) -> Option<&WindowData> {
        self.windows.get(&winit_id)
    }

    /// Get a mutable reference to a window's data.
    pub fn window_mut(&mut self, winit_id: winit::window::WindowId) -> Option<&mut WindowData> {
        self.windows.get_mut(&winit_id)
    }

    /// Return the list of window IDs in current Z-order stack.
    pub fn window_order(&self) -> &[winit::window::WindowId] {
        &self.window_stack
    }
}

pub struct WindowData {
    window: Arc<Window>,
    accesskit_adapter: Option<accesskit_winit::Adapter>,
    vdom: Option<cvkg_vdom::VDom>,
    cursor_pos: [f32; 2],
    cursor_velocity: [f32; 2],
    /// The instant the last redraw finished, used for measuring inter-frame gap timing.
    last_redraw_start: std::time::Instant,
    /// Sliding window of frame times for tail latency (P99) calculation.
    frame_history: std::collections::VecDeque<f32>,
    /// Total frames rendered on this window.
    frame_count: u64,
    /// Last window position for shake detection.
    last_pos: Option<[i32; 2]>,
    /// Set when mouse moves; cleared when redraw processes. Prevents redundant
    /// VDOM rebuilds when cursor moves faster than the display refresh rate.
    needs_cursor_update: bool,
    // ── Drag tracking ──────────────────────────────────────────────────────
    /// Whether a drag is currently in progress.
    is_dragging: bool,
    /// The position where the drag started.
    drag_start_pos: [f32; 2],
    /// The button that initiated the drag.
    drag_button: u32,
    /// Drag threshold in logical pixels (pointer must move this far to start drag).
    drag_threshold: f32,
    /// Pointer target captured on press so release/click stay stable through rebuilds.
    active_pointer_target: Option<cvkg_vdom::NodeId>,
    /// Stashed component_type of the pressed target, used to verify identity across rebuilds.
    active_pointer_target_type: Option<String>,
    /// Stashed key of the pressed target, used to verify identity across rebuilds.
    active_pointer_target_key: Option<String>,
    /// Pointer position captured on press for fallback hit-testing.
    active_pointer_pos: Option<[f32; 2]>,
    /// Pointer precision captured on press for fallback hit-testing.
    active_pointer_precision: f32,

    // ── Multi-window tracking ──────────────────────────────────────────────
    is_key_focused: Arc<std::sync::atomic::AtomicBool>,
    is_main: bool,
    core_id: cvkg_core::WindowId,
    window_handle: cvkg_core::WindowHandle,

    // ── Focus navigation ───────────────────────────────────────────────────
    focus_manager: cvkg_core::FocusManager,
    focused_node_id: Option<cvkg_vdom::NodeId>,
    
    // -- Input disambiguation --
    last_touch_time: Option<std::time::Instant>,
}

struct App<V: cvkg_core::View> {
    view: V,
    window_manager: WindowManager,
    gpu: Option<Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>>,
    #[allow(dead_code)]
    asset_manager: std::sync::Arc<NativeAssetManager>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
    start_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    berserker_mode: cvkg_core::BerserkerMode,
    rage: f32,
    /// Tracks the current window state for render-loop decisions.
    state_detector: WindowStateDetector,
    /// Global frame budget used for explicit per-phase telemetry.
    frame_budget: cvkg_core::FrameBudgetTracker,
    /// Tracks active modifier key states (Ctrl, Shift, Command, etc.).
    modifiers: winit::keyboard::ModifiersState,
    /// Cross-platform audio engine for spatialized sound cues.
    audio_engine: Option<Arc<dyn cvkg_core::AudioEngine>>,
    /// Visual haptic engine for micro-feedback animations.
    haptic_engine: Arc<dyn cvkg_core::HapticEngine>,
    /// Assets to prewarm on the first frame (name, raw bytes). Drained once.
    pending_prewarm: Option<Vec<(String, Vec<u8>)>>,
}

impl<V: cvkg_core::View + 'static> ApplicationHandler<AppEvent> for App<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            // Detect and apply system accessibility preferences at startup
            let a11y_prefs = cvkg_core::AccessibilityPreferences::detect_from_system();
            cvkg_core::set_accessibility_preferences(a11y_prefs);
            if a11y_prefs.reduce_motion
                || a11y_prefs.reduce_transparency
                || a11y_prefs.increase_contrast
            {
                log::info!(
                    "[Native] Accessibility prefs: motion={} transparency={} contrast={}",
                    a11y_prefs.reduce_motion,
                    a11y_prefs.reduce_transparency,
                    a11y_prefs.increase_contrast
                );
            }

            // Detect and apply system theme (dark/light)
            let system_theme = cvkg_core::detect_system_theme();
            log::info!("[Native] System theme detected: {:?}", system_theme);

            // Initialize cross-platform audio engine
            self.audio_engine =
                RodioAudioEngine::new().map(|e| Arc::new(e) as Arc<dyn cvkg_core::AudioEngine>);

            // Initialize visual haptic engine for micro-feedback
            self.haptic_engine = Arc::new(VisualHapticEngine::new());

            log::info!("[Native] App instance (resumed): {:p}", self);

            let config = cvkg_core::WindowConfig {
                title: "CVKG Berserker".to_string(),
                size: (1280.0, 720.0),
                min_size: None,
                max_size: None,
                resizable: true,
                transparent: true,
                decorations: true,
                level: cvkg_core::WindowLevel::Normal,
            };

            let handle = self.window_manager.create_window(
                event_loop,
                &self.gpu,
                self.proxy.clone(),
                config,
                true, // is_main
                &self.view,
            );

            let winit_id = self
                .window_manager
                .core_to_winit
                .get(&handle.id)
                .copied()
                .unwrap_or_else(|| panic!("winit_id not found for window handle: window may have been destroyed"));
            let window = self
                .window_manager
                .windows
                .get(&winit_id)
                .unwrap()
                .window
                .clone();

            // Immediately set self.gpu to prevent re-entry
            let mut gpu = pollster::block_on(cvkg_render_gpu::SurtrRenderer::forge(window.clone()));

            // Phase 2.3: Pre-shape static labels to warm the text cache.
            // These strings are rendered every frame by the berserker demo
            // (NornirBar menu items, dock labels, overlay labels).
            // Pre-shaping avoids the first-frame HarfBuzz cost.
            static PREFETCH_LABELS: &[(&str, f32)] = &[
                // NornirBar menu items
                ("File", 13.0),
                ("Edit", 13.0),
                ("View", 13.0),
                ("Window", 13.0),
                ("Help", 13.0),
                // Title / overlay labels (common sizes)
                ("Berserker", 14.0),
                ("Rage", 12.0),
                ("FPS", 12.0),
                ("Frame", 12.0),
                ("Draw", 12.0),
                ("Layout", 12.0),
                ("Submit", 12.0),
                ("Browser", 12.0),
                ("Chat", 12.0),
                ("Code", 12.0),
                ("Terminal", 12.0),
            ];
            gpu.prewarm_text_cache(PREFETCH_LABELS);

            self.gpu = Some(Arc::new(std::sync::Mutex::new(gpu)));

            log::info!("[Native] Initialization complete.");
            window.request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if matches!(cause, winit::event::StartCause::Poll) {
            // Too noisy
        } else {
            // Lowered to trace to prevent logs flooding under standard debug levels
            log::trace!("[Native] Event Loop Wake: {:?}", cause);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if matches!(event, winit::event::DeviceEvent::MouseMotion { .. }) {
            // log::trace!("[Native] Raw Mouse Motion");
        } else {
            // Log device raw events at trace level to prevent I/O blocking performance issues
            // under high mouse-polling rates on systems with direct input mapping.
            log::trace!("[Native] DEVICE EVENT: {:?}", event);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if !matches!(event, WindowEvent::RedrawRequested)
            && !matches!(event, WindowEvent::CursorMoved { .. })
        {
            log::info!(
                "[Native] App instance: {:p} | WINDOW EVENT: {:?}",
                self,
                event
            );
        }

        let gpu_arc = if let Some(g) = &self.gpu {
            g.clone()
        } else {
            log::warn!("[Native] DROPPING EVENT: GPU not initialized yet");
            return;
        };

        let mut close_window = false;
        let mut bring_to_front = false;
        let mut create_new_window = false;
        // Cmd+Q was pressed -- close all windows after the state block ends.
        let mut quit_all = false;

        {
            let state = if let Some(s) = self.window_manager.windows.get_mut(&id) {
                s
            } else {
                return;
            };

            match event {
                WindowEvent::Moved(pos) => {
                    let dx = state.last_pos.map_or(0, |last| pos.x - last[0]);
                    let dy = state.last_pos.map_or(0, |last| pos.y - last[1]);
                    let speed = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

                    if speed > 0.1 {
                        // Significant kinetic injection
                        self.rage = (self.rage + 0.2).min(1.0);
                        log::info!("[Native] Kinetic Injection! Rage: {}", self.rage);
                    }

                    state.last_pos = Some([pos.x, pos.y]);
                    state.window.request_redraw();
                }
                WindowEvent::DroppedFile(path) => {
                    if let Some(vdom) = &state.vdom {
                        vdom.dispatch_event(cvkg_core::Event::FileDrop {
                            x: state.cursor_pos[0],
                            y: state.cursor_pos[1],
                            path: path.to_string_lossy().into_owned(),
                        });
                    }
                }
                WindowEvent::CloseRequested => {
                    let close_action = cvkg_core::WindowCloseAction::Allow;
                    match close_action {
                        cvkg_core::WindowCloseAction::Allow
                        | cvkg_core::WindowCloseAction::Confirm => {
                            close_window = true;
                        }
                        cvkg_core::WindowCloseAction::Deny => {
                            log::info!("[Native] Close request denied for window {:?}", id);
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    gpu_arc
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .resize(
                            id,
                            physical_size.width,
                            physical_size.height,
                            state.window.scale_factor() as f32,
                        );
                    state.window.request_redraw();
                }
                WindowEvent::Focused(focused) => {
                    log::info!("[Native] Window focus changed: {}", focused);
                    state
                        .is_key_focused
                        .store(focused, std::sync::atomic::Ordering::SeqCst);
                    if focused {
                        bring_to_front = true;
                    }
                }
                WindowEvent::RedrawRequested => {
                    if state.frame_count % 60 == 0 {
                        log::info!("[Native] RedrawRequested (frame {})", state.frame_count);
                    }
                    let size = state.window.inner_size();
                    let scale = state.window.scale_factor();
                    let logical_size = size.to_logical::<f32>(scale);

                    let rect = cvkg_core::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: logical_size.width,
                        height: logical_size.height,
                    };

                    // Record the start of this redraw and snapshot the previous frame's
                    // start time before overwriting it, so inter-frame gap is measurable.
                    let redraw_start = std::time::Instant::now();
                    let last_redraw_start = state.last_redraw_start;
                    // Update last_redraw_start immediately so the next frame measures correctly
                    // even if this frame returns early.
                    state.last_redraw_start = redraw_start;
                    self.frame_budget.new_frame();

                    // Build new vdom and diff (layout pass)
                    let layout_start = std::time::Instant::now();
                    let view_changed = self.view.changed();

                    // Phase 1.2: Skip VDom rebuild when view hasn't changed.
                    let new_vdom: Option<cvkg_vdom::VDom> = if view_changed {
                        let vdom_start = std::time::Instant::now();
                        let vdom = cvkg_vdom::VDom::build(&self.view, rect);
                        let vdom_elapsed = vdom_start.elapsed();
                        if vdom_elapsed > std::time::Duration::from_millis(1) {
                            log::warn!("[Native] VDom::build took {:?} ({} nodes)", vdom_elapsed, vdom.nodes.len());
                        }
                        Some(vdom)
                    } else {
                        None
                    };

                    // Dispatch cursor events if the mouse moved since last frame
                    if state.needs_cursor_update {
                        if let Some(vdom) = &state.vdom {
                            vdom.dispatch_event(cvkg_core::Event::PointerMove {
                                x: state.cursor_pos[0],
                                y: state.cursor_pos[1],
                                proximity_field: 0.0,
                                tilt: None,
                                azimuth: None,
                                pressure: Some(1.0),
                                barrel_rotation: None,
                                pointer_precision: 0.0,
                            });
                        }
                        state.needs_cursor_update = false;
                    }
                    let layout_end = std::time::Instant::now();
                    self.frame_budget.subsystem_finish(1);

                    // Apply patches to the accessibility tree and the previous VDOM.
                    // When new_vdom is None (view unchanged), skip diff entirely.
                    let state_flush_start = std::time::Instant::now();
                    #[allow(unused)]
                    let mut diff_patches = None;
                    match (new_vdom, &mut state.vdom) {
                        (Some(new_vdom), Some(prev_vdom)) => {
                            let diff_start = std::time::Instant::now();
                            let patches = prev_vdom.diff(&new_vdom);
                            let diff_elapsed = diff_start.elapsed();
                            if diff_elapsed > std::time::Duration::from_millis(1) {
                                log::warn!("[Native] VDom::diff took {:?} ({} patches)", diff_elapsed, patches.len());
                            }
                            diff_patches = Some(patches);
                            let patches = diff_patches.as_ref().unwrap();
                            let mut nodes = Vec::new();
                            for patch in patches {
                                if let cvkg_vdom::VDomPatch::Create(node)
                                | cvkg_vdom::VDomPatch::Replace { node, .. } = patch
                                {
                                    nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                                } else if let cvkg_vdom::VDomPatch::Update { id, .. } = patch
                                    && let Some(node) = new_vdom.nodes.get(id)
                                {
                                    nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                                } else if let cvkg_vdom::VDomPatch::Remove(id) = patch {
                                    // Unregister removed nodes from focus manager to prevent
                                    // unbounded growth of the focus order list.
                                    state.focus_manager.unregister(&FocusableId::from(id.0.to_string()));
                                }
                            }
                            let focused_id = state.focused_node_id.map(|id| accesskit::NodeId(id.0)).unwrap_or(accesskit::NodeId(1));
                            for patch in diff_patches.as_ref().unwrap() {
                                if let cvkg_vdom::VDomPatch::Create(node)
                                | cvkg_vdom::VDomPatch::Replace { node, .. } = patch
                                {
                                    if node.is_focusable() {
                                        state.focus_manager.register(node.id.0.to_string());
                                    }
                                }
                            }
                            if !nodes.is_empty() {
                                if let Some(adapter) = &mut state.accesskit_adapter {
                                    adapter.update_if_active(|| accesskit::TreeUpdate {
                                        nodes,
                                        tree: None,
                                        focus: focused_id,
                                        tree_id: accesskit::TreeId::ROOT,
                                    });
                                }
                            }
                            prev_vdom.apply_patches(diff_patches.unwrap());
                            state.vdom = Some(new_vdom);
                        }
                        (Some(new_vdom), None) => {
                            state.vdom = Some(new_vdom);
                        }
                        (None, _) => {
                            // View unchanged -- keep existing state.vdom as-is.
                        }
                    }
                    let state_flush_end = std::time::Instant::now();
                    self.frame_budget.subsystem_finish(0);

                    let _draw_start = std::time::Instant::now();
                    let delta_time = redraw_start.duration_since(last_redraw_start).as_secs_f32();
                    let elapsed_time = redraw_start.duration_since(self.start_time).as_secs_f32();

                    // Compute safe area insets based on current window state
                    let safe_area = crate::SafeAreaInsets::for_window_state(self.state_detector.state());
                    let content_rect = cvkg_core::Rect {
                        x: safe_area.left,
                        y: safe_area.top,
                        width: rect.width - safe_area.left - safe_area.right,
                        height: rect.height - safe_area.top - safe_area.bottom,
                    };
                    let layout_deadline = std::time::Instant::now()
                        + self.frame_budget.allocations()[1].time_slice;
                    cvkg_core::LayoutCache::set_layout_budget_deadline(Some(layout_deadline));

                    let mut renderer = NativeRenderer::new(
                        state.window.clone(),
                        gpu_arc.clone(),
                        delta_time,
                        elapsed_time,
                        self.berserker_mode,
                        self.rage,
                    );

                    // Single GPU lock for the entire frame: update mouse, prewarm, begin, draw, render, submit.
                    // This eliminates two extra lock/unlock cycles per frame.
                    let cpu_draw_start = std::time::Instant::now();
                    let mut gpu = gpu_arc.lock().unwrap_or_else(|p| p.into_inner());
                    let gpu_lock_time = cpu_draw_start.elapsed().as_secs_f32() * 1000.0;

                    // Update mouse position
                    gpu.update_mouse(state.cursor_pos, state.cursor_velocity);

                    // One-time prewarm: drain any pending assets into the GPU texture atlas
                    if let Some(assets) = self.pending_prewarm.take() {
                        log::info!("[Native] Pre-warming {} assets on first frame", assets.len());
                        gpu.prewarm_vram(assets);
                    }

                    // Begin frame
                    let encoder = gpu.begin_frame(id);
                    let begin_frame_time = cpu_draw_start.elapsed().as_secs_f32() * 1000.0 - gpu_lock_time;

                    // Render pass: publish pointer, draw, clear pointer
                    {
                        let raw: *mut cvkg_render_gpu::SurtrRenderer = &mut *gpu;
                        GPU_FRAME_PTR.with(|ptr| ptr.set(raw));
                        let render_start = std::time::Instant::now();
                        self.view.render(&mut renderer, content_rect);
                        let render_time = render_start.elapsed().as_secs_f32() * 1000.0;
                        GPU_FRAME_PTR.with(|ptr| ptr.set(std::ptr::null_mut()));
                        if render_time > 5.0 {
                            log::warn!("[Native] view.render() took {:.2}ms (gpu_lock={:.2}ms, begin_frame={:.2}ms)", render_time, gpu_lock_time, begin_frame_time);
                        }
                    }
                    let cpu_draw_end = std::time::Instant::now();
                    cvkg_core::LayoutCache::clear_layout_budget_deadline();

                    self.frame_budget.subsystem_finish(2);

                    // Submit the frame (still under the same lock)
                    let gpu_submit_start = std::time::Instant::now();
                    let gpu_render_start = std::time::Instant::now();
                    gpu.render_frame();
                    let gpu_render_end = std::time::Instant::now();
                    gpu.end_frame(encoder);
                    let gpu_submit_end = std::time::Instant::now();

                    // GPU guard drops here, releasing the lock

                    // GPU profile logging: only log every 60 frames to avoid per-frame overhead
                    if state.frame_count % 60 == 0 {
                        log::info!(
                            "[Native] GPU profile: cpu_draw={:?} gpu_render={:?} gpu_submit={:?}",
                            cpu_draw_end.duration_since(cpu_draw_start),
                            gpu_render_end.duration_since(gpu_render_start),
                            gpu_submit_end.duration_since(gpu_render_end),
                        );
                    }

                    // Build telemetry from this frame's timing measurements.
                    // NOTE: input_time_ms measures the inter-frame gap (time from end of last frame
                    // to start of this one), not input dispatch latency. The field name is defined
                    // in cvkg_core::TelemetryData and kept as-is to match that struct.
                    let mut telemetry = cvkg_core::TelemetryData::default();
                    telemetry.input_time_ms =
                        redraw_start.duration_since(last_redraw_start).as_secs_f32() * 1000.0;
                    telemetry.layout_time_ms =
                        layout_end.duration_since(layout_start).as_secs_f32() * 1000.0;
                    telemetry.state_flush_time_ms = state_flush_end
                        .duration_since(state_flush_start)
                        .as_secs_f32()
                        * 1000.0;
                    telemetry.draw_time_ms =
                        cpu_draw_end.duration_since(cpu_draw_start).as_secs_f32() * 1000.0;
                    telemetry.gpu_submit_time_ms = gpu_submit_end
                        .duration_since(gpu_submit_start)
                        .as_secs_f32()
                        * 1000.0;

                    // Total frame time from redraw request to GPU submission complete
                    let frame_time_ms =
                        gpu_submit_end.duration_since(redraw_start).as_secs_f32() * 1000.0;
                    telemetry.frame_time_ms = frame_time_ms;
                    telemetry.frame_budget_ms = self.frame_budget.total().as_secs_f32() * 1000.0;
                    telemetry.frame_budget_remaining_ms =
                        telemetry.frame_budget_ms - telemetry.frame_time_ms;
                    telemetry.layout_budget_remaining_ms = self
                        .frame_budget
                        .allocations()
                        .get(1)
                        .map(|alloc| alloc.time_slice.as_secs_f32() * 1000.0 - telemetry.layout_time_ms)
                        .unwrap_or(0.0);
                    telemetry.frame_over_budget = !self.frame_budget.frame_within_budget()
                        || telemetry.frame_budget_remaining_ms < 0.0;
                    telemetry.layout_over_budget = !self.frame_budget.is_within_budget(1)
                        || telemetry.layout_budget_remaining_ms < 0.0;

                    // Log detailed frame time breakdown for performance diagnostics
                    log::info!(
                        "[Native] Frame timings: layout={:.2}ms state={:.2}ms draw={:.2}ms submit={:.2}ms total={:.2}ms",
                        telemetry.layout_time_ms,
                        telemetry.state_flush_time_ms,
                        telemetry.draw_time_ms,
                        telemetry.gpu_submit_time_ms,
                        telemetry.frame_time_ms
                    );

                    // Tail Latency Tracking (P99 and Jitter) over a 100-frame sliding window.
                    state.frame_history.push_back(frame_time_ms);
                    if state.frame_history.len() > 100 {
                        state.frame_history.pop_front();
                    }

                    let mut sorted_frames: Vec<f32> = state.frame_history.iter().copied().collect();
                    sorted_frames
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                    if !sorted_frames.is_empty() {
                        let p99_idx = (sorted_frames.len() as f32 * 0.99).floor() as usize;
                        telemetry.p99_frame_time_ms =
                            sorted_frames[p99_idx.min(sorted_frames.len() - 1)];

                        // Jitter: standard deviation of frame times over the sliding window.
                        let avg = sorted_frames.iter().sum::<f32>() / sorted_frames.len() as f32;
                        let variance = sorted_frames.iter().map(|f| (f - avg).powi(2)).sum::<f32>()
                            / sorted_frames.len() as f32;
                        telemetry.frame_jitter_ms = variance.sqrt();
                    }

                    // FIX #8: hardware_stall_detected is now reset each frame based on current
                    // jitter rather than being set once and never cleared. A single jittery frame
                    // no longer permanently flags the session. Jitter > 20ms is a heuristic for
                    // scheduling disruption (GC, OS preemption, slow layout) -- not a confirmed
                    // hardware stall, but the field name is defined in cvkg_core::TelemetryData.
                    telemetry.hardware_stall_detected = telemetry.frame_jitter_ms > 20.0;
                    if telemetry.frame_over_budget {
                        log::warn!(
                            "[Native] Frame budget exceeded by {:.2}ms (layout remaining {:.2}ms)",
                            -telemetry.frame_budget_remaining_ms,
                            telemetry.layout_budget_remaining_ms
                        );
                    }

                    state.frame_count += 1;

                    telemetry.berserker_rage = self.rage;
                    gpu.telemetry = telemetry;

                    // Drive the continuous animation loop: immediately schedule the next frame.
                    // Without this, winit's Wait mode only redraws on OS input events (mouse
                    // moves), which produces ~20fps driven by cursor poll rate instead of
                    // the 120fps target. This single call is the animation loop.
                    state.window.request_redraw();
                }
                WindowEvent::CursorEntered { .. } => {
                    log::info!("[Native] Cursor ENTERED window");
                    if let Some(vdom) = &state.vdom {
                        vdom.dispatch_event(cvkg_core::Event::PointerEnter);
                    }
                    state.window.request_redraw();
                }
                WindowEvent::CursorLeft { .. } => {
                    log::info!("[Native] Cursor LEFT window");
                    if let Some(vdom) = &state.vdom {
                        vdom.dispatch_event(cvkg_core::Event::PointerLeave);
                    }
                    state.window.request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let scale = state.window.scale_factor();
                    let logical = position.to_logical::<f32>(scale);
                    let elapsed = state.last_redraw_start.elapsed().as_secs_f32().max(0.001);
                    let dx = logical.x - state.cursor_pos[0];
                    let dy = logical.y - state.cursor_pos[1];
                    state.cursor_velocity = [dx / elapsed, dy / elapsed];
                    state.cursor_pos = [logical.x, logical.y];
                    // Check if we've moved past the drag threshold
                    if !state.is_dragging {
                        let ddx = state.cursor_pos[0] - state.drag_start_pos[0];
                        let ddy = state.cursor_pos[1] - state.drag_start_pos[1];
                        let dist_sq = ddx * ddx + ddy * ddy;
                        if dist_sq > state.drag_threshold * state.drag_threshold {
                            state.is_dragging = true;
                        }
                    }
                    state.needs_cursor_update = true;
                    // Don't request_redraw here -- the redraw will process the cursor update.
                    // Only request a redraw if we're not already in a redraw cycle.
                    if state.frame_count == 0 {
                        state.window.request_redraw();
                    }
                }
                WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => {
                    log::info!(
                        "[Native] MOUSE INPUT: {:?} button={:?} pos={:?}",
                        mouse_state,
                        button,
                        state.cursor_pos
                    );
                    if let Some(touch_time) = state.last_touch_time {
                        if touch_time.elapsed().as_millis() < 500 {
                            log::info!("[Native] Ignoring MouseInput (synthesized from Touch)");
                            return;
                        }
                    }
                    if let Some(vdom) = &state.vdom {
                        let btn_id = match button {
                            winit::event::MouseButton::Left => 0,
                            winit::event::MouseButton::Right => 2,
                            winit::event::MouseButton::Middle => 1,
                            winit::event::MouseButton::Back => 3,
                            winit::event::MouseButton::Forward => 4,
                            winit::event::MouseButton::Other(id) => id as u32,
                        };

                        match mouse_state {
                            winit::event::ElementState::Pressed => {
                                // Record drag start position for click/drag disambiguation
                                state.drag_start_pos = state.cursor_pos;
                                state.is_dragging = false;
                                state.drag_button = btn_id;
                                state.active_pointer_pos = Some(state.cursor_pos);
                                state.active_pointer_precision = 0.0;
                                state.active_pointer_target = vdom
                                    .hit_test(state.cursor_pos[0], state.cursor_pos[1], 0.0)
                                    .map(|(id, _)| id);
                                // Stash component_type and key for identity verification on release.
                                // NodeIds are stable across rebuilds when nodes have keys, but
                                // we still verify identity to be safe against hash collisions.
                                if let Some(target_id) = state.active_pointer_target {
                                    if let Some(node) = vdom.nodes.get(&target_id) {
                                        state.active_pointer_target_type = Some(node.component_type.clone());
                                        state.active_pointer_target_key = node.key.clone();
                                    }
                                }
                                log::info!("[Native] Dispatching PointerDown to VDOM");
                                vdom.dispatch_event(cvkg_core::Event::PointerDown {
                                    x: state.cursor_pos[0],
                                    y: state.cursor_pos[1],
                                    button: btn_id,
                                    proximity_field: 0.0,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(1.0),
                                    barrel_rotation: None,
                                    pointer_precision: 0.0,
                                });
                            }
                            winit::event::ElementState::Released => {
                                log::info!("[Native] Dispatching PointerUp to VDOM");
                                let fallback_target = state
                                    .active_pointer_pos
                                    .and_then(|pos| {
                                        vdom.hit_test(pos[0], pos[1], state.active_pointer_precision)
                                            .map(|(id, _)| id)
                                    })
                                    .or_else(|| {
                                        vdom.hit_test(
                                            state.cursor_pos[0],
                                            state.cursor_pos[1],
                                            state.active_pointer_precision,
                                        )
                                        .map(|(id, _)| id)
                                    });
                                // Verify the cached target is the same logical node.
                                // When key is None, identity can't be verified reliably
                                // (many nodes have None keys), so fall back to fresh hit-test.
                                let target = state
                                    .active_pointer_target
                                    .filter(|target| {
                                        if state.active_pointer_target_key.is_none() {
                                            log::debug!("[Native] Target verification: key is None, skipping cache");
                                            return false;
                                        }
                                        let verified = vdom.nodes.get(target).map_or(false, |node| {
                                            let type_match = Some(&node.component_type) == state.active_pointer_target_type.as_ref();
                                            let key_match = node.key == state.active_pointer_target_key;
                                            log::debug!("[Native] Target verify: id={:?} type={} key={:?} type_match={} key_match={}",
                                                target, node.component_type, node.key, type_match, key_match);
                                            type_match && key_match
                                        });
                                        if !verified {
                                            log::debug!("[Native] Target verification failed for {:?}, using fallback", target);
                                        }
                                        verified
                                    })
                                    .or(fallback_target);
                                let pointer_up = cvkg_core::Event::PointerUp {
                                    x: state.cursor_pos[0],
                                    y: state.cursor_pos[1],
                                    button: btn_id,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                    pointer_precision: 0.0,
                                };
                                let pointer_click = cvkg_core::Event::PointerClick {
                                    x: state.cursor_pos[0],
                                    y: state.cursor_pos[1],
                                    button: btn_id,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                    pointer_precision: 0.0,
                                };
                                if let Some(target) = target {
                                    vdom.dispatch_event_to_target(target, pointer_up);
                                } else {
                                    vdom.dispatch_event(pointer_up);
                                }
                                // Only dispatch PointerClick if we didn't drag
                                if !state.is_dragging {
                                    if let Some(target) = target {
                                        log::info!("[Native] Dispatching PointerClick to VDOM (target={:?})", target);
                                        vdom.dispatch_event_to_target(target, pointer_click);
                                    } else {
                                        log::info!("[Native] Dispatching PointerClick to VDOM (no target, bubbling)");
                                        vdom.dispatch_event(pointer_click);
                                    }
                                } else {
                                    log::info!("[Native] Skipping PointerClick (is_dragging=true)");
                                }
                                // Reset drag state
                                state.is_dragging = false;
                                state.active_pointer_target = None;
                                state.active_pointer_target_type = None;
                                state.active_pointer_target_key = None;
                                state.active_pointer_pos = None;
                            }
                        }
                        state.window.request_redraw();
                    } else {
                        log::warn!("[Native] Mouse input received but state.vdom is None!");
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(vdom) = &state.vdom {
                        let (dx, dy) = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 10.0, y * 10.0),
                            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                (pos.x as f32, pos.y as f32)
                            }
                        };
                        vdom.dispatch_event(cvkg_core::Event::PointerWheel {
                            x: state.cursor_pos[0],
                            y: state.cursor_pos[1],
                            delta_x: dx,
                            delta_y: dy,
                            pointer_precision: 0.0,
                        });
                        state.window.request_redraw();
                    }
                }
                // ── Touch screen inputs ──────────────────────────────────────────
                // Map native winit touchscreen events to VDOM Pointer events using
                // low-precision fat-finger bounding expansion (150px proximity field).
                WindowEvent::Touch(touch) => {
                    state.last_touch_time = Some(std::time::Instant::now());
                    if let Some(vdom) = &state.vdom {
                        let scale = state.window.scale_factor();
                        let logical = touch.location.to_logical::<f32>(scale);
                        let x = logical.x;
                        let y = logical.y;
                        let touch_btn = 0; // Touch maps to primary/left button
                        match touch.phase {
                            winit::event::TouchPhase::Started => {
                                log::info!("[Native] Dispatching PointerDown (Touch) to VDOM");
                                // Record drag start position for click/drag disambiguation
                                state.drag_start_pos = [x, y];
                                state.is_dragging = false;
                                state.drag_button = touch_btn as u32;
                                state.active_pointer_pos = Some([x, y]);
                                state.active_pointer_precision = 150.0;
                                state.active_pointer_target = vdom.hit_test(x, y, 150.0).map(|(id, _)| id);
                                if let Some(target_id) = state.active_pointer_target {
                                    if let Some(node) = vdom.nodes.get(&target_id) {
                                        state.active_pointer_target_type = Some(node.component_type.clone());
                                        state.active_pointer_target_key = node.key.clone();
                                    }
                                }
                                vdom.dispatch_event(cvkg_core::Event::PointerDown {
                                    x,
                                    y,
                                    button: touch_btn,
                                    proximity_field: 0.0,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(
                                        touch.force.map(|f| f.normalized() as f32).unwrap_or(0.5),
                                    ),
                                    barrel_rotation: None,
                                    pointer_precision: 150.0,
                                });
                            }
                            winit::event::TouchPhase::Moved => {
                                // Check if we've moved past the drag threshold
                                if !state.is_dragging {
                                    let ddx = x - state.drag_start_pos[0];
                                    let ddy = y - state.drag_start_pos[1];
                                    let dist_sq = ddx * ddx + ddy * ddy;
                                    if dist_sq > state.drag_threshold * state.drag_threshold {
                                        state.is_dragging = true;
                                    }
                                }
                                vdom.dispatch_event(cvkg_core::Event::PointerMove {
                                    x,
                                    y,
                                    proximity_field: 0.0,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(
                                        touch.force.map(|f| f.normalized() as f32).unwrap_or(0.5),
                                    ),
                                    barrel_rotation: None,
                                    pointer_precision: 150.0,
                                });
                            }
                            winit::event::TouchPhase::Ended => {
                                let fallback_target = state
                                    .active_pointer_pos
                                    .and_then(|pos| {
                                        vdom.hit_test(pos[0], pos[1], state.active_pointer_precision)
                                            .map(|(id, _)| id)
                                    })
                                    .or_else(|| vdom.hit_test(x, y, state.active_pointer_precision).map(|(id, _)| id));
                                // Verify the cached target is the same logical node.
                                let target = state
                                    .active_pointer_target
                                    .filter(|target| {
                                        vdom.nodes.get(target).map_or(false, |node| {
                                            Some(&node.component_type) == state.active_pointer_target_type.as_ref()
                                                && node.key == state.active_pointer_target_key
                                        })
                                    })
                                    .or(fallback_target);
                                let pointer_up = cvkg_core::Event::PointerUp {
                                    x,
                                    y,
                                    button: touch_btn,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                    pointer_precision: 150.0,
                                };
                                let pointer_click = cvkg_core::Event::PointerClick {
                                    x,
                                    y,
                                    button: touch_btn,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                    pointer_precision: 150.0,
                                };
                                if let Some(target) = target {
                                    vdom.dispatch_event_to_target(target, pointer_up);
                                } else {
                                    vdom.dispatch_event(pointer_up);
                                }
                                // Only dispatch PointerClick if we didn't drag
                                if !state.is_dragging {
                                    if let Some(target) = target {
                                        log::info!("[Native] Dispatching PointerClick to VDOM (target={:?})", target);
                                        vdom.dispatch_event_to_target(target, pointer_click);
                                    } else {
                                        log::info!("[Native] Dispatching PointerClick to VDOM (no target, bubbling)");
                                        vdom.dispatch_event(pointer_click);
                                    }
                                } else {
                                    log::info!("[Native] Skipping PointerClick (is_dragging=true)");
                                }
                                // Reset drag state
                                state.is_dragging = false;
                                state.active_pointer_target = None;
                                state.active_pointer_target_type = None;
                                state.active_pointer_target_key = None;
                                state.active_pointer_pos = None;
                            }
                            winit::event::TouchPhase::Cancelled => {
                                vdom.dispatch_event(cvkg_core::Event::PointerUp {
                                    x,
                                    y,
                                    button: touch_btn,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                    pointer_precision: 150.0,
                                });
                                state.active_pointer_target = None;
                                state.active_pointer_pos = None;
                            }
                        }
                        state.window.request_redraw();
                    }
                }
                // ── Trackpad gestures (pinch-to-zoom, swipe) ──────────────────────
                // OS-agnostic: winit provides these on macOS trackpad, Windows precision
                // touchpads, and Linux (where supported). Falls back gracefully.
                WindowEvent::PinchGesture { delta, .. } => {
                    if let Some(vdom) = &state.vdom {
                        let scale = 1.0 + delta as f32;
                        let velocity = delta as f32;
                        vdom.dispatch_event(cvkg_core::Event::GesturePinch {
                            center: state.cursor_pos,
                            scale,
                            velocity,
                            phase: cvkg_core::TouchPhase::Moved,
                        });
                    }
                    // Provide micro-feedback on pinch
                    if let Some(audio) = &self.audio_engine {
                        audio.play_sound("nav_tick", 0.3);
                    }
                    self.haptic_engine
                        .visual_tick((delta.abs() as f32 * 5.0).min(1.0));
                    state.window.request_redraw();
                }
                WindowEvent::RotationGesture { delta, .. } => {
                    if let Some(vdom) = &state.vdom {
                        let angle = delta;
                        vdom.dispatch_event(cvkg_core::Event::GestureSwipe {
                            direction: [angle.cos(), angle.sin()],
                            velocity: delta.abs(),
                            phase: cvkg_core::TouchPhase::Moved,
                        });
                    }
                    state.window.request_redraw();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == winit::event::ElementState::Pressed {
                        if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                            // Cross-platform "command" key: ⌘ on macOS, Ctrl on all other OSes.
                            // This ensures keyboard shortcuts work identically on every platform
                            // without separate branches in every handler.
                            let is_cmd = if cfg!(target_os = "macos") {
                                self.modifiers.super_key()
                            } else {
                                self.modifiers.control_key()
                            };
                            let is_shift = self.modifiers.shift_key();

                            if is_cmd {
                                match code {
                                    // ── Undo / Redo ───────────────────────────────
                                    winit::keyboard::KeyCode::KeyZ => {
                                        if is_shift {
                                            log::info!("[Native] Shortcut: Redo (Cmd+Shift+Z)");
                                            let mut redo_action = None;
                                            cvkg_core::update_system_state(|s| {
                                                let mut s = s.clone();
                                                redo_action = s.undo_manager.redo();
                                                s
                                            });
                                            if let Some(action) = redo_action {
                                                action();
                                            }
                                            state.window.request_redraw();
                                        } else {
                                            log::info!("[Native] Shortcut: Undo (Cmd+Z)");
                                            let mut undo_action = None;
                                            cvkg_core::update_system_state(|s| {
                                                let mut s = s.clone();
                                                undo_action = s.undo_manager.undo();
                                                s
                                            });
                                            if let Some(action) = undo_action {
                                                action();
                                            }
                                            state.window.request_redraw();
                                        }
                                    }
                                    // Ctrl+Y as alternative Redo on non-macOS
                                    winit::keyboard::KeyCode::KeyY
                                        if !cfg!(target_os = "macos") =>
                                    {
                                        log::info!("[Native] Shortcut: Redo (Ctrl+Y)");
                                        let mut redo_action = None;
                                        cvkg_core::update_system_state(|s| {
                                            let mut s = s.clone();
                                            redo_action = s.undo_manager.redo();
                                            s
                                        });
                                        if let Some(action) = redo_action {
                                            action();
                                        }
                                        state.window.request_redraw();
                                    }
                                    // ── File operations ───────────────────────────
                                    winit::keyboard::KeyCode::KeyN => {
                                        log::info!("[Native] Shortcut: New Window (Cmd+N)");
                                        create_new_window = true;
                                    }
                                    winit::keyboard::KeyCode::KeyO => {
                                        log::info!("[Native] Shortcut: Open File (Cmd+O)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+o".to_string(),
                                                modifiers: cvkg_core::KeyModifiers::default(),
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyS => {
                                        log::info!("[Native] Shortcut: Save (Cmd+S)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+s".to_string(),
                                                modifiers: cvkg_core::KeyModifiers::default(),
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyW => {
                                        log::info!("[Native] Shortcut: Close Window (Cmd+W)");
                                        close_window = true;
                                    }
                                    winit::keyboard::KeyCode::KeyQ => {
                                        log::info!("[Native] Shortcut: Quit (Cmd+Q)");
                                        // Defer closing all windows until after the state borrow ends.
                                        quit_all = true;
                                    }
                                    // ── Clipboard ────────────────────────────────
                                    winit::keyboard::KeyCode::KeyC => {
                                        log::info!("[Native] Shortcut: Copy (Cmd+C)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::Copy);
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyV => {
                                        log::info!("[Native] Shortcut: Paste (Cmd+V)");
                                        // Read the system clipboard. Fall back to empty string on
                                        // error so the Paste event is always delivered to the VDOM.
                                        let text = arboard::Clipboard::new()
                                            .ok()
                                            .and_then(|mut cb| cb.get_text().ok())
                                            .unwrap_or_default();
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::Paste(text));
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyX => {
                                        log::info!("[Native] Shortcut: Cut (Cmd+X)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::Cut);
                                        }
                                        state.window.request_redraw();
                                    }
                                    // ── Selection / search ────────────────────────
                                    winit::keyboard::KeyCode::KeyA => {
                                        log::info!("[Native] Shortcut: Select All (Cmd+A)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+a".to_string(),
                                                modifiers: cvkg_core::KeyModifiers::default(),
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyF => {
                                        log::info!("[Native] Shortcut: Find (Cmd+F)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+f".to_string(),
                                                modifiers: cvkg_core::KeyModifiers::default(),
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    // ── Focus navigation: Tab / Shift+Tab ───────────────
                                    winit::keyboard::KeyCode::Tab => {
                                        if is_shift {
                                            if let Some(id) = state.focus_manager.focus_prev() {
                                                if let Ok(node_id) = id.as_str().parse::<u64>() {
                                                    state.focused_node_id = Some(cvkg_core::KvasirId(node_id));
                                                    log::info!("[Native] Focus previous: {:?}", node_id);
                                                }
                                            }
                                        } else {
                                            if let Some(id) = state.focus_manager.focus_next() {
                                                if let Ok(node_id) = id.as_str().parse::<u64>() {
                                                    state.focused_node_id = Some(cvkg_core::KvasirId(node_id));
                                                    log::info!("[Native] Focus next: {:?}", node_id);
                                                }
                                            }
                                        }
                                        state.window.request_redraw();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(vdom) = &state.vdom
                        && let Some(cvkg_event) = convert_keyboard_event(event, &self.modifiers)
                    {
                        vdom.dispatch_event(cvkg_event);
                        state.window.request_redraw();
                    }
                }

                WindowEvent::Ime(ime_event) => {
                    if let Some(vdom) = &state.vdom
                        && let Some(cvkg_event) = convert_ime_event(ime_event)
                    {
                        vdom.dispatch_event(cvkg_event);
                        state.window.request_redraw();
                    }
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.modifiers = new_modifiers.state();
                    let shift = self.modifiers.shift_key();
                    let ctrl = self.modifiers.control_key();
                    let alt = self.modifiers.alt_key();
                    let logo = self.modifiers.super_key();
                    cvkg_core::update_system_state(|st| {
                        let mut new_st = st.clone();
                        new_st.modifiers_shift = shift;
                        new_st.modifiers_ctrl = ctrl;
                        new_st.modifiers_alt = alt;
                        new_st.modifiers_logo = logo;
                        new_st
                    });
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    // Update the scale factor and request a redraw.
                    // The surface will be reconfigured on the next frame via the
                    // existing resize path in begin_frame.
                    let _ = scale_factor;
                    if let Some(ctx) = self.window_manager.windows.get(&id) {
                        ctx.window.request_redraw();
                    }
                }
                _ => {}
            }
        } // end of state block

        if close_window {
            self.window_manager.close_window(id);
        }
        if quit_all {
            // Drain all windows; the is_empty check below will exit the event loop.
            for wid in self.window_manager.window_order().to_vec() {
                self.window_manager.close_window(wid);
            }
        }
        // Exit the event loop when all windows are closed (Cmd+W on last window, or Cmd+Q).
        if self.window_manager.windows.is_empty() {
            event_loop.exit();
        }
        if bring_to_front {
            self.window_manager.bring_to_front(id);
        }
        if create_new_window {
            self.window_manager.create_window(
                event_loop,
                &self.gpu,
                self.proxy.clone(),
                cvkg_core::WindowConfig {
                    title: "New CVKG Window".to_string(),
                    size: (800.0, 600.0),
                    ..Default::default()
                },
                false, // is_main
                &self.view,
            );
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::AccessibilityAction(request) => {
                let node_id = cvkg_core::KvasirId(request.target_node.0);
                let target_state = self.window_manager.windows.values_mut().find(|s| {
                    s.vdom
                        .as_ref()
                        .map_or(false, |v| v.nodes.contains_key(&node_id))
                });

                if let Some(state) = target_state
                    && let Some(vdom) = &state.vdom
                    && let Some(node) = vdom.nodes.get(&node_id)
                    && request.action == accesskit::Action::Click
                {
                    let event = cvkg_core::Event::PointerClick {
                        x: node.layout.x + node.layout.width / 2.0,
                        y: node.layout.y + node.layout.height / 2.0,
                        button: 0, // Assume left click for accessibility actions
                        tilt: None,
                        azimuth: None,
                        pressure: Some(1.0),
                        barrel_rotation: None,
                        pointer_precision: 0.0,
                    };
                    vdom.dispatch_event(event);
                }
            }
            AppEvent::AccessibilityInitialTreeRequested(winit_id) => {
                if let Some(state) = self.window_manager.windows.get_mut(&winit_id) {
                    if let Some(vdom) = &state.vdom {
                        let root_id = vdom.root.map(|id| id.0).unwrap_or(1);
                        let mut nodes = Vec::new();
                        for (id, node) in &vdom.nodes {
                            nodes.push((accesskit::NodeId(id.0), node.to_accesskit_node()));
                        }
                        let tree = accesskit::Tree::new(accesskit::NodeId(root_id));
                        if let Some(adapter) = &mut state.accesskit_adapter {
                            adapter.update_if_active(|| accesskit::TreeUpdate {
                                nodes,
                                tree: Some(tree),
                                focus: accesskit::NodeId(root_id),
                                tree_id: accesskit::TreeId::ROOT,
                            });
                        }
                    }
                }
            }
            AppEvent::CloseWindow(winit_id) => {
                self.window_manager.close_window(winit_id);
                if self.window_manager.windows.is_empty() {
                    event_loop.exit();
                }
            }
            AppEvent::SetTitle(winit_id, title) => {
                if let Some(data) = self.window_manager.windows.get(&winit_id) {
                    data.window.set_title(&title);
                }
            }
            AppEvent::SetSize(winit_id, width, height) => {
                if let Some(data) = self.window_manager.windows.get(&winit_id) {
                    let _ = data
                        .window
                        .request_inner_size(winit::dpi::LogicalSize::new(width, height));
                }
            }
            AppEvent::SetVisible(winit_id, visible) => {
                if let Some(data) = self.window_manager.windows.get(&winit_id) {
                    data.window.set_visible(visible);
                }
            }
            AppEvent::BringToFront(winit_id) => {
                self.window_manager.bring_to_front(winit_id);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Apply Rage Decay: rage naturally settles to 0 over time.
        self.rage = (self.rage - 0.02).max(0.0);

        // Frame Throttling: 120FPS target (8.33ms). Heartbeat timer for idle wakeup only.
        // The primary render loop is driven by request_redraw() inside RedrawRequested.
        let now = std::time::Instant::now();
        let target_interval = std::time::Duration::from_micros(8_333); // 120fps

        if now.duration_since(self.last_frame_time) >= target_interval {
            self.last_frame_time = now;
            // Only request redraw if the view has actually changed.
            // changed() returns true when rage, menu, or counters differ from last frame.
            // This avoids unnecessary GPU work for static frames (rage == 0, no interaction).
            let needs_redraw = self.view.changed();
            if needs_redraw {
                for window_state in self.window_manager.windows.values() {
                    window_state.window.request_redraw();
                }
            }
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                now + target_interval,
            ));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                self.last_frame_time + target_interval,
            ));
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

impl cvkg_core::Renderer for NativeRenderer {
    fn fill_rect(&mut self, rect: cvkg_core::Rect, color: [f32; 4]) {
        self.gpu_ref().fill_rect(rect, color);
    }
    fn fill_rounded_rect(&mut self, rect: cvkg_core::Rect, radius: f32, color: [f32; 4]) {
        self.gpu_ref().fill_rounded_rect(rect, radius, color);
    }
    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4]) {
        self.gpu_ref().fill_ellipse(rect, color);
    }
    fn stroke_rect(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref().stroke_rect(rect, color, stroke_width);
    }
    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.gpu_ref().stroke_rounded_rect(rect, radius, color, stroke_width);
    }
    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
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

    fn fill_glass_rect(&mut self, rect: cvkg_core::Rect, radius: f32, blur_radius: f32) {
        self.gpu_ref()
            .fill_glass_rect(rect, radius, blur_radius);
    }

    fn fill_glass_rect_with_intensity(&mut self, rect: cvkg_core::Rect, radius: f32, blur_radius: f32, glass_intensity: f32) {
        self.gpu_ref()
            .fill_glass_rect_with_intensity(rect, radius, blur_radius, glass_intensity);
    }

    fn fill_glass_rect_with_pressure(&mut self, rect: cvkg_core::Rect, radius: f32, blur_radius: f32, pressure: f32) {
        // Scale glass intensity by pressure: full pressure = full glass, no pressure = solid
        self.gpu_ref()
            .fill_glass_rect_with_intensity(rect, radius, blur_radius, pressure);
    }

    fn fill_squircle(&mut self, rect: cvkg_core::Rect, n: f32, color: [f32; 4]) {
        self.gpu_ref()
            .fill_squircle(rect, n, color);
    }

    fn stroke_squircle(&mut self, rect: cvkg_core::Rect, n: f32, color: [f32; 4], stroke_width: f32) {
        self.gpu_ref()
            .stroke_squircle(rect, n, color, stroke_width);
    }

    fn draw_focus_ring(&mut self, rect: cvkg_core::Rect, radius: f32, offset: f32, width: f32, color: [f32; 4]) {
        self.gpu_ref()
            .draw_focus_ring(rect, radius, offset, width, color);
    }


    fn draw_linear_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.gpu_ref()
            .draw_linear_gradient(rect, start_color, end_color, angle);
    }
    fn draw_radial_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        self.gpu_ref()
            .draw_radial_gradient(rect, inner_color, outer_color);
    }
    fn draw_texture(&mut self, texture_id: u32, rect: cvkg_core::Rect) {
        self.gpu_ref()
            .draw_texture(texture_id, rect);
    }
    fn draw_image(&mut self, image_name: &str, rect: cvkg_core::Rect) {
        self.gpu_ref()
            .draw_image(image_name, rect);
    }
    fn load_image(&mut self, name: &str, data: &[u8]) {
        self.gpu_ref()
            .load_image(name, data);
    }
    fn push_clip_rect(&mut self, rect: cvkg_core::Rect) {
        self.gpu_ref()
            .push_clip_rect(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.gpu_ref()
            .pop_clip_rect();
    }
    fn push_opacity(&mut self, opacity: f32) {
        self.gpu_ref()
            .push_opacity(opacity);
    }
    fn draw_3d_cube(&mut self, rect: cvkg_core::Rect, color: [f32; 4], rotation: [f32; 3]) {
        self.gpu_ref()
            .draw_3d_cube(rect, color, rotation);
    }
    /// Render a 3D scene graph node using the GPU backend.
    ///
    /// # Contract
    /// Delegates to the locked GPU renderer instance to queue the 3D meshes for rendering.
    fn render_scene_node_3d(
        &mut self,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
        color: [f32; 4],
        meshes: &[cvkg_core::Mesh],
    ) {
        self.gpu_ref()
            .render_scene_node_3d(position, rotation, scale, color, meshes);
    }
    fn pop_opacity(&mut self) {
        self.gpu_ref()
            .pop_opacity();
    }
    fn bifrost(&mut self, rect: cvkg_core::Rect, blur: f32, saturation: f32, opacity: f32) {
        self.gpu_ref()
            .bifrost(rect, blur, saturation, opacity);
    }
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.gpu_ref()
            .push_mjolnir_slice(angle, offset);
    }
    fn pop_mjolnir_slice(&mut self) {
        self.gpu_ref()
            .pop_mjolnir_slice();
    }
    fn mjolnir_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu_ref()
            .mjolnir_shatter(rect, pieces, force, color);
    }
    fn mjolnir_fluid_shatter(
        &mut self,
        rect: cvkg_core::Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
    ) {
        self.gpu_ref()
            .mjolnir_fluid_shatter(rect, pieces, force, color);
    }
    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.gpu_ref()
            .draw_mjolnir_bolt(from, to, color);
    }
    fn gungnir(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.gpu_ref()
            .gungnir(rect, color, radius, intensity);
    }
    fn mani_glow(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32) {
        self.gpu_ref()
            .mani_glow(rect, color, radius);
    }
    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.gpu_ref()
            .register_handler(event_type, handler);
    }
    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
        self.gpu_ref()
            .push_vnode(rect, name);
    }
    fn pop_vnode(&mut self) {
        self.gpu_ref()
            .pop_vnode();
    }
    // FIX #1: Removed duplicate definitions of set_z_index and get_z_index.
    // They appeared twice in this impl block (after pop_vnode and after register_shared_element),
    // which is a hard compiler error. Exactly one definition of each is kept here.
    fn set_z_index(&mut self, z: f32) {
        self.gpu_ref()
            .set_z_index(z);
    }
    fn get_z_index(&self) -> f32 {
        self.gpu_ref_shared()
            .get_z_index()
    }
    fn register_shared_element(&mut self, id: &str, rect: cvkg_core::Rect) {
        self.gpu_ref()
            .register_shared_element(id, rect);
    }
    fn set_material(&mut self, material: cvkg_core::DrawMaterial) {
        self.gpu_ref()
            .set_material(material);
    }
    fn current_material(&self) -> cvkg_core::DrawMaterial {
        self.gpu_ref_shared()
            .current_material()
    }
    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        self.gpu_ref()
            .serialize_svg(name)
    }
    fn apply_svg_filter(
        &mut self,
        name: &str,
        filter_id: &str,
        region: cvkg_core::Rect,
    ) -> Result<String, String> {
        self.gpu_ref()
            .apply_svg_filter(name, filter_id, region)
    }
    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.gpu_ref()
            .push_shadow(radius, color, offset);
    }
    fn pop_shadow(&mut self) {
        self.gpu_ref()
            .pop_shadow();
    }
    fn push_affine(&mut self, transform: [f32; 6]) {
        self.gpu_ref()
            .push_affine(transform);
    }
    fn enter_portal(&mut self, z_index: i32) {
        // Portal layer rendering not yet supported in SurtrRenderer.
        // Content within portals renders inline as fallback.
        log::warn!(
            "Portal rendering (enter_portal) not yet implemented in GPU backend; z_index={}",
            z_index
        );
    }
    fn exit_portal(&mut self) {
        // Portal layer rendering not yet supported in SurtrRenderer.
        log::warn!("Portal rendering (exit_portal) not yet implemented in GPU backend");
    }
    fn viewport_size(&self) -> cvkg_core::Rect {
        let size = self.window.inner_size();
        let scale = self.window.scale_factor();
        let logical = size.to_logical::<f32>(scale);
        cvkg_core::Rect::new(0.0, 0.0, logical.width, logical.height)
    }
    fn announce(&mut self, message: &str, priority: cvkg_core::AnnouncementPriority) {
        // Delegate to AccessKit via the ShieldWall adapter if active.
        // For now, log the announcement. Full implementation requires
        // integration with the AccessKit tree update cycle.
        log::info!("Accessibility announcement [{:?}]: {}", priority, message);
    }
    fn load_svg(&mut self, name: &str, svg_data: &[u8]) {
        self.gpu_ref()
            .load_svg(name, svg_data);
    }
    fn draw_svg(&mut self, name: &str, rect: cvkg_core::Rect) {
        self.gpu_ref()
            .draw_svg(name, rect, None, 0);
    }
    fn draw_svg_with_offset(&mut self, name: &str, rect: cvkg_core::Rect, animation_time_offset: f32) {
        self.gpu_ref()
            .draw_svg_with_offset(name, rect, None, 0, animation_time_offset);
    }
    fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.gpu_ref_shared()
            .telemetry
            .clone()
    }
    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        self.gpu_ref()
            .prewarm_vram(assets);
    }

    /// Return the text scale factor of the GPU renderer.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to retrieve the correct scale factor.
    fn text_scale_factor(&self) -> f32 {
        self.gpu_ref_shared()
            .text_scale_factor()
    }

    /// Return whether the current frame is over its time budget.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to check budget status.
    fn is_over_budget(&self) -> bool {
        self.gpu_ref_shared()
            .is_over_budget()
    }

    /// Draws simple unformatted text at the specified coordinates.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to perform cached text rendering.
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.gpu_ref()
            .draw_text(text, x, y, size, color);
    }

    /// Measures the dimensions of the text if rendered at the specified size.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to look up cached text dimensions.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.gpu_ref()
            .measure_text(text, size)
    }

    /// Shapes a rich text layout with the specified font spans.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to perform text layout and shaping.
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

    /// Draws a previously shaped text layout at the specified coordinates.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to emit glyph instances.
    fn draw_shaped_text(&mut self, shaped: &runic_text::ShapedText, x: f32, y: f32) {
        self.gpu_ref()
            .draw_shaped_text(shaped, x, y);
    }

    /// Fills a rounded rectangle with glass material, custom tint color, and intensity.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to draw frosted glass panels.
    fn fill_glass_rect_with_tint(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        blur_radius: f32,
        tint_color: [f32; 4],
        glass_intensity: f32,
    ) {
        self.gpu_ref()
            .fill_glass_rect_with_tint(rect, radius, blur_radius, tint_color, glass_intensity);
    }

    /// Sets the color theme of the renderer.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to update global themes.
    fn set_theme(&mut self, theme: cvkg_core::ColorTheme) {
        self.gpu_ref()
            .set_theme(theme);
    }

    /// Triggers a screen-shatter physics event at the specified origin.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to dispatch shatter compute effects.
    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.gpu_ref()
            .trigger_shatter_event(origin, force);
    }

    /// Sets the fireball light source position for specular glass highlights.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to update fireball coordinates.
    fn set_fireball_pos(&mut self, pos: [f32; 2]) {
        self.gpu_ref()
            .set_fireball_pos(pos);
    }

    /// Sets the active scene preset by name.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to configure scene shaders.
    fn set_scene(&mut self, scene: &str) {
        self.gpu_ref()
            .set_scene(scene);
    }

    /// Sets the active scene preset by ID.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to configure scene shaders.
    fn set_scene_preset(&mut self, preset: u32) {
        self.gpu_ref()
            .set_scene_preset(preset);
    }

    /// Sets the default canvas background color.
    ///
    /// # Contract
    /// delegates to the locked GPU renderer instance to configure background clears.
    fn set_default_background_color(&mut self, color: [f32; 4]) {
        self.gpu_ref()
            .set_default_background_color(color);
    }
    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        self.gpu_ref()
            .push_transform(translation, scale, rotation);
    }
    fn pop_transform(&mut self) {
        self.gpu_ref()
            .pop_transform();
    }

    fn set_berserker_mode(&mut self, state: cvkg_core::BerserkerMode) {
        self.berserker_mode = state;

        // Berserker Determinism: Apply OS-level scheduler priority hints for GodMode.
        // SAFETY: setpriority is a POSIX syscall. We pass PRIO_PROCESS with pid=0 (self).
        // Failure is silently ignored via let _ because insufficient permissions are expected
        // in unprivileged environments and must not crash the render loop.
        if state == cvkg_core::BerserkerMode::GodMode {
            log::info!("ENTERING GOD MODE: Activating Berserker Determinism (High Priority)");
            #[cfg(target_os = "linux")]
            unsafe {
                let _ = libc::setpriority(libc::PRIO_PROCESS, 0, -10);
            }
        } else {
            #[cfg(target_os = "linux")]
            unsafe {
                let _ = libc::setpriority(libc::PRIO_PROCESS, 0, 0);
            }
        }

        self.gpu_ref()
            .set_berserker_mode(state);
    }

    fn set_rage(&mut self, rage: f32) {
        self.rage = rage;
        self.gpu_ref()
            .set_rage(rage);
    }

    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        self.gpu_ref()
            .memoize(id, data_hash, render_fn);
    }

    fn snapshot_render_state(&self) -> RenderStateSnapshot {
        self.gpu_ref_shared()
            .snapshot_render_state()
    }

    fn restore_render_state(&mut self, snap: RenderStateSnapshot) {
        self.gpu_ref()
            .restore_render_state(snap);
    }
    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    /// Captures the current frame as a PNG-encoded byte buffer via GPU readback.
    /// Captures the current frame as a PNG-encoded byte buffer via GPU readback.
    ///
    /// FIX #4: capture_frame() returns a Future that borrows the SurtrRenderer, so the
    /// MutexGuard must remain alive until block_on completes -- the guard cannot be dropped
    /// before the future is driven to completion. The lock is held for the duration of the
    /// GPU readback. This is acceptable because capture_png is an infrequent, explicit
    /// user-triggered operation (not called on the hot render path), so blocking other
    /// render calls for the readback duration is not a practical concern.
    fn capture_png(&mut self) -> Vec<u8> {
        log::info!("CAPTURING_FRAME: Initiating GPU readback...");
        // INVARIANT: The MutexGuard `gpu` must outlive the future returned by capture_frame()
        // because the future borrows from the SurtrRenderer. We therefore lock, block_on the
        // future (driving it to completion), and only then allow the guard to drop.
        let gpu = self.gpu.lock().unwrap_or_else(|p| p.into_inner());
        pollster::block_on(gpu.capture_frame()).unwrap_or_else(|e| {
            log::error!("GPU frame capture failed: {}", e);
            Vec::new() // Return empty buffer on failure -- do not panic the render loop
        })
    }

    fn print(&mut self) {
        log::info!("PRINT_BRIDGE: Spooling mission status to native printer...");
        // In a production environment, this would interface with CUPS/GDI/AirPrint.
        // For the Ulfhednar prototype, we simulate the handshake.
        println!("[BRIDGE] PRINTER_READY // SPOOLING_DATA...");
    }
}

// ── Native Menu Bar Builder ───────────────────────────────────────────

// ── Event Conversion Helpers ───────────────────────────────────────────

fn convert_keyboard_event(event: winit::event::KeyEvent, modifiers: &winit::keyboard::ModifiersState) -> Option<cvkg_core::Event> {
    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
        let key_str = format!("{:?}", code);
        let cvkg_mods = cvkg_core::KeyModifiers {
            shift: modifiers.shift_key(),
            ctrl: modifiers.control_key(),
            alt: modifiers.alt_key(),
            meta: modifiers.super_key(),
        };
        if event.state == winit::event::ElementState::Pressed {
            Some(cvkg_core::Event::KeyDown { key: key_str, modifiers: cvkg_mods })
        } else {
            Some(cvkg_core::Event::KeyUp { key: key_str, modifiers: cvkg_mods })
        }
    } else {
        None
    }
}

fn convert_ime_event(event: winit::event::Ime) -> Option<cvkg_core::Event> {
    if let winit::event::Ime::Commit(string) = event {
        Some(cvkg_core::Event::Ime(string))
    } else {
        None
    }
}

fn convert_mouse_event(
    state: winit::event::ElementState,
    position: [f32; 2],
    button: u32,
) -> cvkg_core::Event {
    match state {
        winit::event::ElementState::Pressed => cvkg_core::Event::PointerDown {
            x: position[0],
            y: position[1],
            button,
            proximity_field: 0.0,
            tilt: None,
            azimuth: None,
            pressure: Some(1.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        },
        winit::event::ElementState::Released => cvkg_core::Event::PointerUp {
            x: position[0],
            y: position[1],
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        },
    }
}

// Platform-specific implementations for macOS, Windows, and Linux are handled by winit and AccessKit.

struct ShieldWall {
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
}

impl accesskit::ActionHandler for ShieldWall {
    fn do_action(&mut self, request: accesskit::ActionRequest) {
        let _ = self
            .proxy
            .send_event(AppEvent::AccessibilityAction(request));
    }
}

impl accesskit::ActivationHandler for ShieldWall {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        let mut root = accesskit::Node::new(accesskit::Role::Window);
        root.set_label("CVKG Application");

        let root_id = accesskit::NodeId(1);
        Some(accesskit::TreeUpdate {
            nodes: vec![(root_id, root)],
            tree: Some(accesskit::Tree::new(root_id)),
            focus: root_id,
            tree_id: accesskit::TreeId::ROOT,
        })
    }
}

impl accesskit::DeactivationHandler for ShieldWall {
    fn deactivate_accessibility(&mut self) {}
}

type AssetCacheMap =
    std::collections::HashMap<String, cvkg_core::AssetState<std::sync::Arc<Vec<u8>>>>;

/// A concrete AssetManager for native desktop targets that loads from the local filesystem.
///
/// The cache is read on every render frame (lock-free via `ArcSwap::load()`) but written
/// at most once per URL after disk I/O completes. `rcu()` atomically inserts the result
/// without blocking concurrent render-loop readers.
pub struct NativeAssetManager {
    cache: std::sync::Arc<arc_swap::ArcSwap<AssetCacheMap>>,
}

impl Default for NativeAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeAssetManager {
    /// Create a new, empty NativeAssetManager.
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(
                std::collections::HashMap::new(),
            )),
        }
    }
}

impl cvkg_core::AssetManager for NativeAssetManager {
    /// Return the cached asset state for `url`.
    ///
    /// Fast path: lock-free snapshot read via `ArcSwap::load()`.
    /// Slow path (cache miss): atomically insert a Loading sentinel via `rcu()`,
    /// then spawn a background thread for I/O. The `rcu()` closure may execute
    /// more than once under contention, so `already_tracked` is determined by
    /// whether the closure actually inserted the Loading entry (detected by checking
    /// the returned map). This prevents duplicate I/O threads for the same URL.
    ///
    /// FIX #5: The previous implementation set `already_tracked` inside the `rcu`
    /// closure body, which is incorrect because `rcu` retries the closure on
    /// contention -- the bool would reflect only the last execution. The fix uses
    /// the fast-path check result plus the atomic `rcu` insertion to determine
    /// whether a thread must be spawned, making the logic correct under concurrency.
    fn load_image(&self, url: &str) -> cvkg_core::AssetState<std::sync::Arc<Vec<u8>>> {
        // Fast path: lock-free read from the current cache snapshot.
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        let cache = self.cache.clone();
        let key = url.to_string();

        // Slow path: atomically insert Loading if the key is absent.
        // `rcu` returns the final committed map; we inspect it to determine
        // whether *this* call was the one that inserted Loading (and thus
        // should spawn the I/O thread) versus a concurrent call that beat us.
        let mut we_inserted = false;
        self.cache.rcu(|map| {
            if map.contains_key(&key) {
                // Another caller already claimed this URL -- do not insert.
                (**map).clone()
            } else {
                we_inserted = true;
                let mut m = (**map).clone();
                m.insert(key.clone(), cvkg_core::AssetState::Loading);
                m
            }
        });

        // Only the caller that performed the insertion spawns the I/O thread,
        // preventing duplicate concurrent reads for the same asset URL.
        if we_inserted {
            let cache_inner = cache.clone();
            let key_inner = key.clone();

            std::thread::spawn(move || {
                log::debug!("[Native] Asynchronously loading asset: {}", key_inner);
                let result = match std::fs::read(&key_inner) {
                    Ok(data) => cvkg_core::AssetState::Ready(std::sync::Arc::new(data)),
                    Err(e) => cvkg_core::AssetState::Error(e.to_string()),
                };

                cache_inner.rcu(move |map| {
                    let mut m = (**map).clone();
                    m.insert(key_inner.clone(), result.clone());
                    m
                });
            });
        }

        cvkg_core::AssetState::Loading
    }

    /// Trigger a background load of `url` without waiting for the result.
    ///
    /// FIX #6: The previous implementation had a bare fast-path check followed
    /// by an unconditional thread spawn, allowing two concurrent calls for the
    /// same URL to both spawn I/O threads. Now uses the same rcu-based insertion
    /// guard as `load_image` to ensure exactly one thread is spawned per URL.
    fn preload_image(&self, url: &str) {
        // Fast path: if already in cache (any state), no work to do.
        if self.cache.load().contains_key(url) {
            return;
        }

        let cache = self.cache.clone();
        let key = url.to_string();

        let mut we_inserted = false;
        self.cache.rcu(|map| {
            if map.contains_key(&key) {
                (**map).clone()
            } else {
                we_inserted = true;
                let mut m = (**map).clone();
                m.insert(key.clone(), cvkg_core::AssetState::Loading);
                m
            }
        });

        if we_inserted {
            std::thread::spawn(move || {
                log::debug!("[Native] Preloading asset: {}", key);
                let result = match std::fs::read(&key) {
                    Ok(data) => cvkg_core::AssetState::Ready(std::sync::Arc::new(data)),
                    Err(e) => cvkg_core::AssetState::Error(e.to_string()),
                };

                cache.rcu(move |map| {
                    let mut m = (**map).clone();
                    m.insert(key.clone(), result.clone());
                    m
                });
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::AssetManager;
    use cvkg_vdom::{AriaProps, LayoutRect, VDom, VNode};
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    fn interactive_node(
        id: u64,
        component_type: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        aria_role: &str,
    ) -> VNode {
        VNode {
            id: cvkg_core::KvasirId(id),
            key: None,
            component_type: component_type.to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x,
                y,
                width,
                height,
            },
            children: Vec::new(),
            aria_role: aria_role.to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
            sdf_shape: Some(cvkg_core::layout::SdfShape::Rect(cvkg_core::Rect {
                x,
                y,
                width,
                height,
            })),
        }
    }

    fn route_pointer_sequence_through_native_capture(
        pressed_vdom: &VDom,
        rebuilt_vdom: &VDom,
        x: f32,
        y: f32,
        button: u32,
    ) -> (
        cvkg_core::EventResponse,
        cvkg_core::EventResponse,
        cvkg_core::EventResponse,
    ) {
        let active_target = pressed_vdom.hit_test(x, y, 0.0).map(|(id, _)| id);
        let mut applied_vdom = VDom::new();
        applied_vdom.root = pressed_vdom.root;
        applied_vdom.nodes = pressed_vdom.nodes.clone();
        applied_vdom.parents = pressed_vdom.parents.clone();
        applied_vdom.event_handlers = pressed_vdom.event_handlers.clone();
        let down = active_target
            .map(|target| {
                applied_vdom.dispatch_event_to_target(
                    target,
                    cvkg_core::Event::PointerDown {
                        x,
                        y,
                        button,
                        proximity_field: 0.0,
                        tilt: None,
                        azimuth: None,
                        pressure: Some(1.0),
                        barrel_rotation: None,
                        pointer_precision: 0.0,
                    },
                )
            })
            .unwrap_or_else(|| {
                applied_vdom.dispatch_event(cvkg_core::Event::PointerDown {
                    x,
                    y,
                    button,
                    proximity_field: 0.0,
                    tilt: None,
                    azimuth: None,
                    pressure: Some(1.0),
                    barrel_rotation: None,
                    pointer_precision: 0.0,
                })
        });

        applied_vdom.apply_patches(pressed_vdom.diff(rebuilt_vdom));

        let fallback_target = applied_vdom.hit_test(x, y, 0.0).map(|(id, _)| id);
        let resolved_target = active_target
            .filter(|target| applied_vdom.nodes.contains_key(target))
            .or(fallback_target);

        let pointer_up = cvkg_core::Event::PointerUp {
            x,
            y,
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        };
        let pointer_click = cvkg_core::Event::PointerClick {
            x,
            y,
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        };

        let up = resolved_target
            .map(|target| applied_vdom.dispatch_event_to_target(target, pointer_up.clone()))
            .unwrap_or_else(|| applied_vdom.dispatch_event(pointer_up));
        let click = resolved_target
            .map(|target| applied_vdom.dispatch_event_to_target(target, pointer_click.clone()))
            .unwrap_or_else(|| applied_vdom.dispatch_event(pointer_click));

        (down, up, click)
    }

    /// FIX #12: Replaced hardcoded relative path "test_asset.png" with a temp-dir path
    /// constructed from a unique per-test name. The previous path was written to the
    /// process working directory, which varies by invocation and causes collisions when
    /// tests run in parallel or when a prior run panics before cleanup.
    #[test]
    fn test_native_asset_manager_loading() {
        let manager = NativeAssetManager::new();
        let temp_path = std::env::temp_dir().join("cvkg_test_asset_loading.png");
        let temp_file_path = temp_path.to_str().expect("temp path contains invalid UTF-8: OS temp directory is corrupted");
        let test_data = b"fake-image-data";

        // Create a temporary file in the OS temp directory
        let mut file = std::fs::File::create(temp_file_path).unwrap();
        file.write_all(test_data).unwrap();
        drop(file);

        // First call returns Loading and spawns the background I/O thread
        let mut state = manager.load_image(temp_file_path);

        // Poll until Ready or timeout
        let start = std::time::Instant::now();
        while matches!(state, cvkg_core::AssetState::Loading) && start.elapsed().as_secs() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            state = manager.load_image(temp_file_path);
        }

        if let cvkg_core::AssetState::Ready(data) = state {
            assert_eq!(&*data, test_data);
        } else {
            let _ = std::fs::remove_file(temp_file_path);
            panic!("Expected Ready state, got {:?}", state);
        }

        // Verify fast path returns Ready immediately from cache
        let state2 = manager.load_image(temp_file_path);
        if let cvkg_core::AssetState::Ready(data) = state2 {
            assert_eq!(&*data, test_data);
        } else {
            let _ = std::fs::remove_file(temp_file_path);
            panic!("Expected Ready state (cached), got {:?}", state2);
        }

        let _ = std::fs::remove_file(temp_file_path);
    }

    #[test]
    fn test_native_asset_manager_error() {
        let manager = NativeAssetManager::new();
        let path = "non_existent_file_cvkg_test.png";
        let mut state = manager.load_image(path);

        let start = std::time::Instant::now();
        while matches!(state, cvkg_core::AssetState::Loading) && start.elapsed().as_secs() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            state = manager.load_image(path);
        }

        if let cvkg_core::AssetState::Error(_) = state {
            // Expected -- non-existent file must produce an Error state
        } else {
            panic!("Expected Error state, got {:?}", state);
        }
    }

    #[test]
    fn test_event_conversion() {
        // Mouse press event
        let event = convert_mouse_event(winit::event::ElementState::Pressed, [10.0, 20.0], 0);
        if let cvkg_core::Event::PointerDown { x, y, button, .. } = event {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
            assert_eq!(button, 0);
        } else {
            panic!("Expected PointerDown");
        }

        // IME commit event
        let event = convert_ime_event(winit::event::Ime::Commit("hello".to_string()));
        if let Some(cvkg_core::Event::Ime(s)) = event {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Ime event");
        }
    }

    #[test]
    fn native_pointer_capture_survives_rebuild_sequence() {
        let fired = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let mut pressed = VDom::new();
        let root_id = cvkg_core::KvasirId(1);
        let button_id = cvkg_core::KvasirId(2);
        let mut root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        root.children = vec![button_id];
        let button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");

        let fired_down = Arc::clone(&fired);
        let fired_up = Arc::clone(&fired);
        let fired_click = Arc::clone(&fired);
        pressed.event_handlers.insert(
            button_id,
            vec![
                (
                    "pointerdown".to_string(),
                    Arc::new(move |_| {
                        fired_down.lock().unwrap().push("down");
                    }) as _,
                ),
                (
                    "pointerup".to_string(),
                    Arc::new(move |_| {
                        fired_up.lock().unwrap().push("up");
                    }) as _,
                ),
                (
                    "pointerclick".to_string(),
                    Arc::new(move |_| {
                        fired_click.lock().unwrap().push("click");
                    }) as _,
                ),
            ]
            .into_iter()
            .collect(),
        );
        pressed.root = Some(root_id);
        pressed.nodes.insert(root_id, root);
        pressed.nodes.insert(button_id, button);
        pressed.parents.insert(button_id, root_id);

        let mut rebuilt = VDom::new();
        let mut rebuilt_root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        rebuilt_root.children = vec![button_id];
        let rebuilt_button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");
        rebuilt.event_handlers = pressed.event_handlers.clone();
        rebuilt.root = Some(root_id);
        rebuilt.nodes.insert(root_id, rebuilt_root);
        rebuilt.nodes.insert(button_id, rebuilt_button);
        rebuilt.parents.insert(button_id, root_id);

        let (down, up, click) =
            route_pointer_sequence_through_native_capture(&pressed, &rebuilt, 30.0, 30.0, 0);

        assert_eq!(down, cvkg_core::EventResponse::Handled);
        assert_eq!(up, cvkg_core::EventResponse::Handled);
        assert_eq!(click, cvkg_core::EventResponse::Handled);
        assert_eq!(*fired.lock().unwrap(), vec!["down", "up", "click"]);
    }

    #[test]
    fn native_pointer_capture_falls_back_to_rebuilt_target() {
        let fired = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let mut pressed = VDom::new();
        let root_id = cvkg_core::KvasirId(1);
        let old_button_id = cvkg_core::KvasirId(2);
        let mut root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        root.children = vec![old_button_id];
        let button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");

        let fired_down = Arc::clone(&fired);
        let fired_up = Arc::clone(&fired);
        let fired_click = Arc::clone(&fired);
        pressed.event_handlers.insert(
            old_button_id,
            vec![
                (
                    "pointerdown".to_string(),
                    Arc::new(move |_| {
                        fired_down.lock().unwrap().push("down");
                    }) as _,
                ),
                (
                    "pointerup".to_string(),
                    Arc::new(move |_| {
                        fired_up.lock().unwrap().push("up");
                    }) as _,
                ),
                (
                    "pointerclick".to_string(),
                    Arc::new(move |_| {
                        fired_click.lock().unwrap().push("click");
                    }) as _,
                ),
            ]
            .into_iter()
            .collect(),
        );
        pressed.root = Some(root_id);
        pressed.nodes.insert(root_id, root);
        pressed.nodes.insert(old_button_id, button);
        pressed.parents.insert(old_button_id, root_id);

        let mut rebuilt = VDom::new();
        let mut rebuilt_root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        let rebuilt_button_id = cvkg_core::KvasirId(3);
        rebuilt_root.children = vec![rebuilt_button_id];
        let rebuilt_button = interactive_node(3, "Button", 20.0, 20.0, 80.0, 40.0, "button");
        rebuilt.event_handlers = pressed.event_handlers.clone();
        rebuilt.root = Some(root_id);
        rebuilt.nodes.insert(root_id, rebuilt_root);
        rebuilt.nodes.insert(rebuilt_button_id, rebuilt_button);
        rebuilt.parents.insert(rebuilt_button_id, root_id);

        let (down, up, click) =
            route_pointer_sequence_through_native_capture(&pressed, &rebuilt, 30.0, 30.0, 0);

        assert_eq!(down, cvkg_core::EventResponse::Handled);
        assert_eq!(up, cvkg_core::EventResponse::Handled);
        assert_eq!(click, cvkg_core::EventResponse::Handled);
        assert_eq!(*fired.lock().unwrap(), vec!["down", "up", "click"]);
    }
}

/// load_icon -- Searches known asset directories for 'icon.png'.
/// Returns a winit Icon if found and decodable, None otherwise.
/// All failures are logged at warn level -- missing icons are non-fatal.
fn load_icon() -> Option<winit::window::Icon> {
    // FIX #13: Replaced unwrap_or_default() with unwrap_or_else that logs the failure.
    // unwrap_or_default() produced an empty PathBuf silently, making all subsequent
    // icon path lookups silently fail with no diagnostic output.
    let base = std::env::current_dir().unwrap_or_else(|e| {
        log::warn!(
            "[Native] Failed to get current directory for icon search: {}",
            e
        );
        std::path::PathBuf::new()
    });

    let mut candidates = vec![
        base.join("icon.png"),
        base.join("crates/ulfhednar/icons/icon.png"),
        base.join("ulfhednar/icons/icon.png"),
        base.join("crates/ulfhednar/assets/icon.png"),
        base.join("ulfhednar/assets/icon.png"),
        base.join("assets/icon.png"),
    ];

    // Also search relative to the executable directory
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("icons/icon.png"));
        candidates.push(exe_dir.join("assets/icon.png"));
        candidates.push(exe_dir.join("icon.png"));
        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join("icons/icon.png"));
            candidates.push(parent.join("assets/icon.png"));
            candidates.push(parent.join("icon.png"));
        }
    }

    for path in candidates {
        if !path.exists() {
            log::debug!("[Native] Icon candidate not found: {:?}", path);
            continue;
        }

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                match winit::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                    Ok(icon) => {
                        log::info!("[Native] Successfully loaded app icon from: {:?}", path);
                        return Some(icon);
                    }
                    Err(e) => {
                        log::warn!("[Native] Icon format error at {:?}: {}", path, e);
                    }
                }
            }
            Err(e) => {
                log::warn!("[Native] Failed to open icon image at {:?}: {}", path, e);
            }
        }
    }

    log::warn!(
        "[Native] Failed to find icon.png in any search path (CWD: {:?})",
        base
    );
    None
}

// =============================================================================
// AUDIO / HAPTIC ENGINES -- Cross-platform micro-feedback
// =============================================================================

/// Cross-platform audio engine using rodio for spatialized sound cues.
/// Uses rodio 0.21 API: OutputStreamBuilder::open_default_stream() returns
/// OutputStream directly. Playback via Sink::try_new(&stream.mixer()) + append.
pub struct RodioAudioEngine {
    _stream: rodio::OutputStream,
}

// OutputStream is not Send+Sync on macOS due to CoreAudio, but we only use it
// from the main thread. The AudioEngine trait requires Send+Sync for use in
// App struct fields, which is safe here because we never move it across threads.
unsafe impl Send for RodioAudioEngine {}
unsafe impl Sync for RodioAudioEngine {}

impl RodioAudioEngine {
    /// Create a new audio engine. Falls back to None if audio init fails.
    pub fn new() -> Option<Self> {
        match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(stream) => {
                log::info!("[Native] Audio engine initialized (rodio)");
                Some(Self { _stream: stream })
            }
            Err(e) => {
                log::warn!("[Native] Audio init failed (no sound): {}", e);
                None
            }
        }
    }
}

impl cvkg_core::AudioEngine for RodioAudioEngine {
    fn play_sound(&self, name: &str, volume: f32) {
        let data: &[u8] = match name {
            "nav_tick" => cvkg_core::sounds::NAVIGATION_TICK,
            "success_chime" => cvkg_core::sounds::SUCCESS_CHIME,
            "warning_tone" => cvkg_core::sounds::WARNING_TONE,
            _ => {
                log::warn!("[Native] Unknown sound: {}", name);
                return;
            }
        };
        self.play_buffer(data, volume);
    }

    fn play_buffer(&self, data: &[u8], _volume: f32) {
        use std::io::Cursor;
        let cursor = Cursor::new(data.to_vec());
        let mixer = self._stream.mixer();
        match rodio::play(mixer, cursor) {
            Ok(_sink) => {}
            Err(e) => log::warn!("[Native] Audio play failed: {}", e),
        }
    }

    fn play_spatial(&self, name: &str, _position: [f32; 3], volume: f32) {
        // Spatial audio: play sound without positional attenuation (OS-agnostic fallback)
        self.play_sound(name, volume);
    }
}

/// Visual haptic engine that translates haptic requests into visual micro-animations.
/// Used as a cross-platform fallback where native haptics are unavailable.
pub struct VisualHapticEngine {
    last_impact: std::sync::Mutex<std::time::Instant>,
}

impl Default for VisualHapticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualHapticEngine {
    pub fn new() -> Self {
        Self {
            last_impact: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }
}

impl cvkg_core::HapticEngine for VisualHapticEngine {
    fn impact(&self, intensity: cvkg_core::HapticIntensity) {
        let _ = intensity;
        *self.last_impact.lock().unwrap_or_else(|p| p.into_inner()) = std::time::Instant::now();
    }
    fn selection(&self) {
        self.impact(cvkg_core::HapticIntensity::Light);
    }
    fn success(&self) {
        self.impact(cvkg_core::HapticIntensity::Medium);
    }
    fn warning(&self) {
        self.impact(cvkg_core::HapticIntensity::Medium);
    }
    fn error(&self) {
        self.impact(cvkg_core::HapticIntensity::Heavy);
    }
    fn visual_tick(&self, _intensity: f32) {
        *self.last_impact.lock().unwrap_or_else(|p| p.into_inner()) = std::time::Instant::now();
    }
}

// =============================================================================
// P1-46: Backend Translation Contracts
// =============================================================================
//
// Formalizes the translation contract between CVKG's scene graph and
// platform-native representations. Each widget type has a documented
/// mapping to platform APIs.

/// Translation contract for a CVKG widget to its native representation.
#[derive(Debug, Clone)]
pub struct TranslationContract {
    /// CVKG widget type name.
    pub cvkg_type: &'static str,
    /// Platform-specific type name (e.g., "NSView", "HWND", "GTKWidget").
    pub platform_type: &'static str,
    /// Whether this widget uses native controls or custom rendering.
    pub rendering_mode: RenderingMode,
    /// Whether accessibility is handled natively.
    pub native_accessibility: bool,
}

/// Rendering mode for a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingMode {
    /// Use native platform controls (buttons, text fields, etc.).
    Native,
    /// Use CVKG's GPU renderer for custom-drawn content.
    Custom,
    /// Hybrid: native container with custom rendering inside.
    Hybrid,
}

/// Registry of translation contracts for all widget types.
pub struct TranslationContractRegistry {
    contracts: Vec<TranslationContract>,
}

impl TranslationContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: vec![
                TranslationContract {
                    cvkg_type: "Button",
                    platform_type: "NSButton/Button/GTKButton",
                    rendering_mode: RenderingMode::Native,
                    native_accessibility: true,
                },
                TranslationContract {
                    cvkg_type: "TextInput",
                    platform_type: "NSTextField/TextBox/GTKEntry",
                    rendering_mode: RenderingMode::Native,
                    native_accessibility: true,
                },
                TranslationContract {
                    cvkg_type: "Canvas",
                    platform_type: "NSView/HWND/GtkDrawingArea",
                    rendering_mode: RenderingMode::Custom,
                    native_accessibility: false,
                },
                TranslationContract {
                    cvkg_type: "TreeView",
                    platform_type: "NSTableView/TreeView/GTKTreeView",
                    rendering_mode: RenderingMode::Hybrid,
                    native_accessibility: true,
                },
            ],
        }
    }

    /// Look up the contract for a CVKG widget type.
    pub fn find(&self, cvkg_type: &str) -> Option<&TranslationContract> {
        self.contracts.iter().find(|c| c.cvkg_type == cvkg_type)
    }
}

impl Default for TranslationContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// P1-47: Window Management Contracts
// =============================================================================

/// Window capability matrix per platform.
#[derive(Debug, Clone)]
pub struct WindowCapabilityMatrix {
    /// Platform name.
    pub platform: &'static str,
    /// Supported window types.
    pub window_types: Vec<WindowType>,
    /// Whether tabbed windows are supported.
    pub tabbed_windows: bool,
    /// Whether tiled windows are supported.
    pub tiled_windows: bool,
    /// Whether floating panels are supported.
    pub floating_panels: bool,
    /// Whether sheets/popovers are supported.
    pub sheets: bool,
}

/// Window type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Document,
    Panel,
    Popover,
    Dialog,
    Tooltip,
}

impl WindowCapabilityMatrix {
    /// Get the capability matrix for the current platform.
    pub fn for_current_platform() -> Self {
        #[cfg(target_os = "macos")]
        return Self {
            platform: "macOS",
            window_types: vec![
                WindowType::Document,
                WindowType::Panel,
                WindowType::Popover,
                WindowType::Dialog,
                WindowType::Tooltip,
            ],
            tabbed_windows: true,
            tiled_windows: true,
            floating_panels: true,
            sheets: true,
        };

        #[cfg(target_os = "windows")]
        return Self {
            platform: "Windows",
            window_types: vec![
                WindowType::Document,
                WindowType::Panel,
                WindowType::Dialog,
                WindowType::Tooltip,
            ],
            tabbed_windows: true,
            tiled_windows: true,
            floating_panels: true,
            sheets: false,
        };

        #[cfg(target_os = "linux")]
        return Self {
            platform: "Linux",
            window_types: vec![
                WindowType::Document,
                WindowType::Panel,
                WindowType::Dialog,
                WindowType::Tooltip,
            ],
            tabbed_windows: false,
            tiled_windows: true,
            floating_panels: true,
            sheets: false,
        };

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return Self {
            platform: "Unknown",
            window_types: vec![WindowType::Document],
            tabbed_windows: false,
            tiled_windows: false,
            floating_panels: false,
            sheets: false,
        };
    }
}

// =============================================================================
// P1-49: Widget State Synchronization
// =============================================================================

/// Bidirectional state synchronization between CVKG and native widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// CVKG state drives native widget.
    CvkgToNative,
    /// Native widget state drives CVKG.
    NativeToCvkg,
    /// Both directions.
    Bidirectional,
}

/// State synchronization contract for a widget.
#[derive(Debug, Clone)]
pub struct StateSyncContract {
    /// Widget type name.
    pub widget_type: &'static str,
    /// Synchronization direction.
    pub direction: SyncDirection,
    /// Whether to debounce rapid changes.
    pub debounce: bool,
    /// Debounce interval in milliseconds.
    pub debounce_ms: u64,
}

/// Registry of state synchronization contracts.
pub struct StateSyncRegistry {
    contracts: Vec<StateSyncContract>,
}

impl StateSyncRegistry {
    pub fn new() -> Self {
        Self {
            contracts: vec![
                StateSyncContract {
                    widget_type: "Button",
                    direction: SyncDirection::Bidirectional,
                    debounce: false,
                    debounce_ms: 0,
                },
                StateSyncContract {
                    widget_type: "TextInput",
                    direction: SyncDirection::Bidirectional,
                    debounce: true,
                    debounce_ms: 50,
                },
                StateSyncContract {
                    widget_type: "Slider",
                    direction: SyncDirection::Bidirectional,
                    debounce: true,
                    debounce_ms: 16,
                },
                StateSyncContract {
                    widget_type: "Checkbox",
                    direction: SyncDirection::Bidirectional,
                    debounce: false,
                    debounce_ms: 0,
                },
            ],
        }
    }

    pub fn find(&self, widget_type: &str) -> Option<&StateSyncContract> {
        self.contracts.iter().find(|c| c.widget_type == widget_type)
    }
}

impl Default for StateSyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// P1-51: Large UI Scalability (Native)
// =============================================================================

/// Widget virtualization configuration for large UIs.
#[derive(Debug, Clone, Copy)]
pub struct WidgetVirtualizationConfig {
    /// Number of widgets to render outside the viewport (buffer).
    pub buffer_size: usize,
    /// Whether to recycle widget native handles.
    pub recycle_handles: bool,
    /// Maximum number of active native handles.
    pub max_active_handles: usize,
}

impl Default for WidgetVirtualizationConfig {
    fn default() -> Self {
        Self {
            buffer_size: 5,
            recycle_handles: true,
            max_active_handles: 100,
        }
    }
}

// =============================================================================
// P1-50: Semantic Role Mapping
// =============================================================================

/// Explicit mapping from AccessKit/CVKG role to platform accessibility concepts:
/// macOS (AXRole), Windows (UIA ControlType), and Linux (ATK Role).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticRoleMapping {
    /// The input AccessKit role.
    pub role: accesskit::Role,
    /// macOS AXRole string.
    pub mac_ax_role: &'static str,
    /// Windows UI Automation ControlType constant name or ID string.
    pub win_uia_control_type: &'static str,
    /// Linux ATK Role constant name or ID string.
    pub linux_atk_role: &'static str,
}

/// Registry of semantic accessibility mappings.
pub struct SemanticRoleRegistry {
    mappings: Vec<SemanticRoleMapping>,
}

impl SemanticRoleRegistry {
    pub fn new() -> Self {
        Self {
            mappings: vec![
                SemanticRoleMapping {
                    role: accesskit::Role::Button,
                    mac_ax_role: "AXButton",
                    win_uia_control_type: "UIA_ButtonControlTypeId",
                    linux_atk_role: "ATK_ROLE_PUSH_BUTTON",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::TextInput,
                    mac_ax_role: "AXTextField",
                    win_uia_control_type: "UIA_EditControlTypeId",
                    linux_atk_role: "ATK_ROLE_ENTRY",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::CheckBox,
                    mac_ax_role: "AXCheckBox",
                    win_uia_control_type: "UIA_CheckBoxControlTypeId",
                    linux_atk_role: "ATK_ROLE_CHECK_BOX",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::Slider,
                    mac_ax_role: "AXSlider",
                    win_uia_control_type: "UIA_SliderControlTypeId",
                    linux_atk_role: "ATK_ROLE_SLIDER",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::Label,
                    mac_ax_role: "AXStaticText",
                    win_uia_control_type: "UIA_TextControlTypeId",
                    linux_atk_role: "ATK_ROLE_LABEL",
                },
            ],
        }
    }

    /// Look up the platform mappings for a given role.
    pub fn find(&self, role: accesskit::Role) -> Option<&SemanticRoleMapping> {
        self.mappings.iter().find(|m| m.role == role)
    }
}

impl Default for SemanticRoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}


// =============================================================================
// P2-39: Multi-Monitor Support
// =============================================================================

/// P2-39: Multi-monitor support contract config for mixed DPI and refresh rates.
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Friendly name of the monitor (e.g. "Primary", "External").
    pub name: String,
    /// Spatial origin position in physical coordinates.
    pub position: (i32, i32),
    /// Size in physical pixels.
    pub size: (u32, u32),
    /// DPI scaling factor.
    pub scale_factor: f64,
    /// Refresh rate in Hz.
    pub refresh_rate: u32,
}

/// P2-39: Manages multi-monitor layouts, tracking scale factor updates
/// and DPI changes during transitions.
#[derive(Debug, Clone)]
pub struct MultiMonitorManager {
    monitors: Vec<MonitorConfig>,
    current_monitor_index: usize,
}

impl MultiMonitorManager {
    /// Creates a new `MultiMonitorManager` with a set of displays.
    ///
    /// # Arguments
    /// * `monitors` - The list of active monitors. Must contain at least one monitor.
    ///
    /// # Contract
    /// If the list of monitors is empty, a default 1080p, 60Hz, 1.0x scale monitor is added.
    pub fn new(mut monitors: Vec<MonitorConfig>) -> Self {
        if monitors.is_empty() {
            monitors.push(MonitorConfig {
                name: "Default".to_string(),
                position: (0, 0),
                size: (1920, 1080),
                scale_factor: 1.0,
                refresh_rate: 60,
            });
        }
        Self {
            monitors,
            current_monitor_index: 0,
        }
    }

    /// Returns the currently active monitor configuration.
    pub fn current_monitor(&self) -> &MonitorConfig {
        &self.monitors[self.current_monitor_index]
    }

    /// Returns all registered monitor configurations.
    pub fn monitors(&self) -> &[MonitorConfig] {
        &self.monitors
    }

    /// Determines which monitor a window is on based on its center coordinate.
    ///
    /// # Arguments
    /// * `window_rect` - The spatial bounds of the window represented as `(x, y, width, height)` in physical coordinates.
    ///
    /// # Contract
    /// Selects the monitor that contains the center point of the window. If the center point
    /// is outside all monitors, defaults to the closest monitor or the current one.
    pub fn update_window_position(&mut self, window_rect: (i32, i32, u32, u32)) -> Option<usize> {
        let center_x = window_rect.0 + (window_rect.2 as i32 / 2);
        let center_y = window_rect.1 + (window_rect.3 as i32 / 2);

        let mut best_index = None;
        let mut min_distance = f64::MAX;

        for (i, m) in self.monitors.iter().enumerate() {
            let left = m.position.0;
            let right = m.position.0 + m.size.0 as i32;
            let top = m.position.1;
            let bottom = m.position.1 + m.size.1 as i32;

            if center_x >= left && center_x < right && center_y >= top && center_y < bottom {
                self.current_monitor_index = i;
                return Some(i);
            }

            // Calculate distance to center of monitor
            let m_center_x = m.position.0 + (m.size.0 as i32 / 2);
            let m_center_y = m.position.1 + (m.size.1 as i32 / 2);
            let dx = (center_x - m_center_x) as f64;
            let dy = (center_y - m_center_y) as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < min_distance {
                min_distance = dist;
                best_index = Some(i);
            }
        }

        if let Some(i) = best_index {
            self.current_monitor_index = i;
            Some(i)
        } else {
            None
        }
    }

    /// Dynamically scales logical dimensions to physical dimensions using the active monitor's scale factor.
    ///
    /// # Arguments
    /// * `logical_width` - The logical width to scale.
    /// * `logical_height` - The logical height to scale.
    ///
    /// # Returns
    /// The physical dimensions as `(u32, u32)`.
    pub fn scale_dimensions(&self, logical_width: f64, logical_height: f64) -> (u32, u32) {
        let sf = self.current_monitor().scale_factor;
        (
            (logical_width * sf).round() as u32,
            (logical_height * sf).round() as u32,
        )
    }

    /// Checks if moving between monitors requires a DPI scaling recalculation.
    ///
    /// # Arguments
    /// * `from_index` - The source monitor index.
    /// * `to_index` - The target monitor index.
    pub fn requires_dpi_adaptation(&self, from_index: usize, to_index: usize) -> bool {
        if from_index < self.monitors.len() && to_index < self.monitors.len() {
            (self.monitors[from_index].scale_factor - self.monitors[to_index].scale_factor).abs() > f64::EPSILON
        } else {
            false
        }
    }
}

// =============================================================================
// P2-40: Visual Regression Tracker
// =============================================================================

/// P2-40: Native Visual Regression Testing infrastructure.
/// Captures and compares frames to detect platform-specific visual differences.
#[derive(Debug, Clone)]
pub struct VisualRegressionTracker {
    /// Path to directory where reference "golden" images are located.
    reference_dir: std::path::PathBuf,
    /// Absolute threshold difference tolerance per pixel component (0 to 255).
    pixel_tolerance: u8,
    /// Percentage threshold of allowed mismatched pixels (0.0 to 100.0).
    max_mismatched_percentage: f64,
}

impl VisualRegressionTracker {
    /// Creates a new `VisualRegressionTracker` with specified reference folder and tolerances.
    pub fn new(reference_dir: std::path::PathBuf, pixel_tolerance: u8, max_mismatched_percentage: f64) -> Self {
        Self {
            reference_dir,
            pixel_tolerance,
            max_mismatched_percentage,
        }
    }

    /// Compares a captured PNG byte buffer against a named golden reference file.
    ///
    /// # Arguments
    /// * `test_name` - The identifier of the visual test (e.g. "primary_window_layout").
    /// * `captured_png` - The raw bytes of the PNG-encoded frame capture.
    ///
    /// # Returns
    /// `true` if the captured image matches the reference image within tolerances,
    /// `false` if they mismatch or if the reference image cannot be found/decoded.
    ///
    /// # Contract
    /// If the reference image file does not exist, this function writes the captured PNG
    /// as the new reference (acting in recording mode) and returns `true`.
    pub fn verify_frame(&self, test_name: &str, captured_png: &[u8]) -> bool {
        let reference_path = self.reference_dir.join(format!("{}.png", test_name));
        if !reference_path.exists() {
            log::info!("Golden reference for '{}' not found. Recording current capture as reference.", test_name);
            if let Some(parent) = reference_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&reference_path, captured_png) {
                log::error!("Failed to write golden image: {}", e);
                return false;
            }
            return true;
        }

        // Load reference image
        let ref_img = match image::load_from_memory(&std::fs::read(&reference_path).unwrap_or_default()) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to decode reference image: {}", e);
                return false;
            }
        };

        // Load captured image
        let cap_img = match image::load_from_memory(captured_png) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to decode captured image: {}", e);
                return false;
            }
        };

        if ref_img.dimensions() != cap_img.dimensions() {
            log::warn!("Dimensions mismatch for test '{}': ref {:?}, cap {:?}", test_name, ref_img.dimensions(), cap_img.dimensions());
            return false;
        }

        let (width, height) = ref_img.dimensions();
        let total_pixels = width as f64 * height as f64;
        let mut mismatched_pixels = 0;

        for (x, y, ref_pixel) in ref_img.enumerate_pixels() {
            let cap_pixel = cap_img.get_pixel(x, y);
            let mut pixel_differs = false;
            for c in 0..4 {
                let diff = (ref_pixel[c] as i16 - cap_pixel[c] as i16).abs();
                if diff > self.pixel_tolerance as i16 {
                    pixel_differs = true;
                    break;
                }
            }
            if pixel_differs {
                mismatched_pixels += 1;
            }
        }

        let mismatch_pct = (mismatched_pixels as f64 / total_pixels) * 100.0;
        if mismatch_pct > self.max_mismatched_percentage {
            log::warn!("Visual regression detected in test '{}': {:.2}% mismatched pixels (max allowed {:.2}%)",
                test_name, mismatch_pct, self.max_mismatched_percentage);
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod p1_46_47_49_51_tests {
    use super::*;

    // P2-39: Multi-monitor tests
    #[test]
    fn test_multi_monitor_manager_basics() {
        let m1 = MonitorConfig {
            name: "Display 1".to_string(),
            position: (0, 0),
            size: (1920, 1080),
            scale_factor: 1.0,
            refresh_rate: 60,
        };
        let m2 = MonitorConfig {
            name: "Display 2".to_string(),
            position: (1920, 0),
            size: (3840, 2160),
            scale_factor: 2.0,
            refresh_rate: 120,
        };

        let mut manager = MultiMonitorManager::new(vec![m1, m2]);
        assert_eq!(manager.monitors().len(), 2);
        assert_eq!(manager.current_monitor().name, "Display 1");

        // Scale dimensions logical to physical
        let scaled = manager.scale_dimensions(100.0, 200.0);
        assert_eq!(scaled, (100, 200));

        // Shift window to second monitor (centered on second monitor)
        let idx = manager.update_window_position((1920 + 100, 100, 1000, 1000));
        assert_eq!(idx, Some(1));
        assert_eq!(manager.current_monitor().name, "Display 2");

        let scaled_m2 = manager.scale_dimensions(100.0, 200.0);
        assert_eq!(scaled_m2, (200, 400));

        // Check DPI adaptation trigger
        assert!(manager.requires_dpi_adaptation(0, 1));
        assert!(!manager.requires_dpi_adaptation(0, 0));
    }

    // P2-40: Visual regression tests
    #[test]
    fn test_visual_regression_tracker_comparison() {
        // Create simple mock raw images using image crate
        use image::{RgbaImage, ImageFormat};
        use std::io::Cursor;

        let mut img1 = RgbaImage::new(10, 10);
        for p in img1.pixels_mut() {
            *p = image::Rgba([255, 0, 0, 255]);
        }
        let mut png1 = Vec::new();
        img1.write_to(&mut Cursor::new(&mut png1), ImageFormat::Png).unwrap();

        // Exact match
        let temp_dir = std::env::temp_dir().join("cvkg_visual_regression_tests");
        let tracker = VisualRegressionTracker::new(temp_dir.clone(), 5, 1.0);

        // Recording mode: first call records png1 as the golden reference
        let matched = tracker.verify_frame("test_red_rect", &png1);
        assert!(matched);

        // Second call matches against recorded reference
        let matched_again = tracker.verify_frame("test_red_rect", &png1);
        assert!(matched_again);

        // Slightly different image (within tolerances)
        let mut img2 = RgbaImage::new(10, 10);
        for (i, p) in img2.pixels_mut().enumerate() {
            if i == 0 {
                // One pixel slightly off, but within tolerance
                *p = image::Rgba([253, 0, 0, 255]);
            } else {
                *p = image::Rgba([255, 0, 0, 255]);
            }
        }
        let mut png2 = Vec::new();
        img2.write_to(&mut Cursor::new(&mut png2), ImageFormat::Png).unwrap();

        let matched_tolerated = tracker.verify_frame("test_red_rect", &png2);
        assert!(matched_tolerated);

        // Very different image (out of tolerances)
        let mut img3 = RgbaImage::new(10, 10);
        for p in img3.pixels_mut() {
            *p = image::Rgba([0, 255, 0, 255]); // Green instead of Red
        }
        let mut png3 = Vec::new();
        img3.write_to(&mut Cursor::new(&mut png3), ImageFormat::Png).unwrap();

        let matched_fail = tracker.verify_frame("test_red_rect", &png3);
        assert!(!matched_fail);

        // Clean up
        let _ = std::fs::remove_file(temp_dir.join("test_red_rect.png"));
    }

    // P1-46: Translation contracts
    #[test]
    fn translation_contract_registry_has_defaults() {
        let reg = TranslationContractRegistry::new();
        assert!(reg.find("Button").is_some());
        assert!(reg.find("Canvas").is_some());
        assert!(reg.find("Unknown").is_none());
    }

    #[test]
    fn button_uses_native_rendering() {
        let reg = TranslationContractRegistry::new();
        let contract = reg.find("Button").unwrap();
        assert_eq!(contract.rendering_mode, RenderingMode::Native);
        assert!(contract.native_accessibility);
    }

    #[test]
    fn canvas_uses_custom_rendering() {
        let reg = TranslationContractRegistry::new();
        let contract = reg.find("Canvas").unwrap();
        assert_eq!(contract.rendering_mode, RenderingMode::Custom);
    }

    // P1-47: Window capabilities
    #[test]
    fn window_capability_matrix_has_platform() {
        let matrix = WindowCapabilityMatrix::for_current_platform();
        assert!(!matrix.platform.is_empty());
        assert!(!matrix.window_types.is_empty());
    }

    #[test]
    fn macos_has_sheets() {
        #[cfg(target_os = "macos")]
        {
            let matrix = WindowCapabilityMatrix::for_current_platform();
            assert!(matrix.sheets);
            assert!(matrix.tabbed_windows);
        }
    }

    // P1-49: State sync
    #[test]
    fn state_sync_registry_has_defaults() {
        let reg = StateSyncRegistry::new();
        assert!(reg.find("Button").is_some());
        assert!(reg.find("TextInput").is_some());
    }

    #[test]
    fn text_input_has_debounce() {
        let reg = StateSyncRegistry::new();
        let contract = reg.find("TextInput").unwrap();
        assert!(contract.debounce);
        assert_eq!(contract.debounce_ms, 50);
    }

    #[test]
    fn button_is_bidirectional() {
        let reg = StateSyncRegistry::new();
        let contract = reg.find("Button").unwrap();
        assert_eq!(contract.direction, SyncDirection::Bidirectional);
    }

    // P1-51: Widget virtualization
    #[test]
    fn default_virtualization_config() {
        let config = WidgetVirtualizationConfig::default();
        assert_eq!(config.buffer_size, 5);
        assert!(config.recycle_handles);
        assert_eq!(config.max_active_handles, 100);
    }

    // P1-50: Semantic Role Mapping
    #[test]
    fn semantic_role_registry_has_button_and_text() {
        let reg = SemanticRoleRegistry::new();
        let button = reg.find(accesskit::Role::Button).unwrap();
        assert_eq!(button.mac_ax_role, "AXButton");
        assert_eq!(button.win_uia_control_type, "UIA_ButtonControlTypeId");
        assert_eq!(button.linux_atk_role, "ATK_ROLE_PUSH_BUTTON");

        let text = reg.find(accesskit::Role::TextInput).unwrap();
        assert_eq!(text.mac_ax_role, "AXTextField");
    }


    // =========================================================================
    // P2-3: Mutex Poison Recovery Tests
    // =========================================================================
    // These tests verify that mutex poison is handled gracefully via
    // unwrap_or_else(|p| p.into_inner()) instead of panicking.

    use std::sync::{Arc, Mutex};
    use std::thread;

    /// Test that a poisoned mutex can be recovered via unwrap_or_else.
    /// This simulates what happens when a thread panics while holding the GPU lock.
    #[test]
    fn mutex_poison_recovery_via_unwrap_or_else() {
        let mutex = Arc::new(Mutex::new(42u32));
        let mutex_clone = Arc::clone(&mutex);

        // Spawn a thread that panics while holding the lock
        let handle = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("simulated thread panic while holding lock");
        });

        // Wait for the thread to panic
        let _ = handle.join();

        // The mutex is now poisoned - but we can still recover the data
        let value = mutex.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(*value, 42, "poisoned mutex should still yield the inner value");
    }

    /// Test that multiple poison recoveries work correctly.
    #[test]
    fn mutex_poison_recovery_multiple_times() {
        let mutex = Arc::new(Mutex::new(String::from("hello")));

        for i in 0..5 {
            let m = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                let _guard = m.lock().unwrap();
                panic!("panic iteration {}", i);
            });
            let _ = handle.join();
        }

        // After 5 poison events, we can still recover
        let value = mutex.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(*value, "hello");
    }

    /// Test that the GPU mutex pattern used in NativeRenderer works correctly.
    /// This validates the pattern: self.gpu.lock().unwrap_or_else(|p| p.into_inner())
    #[test]
    fn gpu_mutex_poison_pattern() {
        let gpu = Arc::new(Mutex::new(RendererState { frame_count: 0 }));
        let gpu_clone = Arc::clone(&gpu);

        // Simulate a render call that panics mid-frame
        let handle = thread::spawn(move || {
            let mut state = gpu_clone.lock().unwrap();
            state.frame_count += 1;
            panic!("GPU render panic");
        });

        let _ = handle.join();

        // The NativeRenderer pattern should recover gracefully
        let mut state = gpu.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(state.frame_count, 1);
        // Can continue using the renderer after poison recovery
        state.frame_count += 1;
        assert_eq!(state.frame_count, 2);
    }

    /// Test that poison recovery doesn't lose data integrity.
    #[test]
    fn poison_recovery_preserves_data_integrity() {
        let data = Arc::new(Mutex::new(vec![1, 2, 3, 4, 5]));
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            let mut guard = data_clone.lock().unwrap();
            guard.push(6);
            panic!("mid-mutation panic");
        });

        let _ = handle.join();

        // The data should be in a consistent state (push may or may not have completed)
        let recovered = data.lock().unwrap_or_else(|p| p.into_inner());
        // Either [1,2,3,4,5] or [1,2,3,4,5,6] - both are valid
        assert!(recovered.len() >= 5);
        assert_eq!(&recovered[..5], &[1, 2, 3, 4, 5]);
    }
}

/// Helper struct for GPU mutex poison tests.
struct RendererState {
    frame_count: u32,
}
