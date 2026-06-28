use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::asset_manager::NativeAssetManager;
use crate::audio::{RodioAudioEngine, VisualHapticEngine};
use crate::events::{convert_ime_event, convert_keyboard_event};
use crate::renderer::{GpuFramePtrGuard, GPU_FRAME_PTR, NativeRenderer};
use crate::window::{SafeAreaInsets, WindowManager, WindowState, WindowStateDetector};
use cvkg_core::{
    AccessibilityPreferences, ColorTheme, FocusableId, FrameBudgetTracker, FrameRenderer,
    RenderIntensityMode, Renderer, TelemetryData, View, WindowConfig, detect_system_theme,
    set_accessibility_preferences, update_system_state,
};

/// Custom events for the native application event loop, handling accessibility
/// callbacks and routing window lifecycle control events from background threads.
#[derive(Debug)]
pub enum AppEvent {
    /// Action request from the accessibility subsystem.
    AccessibilityAction(accesskit::ActionRequest),
    /// Request to close a specific window.
    CloseWindow(WindowId),
    /// Request to set the title bar string of a window.
    SetTitle(WindowId, String),
    /// Request to resize a window.
    SetSize(WindowId, f32, f32),
    /// Request to change visibility of a window.
    SetVisible(WindowId, bool),
    /// Request to bring a window to the front and focus it.
    BringToFront(WindowId),
    /// Initial accessibility tree requested by screen reader.
    AccessibilityInitialTreeRequested(WindowId),
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

pub struct App<V: View> {
    pub(crate) view: V,
    pub(crate) window_manager: WindowManager,
    pub(crate) gpu: Option<Arc<std::sync::Mutex<cvkg_render_gpu::GpuRenderer>>>,
    #[allow(dead_code)]
    pub(crate) asset_manager: std::sync::Arc<NativeAssetManager>,
    pub(crate) proxy: EventLoopProxy<AppEvent>,
    pub(crate) start_time: std::time::Instant,
    pub(crate) last_frame_time: std::time::Instant,
    pub(crate) berserker_mode: RenderIntensityMode,
    pub(crate) rage: f32,
    pub(crate) state_detector: WindowStateDetector,
    pub(crate) frame_budget: FrameBudgetTracker,
    pub(crate) modifiers: winit::keyboard::ModifiersState,
    pub(crate) audio_engine: Option<Arc<dyn cvkg_core::AudioEngine>>,
    pub(crate) haptic_engine: Arc<dyn cvkg_core::HapticEngine>,
    pub(crate) pending_prewarm: Option<Vec<(String, Vec<u8>)>>,
}

impl<V: View + 'static> ApplicationHandler<AppEvent> for App<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            let a11y_prefs = AccessibilityPreferences::detect_from_system();
            set_accessibility_preferences(a11y_prefs);
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

            let system_theme = detect_system_theme();
            log::info!("[Native] System theme detected: {:?}", system_theme);

            self.audio_engine =
                RodioAudioEngine::new().map(|e| Arc::new(e) as Arc<dyn cvkg_core::AudioEngine>);

            self.haptic_engine = Arc::new(VisualHapticEngine::new());

            log::info!("[Native] App instance (resumed): {:p}", self);

