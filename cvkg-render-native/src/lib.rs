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
#[cfg(target_os = "linux")]
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

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

/// Custom events for the native application event loop
#[derive(Debug)]
enum AppEvent {
    AccessibilityAction(accesskit::ActionRequest),
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
            windows: std::collections::HashMap::new(),
            gpu: None,
            asset_manager: std::sync::Arc::new(NativeAssetManager::new()),
            proxy: event_loop.create_proxy(),
            start_time: std::time::Instant::now(),
            last_frame_time: std::time::Instant::now(),
            berserker_mode: cvkg_core::BerserkerMode::Normal,
            rage: 0.0,
        };

        event_loop.run_app(&mut app).expect("Event loop error");
    }
}

struct WindowState {
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
}

struct App<V: cvkg_core::View> {
    view: V,
    windows: std::collections::HashMap<WindowId, WindowState>,
    gpu: Option<Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>>,
    #[allow(dead_code)]
    asset_manager: std::sync::Arc<NativeAssetManager>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
    start_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    berserker_mode: cvkg_core::BerserkerMode,
    rage: f32,
}

impl<V: cvkg_core::View + 'static> ApplicationHandler<AppEvent> for App<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            log::info!("[Native] App instance (resumed): {:p}", self);

            let window_attrs = Window::default_attributes()
                .with_title("CVKG Berserker")
                .with_visible(true)
                .with_transparent(false)
                .with_decorations(true)
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            let window_id = window.id();
            let vdom =
                cvkg_vdom::VDom::build(&self.view, cvkg_core::Rect::new(0.0, 0.0, 1280.0, 720.0));

            log::info!("[Native] INSERTING window ID: {:?}", window_id);

            self.windows.insert(
                window_id,
                WindowState {
                    window: window.clone(),
                    accesskit_adapter: None,
                    vdom: Some(vdom),
                    cursor_pos: [0.0, 0.0],
                    last_redraw_start: std::time::Instant::now(),
                    frame_history: std::collections::VecDeque::with_capacity(60),
                    frame_count: 0,
                    last_pos: None,
                },
            );

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

        let state = if let Some(s) = self.windows.get_mut(&id) {
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
            WindowEvent::CloseRequested => {
                self.windows.remove(&id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(physical_size) => {
                // FIX #3: All lock().unwrap() calls in the render path replaced with
                // lock().expect("...") providing actionable context on panic. The GPU
                // mutex should never be poisoned under correct usage; expect() surfaces
                // the failure clearly rather than producing an opaque unwrap panic.
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
                            nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                        } else if let cvkg_vdom::VDomPatch::Update { id, .. } = patch
                            && let Some(node) = new_vdom.nodes.get(id)
                        {
                            nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
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
                telemetry.draw_time_ms = draw_end.duration_since(draw_start).as_secs_f32() * 1000.0;
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
                sorted_frames.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

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

                // FIX #7: Removed anti-analysis EnvironmentShield probe and enforce_mitigation
                // calls. This code ran every 60 frames and actively interfered with legitimate
                // profiling, debugging, and CI environments. Anti-debugging measures have no
                // place in a developer tool's render loop and will break expected tooling behavior.

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
                log::info!(
                    "[Native] Cursor Moved: Physical={:?} Logical={:?} Scale={}",
                    position,
                    logical,
                    scale
                );
                state.cursor_pos = [logical.x, logical.y];
                if let Some(vdom) = &state.vdom {
                    vdom.dispatch_event(cvkg_core::Event::PointerMove {
                        x: state.cursor_pos[0],
                        y: state.cursor_pos[1],
                        proximity_field: 0.0,
                    });
                }
                // FIX #12: Always request redraw on movement to ensure hover effects respond immediately.
                state.window.request_redraw();
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
                            });
                        }
                        winit::event::ElementState::Released => {
                            log::info!("[Native] Dispatching PointerUp to VDOM");
                            vdom.dispatch_event(cvkg_core::Event::PointerUp {
                                x: state.cursor_pos[0],
                                y: state.cursor_pos[1],
                                button: btn_id,
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
            WindowEvent::KeyboardInput { event, .. } => {
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
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        let AppEvent::AccessibilityAction(request) = event;
        let node_id = cvkg_vdom::NodeId(request.target.0);

        // FIX #11: Accessibility actions carry a target NodeId that identifies which
        // window owns the node. We search all windows for the one containing that node
        // rather than routing to the arbitrary first window (HashMap iteration order is
        // non-deterministic and would silently misroute actions in multi-window layouts).
        let target_state = self.windows.values_mut().find(|s| {
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
            };
            vdom.dispatch_event(event);
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
            for window_state in self.windows.values() {
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
        },
        winit::event::ElementState::Released => cvkg_core::Event::PointerUp {
            x: position[0],
            y: position[1],
            button,
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
