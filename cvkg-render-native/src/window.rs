use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::main_loop::AppEvent;
use cvkg_core::{FocusManager, FocusableId, WindowConfig, WindowHandle, WindowId as CoreWindowId};

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
    pub fn update_from_window(&mut self, window: &Window) -> Option<WindowState> {
        let old_state = self.state;
        if window.is_minimized().unwrap_or(false) {
            self.state = WindowState::Minimized;
        } else if window.fullscreen().is_some() {
            self.state = WindowState::Fullscreen;
        } else if self.state == WindowState::Minimized || self.state == WindowState::Fullscreen {
            self.state = WindowState::Normal;
        }
        if self.state != old_state {
            Some(self.state)
        } else {
            None
        }
    }

    /// Returns `true` if the window should render a frame in the current state.
    pub fn should_render(&self) -> bool {
        !matches!(
            self.state,
            WindowState::Occluded | WindowState::Minimized | WindowState::Hidden
        )
    }

    /// Returns the appropriate [`ControlFlow`] for the current state.
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
pub struct ResizeHitTest {
    window_size: winit::dpi::PhysicalSize<u32>,
    corner_radius: f32,
    expansion: f32,
}

impl ResizeHitTest {
    /// Creates a new hit-test helper.
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

    /// Tests whether `pos` falls within the expanded resize-hit region.
    pub fn hit_test(&self, pos: winit::dpi::PhysicalPosition<f32>, corner_radius: f32) -> bool {
        let r = corner_radius + self.expansion;
        let w = self.window_size.width as f32;
        let h = self.window_size.height as f32;
        let px = pos.x;
        let py = pos.y;

        if px <= r && py <= r {
            return true;
        }
        if px >= w - r && py <= r {
            return true;
        }
        if px <= r && py >= h - r {
            return true;
        }
        if px >= w - r && py >= h - r {
            return true;
        }
        false
    }
}

/// Platform safe area insets (menu bar, notch, etc.).
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

/// Native implementation of the cvkg_core::Window trait.
pub struct NativeWindowWrapper {
    pub(crate) winit_id: WindowId,
    pub(crate) window: Arc<Window>,
    pub(crate) proxy: EventLoopProxy<AppEvent>,
    pub(crate) is_key: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) is_main: bool,
}

impl cvkg_core::Window for NativeWindowWrapper {
    fn close(&self) {
        let _ = self.proxy.send_event(AppEvent::CloseWindow(self.winit_id));
    }

    fn set_title(&self, title: &str) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetTitle(self.winit_id, title.to_string()));
    }

    fn set_size(&self, width: f32, height: f32) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetSize(self.winit_id, width, height));
    }

    fn is_key(&self) -> bool {
        self.is_key.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn is_main(&self) -> bool {
        self.is_main
    }

    fn is_visible(&self) -> bool {
        self.window.is_visible().unwrap_or(false)
    }

    fn set_visible(&self, visible: bool) {
        let _ = self
            .proxy
            .send_event(AppEvent::SetVisible(self.winit_id, visible));
    }

    fn bring_to_front(&self) {
        let _ = self.proxy.send_event(AppEvent::BringToFront(self.winit_id));
    }
}

/// Dynamic manager for all active native windows and their rendering contexts.
pub struct WindowManager {
    pub windows: std::collections::HashMap<WindowId, WindowData>,
    pub window_stack: Vec<WindowId>,
    pub winit_to_core: std::collections::HashMap<WindowId, CoreWindowId>,
    pub core_to_winit: std::collections::HashMap<CoreWindowId, WindowId>,
    pub next_core_id: u64,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: std::collections::HashMap::new(),
            window_stack: Vec::new(),
            winit_to_core: std::collections::HashMap::new(),
            core_to_winit: std::collections::HashMap::new(),
            next_core_id: 1,
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        gpu: &Option<Arc<std::sync::Mutex<cvkg_render_gpu::GpuRenderer>>>,
        proxy: EventLoopProxy<AppEvent>,
        config: WindowConfig,
        is_main: bool,
        view: &impl cvkg_core::View,
    ) -> WindowHandle {
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
            use winit::platform::windows::WindowAttributesExtWindows;
            window_attrs = window_attrs.with_undecorated_shadow(true);
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("failed to create native window"),
        );

        let winit_id = window.id();
        let core_id = CoreWindowId(self.next_core_id);
        self.next_core_id += 1;

        let is_key_focused = Arc::new(std::sync::atomic::AtomicBool::new(true));

        let wrapper = Arc::new(NativeWindowWrapper {
            winit_id,
            window: window.clone(),
            proxy: proxy.clone(),
            is_key: is_key_focused.clone(),
            is_main,
        });

        let handle = WindowHandle::new(core_id, wrapper);

        let vdom = cvkg_vdom::VDom::build(
            view,
            cvkg_core::Rect::new(0.0, 0.0, config.size.0, config.size.1),
        );

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
            focus_manager: FocusManager::new(),
            focused_node_id: None,
            last_touch_time: None,
            last_bounds: None,
        };

        self.windows.insert(winit_id, data);
        self.window_stack.push(winit_id);
        self.winit_to_core.insert(winit_id, core_id);
        self.core_to_winit.insert(core_id, winit_id);

        if let Some(gpu_mutex) = gpu {
            gpu_mutex
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .register_window(window.clone());
        }

        handle
    }

    pub fn close_window(&mut self, winit_id: WindowId) {
        self.windows.remove(&winit_id);
        self.window_stack.retain(|id| *id != winit_id);
        if let Some(core_id) = self.winit_to_core.remove(&winit_id) {
            self.core_to_winit.remove(&core_id);
        }
    }

    pub fn bring_to_front(&mut self, winit_id: WindowId) {
        self.window_stack.retain(|id| *id != winit_id);
        self.window_stack.push(winit_id);
        if let Some(data) = self.windows.get(&winit_id) {
            data.window.focus_window();
        }
    }

    pub fn window(&self, winit_id: WindowId) -> Option<&WindowData> {
        self.windows.get(&winit_id)
    }

    pub fn window_mut(&mut self, winit_id: WindowId) -> Option<&mut WindowData> {
        self.windows.get_mut(&winit_id)
    }

    pub fn window_order(&self) -> &[WindowId] {
        &self.window_stack
    }
}

