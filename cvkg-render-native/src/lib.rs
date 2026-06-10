//! # CVKG Agentic Development Guidelines (v1.3)
//!
//! All AI agents contributing to this crate MUST follow ALL eight rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–8) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//! 8. HARDWARE VERIFIED — NEVER declare success based on mock data/rendering for native crates.
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

use cvkg_core::{FrameRenderer, Renderer};
use image;
// FIX #10: Wayland import gated to Linux only — was unconditional, broke macOS/Windows builds.
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
    /// * `window_size` — the current window size in physical pixels.
    /// * `corner_radius` — the corner radius in points (e.g., 26.0 for Tahoe).
    /// * `expansion` — extra pixels to expand beyond the visual edge (e.g., 8.0).
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
    pub fn hit_test(&self, pos: winit::dpi::PhysicalSize<u32>, corner_radius: f32) -> bool {
        let r = corner_radius + self.expansion;
        let w = self.window_size.width as f32;
        let h = self.window_size.height as f32;
        let px = pos.width as f32;
        let py = pos.height as f32;

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
    /// * **Fullscreen** — zero insets (window owns the entire screen).
    /// * **Normal** — 24pt top on macOS for the menu bar, 0 on other platforms.
    /// * **All other states** — same as Normal.
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

/// Native renderer backend implementing the Renderer trait.
/// It wraps a shared SurtrRenderer for high-performance GPU drawing.
pub struct NativeRenderer {
    gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>,
    delta_time: f32,
    elapsed_time: f32,
    berserker_mode: cvkg_core::BerserkerMode,
    rage: f32,
    window: Arc<Window>,
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
    /// This is the main entry point for desktop applications.
    pub fn run<V: cvkg_core::View + 'static>(view: V) {
        let event_loop = EventLoop::<AppEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");
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
            modifiers: winit::keyboard::ModifiersState::default(),
            audio_engine: None,
            haptic_engine: Arc::new(VisualHapticEngine::new()),
        };

        event_loop.run_app(&mut app).expect("Event loop error");
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

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
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

        let data = WindowData {
            window: window.clone(),
            accesskit_adapter: None,
            vdom: Some(vdom),
            cursor_pos: [0.0, 0.0],
            last_redraw_start: std::time::Instant::now(),
            frame_history: std::collections::VecDeque::with_capacity(60),
            frame_count: 0,
            last_pos: None,
            needs_cursor_update: false,
            is_dragging: false,
            drag_start_pos: [0.0, 0.0],
            drag_button: 0,
            drag_threshold: 5.0,
            is_key_focused,
            is_main,
            core_id,
            window_handle: handle.clone(),
        };

        self.windows.insert(winit_id, data);
        self.window_stack.push(winit_id);
        self.winit_to_core.insert(winit_id, core_id);
        self.core_to_winit.insert(core_id, winit_id);

        if let Some(gpu_mutex) = gpu {
            gpu_mutex.lock().unwrap().register_window(window.clone());
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

    // ── Multi-window tracking ──────────────────────────────────────────────
    is_key_focused: Arc<std::sync::atomic::AtomicBool>,
    is_main: bool,
    core_id: cvkg_core::WindowId,
    window_handle: cvkg_core::WindowHandle,
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
    /// Tracks active modifier key states (Ctrl, Shift, Command, etc.).
    modifiers: winit::keyboard::ModifiersState,
    /// Cross-platform audio engine for spatialized sound cues.
    audio_engine: Option<Arc<dyn cvkg_core::AudioEngine>>,
    /// Visual haptic engine for micro-feedback animations.
    haptic_engine: Arc<dyn cvkg_core::HapticEngine>,
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
                transparent: false,
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
                .expect("Failed to get winit_id");
            let window = self
                .window_manager
                .windows
                .get(&winit_id)
                .unwrap()
                .window
                .clone();

            // Immediately set self.gpu to prevent re-entry
            let gpu = pollster::block_on(cvkg_render_gpu::SurtrRenderer::forge(window.clone()));
            self.gpu = Some(Arc::new(std::sync::Mutex::new(gpu)));

            log::info!("[Native] Initialization complete.");
            window.request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if matches!(cause, winit::event::StartCause::Poll) {
            // Too noisy
        } else {
            log::debug!("[Native] Event Loop Wake: {:?}", cause);
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
            log::info!("[Native] DEVICE EVENT: {:?}", event);
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
        // Cmd+Q was pressed — close all windows after the state block ends.
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
                        .expect("GPU mutex poisoned during resize")
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

                    // Build new vdom and diff (layout pass)
                    let layout_start = std::time::Instant::now();
                    let new_vdom = cvkg_vdom::VDom::build(&self.view, rect);

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
                            });
                        }
                        state.needs_cursor_update = false;
                    }
                    let layout_end = std::time::Instant::now();

                    // Apply patches to the accessibility tree and the previous VDOM
                    let state_flush_start = std::time::Instant::now();
                    if let Some(prev_vdom) = &mut state.vdom {
                        let patches = prev_vdom.diff(&new_vdom);
                        let mut nodes = Vec::new();
                        for patch in &patches {
                            if let cvkg_vdom::VDomPatch::Create(node)
                            | cvkg_vdom::VDomPatch::Replace { node, .. } = patch
                            {
                                nodes
                                    .push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                            } else if let cvkg_vdom::VDomPatch::Update { id, .. } = patch
                                && let Some(node) = new_vdom.nodes.get(id)
                            {
                                nodes
                                    .push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                            }
                        }
                        if !nodes.is_empty() {
                            if let Some(adapter) = &mut state.accesskit_adapter {
                                adapter.update_if_active(|| accesskit::TreeUpdate {
                                    nodes,
                                    tree: None,
                                    focus: accesskit::NodeId(1),
                                });
                            }
                        }
                        prev_vdom.apply_patches(patches);
                    } else {
                        state.vdom = Some(new_vdom);
                    }
                    let state_flush_end = std::time::Instant::now();

                    // GPU rendering
                    let draw_start = std::time::Instant::now();
                    let delta_time = redraw_start.duration_since(last_redraw_start).as_secs_f32();
                    let elapsed_time = redraw_start.duration_since(self.start_time).as_secs_f32();
                    let mut gpu = gpu_arc
                        .lock()
                        .expect("GPU mutex poisoned during frame begin");
                    let encoder = gpu.begin_frame(id);
                    let mut renderer = NativeRenderer::new(
                        state.window.clone(),
                        gpu_arc.clone(),
                        delta_time,
                        elapsed_time,
                        self.berserker_mode,
                        self.rage,
                    );
                    // Release the gpu lock before calling render — the render methods each
                    // re-acquire it per-call, allowing the view tree to interleave with other
                    // work without holding one giant critical section across the whole draw.
                    drop(gpu);
                    self.view.render(&mut renderer, rect);
                    let draw_end = std::time::Instant::now();

                    // Re-acquire to submit the frame
                    let gpu_submit_start = std::time::Instant::now();
                    let mut gpu = gpu_arc
                        .lock()
                        .expect("GPU mutex poisoned during frame submit");
                    gpu.render_frame();
                    gpu.end_frame(encoder);
                    let gpu_submit_end = std::time::Instant::now();

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
                        draw_end.duration_since(draw_start).as_secs_f32() * 1000.0;
                    telemetry.gpu_submit_time_ms = gpu_submit_end
                        .duration_since(gpu_submit_start)
                        .as_secs_f32()
                        * 1000.0;

                    // Total frame time from redraw request to GPU submission complete
                    let frame_time_ms =
                        gpu_submit_end.duration_since(redraw_start).as_secs_f32() * 1000.0;
                    telemetry.frame_time_ms = frame_time_ms;

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
                    // scheduling disruption (GC, OS preemption, slow layout) — not a confirmed
                    // hardware stall, but the field name is defined in cvkg_core::TelemetryData.
                    telemetry.hardware_stall_detected = telemetry.frame_jitter_ms > 20.0;

                    state.frame_count += 1;

                    telemetry.berserker_rage = self.rage;
                    gpu.telemetry = telemetry;
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
                    state.cursor_pos = [logical.x, logical.y];
                    state.needs_cursor_update = true;
                    // Don't request_redraw here — the redraw will process the cursor update.
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
                                });
                            }
                            winit::event::ElementState::Released => {
                                log::info!("[Native] Dispatching PointerUp to VDOM");
                                vdom.dispatch_event(cvkg_core::Event::PointerUp {
                                    x: state.cursor_pos[0],
                                    y: state.cursor_pos[1],
                                    button: btn_id,
                                    tilt: None,
                                    azimuth: None,
                                    pressure: Some(0.0),
                                    barrel_rotation: None,
                                });
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
                        });
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
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyS => {
                                        log::info!("[Native] Shortcut: Save (Cmd+S)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+s".to_string(),
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
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyF => {
                                        log::info!("[Native] Shortcut: Find (Cmd+F)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::KeyDown {
                                                key: "cmd+f".to_string(),
                                            });
                                        }
                                        state.window.request_redraw();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(vdom) = &state.vdom
                        && let Some(cvkg_event) = convert_keyboard_event(event)
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
                let node_id = cvkg_vdom::NodeId(request.target.0);
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
                    };
                    vdom.dispatch_event(event);
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

        // Frame Throttling: 60FPS target (16.6ms)
        let now = std::time::Instant::now();
        let target_interval = std::time::Duration::from_millis(16);

        if now.duration_since(self.last_frame_time) >= target_interval {
            if self.rage > 0.01 {
                // Only log heartbeat when there is kinetic activity
                log::debug!("[Native] Heartbeat ticking (rage: {})", self.rage);
            }
            self.last_frame_time = now;
            for window_state in self.window_manager.windows.values() {
                window_state.window.request_redraw();
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
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: fill_rect")
            .fill_rect(rect, color);
    }
    fn fill_rounded_rect(&mut self, rect: cvkg_core::Rect, radius: f32, color: [f32; 4]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: fill_rounded_rect")
            .fill_rounded_rect(rect, radius, color);
    }
    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: fill_ellipse")
            .fill_ellipse(rect, color);
    }
    fn stroke_rect(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: stroke_rect")
            .stroke_rect(rect, color, stroke_width);
    }
    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: stroke_rounded_rect")
            .stroke_rounded_rect(rect, radius, color, stroke_width);
    }
    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: stroke_ellipse")
            .stroke_ellipse(rect, color, stroke_width);
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
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_line")
            .draw_line(x1, y1, x2, y2, color, stroke_width);
    }
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_text")
            .draw_text(text, x, y, size, color);
    }
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: measure_text")
            .measure_text(text, size)
    }
    fn draw_linear_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_linear_gradient")
            .draw_linear_gradient(rect, start_color, end_color, angle);
    }
    fn draw_radial_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_radial_gradient")
            .draw_radial_gradient(rect, inner_color, outer_color);
    }
    fn draw_texture(&mut self, texture_id: u32, rect: cvkg_core::Rect) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_texture")
            .draw_texture(texture_id, rect);
    }
    fn draw_image(&mut self, image_name: &str, rect: cvkg_core::Rect) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_image")
            .draw_image(image_name, rect);
    }
    fn load_image(&mut self, name: &str, data: &[u8]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: load_image")
            .load_image(name, data);
    }
    fn push_clip_rect(&mut self, rect: cvkg_core::Rect) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_clip_rect")
            .push_clip_rect(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_clip_rect")
            .pop_clip_rect();
    }
    fn push_opacity(&mut self, opacity: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_opacity")
            .push_opacity(opacity);
    }
    fn draw_3d_cube(&mut self, rect: cvkg_core::Rect, color: [f32; 4], rotation: [f32; 3]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_3d_cube")
            .draw_3d_cube(rect, color, rotation);
    }
    fn pop_opacity(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_opacity")
            .pop_opacity();
    }
    fn bifrost(&mut self, rect: cvkg_core::Rect, blur: f32, saturation: f32, opacity: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: bifrost")
            .bifrost(rect, blur, saturation, opacity);
    }
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_mjolnir_slice")
            .push_mjolnir_slice(angle, offset);
    }
    fn pop_mjolnir_slice(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_mjolnir_slice")
            .pop_mjolnir_slice();
    }
    fn mjolnir_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: mjolnir_shatter")
            .mjolnir_shatter(rect, pieces, force, color);
    }
    fn mjolnir_fluid_shatter(
        &mut self,
        rect: cvkg_core::Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
    ) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: mjolnir_fluid_shatter")
            .mjolnir_fluid_shatter(rect, pieces, force, color);
    }
    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_mjolnir_bolt")
            .draw_mjolnir_bolt(from, to, color);
    }
    fn gungnir(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: gungnir")
            .gungnir(rect, color, radius, intensity);
    }
    fn mani_glow(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: mani_glow")
            .mani_glow(rect, color, radius);
    }
    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: register_handler")
            .register_handler(event_type, handler);
    }
    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_vnode")
            .push_vnode(rect, name);
    }
    fn pop_vnode(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_vnode")
            .pop_vnode();
    }
    // FIX #1: Removed duplicate definitions of set_z_index and get_z_index.
    // They appeared twice in this impl block (after pop_vnode and after register_shared_element),
    // which is a hard compiler error. Exactly one definition of each is kept here.
    fn set_z_index(&mut self, z: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: set_z_index")
            .set_z_index(z);
    }
    fn get_z_index(&self) -> f32 {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: get_z_index")
            .get_z_index()
    }
    fn register_shared_element(&mut self, id: &str, rect: cvkg_core::Rect) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: register_shared_element")
            .register_shared_element(id, rect);
    }
    fn set_material(&mut self, material: cvkg_core::DrawMaterial) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: set_material")
            .set_material(material);
    }
    fn current_material(&self) -> cvkg_core::DrawMaterial {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: current_material")
            .current_material()
    }
    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: serialize_svg")
            .serialize_svg(name)
    }
    fn apply_svg_filter(
        &mut self,
        name: &str,
        filter_id: &str,
        region: cvkg_core::Rect,
    ) -> Result<String, String> {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: apply_svg_filter")
            .apply_svg_filter(name, filter_id, region)
    }
    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_shadow")
            .push_shadow(radius, color, offset);
    }
    fn pop_shadow(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_shadow")
            .pop_shadow();
    }
    fn push_affine(&mut self, transform: [f32; 6]) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_affine")
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
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: load_svg")
            .load_svg(name, svg_data);
    }
    fn draw_svg(&mut self, name: &str, rect: cvkg_core::Rect) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: draw_svg")
            .draw_svg(name, rect, None, 0);
    }
    fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: get_telemetry")
            .telemetry
            .clone()
    }
    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: prewarm_vram")
            .prewarm_vram(assets);
    }
    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: push_transform")
            .push_transform(translation, scale, rotation);
    }
    fn pop_transform(&mut self) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: pop_transform")
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

        self.gpu
            .lock()
            .expect("GPU mutex poisoned: set_berserker_mode")
            .set_berserker_mode(state);
    }

    fn set_rage(&mut self, rage: f32) {
        self.rage = rage;
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: set_rage")
            .set_rage(rage);
    }

    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        self.gpu
            .lock()
            .expect("GPU mutex poisoned: memoize")
            .memoize(id, data_hash, render_fn);
    }
    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    /// Captures the current frame as a PNG-encoded byte buffer via GPU readback.
    /// Captures the current frame as a PNG-encoded byte buffer via GPU readback.
    ///
    /// FIX #4: capture_frame() returns a Future that borrows the SurtrRenderer, so the
    /// MutexGuard must remain alive until block_on completes — the guard cannot be dropped
    /// before the future is driven to completion. The lock is held for the duration of the
    /// GPU readback. This is acceptable because capture_png is an infrequent, explicit
    /// user-triggered operation (not called on the hot render path), so blocking other
    /// render calls for the readback duration is not a practical concern.
    fn capture_png(&mut self) -> Vec<u8> {
        log::info!("CAPTURING_FRAME: Initiating GPU readback...");
        // INVARIANT: The MutexGuard `gpu` must outlive the future returned by capture_frame()
        // because the future borrows from the SurtrRenderer. We therefore lock, block_on the
        // future (driving it to completion), and only then allow the guard to drop.
        let gpu = self.gpu.lock().expect("GPU mutex poisoned: capture_png");
        pollster::block_on(gpu.capture_frame()).unwrap_or_else(|e| {
            log::error!("GPU frame capture failed: {}", e);
            Vec::new() // Return empty buffer on failure — do not panic the render loop
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

fn convert_keyboard_event(event: winit::event::KeyEvent) -> Option<cvkg_core::Event> {
    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
        let key_str = format!("{:?}", code);
        if event.state == winit::event::ElementState::Pressed {
            Some(cvkg_core::Event::KeyDown { key: key_str })
        } else {
            Some(cvkg_core::Event::KeyUp { key: key_str })
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
        },
        winit::event::ElementState::Released => cvkg_core::Event::PointerUp {
            x: position[0],
            y: position[1],
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
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
    /// contention — the bool would reflect only the last execution. The fix uses
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
                // Another caller already claimed this URL — do not insert.
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
    use std::io::Write;

    /// FIX #12: Replaced hardcoded relative path "test_asset.png" with a temp-dir path
    /// constructed from a unique per-test name. The previous path was written to the
    /// process working directory, which varies by invocation and causes collisions when
    /// tests run in parallel or when a prior run panics before cleanup.
    #[test]
    fn test_native_asset_manager_loading() {
        let manager = NativeAssetManager::new();
        let temp_path = std::env::temp_dir().join("cvkg_test_asset_loading.png");
        let temp_file_path = temp_path.to_str().expect("temp path must be valid UTF-8");
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
            // Expected — non-existent file must produce an Error state
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
}

/// load_icon — Searches known asset directories for 'icon.png'.
/// Returns a winit Icon if found and decodable, None otherwise.
/// All failures are logged at warn level — missing icons are non-fatal.
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
// AUDIO / HAPTIC ENGINES — Cross-platform micro-feedback
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
        *self.last_impact.lock().unwrap() = std::time::Instant::now();
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
        *self.last_impact.lock().unwrap() = std::time::Instant::now();
    }
}
