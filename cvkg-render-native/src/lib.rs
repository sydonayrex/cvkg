//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! Platform-native widget delegation using winit and AccessKit
//!
//! This crate provides platform-specific rendering backends for native desktop targets
//  using winit for window/event handling and AccessKit for accessibility tree integration.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};


/// Native renderer backend implementing the Renderer trait.
/// It wraps a shared SurtrRenderer for high-performance GPU drawing.
pub struct NativeRenderer {
    gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>,
    delta_time: f32,
    elapsed_time: f32,
}

/// Custom events for the native application event loop
#[derive(Debug)]
enum AppEvent {
    AccessibilityAction(accesskit::ActionRequest),
}

impl NativeRenderer {
    /// Create a new NativeRenderer (internal use by App)
    fn new(_window: Arc<Window>, gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>, delta_time: f32, elapsed_time: f32) -> Self {
        Self { gpu, delta_time, elapsed_time }
    }


    /// Start the CVKG native application with the given view.
    /// This is the main entry point for desktop applications.
    pub fn run<V: cvkg_core::View + 'static>(view: V) {
        let event_loop = EventLoop::<AppEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App {
            view,
            windows: std::collections::HashMap::new(),
            gpu: None,
            asset_manager: std::sync::Arc::new(NativeAssetManager::new()),
            proxy: event_loop.create_proxy(),
            start_time: std::time::Instant::now(),
        };

        event_loop.run_app(&mut app).expect("Event loop error");
    }
}

struct WindowState {
    window: Arc<Window>,
    accesskit_adapter: Option<accesskit_winit::Adapter>,
    vdom: Option<cvkg_vdom::VDom>,
    cursor_pos: [f32; 2],
    /// The instant the last redraw finished, used for measuring inter-frame timing.
    last_redraw_start: std::time::Instant,
}

struct App<V: cvkg_core::View> {
    view: V,
    windows: std::collections::HashMap<WindowId, WindowState>,
    gpu: Option<Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>>,
    asset_manager: std::sync::Arc<NativeAssetManager>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
    start_time: std::time::Instant,
}

