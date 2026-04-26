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
}

/// Custom events for the native application event loop
#[derive(Debug)]
enum AppEvent {
    AccessibilityAction(accesskit::ActionRequest),
}

impl NativeRenderer {
    /// Create a new NativeRenderer (internal use by App)
    fn new(_window: Arc<Window>, gpu: Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>) -> Self {
        Self { gpu }
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
        };

        event_loop.run_app(&mut app).expect("Event loop error");
    }
}

struct WindowState {
    window: Arc<Window>,
    accesskit_adapter: Option<accesskit_winit::Adapter>,
    vdom: Option<cvkg_vdom::VDom>,
    cursor_pos: [f32; 2],
}

struct App<V: cvkg_core::View> {
    view: V,
    windows: std::collections::HashMap<WindowId, WindowState>,
    gpu: Option<Arc<std::sync::Mutex<cvkg_render_gpu::SurtrRenderer>>>,
    asset_manager: std::sync::Arc<NativeAssetManager>,
    proxy: winit::event_loop::EventLoopProxy<AppEvent>,
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

                let new_vdom = cvkg_vdom::VDom::build(&self.view, rect);
                if let Some(prev_vdom) = &mut state.vdom {
                    let patches = prev_vdom.diff(&new_vdom);
                    if let Some(adapter) = &mut state.accesskit_adapter {
                        let mut nodes = Vec::new();
                        for patch in &patches {
                            match patch {
                                cvkg_vdom::VDomPatch::Create(node)
                                | cvkg_vdom::VDomPatch::Replace { node, .. } => {
                                    nodes.push((accesskit::NodeId(node.id.0 as u64), node.to_accesskit_node()));
                                }
                                cvkg_vdom::VDomPatch::Update { id, .. } => {
                                    if let Some(node) = new_vdom.nodes.get(id) {
                                        nodes.push((accesskit::NodeId(node.id.0 as u64), node.to_accesskit_node()));
                                    }
                                }
                                _ => {}
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

                {
                    let mut gpu = gpu_arc.lock().unwrap();
                    let encoder = gpu.begin_frame(id);
                    let mut renderer = NativeRenderer::new(state.window.clone(), gpu_arc.clone());
                    self.view.render(&mut renderer, rect);
                    gpu.end_frame(encoder);
                }
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
                if let Some(vdom) = &state.vdom {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        let key_str = format!("{:?}", code);
                        let cvkg_event = if event.state == winit::event::ElementState::Pressed {
                            cvkg_core::Event::KeyDown { key: key_str }
                        } else {
                            cvkg_core::Event::KeyUp { key: key_str }
                        };
                        vdom.dispatch_event(cvkg_event);
                    }
                }
            }
            WindowEvent::Ime(ime_event) => {
                if let Some(vdom) = &state.vdom {
                    match ime_event {
                        winit::event::Ime::Commit(string) => {
                            vdom.dispatch_event(cvkg_core::Event::Ime(string));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::AccessibilityAction(request) => {
                let node_id = cvkg_vdom::NodeId(request.target.0 as u64);
                // For accessibility, we'll route to the first window for now
                if let Some(state) = self.windows.values_mut().next() {
                    if let Some(vdom) = &state.vdom {
                        if let Some(node) = vdom.nodes.get(&node_id) {
                            match request.action {
                                accesskit::Action::Click => {
                                    let event = cvkg_core::Event::PointerClick {
                                        x: node.layout.x + node.layout.width / 2.0,
                                        y: node.layout.y + node.layout.height / 2.0,
                                    };
                                    vdom.dispatch_event(event);
                                }
                                _ => ()
                            }
                        }
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for state in self.windows.values() {
            state.window.request_redraw();
        }
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
pub struct NativeAssetManager {
    cache: std::sync::Arc<
        std::sync::RwLock<
            std::collections::HashMap<String, cvkg_core::AssetState<std::sync::Arc<Vec<u8>>>>,
        >,
    >,
}

impl NativeAssetManager {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl cvkg_core::AssetManager for NativeAssetManager {
    fn load_image(&self, url: &str) -> cvkg_core::AssetState<std::sync::Arc<Vec<u8>>> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(state) = cache.get(url) {
                return state.clone();
            }
        }

        // Real filesystem I/O (simplistic implementation for now)
        match std::fs::read(url) {
            Ok(data) => {
                let state = cvkg_core::AssetState::Ready(std::sync::Arc::new(data));
                let mut cache = self.cache.write().unwrap();
                cache.insert(url.to_string(), state.clone());
                state
            }
            Err(e) => {
                let state = cvkg_core::AssetState::Error(e.to_string());
                let mut cache = self.cache.write().unwrap();
                cache.insert(url.to_string(), state.clone());
                state
            }
        }
    }

    fn preload_image(&self, _url: &str) {
        // Implementation for async preloading could go here
    }
}