pub struct WindowData {
    pub(crate) window: Arc<Window>,
    pub(crate) accesskit_adapter: Option<accesskit_winit::Adapter>,
    pub(crate) vdom: Option<cvkg_vdom::VDom>,
    pub(crate) cursor_pos: [f32; 2],
    pub(crate) cursor_velocity: [f32; 2],
    pub(crate) last_redraw_start: std::time::Instant,
    pub(crate) frame_history: std::collections::VecDeque<f32>,
    pub(crate) frame_count: u64,
    pub(crate) last_pos: Option<[i32; 2]>,
    pub(crate) needs_cursor_update: bool,
    pub(crate) is_dragging: bool,
    pub(crate) drag_start_pos: [f32; 2],
    pub(crate) drag_button: u32,
    pub(crate) drag_threshold: f32,
    pub(crate) active_pointer_target: Option<cvkg_vdom::NodeId>,
    pub(crate) active_pointer_target_type: Option<String>,
    pub(crate) active_pointer_target_key: Option<String>,
    pub(crate) active_pointer_pos: Option<[f32; 2]>,
    pub(crate) active_pointer_precision: f32,
    pub(crate) is_key_focused: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) is_main: bool,
    pub(crate) core_id: CoreWindowId,
    pub(crate) window_handle: WindowHandle,
    pub(crate) focus_manager: FocusManager,
    pub(crate) focused_node_id: Option<cvkg_vdom::NodeId>,
    pub(crate) last_touch_time: Option<std::time::Instant>,
    pub(crate) last_bounds: Option<cvkg_core::Rect>,
}

// =============================================================================
// Window Capability Matrix and Multi-Monitor configurations
// =============================================================================

/// Window type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Document,
    Panel,
    Popover,
    Dialog,
    Tooltip,
}

/// Window capability matrix per platform.
#[derive(Debug, Clone)]
pub struct WindowCapabilityMatrix {
    pub platform: &'static str,
    pub window_types: Vec<WindowType>,
    pub tabbed_windows: bool,
    pub tiled_windows: bool,
    pub floating_panels: bool,
    pub sheets: bool,
}

impl WindowCapabilityMatrix {
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

/// Monitor configuration.
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub name: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub scale_factor: f64,
    pub refresh_rate: u32,
}

/// Manages multi-monitor layouts.
#[derive(Debug, Clone)]
pub struct MultiMonitorManager {
    monitors: Vec<MonitorConfig>,
    current_monitor_index: usize,
}

impl MultiMonitorManager {
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

    pub fn current_monitor(&self) -> &MonitorConfig {
        &self.monitors[self.current_monitor_index]
    }

    pub fn monitors(&self) -> &[MonitorConfig] {
        &self.monitors
    }

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

    pub fn scale_dimensions(&self, logical_width: f64, logical_height: f64) -> (u32, u32) {
        let sf = self.current_monitor().scale_factor;
        (
            (logical_width * sf).round() as u32,
            (logical_height * sf).round() as u32,
        )
    }

    pub fn requires_dpi_adaptation(&self, from_index: usize, to_index: usize) -> bool {
        if from_index < self.monitors.len() && to_index < self.monitors.len() {
            (self.monitors[from_index].scale_factor - self.monitors[to_index].scale_factor).abs()
                > f64::EPSILON
        } else {
            false
        }
    }
}