            let config = WindowConfig {
                title: "CVKG Gallery".to_string(),
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
                .unwrap_or_else(|| {
                    log::error!("[Native] winit_id not found for window handle: window may have been destroyed");
                    std::process::exit(1);
                });
            let window = self
                .window_manager
                .windows
                .get(&winit_id)
                .unwrap()
                .window
                .clone();

            let mut gpu = pollster::block_on(cvkg_render_gpu::GpuRenderer::forge(window.clone()));

            static PREFETCH_LABELS: &[(&str, f32)] = &[
                ("File", 13.0),
                ("Edit", 13.0),
                ("View", 13.0),
                ("Window", 13.0),
                ("Help", 13.0),
                ("Gallery", 14.0),
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
        if !matches!(cause, winit::event::StartCause::Poll) {
            log::trace!("[Native] Event Loop Wake: {:?}", cause);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if !matches!(event, DeviceEvent::MouseMotion { .. }) {
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
                    close_window = true;
                }
                WindowEvent::Resized(physical_size) => {
                    gpu_arc.lock().unwrap_or_else(|p| p.into_inner()).resize(
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

                    let redraw_start = std::time::Instant::now();
                    let last_redraw_start = state.last_redraw_start;
                    state.last_redraw_start = redraw_start;
                    self.frame_budget.new_frame();

                    let layout_start = std::time::Instant::now();
                    let view_changed = self.view.changed();

                     let bounds_changed = state.last_bounds.map_or(true, |b| b != rect);
                     let new_vdom: Option<cvkg_vdom::VDom> = if view_changed || bounds_changed {
                         state.last_bounds = Some(rect);
                         let vdom_start = std::time::Instant::now();
                         let vdom = cvkg_vdom::VDom::build(&self.view, rect);
                         let vdom_elapsed = vdom_start.elapsed();
                         if vdom_elapsed > std::time::Duration::from_millis(1) {
                             log::warn!(
                                 "[Native] VDom::build took {:?} ({} nodes)",
                                 vdom_elapsed,
                                 vdom.nodes.len()
                             );
                         }
                         Some(vdom)
                     } else {
                         None
                     };

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

                    let state_flush_start = std::time::Instant::now();
                    #[allow(unused_assignments)]
                    let mut diff_patches = None;
                    match (new_vdom, &mut state.vdom) {
                        (Some(new_vdom), Some(prev_vdom)) => {
                            let diff_start = std::time::Instant::now();
                            let patches = prev_vdom.diff(&new_vdom);
                            let diff_elapsed = diff_start.elapsed();
                            if diff_elapsed > std::time::Duration::from_millis(1) {
                                log::warn!(
                                    "[Native] VDom::diff took {:?} ({} patches)",
                                    diff_elapsed,
                                    patches.len()
                                );
                            }
                            diff_patches = Some(patches);
                            // ponytail: if diff returned None/empty, skip patching (no UI change this frame)
                            let patches = diff_patches.as_deref().unwrap_or_default();
                            let mut nodes = Vec::new();
                            for patch in patches {
                                if let cvkg_vdom::VDomPatch::Create(node)
                                | cvkg_vdom::VDomPatch::Replace { node, .. } = patch
                                {
                                    nodes.push((
                                        accesskit::NodeId(node.id.0),
                                        node.to_accesskit_node(),
                                    ));
                                } else if let cvkg_vdom::VDomPatch::Update { id, .. } = patch
                                    && let Some(node) = new_vdom.nodes.get(id)
                                {
                                    nodes.push((
                                        accesskit::NodeId(node.id.0),
                                        node.to_accesskit_node(),
                                    ));
                                } else if let cvkg_vdom::VDomPatch::Remove(id) = patch {
                                    state
                                        .focus_manager
                                        .unregister(&FocusableId::from(id.0.to_string()));
                                }
                            }
                            let focused_id = state
                                .focused_node_id
                                .map(|id| accesskit::NodeId(id.0))
                                .unwrap_or(accesskit::NodeId(1));
                            for patch in diff_patches.as_deref().unwrap_or_default() {
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
                            prev_vdom.apply_patches(diff_patches.unwrap_or_default());
                            state.vdom = Some(new_vdom);
                        }
                        (Some(new_vdom), None) => {
                            state.vdom = Some(new_vdom);
                        }
                        (None, _) => {}
                    }
                    let state_flush_end = std::time::Instant::now();
                    self.frame_budget.subsystem_finish(0);

                    let delta_time = redraw_start.duration_since(last_redraw_start).as_secs_f32();
                    let elapsed_time = redraw_start.duration_since(self.start_time).as_secs_f32();

                    let safe_area = SafeAreaInsets::for_window_state(self.state_detector.state());
                    let content_rect = cvkg_core::Rect {
                        x: safe_area.left,
                        y: safe_area.top,
                        width: rect.width - safe_area.left - safe_area.right,
                        height: rect.height - safe_area.top - safe_area.bottom,
                    };
                    let layout_deadline =
                        std::time::Instant::now() + self.frame_budget.allocations()[1].time_slice;
                    cvkg_core::LayoutCache::set_layout_budget_deadline(Some(layout_deadline));

                    let mut renderer = NativeRenderer::new(
                        state.window.clone(),
                        gpu_arc.clone(),
                        delta_time,
                        elapsed_time,
                        self.berserker_mode,
                        self.rage,
                    );

                    let cpu_draw_start = std::time::Instant::now();
                    let mut gpu = gpu_arc.lock().unwrap_or_else(|p| p.into_inner());
                    let gpu_lock_time = cpu_draw_start.elapsed().as_secs_f32() * 1000.0;

                    gpu.update_mouse(state.cursor_pos, state.cursor_velocity);

                    if let Some(assets) = self.pending_prewarm.take() {
                        log::info!(
                            "[Native] Pre-warming {} assets on first frame",
                            assets.len()
                        );
                        gpu.prewarm_vram(assets);
                    }

                    let encoder = gpu.begin_frame(id);
                    let begin_frame_time =
                        cpu_draw_start.elapsed().as_secs_f32() * 1000.0 - gpu_lock_time;

                    {
                        let raw: *mut cvkg_render_gpu::GpuRenderer = &mut *gpu;
                        // SAFETY: `gpu` MutexGuard outlives this guard (scope ends after render)
                        let _guard = unsafe { GpuFramePtrGuard::set(raw) };
                        let render_start = std::time::Instant::now();
                        self.view.render(&mut renderer, content_rect);
                        let render_time = render_start.elapsed().as_secs_f32() * 1000.0;
                        // _guard drops here, clearing GPU_FRAME_PTR even on panic
                        if render_time > 5.0 {
                            log::warn!(
                                "[Native] view.render() took {:.2}ms (gpu_lock={:.2}ms, begin_frame={:.2}ms)",
                                render_time,
                                gpu_lock_time,
                                begin_frame_time
                            );
                        }
                    }
                    let cpu_draw_end = std::time::Instant::now();
                    cvkg_core::LayoutCache::clear_layout_budget_deadline();

                    self.frame_budget.subsystem_finish(2);

                    let gpu_render_start = std::time::Instant::now();
                    gpu.render_frame();
                    let gpu_render_end = std::time::Instant::now();

                    gpu.end_frame(encoder);
                    let gpu_submit_end = std::time::Instant::now();

                    if state.frame_count % 60 == 0 {
                        let cpu_draw = cpu_draw_end.duration_since(cpu_draw_start);
                        let gpu_render = gpu_render_end.duration_since(gpu_render_start);
                        let gpu_submit = gpu_submit_end.duration_since(gpu_render_end);
                        let total = gpu_submit_end.duration_since(redraw_start);
                        log::info!(
                            "[Native] Frame breakdown: cpu_draw={:?} gpu_render={:?} gpu_submit(end_frame)={:?} total={:?}",
                            cpu_draw,
                            gpu_render,
                            gpu_submit,
                            total
                        );
                    }

                    let mut telemetry = TelemetryData::default();
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
                    telemetry.gpu_submit_time_ms =
                        gpu_submit_end.duration_since(cpu_draw_end).as_secs_f32() * 1000.0;

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
                        .map(|alloc| {
                            alloc.time_slice.as_secs_f32() * 1000.0 - telemetry.layout_time_ms
                        })
                        .unwrap_or(0.0);
                    telemetry.frame_over_budget = !self.frame_budget.frame_within_budget()
                        || telemetry.frame_budget_remaining_ms < 0.0;
                    telemetry.layout_over_budget = !self.frame_budget.is_within_budget(1)
                        || telemetry.layout_budget_remaining_ms < 0.0;

                    // Record frame budget telemetry
                    if telemetry.frame_over_budget {
                        log::warn!(
                            "[Telemetry] Frame budget exceeded by {:.2}ms (frame={:.2}ms budget={:.2}ms)",
                            telemetry.frame_time_ms - telemetry.frame_budget_ms,
                            telemetry.frame_time_ms,
                            telemetry.frame_budget_ms
                        );
                    }

                    log::info!(
                        "[Native] Frame timings: layout={:.2}ms state={:.2}ms draw={:.2}ms submit={:.2}ms total={:.2}ms",
                        telemetry.layout_time_ms,
                        telemetry.state_flush_time_ms,
                        telemetry.draw_time_ms,
                        telemetry.gpu_submit_time_ms,
                        telemetry.frame_time_ms
                    );

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

                        let avg = sorted_frames.iter().sum::<f32>() / sorted_frames.len() as f32;
                        let variance = sorted_frames.iter().map(|f| (f - avg).powi(2)).sum::<f32>()
                            / sorted_frames.len() as f32;
                        telemetry.frame_jitter_ms = variance.sqrt();
                    }

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
                    if !state.is_dragging {
                        let ddx = state.cursor_pos[0] - state.drag_start_pos[0];
                        let ddy = state.cursor_pos[1] - state.drag_start_pos[1];
                        let dist_sq = ddx * ddx + ddy * ddy;
                        if dist_sq > state.drag_threshold * state.drag_threshold {
                            state.is_dragging = true;
                        }
                    }
                    state.needs_cursor_update = true;
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
                                state.drag_start_pos = state.cursor_pos;
                                state.is_dragging = false;
                                state.drag_button = btn_id;
                                state.active_pointer_pos = Some(state.cursor_pos);
                                state.active_pointer_precision = 0.0;
                                state.active_pointer_target = vdom
                                    .hit_test(state.cursor_pos[0], state.cursor_pos[1], 0.0)
                                    .map(|(id, _)| id);
                                if let Some(target_id) = state.active_pointer_target {
                                    if let Some(node) = vdom.nodes.get(&target_id) {
                                        state.active_pointer_target_type =
                                            Some(node.component_type.clone());
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
                                        vdom.hit_test(
                                            pos[0],
                                            pos[1],
                                            state.active_pointer_precision,
                                        )
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
                                if !state.is_dragging {
                                    if let Some(target) = target {
                                        log::info!(
                                            "[Native] Dispatching PointerClick to VDOM (target={:?})",
                                            target
                                        );
                                        vdom.dispatch_event_to_target(target, pointer_click);
                                    } else {
                                        log::info!(
                                            "[Native] Dispatching PointerClick to VDOM (no target, bubbling)"
                                        );
                                        vdom.dispatch_event(pointer_click);
                                    }
                                } else {
                                    log::info!("[Native] Skipping PointerClick (is_dragging=true)");
                                }
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
                WindowEvent::Touch(touch) => {
                    state.last_touch_time = Some(std::time::Instant::now());
                    if let Some(vdom) = &state.vdom {
                        let scale = state.window.scale_factor();
                        let logical = touch.location.to_logical::<f32>(scale);
                        let x = logical.x;
                        let y = logical.y;
                        let touch_btn = 0;

                        match touch.phase {
                            winit::event::TouchPhase::Started => {
                                log::info!("[Native] Dispatching PointerDown (Touch) to VDOM");
                                state.drag_start_pos = [x, y];
                                state.is_dragging = false;
                                state.drag_button = touch_btn;
                                state.active_pointer_pos = Some([x, y]);
                                state.active_pointer_precision = 150.0;
                                state.active_pointer_target =
                                    vdom.hit_test(x, y, 150.0).map(|(id, _)| id);
                                if let Some(target_id) = state.active_pointer_target {
                                    if let Some(node) = vdom.nodes.get(&target_id) {
                                        state.active_pointer_target_type =
                                            Some(node.component_type.clone());
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
                                        vdom.hit_test(
                                            pos[0],
                                            pos[1],
                                            state.active_pointer_precision,
                                        )
                                        .map(|(id, _)| id)
                                    })
                                    .or_else(|| {
                                        vdom.hit_test(x, y, state.active_pointer_precision)
                                            .map(|(id, _)| id)
                                    });
                                let target = state
                                    .active_pointer_target
                                    .filter(|target| {
                                        vdom.nodes.get(target).map_or(false, |node| {
                                            Some(&node.component_type)
                                                == state.active_pointer_target_type.as_ref()
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
                                if !state.is_dragging {
                                    if let Some(target) = target {
                                        log::info!(
                                            "[Native] Dispatching PointerClick to VDOM (target={:?})",
                                            target
                                        );
                                        vdom.dispatch_event_to_target(target, pointer_click);
                                    } else {
                                        log::info!(
                                            "[Native] Dispatching PointerClick to VDOM (no target, bubbling)"
                                        );
                                        vdom.dispatch_event(pointer_click);
                                    }
                                } else {
                                    log::info!("[Native] Skipping PointerClick (is_dragging=true)");
                                }
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
                            let is_cmd = if cfg!(target_os = "macos") {
                                self.modifiers.super_key()
                            } else {
                                self.modifiers.control_key()
                            };
                            let is_shift = self.modifiers.shift_key();

                            if is_cmd {
                                match code {
                                    winit::keyboard::KeyCode::KeyZ => {
                                        if is_shift {
                                            log::info!("[Native] Shortcut: Redo (Cmd+Shift+Z)");
                                            let mut redo_action = None;
                                            update_system_state(|s| {
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
                                            update_system_state(|s| {
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
                                    winit::keyboard::KeyCode::KeyY
                                        if !cfg!(target_os = "macos") =>
                                    {
                                        log::info!("[Native] Shortcut: Redo (Ctrl+Y)");
                                        let mut redo_action = None;
                                        update_system_state(|s| {
                                            let mut s = s.clone();
                                            redo_action = s.undo_manager.redo();
                                            s
                                        });
                                        if let Some(action) = redo_action {
                                            action();
                                        }
                                        state.window.request_redraw();
                                    }
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
                                        quit_all = true;
                                    }
                                    winit::keyboard::KeyCode::KeyC => {
                                        log::info!("[Native] Shortcut: Copy (Cmd+C)");
                                        if let Some(vdom) = &state.vdom {
                                            vdom.dispatch_event(cvkg_core::Event::Copy);
                                        }
                                        state.window.request_redraw();
                                    }
                                    winit::keyboard::KeyCode::KeyV => {
                                        log::info!("[Native] Shortcut: Paste (Cmd+V)");
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
                                    winit::keyboard::KeyCode::F11 => {
                                        let is_fullscreen = state.window.fullscreen().is_some();
                                        if is_fullscreen {
                                            state.window.set_fullscreen(None);
                                            log::info!("[Native] Fullscreen OFF");
                                        } else {
                                            if let Some(monitor) = state.window.current_monitor() {
                                                if let Some(mode) = monitor.video_modes().next() {
                                                    let w = mode.size().width;
                                                    let h = mode.size().height;
                                                    let rr = mode.refresh_rate_millihertz();
                                                    state.window.set_fullscreen(Some(
                                                        winit::window::Fullscreen::Exclusive(mode),
                                                    ));
                                                    log::info!(
                                                        "[Native] Fullscreen ON (exclusive: {}x{}@{:?}Hz)",
                                                        w,
                                                        h,
                                                        rr
                                                    );
                                                }
                                            } else {
                                                state.window.set_fullscreen(Some(
                                                    winit::window::Fullscreen::Borderless(None),
                                                ));
                                                log::info!("[Native] Fullscreen ON (borderless)");
                                            }
                                        }
                                        state.window.request_redraw();
                                    }
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
                                    winit::keyboard::KeyCode::Tab => {
                                        if is_shift {
                                            if let Some(id) = state.focus_manager.focus_prev() {
                                                if let Ok(node_id) = id.as_str().parse::<u64>() {
                                                    state.focused_node_id =
                                                        Some(cvkg_core::KvasirId(node_id));
                                                    log::info!(
                                                        "[Native] Focus previous: {:?}",
                                                        node_id
                                                    );
                                                }
                                            }
                                        } else {
                                            if let Some(id) = state.focus_manager.focus_next() {
                                                if let Ok(node_id) = id.as_str().parse::<u64>() {
                                                    state.focused_node_id =
                                                        Some(cvkg_core::KvasirId(node_id));
                                                    log::info!(
                                                        "[Native] Focus next: {:?}",
                                                        node_id
                                                    );
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
                    update_system_state(|st| {
                        let mut new_st = st.clone();
                        new_st.modifiers_shift = shift;
                        new_st.modifiers_ctrl = ctrl;
                        new_st.modifiers_alt = alt;
                        new_st.modifiers_logo = logo;
                        new_st
                    });
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    if let Some(ctx) = self.window_manager.windows.get(&id) {
                        ctx.window.request_redraw();
                    }
                }
                _ => {}
            }
        }

        if close_window {
            self.window_manager.close_window(id);
        }
        if quit_all {
            for wid in self.window_manager.window_order().to_vec() {
                self.window_manager.close_window(wid);
            }
        }
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
                WindowConfig {
                    title: "New CVKG Window".to_string(),
                    size: (800.0, 600.0),
                    ..Default::default()
                },
                false,
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
                        button: 0,
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
        self.rage = (self.rage - 0.02).max(0.0);

        let now = std::time::Instant::now();
        let target_interval = std::time::Duration::from_micros(8_333);

        if now.duration_since(self.last_frame_time) >= target_interval {
            self.last_frame_time = now;
            let needs_redraw = self.view.changed();
            if needs_redraw {
                for window_state in self.window_manager.windows.values() {
                    window_state.window.request_redraw();
                }
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + target_interval));
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                self.last_frame_time + target_interval,
            ));
        }
    }
}

pub struct ShieldWall {
    pub(crate) proxy: EventLoopProxy<AppEvent>,
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