impl<V: cvkg_core::View + 'static> ApplicationHandler<AppEvent> for App<V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("CVKG Forge")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );
            window.set_ime_allowed(true);

            let adapter = accesskit_winit::Adapter::with_direct_handlers(
                event_loop,
                &window,
                ShieldWall { proxy: self.proxy.clone() },
                ShieldWall { proxy: self.proxy.clone() },
                ShieldWall { proxy: self.proxy.clone() },
            );

            let rt = tokio::runtime::Runtime::new().unwrap();
            let gpu = rt.block_on(cvkg_render_gpu::SurtrRenderer::forge(window.clone()));
            let gpu = Arc::new(std::sync::Mutex::new(gpu));
            self.gpu = Some(gpu);

            self.windows.insert(window.id(), WindowState {
                window,
                accesskit_adapter: Some(adapter),
                vdom: Some(cvkg_vdom::VDom::new()),
                cursor_pos: [0.0, 0.0],
                last_redraw_start: std::time::Instant::now(),
            });

            cvkg_core::env::insert::<cvkg_core::AssetKey>(self.asset_manager.clone());
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let gpu_arc = if let Some(g) = &self.gpu { g.clone() } else { return };
        let state = if let Some(s) = self.windows.get_mut(&id) { s } else { return };

        match event {
            WindowEvent::CloseRequested => {
                self.windows.remove(&id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(physical_size) => {
                gpu_arc.lock().unwrap().resize(
                    id,
                    physical_size.width,
                    physical_size.height,
                    state.window.scale_factor() as f32,
                );
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = state.window.inner_size();
                let scale = state.window.scale_factor();
                let logical_size = size.to_logical::<f32>(scale);

                let rect = cvkg_core::Rect {
                    x: 0.0,
                    y: 0.0,
                    width: logical_size.width,
                    height: logical_size.height,
                };

                // Start timing for this redraw
                let redraw_start = std::time::Instant::now();
                
                // Build new vdom and diff (layout pass)
                let layout_start = std::time::Instant::now();
                let new_vdom = cvkg_vdom::VDom::build(&self.view, rect);
                let layout_end = std::time::Instant::now();

                // Apply patches
                let state_flush_start = std::time::Instant::now();
                if let Some(prev_vdom) = &mut state.vdom {
                    let patches = prev_vdom.diff(&new_vdom);
                    if let Some(adapter) = &mut state.accesskit_adapter {
                        let mut nodes = Vec::new();
                        for patch in &patches {
                            if let cvkg_vdom::VDomPatch::Create(node) | cvkg_vdom::VDomPatch::Replace { node, .. } = patch {
                                nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                            } else if let cvkg_vdom::VDomPatch::Update { id, .. } = patch
                                && let Some(node) = new_vdom.nodes.get(id) {
                                nodes.push((accesskit::NodeId(node.id.0), node.to_accesskit_node()));
                            }
                        }
                        if !nodes.is_empty() {
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
                let delta_time = redraw_start.duration_since(state.last_redraw_start).as_secs_f32();
                let elapsed_time = redraw_start.duration_since(self.start_time).as_secs_f32();
                let mut gpu = gpu_arc.lock().unwrap();
                let encoder = gpu.begin_frame(id);
                let mut renderer = NativeRenderer::new(state.window.clone(), gpu_arc.clone(), delta_time, elapsed_time);
                self.view.render(&mut renderer, rect);
                let draw_end = std::time::Instant::now();

                // Submission
                let gpu_submit_start = std::time::Instant::now();
                gpu.end_frame(encoder);
                let gpu_submit_end = std::time::Instant::now();

                // Update telemetry
                let mut telemetry = gpu.telemetry.clone();
                // input_time_ms uses the previous frame's completion to this frame's start as a proxy
                telemetry.input_time_ms = redraw_start.duration_since(state.last_redraw_start).as_secs_f32() * 1000.0;
                telemetry.layout_time_ms = layout_end.duration_since(layout_start).as_secs_f32() * 1000.0;
                telemetry.state_flush_time_ms = state_flush_end.duration_since(state_flush_start).as_secs_f32() * 1000.0;
                telemetry.draw_time_ms = draw_end.duration_since(draw_start).as_secs_f32() * 1000.0;
                telemetry.gpu_submit_time_ms = gpu_submit_end.duration_since(gpu_submit_start).as_secs_f32() * 1000.0;
                
                // Total frame time
                telemetry.frame_time_ms = gpu_submit_end.duration_since(redraw_start).as_secs_f32() * 1000.0;
                
                gpu.telemetry = telemetry;
                state.last_redraw_start = gpu_submit_end;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = state.window.scale_factor();
                let logical = position.to_logical::<f32>(scale);
                state.cursor_pos = [logical.x, logical.y];
                if let Some(vdom) = &state.vdom {
                    vdom.dispatch_event(cvkg_core::Event::PointerMove {
                        x: state.cursor_pos[0],
                        y: state.cursor_pos[1],
                    });
                }
            }
            WindowEvent::MouseInput { state: mouse_state, .. } => {
                if let Some(vdom) = &state.vdom {
                    let event = match mouse_state {
                        winit::event::ElementState::Pressed => {
                            cvkg_core::Event::PointerDown {
                                x: state.cursor_pos[0],
                                y: state.cursor_pos[1],
                            }
                        }
                        winit::event::ElementState::Released => cvkg_core::Event::PointerUp {
                            x: state.cursor_pos[0],
                            y: state.cursor_pos[1],
                        },
                    };
                    vdom.dispatch_event(event);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(vdom) = &state.vdom
                    && let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        let key_str = format!("{:?}", code);
                        let cvkg_event = if event.state == winit::event::ElementState::Pressed {
                            cvkg_core::Event::KeyDown { key: key_str }
                        } else {
                            cvkg_core::Event::KeyUp { key: key_str }
                        };
                        vdom.dispatch_event(cvkg_event);
                }
            }
            WindowEvent::Ime(ime_event) => {
                if let Some(vdom) = &state.vdom
                    && let winit::event::Ime::Commit(string) = ime_event {
                        vdom.dispatch_event(cvkg_core::Event::Ime(string));
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        let AppEvent::AccessibilityAction(request) = event;
            let node_id = cvkg_vdom::NodeId(request.target.0);
            // For accessibility, we'll route to the first window for now
            if let Some(state) = self.windows.values_mut().next()
                && let Some(vdom) = &state.vdom
                && let Some(node) = vdom.nodes.get(&node_id)
                && request.action == accesskit::Action::Click {
                    let event = cvkg_core::Event::PointerClick {
                        x: node.layout.x + node.layout.width / 2.0,
                        y: node.layout.y + node.layout.height / 2.0,
                    };
                    vdom.dispatch_event(event);
            }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for state in self.windows.values() {
            state.window.request_redraw();
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
        self.gpu.lock().unwrap().fill_rect(rect, color);
    }
    fn fill_rounded_rect(&mut self, rect: cvkg_core::Rect, radius: f32, color: [f32; 4]) {
        self.gpu.lock().unwrap().fill_rounded_rect(rect, radius, color);
    }
    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4]) {
        self.gpu.lock().unwrap().fill_ellipse(rect, color);
    }
    fn stroke_rect(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu.lock().unwrap().stroke_rect(rect, color, stroke_width);
    }
    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.gpu.lock().unwrap().stroke_rounded_rect(rect, radius, color, stroke_width);
    }
    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, color: [f32; 4], stroke_width: f32) {
        self.gpu.lock().unwrap().stroke_ellipse(rect, color, stroke_width);
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
        self.gpu.lock().unwrap().draw_line(x1, y1, x2, y2, color, stroke_width);
    }
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.gpu.lock().unwrap().draw_text(text, x, y, size, color);
    }
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.gpu.lock().unwrap().measure_text(text, size)
    }
    fn draw_texture(&mut self, texture_id: u32, rect: cvkg_core::Rect) {
        self.gpu.lock().unwrap().draw_texture(texture_id, rect);
    }
    fn draw_image(&mut self, image_name: &str, rect: cvkg_core::Rect) {
        self.gpu.lock().unwrap().draw_image(image_name, rect);
    }
    fn load_image(&mut self, name: &str, data: &[u8]) {
        self.gpu.lock().unwrap().load_image(name, data);
    }
    fn push_clip_rect(&mut self, rect: cvkg_core::Rect) {
        self.gpu.lock().unwrap().push_clip_rect(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.gpu.lock().unwrap().pop_clip_rect();
    }
    fn push_opacity(&mut self, opacity: f32) {
        self.gpu.lock().unwrap().push_opacity(opacity);
    }
    fn pop_opacity(&mut self) {
        self.gpu.lock().unwrap().pop_opacity();
    }
    fn bifrost(&mut self, rect: cvkg_core::Rect, blur: f32, saturation: f32, opacity: f32) {
        self.gpu.lock().unwrap().bifrost(rect, blur, saturation, opacity);
    }
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.gpu.lock().unwrap().push_mjolnir_slice(angle, offset);
    }
    fn pop_mjolnir_slice(&mut self) {
        self.gpu.lock().unwrap().pop_mjolnir_slice();
    }
    fn mjolnir_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu.lock().unwrap().mjolnir_shatter(rect, pieces, force, color);
    }
    fn mjolnir_fluid_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.gpu.lock().unwrap().mjolnir_fluid_shatter(rect, pieces, force, color);
    }
    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.gpu.lock().unwrap().draw_mjolnir_bolt(from, to, color);
    }
    fn register_shared_element(&mut self, id: &str, rect: cvkg_core::Rect) {
        self.gpu.lock().unwrap().register_shared_element(id, rect);
    }
    fn set_z_index(&mut self, z: f32) {
        self.gpu.lock().unwrap().set_z_index(z);
    }
    fn get_z_index(&self) -> f32 {
        self.gpu.lock().unwrap().get_z_index()
    }
    fn load_svg(&mut self, name: &str, svg_data: &[u8]) {
        self.gpu.lock().unwrap().load_svg(name, svg_data);
    }
    fn draw_svg(&mut self, name: &str, rect: cvkg_core::Rect) {
        self.gpu.lock().unwrap().draw_svg(name, rect, None, 0);
    }
    fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.gpu.lock().unwrap().telemetry.clone()
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        self.gpu.lock().unwrap().push_transform(translation, scale, rotation);
    }

    fn pop_transform(&mut self) {
        self.gpu.lock().unwrap().pop_transform();
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

/// A concrete AssetManager for native desktop targets that loads from the local filesystem.
///
/// The cache is read on every render frame (lock-free via `ArcSwap::load()`) but written
/// at most once per URL after disk I/O completes. `rcu()` atomically inserts the result
/// without blocking concurrent render-loop readers.
pub struct NativeAssetManager {
    cache: std::sync::Arc<
        arc_swap::ArcSwap<
            std::collections::HashMap<String, cvkg_core::AssetState<std::sync::Arc<Vec<u8>>>>,
        >,
    >,
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
    /// Slow path (cache miss): perform filesystem I/O, then publish the result
    /// with `rcu()` — no lock is held while reading the disk.
    fn load_image(&self, url: &str) -> cvkg_core::AssetState<std::sync::Arc<Vec<u8>>> {
        // Fast path: lock-free read from current cache snapshot
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        // Slow path: disk I/O, then atomic rcu insert
        let result = match std::fs::read(url) {
            Ok(data) => cvkg_core::AssetState::Ready(std::sync::Arc::new(data)),
            Err(e) => cvkg_core::AssetState::Error(e.to_string()),
        };
        let result_clone = result.clone();
        let key = url.to_string();
        self.cache.rcu(move |map| {
            let mut m = (**map).clone();
            m.insert(key.clone(), result_clone.clone());
            m
        });
        result
    }

    fn preload_image(&self, _url: &str) {
        // Async preloading could be wired to a background thread here
    }
}
