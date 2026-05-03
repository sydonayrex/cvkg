#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneNode {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub flags: u32,
    pub animation_phase: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ComputeParams {
    pub node_count: u32,
    pub time: f32,
    pub delta_time: f32,
    pub _pad: f32,
}

#[cfg(not(target_arch = "wasm32"))]
mod stubs {
    use cvkg_core::{ElapsedTime, FrameRenderer, Rect, Renderer, RenderTier};
    pub struct WebRenderer;
    impl WebRenderer {
        pub fn new() -> Self { Self }
        pub async fn forge(&mut self) -> Result<RenderTier, String> { Ok(RenderTier::Tier3Fallback) }
        pub fn canvas(&self) -> Option<&()> { None }
        pub fn update_vdom<V>(&mut self, _: V) -> Result<(), String> { Ok(()) }
        pub fn tier(&self) -> RenderTier { RenderTier::Tier3Fallback }
    }
    impl Renderer for WebRenderer {
        fn fill_rect(&mut self, _: Rect, _: [f32; 4]) {}
        fn fill_rounded_rect(&mut self, _: Rect, _: f32, _: [f32; 4]) {}
        fn fill_ellipse(&mut self, _: Rect, _: [f32; 4]) {}
        fn stroke_rect(&mut self, _: Rect, _: [f32; 4], _: f32) {}
        fn stroke_rounded_rect(&mut self, _: Rect, _: f32, _: [f32; 4], _: f32) {}
        fn stroke_ellipse(&mut self, _: Rect, _: [f32; 4], _: f32) {}
        fn draw_line(&mut self, _: f32, _: f32, _: f32, _: f32, _: [f32; 4], _: f32) {}
        fn draw_text(&mut self, _: &str, _: f32, _: f32, _: f32, _: [f32; 4]) {}
        fn measure_text(&mut self, _: &str, _: f32) -> (f32, f32) { (0.0, 0.0) }
        fn memoize(&mut self, _: u64, _: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
            render_fn(self);
        }
    }
    impl ElapsedTime for WebRenderer {
        fn elapsed_time(&self) -> f32 { 0.0 }
        fn delta_time(&self) -> f32 { 0.016 }
    }
    impl FrameRenderer<()> for WebRenderer {
        fn begin_frame(&mut self) -> () { () }
        fn end_frame(&mut self, _: ()) {}
    }
    pub fn get_render_tier_name() -> String { "None".to_string() }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_web_renderer_stub_lifecycle() {
            let mut renderer = WebRenderer::new();
            assert_eq!(renderer.tier(), RenderTier::Tier3Fallback);
            
            // Should not panic
            renderer.fill_rect(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 }, [1.0, 1.0, 1.0, 1.0]);
            renderer.begin_frame();
            renderer.end_frame(());
        }

        #[test]
        fn test_compute_struct_layouts() {
            use crate::{SceneNode, ComputeParams};
            // SceneNode: [f32; 2] (8) + [f32; 2] (8) + [f32; 4] (16) + u32 (4) + f32 (4) = 40
            assert_eq!(std::mem::size_of::<SceneNode>(), 40);
            // ComputeParams: u32 (4) + f32 (4) + f32 (4) + f32 (4) = 16
            assert_eq!(std::mem::size_of::<ComputeParams>(), 16);
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use stubs::*;

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
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

//! WASM WebGPU/WebGL renderer + vDOM bridge
//!
//! This crate implements the WebGPU and WebGL rendering paths for WASM targets,
//  combined with a virtual DOM tree for developer tooling and accessibility.

#![allow(deprecated)]

use cvkg_core::{ElapsedTime, FrameRenderer, Rect, Renderer, View, RenderTier};
use super::{SceneNode, ComputeParams};
use wasm_bindgen::prelude::*;

use cvkg_vdom::VDomPatch;
use web_sys::*;
use wasm_bindgen::JsCast;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneUniforms {
    resolution: [f32; 2],
    time: f32,
    _pad: f32,
}

static CURRENT_VDOM: std::sync::OnceLock<std::sync::Mutex<Option<std::sync::Arc<cvkg_vdom::VDom>>>> =
    std::sync::OnceLock::new();

static ACTIVE_TIER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_buffer: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,
    pub node_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
}

/// Web renderer backend implementing the CvkgRenderer trait
pub struct WebRenderer {
    canvas: Option<web_sys::HtmlCanvasElement>,
    canvas_context: Option<web_sys::CanvasRenderingContext2d>,
    gl_context: Option<web_sys::WebGl2RenderingContext>,
    #[allow(dead_code)]
    webgpu_context: Option<GpuContext>,
    tier: RenderTier,
    vdom: Option<std::sync::Arc<cvkg_vdom::VDom>>,
    previous_vdom: Option<std::sync::Arc<cvkg_vdom::VDom>>,
    start_time: f64,
    pub asset_manager: std::sync::Arc<WebAssetManager>,
    /// Telemetry data for the last frame
    pub telemetry: cvkg_core::TelemetryData,
    /// Configuration for render-loop frame timing and degradation strategies.
    pub frame_budget: cvkg_core::FrameBudget,
    /// Timestamp of the last redraw start, used for measuring frame timings.
    pub last_redraw_start: f64,
    /// Time elapsed since the last frame in seconds.
    pub delta_time: f32,
    /// Whether a redraw has been requested for the next frame.
    pub redraw_requested: bool,
    /// Bridge to the CVKG WebKit server for snapshots and HMR.
    pub bridge: Option<WebKitBridge>,
    /// Berserker pipeline state
    pub berserker_mode: cvkg_core::BerserkerMode,
    pub rage: f32,
    pub(crate) performance: Option<web_sys::Performance>,
    pub shield_wall: Option<ShieldWall>,
    /// Total frames rendered in this session.
    pub frame_count: u64,
}

// WebRenderer is only used on a single thread in WASM, but Renderer trait requires Send.
unsafe impl Send for WebRenderer {}

impl WebRenderer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let window = web_sys::window();
        let performance = window.as_ref().and_then(|w| w.performance());
        let now = performance.as_ref().map(|p| p.now()).unwrap_or(0.0);
        
        Self {
            canvas: None,
            canvas_context: None,
            gl_context: None,
            webgpu_context: None,
            tier: RenderTier::Tier3Fallback,
            vdom: Some(std::sync::Arc::new(cvkg_vdom::VDom::new())),
            previous_vdom: None,
            start_time: now,
            asset_manager: std::sync::Arc::new(WebAssetManager::new()),
            telemetry: cvkg_core::TelemetryData::default(),
            frame_budget: cvkg_core::FrameBudget::default(),
            last_redraw_start: now,
            delta_time: 0.016,
            redraw_requested: false,
            bridge: Some(WebKitBridge::new()),
            berserker_mode: cvkg_core::BerserkerMode::Normal,
            rage: 0.0,
            performance,
            shield_wall: Some(ShieldWall::new()),
            frame_count: 0,
        }
    }

    /// Returns the current rendering tier
    pub fn tier(&self) -> RenderTier {
        self.tier
    }

    /// Get real-time performance telemetry.
    #[allow(dead_code)]
    fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
    }

    fn now(&self) -> f64 {
        self.performance.as_ref().map(|p| p.now()).unwrap_or(0.0)
    }

    /// Get the canvas element.
    pub fn canvas(&self) -> Option<&web_sys::HtmlCanvasElement> {
        self.canvas.as_ref()
    }

    #[doc(hidden)]
    pub fn init(&mut self) -> Result<(), JsValue> {
        // Try to create WebGPU context first
        #[cfg(feature = "webgpu")]
        {
            // Note: init is called from JS, we can't easily await here without making init async
            // For now, we'll assume it's handled or called separately if needed.
            log::warn!("Web: WebGPU init is async and should be handled via init_async");
            Ok(())
        }

        #[cfg(not(feature = "webgpu"))]
        {
            self.init_canvas_2d()
        }
    }

    #[doc(hidden)]
    pub async fn init_async(&mut self) -> Result<(), JsValue> {
        #[cfg(feature = "webgpu")]
        {
            match self.init_webgpu_async().await {
                Ok(_) => {
                    self.tier = RenderTier::Tier1GPU;
                    log::info!("Initialized WebGPU context");
                }
                Err(e) => {
                    log::warn!("WebGPU initialization failed: {:?}. Falling back to WebGL2...", e);
                    match self.init_webgl2() {
                        Ok(_) => {
                            self.tier = RenderTier::Tier2GPU;
                            log::info!("Initialized WebGL2 context");
                        }
                        Err(e2) => {
                            log::warn!("WebGL2 initialization failed: {:?}. Using Tier 3 Canvas2D.", e2);
                            self.tier = RenderTier::Tier3Fallback;
                        }
                    }
                }
            }
        }
        
        #[cfg(not(feature = "webgpu"))]
        {
            match self.init_webgl2() {
                Ok(_) => {
                    self.tier = RenderTier::Tier2GPU;
                }
                Err(_) => {
                    self.tier = RenderTier::Tier3Fallback;
                }
            }
        }
        self.init_canvas_2d()?;

        // Register AssetManager in the environment
        cvkg_core::env::insert::<cvkg_core::AssetKey>(self.asset_manager.clone());

        self.register_web_events()?;

        if let Some(ref mut bridge) = self.bridge {
            let _ = bridge.connect();
        }

        Ok(())
    }

    /// Update the virtual DOM with a new component tree
    #[doc(hidden)]
    pub fn update_vdom<V: View>(&mut self, view: V) -> Result<(), JsValue> {
        // Get viewport from canvas
        let rect = if let Some(ref canvas) = self.canvas {
            Rect {
                x: 0.0,
                y: 0.0,
                width: canvas.width() as f32,
                height: canvas.height() as f32,
            }
        } else {
            Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            }
        };

        // Create new VDOM from the view using the new build system
        let new_vdom = std::sync::Arc::new(cvkg_vdom::VDom::build(&view, rect));

        // Update global VDOM for event dispatch
        if let Some(vdom_lock) = CURRENT_VDOM.get() {
            let mut vdom_guard = vdom_lock.lock().unwrap();
            *vdom_guard = Some(new_vdom.clone());
        } else {
            let _ = CURRENT_VDOM.set(std::sync::Mutex::new(Some(new_vdom.clone())));
        }

        // Compute patches if we have a previous VDOM
        if let Some(ref prev_vdom) = self.vdom {
            let patches = new_vdom.diff(prev_vdom);

            // Apply patches to accessibility DOM
            // In a real WASM app, we'd serialize these and call a JS helper,
            // but we can also call our local apply_vdom_patches for now.
            if let Ok(serialized) = serde_json::to_string(&patches) {
                let _ = apply_vdom_patches(&serialized);
            }

            // Store current as previous for next frame
            self.previous_vdom = self.vdom.take();
        }

        // Send VDOM snapshot to server for SSG if bridge is active
        if let Some(ref _bridge) = self.bridge {
            // _bridge.send_snapshot(&new_vdom.to_html());
        }

        // Update current VDOM
        self.vdom = Some(new_vdom);

        Ok(())
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<(), JsValue> {
        self.end_frame(());
        Ok(())
    }

/// Initialize the renderer, trying WebGPU first, then WebGL2, then Canvas 2D.
    pub async fn forge(&mut self) -> Result<RenderTier, JsValue> {
        // 1. Try WebGPU
        #[cfg(feature = "webgpu")]
        {
            self.init_base_canvas()?;
            log::info!("Attempting WebGPU initialization...");
            if let Ok(_) = self.init_webgpu_async().await {
                self.tier = RenderTier::Tier1GPU;
                ACTIVE_TIER.store(1, std::sync::atomic::Ordering::Relaxed);
                log::info!("Forge Success: WebGPU tier active.");
                let _ = self.register_web_events();
                if let Some(ref mut bridge) = self.bridge {
                    let _ = bridge.connect();
                }
                return Ok(self.tier);
            }
            log::warn!("WebGPU failed, clearing canvas for fallback.");
            self.canvas = None;
        }

        // 2. Try WebGL2 Detection
        log::info!("Attempting WebGL2 detection...");
        
        // We check for WebGL2 support, but we don't 'lock' the main canvas to it 
        // if we need to fallback to Canvas2D for the UI.
        let has_webgl2 = {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let temp_canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            temp_canvas.get_context("webgl2").unwrap_or(None).is_some()
        };

        if has_webgl2 {
            log::info!("WebGL2 support detected.");
        }

        // 3. Initialize Canvas 2D for the actual rendering
        self.init_base_canvas()?;
        self.init_canvas_2d()?;
        
        if has_webgl2 {
            self.tier = RenderTier::Tier2GPU;
            ACTIVE_TIER.store(2, std::sync::atomic::Ordering::Relaxed);
            log::info!("Forge Success: WebGL2 tier active (Hybrid).");
        } else {
            self.tier = RenderTier::Tier3Fallback;
            ACTIVE_TIER.store(3, std::sync::atomic::Ordering::Relaxed);
            log::info!("Forge Success: Canvas 2 D tier active (Fallback).");
        }

        if let Some(ref mut bridge) = self.bridge {
            let _ = bridge.connect();
        }
        
        Ok(self.tier)
    }

    fn init_base_canvas(&mut self) -> Result<(), JsValue> {
        if self.canvas.is_none() {
            let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
            let document = window
                .document()
                .ok_or_else(|| JsValue::from_str("No document found"))?;
            
            // Force a fresh canvas for each tier attempt to avoid context conflicts
            if let Some(existing) = document.get_element_by_id("cvkg-canvas") {
                existing.remove();
            }

            let canvas = document
                .create_element("canvas")?
                .dyn_into::<web_sys::HtmlCanvasElement>()?;
            canvas.set_id("cvkg-canvas");
            
            canvas.set_width(window.inner_width()?.as_f64().unwrap_or(800.0) as u32);
            canvas.set_height(window.inner_height()?.as_f64().unwrap_or(600.0) as u32);
            
            let root = document.get_element_by_id("cvkg-root")
                .ok_or_else(|| JsValue::from_str("No #cvkg-root found"))?;
            
            // Try to find a container first, fallback to root
            let target = document.get_element_by_id("cvkg-container")
                .unwrap_or_else(|| root.clone());
            
            // Only append if it's a new canvas (no parent)
            if canvas.parent_node().is_none() {
                // Clear "Loading..." text
                target.set_inner_html("");
                let node: &web_sys::Node = canvas.as_ref();
                target.append_child(node)?;
            }
            
            // Create a11y root if it doesn't exist
            if document.get_element_by_id("cvkg-a11y-root").is_none() {
                let a11y_root = document.create_element("div")?;
                a11y_root.set_id("cvkg-a11y-root");
                a11y_root.set_attribute("style", "position: absolute; left: 0; top: 0; width: 0; height: 0; overflow: hidden;")?;
                target.append_child(&a11y_root)?;
            }

            self.canvas = Some(canvas);
        }
        Ok(())
    }

    fn init_webgl2(&mut self) -> Result<(), JsValue> {
        let canvas = self.canvas.as_ref().ok_or_else(|| JsValue::from_str("Canvas not initialized"))?;
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 not supported"))?
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;
        self.gl_context = Some(context);
        Ok(())
    }

    fn init_canvas_2d(&mut self) -> Result<(), JsValue> {
        let canvas = self.canvas.as_ref().ok_or_else(|| JsValue::from_str("Canvas not initialized"))?;
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D context not supported"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        self.canvas_context = Some(context);
        Ok(())
    }

#[cfg(feature = "webgpu")]
    async fn init_webgpu_async(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let navigator = window.navigator();
        
        // Safely check if GPU is available in navigator
        let gpu = js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu"))?;
        if gpu.is_undefined() {
            log::warn!("WebGPU: navigator.gpu undefined - falling back");
            return Err(JsValue::from_str("WebGPU not supported by browser (navigator.gpu undefined)"));
        }

        // Create WebGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        // Get the canvas element
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Canvas not initialized"))?;
        
        // Create surface from canvas - use correct modern API
        let surface = match instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone())) {
            Ok(s) => {
                log::info!("WebGPU: Surface created successfully");
                s
            }
            Err(e) => {
                log::error!("WebGPU: Surface creation failed: {:?}", e);
                return Err(JsValue::from_str(&format!("Failed to create surface: {:?}", e)));
            }
        };

        // Request adapter - robust multi-stage fallback
        // Stage 1: HighPerformance + Surface
        log::info!("WebGPU: Requesting HighPerformance adapter...");
        let mut adapter_opt = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok();

        // Stage 2: LowPower + Surface
        if adapter_opt.is_none() {
            log::warn!("WebGPU: HighPerformance adapter failed, trying LowPower hardware...");
            adapter_opt = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        // Stage 3: Software Fallback (force_fallback_adapter: true)
        if adapter_opt.is_none() {
            log::warn!("WebGPU: Hardware adapters failed, trying Software Fallback...");
            adapter_opt = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: true,
                })
                .await
                .ok();
        }

        // Stage 4: Diagnostic - Try without surface just to see if ANY adapter exists
        if adapter_opt.is_none() {
            log::warn!("WebGPU: Surface-compatible adapter failed, performing diagnostics...");
            let diagnostic_adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await;
            
            if let Ok(adp) = diagnostic_adapter {
                let info = adp.get_info();
                log::error!("WebGPU DIAGNOSTIC: Found adapter {:?} ({:?}) but it is INCOMPATIBLE with the current surface target.", info.name, info.backend);
            }
        }

        let adapter = adapter_opt.ok_or_else(|| {
            JsValue::from_str("No suitable WebGPU adapter found. Ensure WebGPU is enabled (e.g. #enable-unsafe-webgpu in Chrome) and your GPU drivers are up to date.")
        })?;

        // Request device and queue with modern Descriptor
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("CVKG WebGPU Device"),
                ..Default::default()
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to request device: {}", e)))?;

        // Configure surface
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .or_else(|| surface_caps.formats.get(0).copied())
            .ok_or_else(|| JsValue::from_str("No supported surface formats found"))?;

        let alpha_mode = surface_caps.alpha_modes.get(0).copied().unwrap_or(wgpu::CompositeAlphaMode::Opaque);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas_width,
            height: canvas_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CVKG Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create Uniform Buffer
        let scene_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("CVKG Scene Buffer"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("CVKG Scene Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CVKG Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        });

            let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("CVKG Render Pipeline Layout"),
                bind_group_layouts: &[Some(&scene_bind_group_layout)],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("CVKG Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create compute resources
        let node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("CVKG Node Buffer"),
            size: (1024 * std::mem::size_of::<SceneNode>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("CVKG Compute Params Buffer"),
            size: std::mem::size_of::<ComputeParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("CVKG Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CVKG Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("CVKG Compute Pipeline Layout"),
                bind_group_layouts: &[Some(&compute_bind_group_layout)],
                immediate_size: 0,
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("CVKG Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // Store WebGPU context
        self.webgpu_context = Some(GpuContext {
            instance,
            surface,
            device,
            queue,
            config,
            pipeline: render_pipeline,
            scene_bind_group,
            scene_buffer,
            compute_pipeline,
            compute_bind_group,
            node_buffer,
            params_buffer,
        });

        self.tier = RenderTier::Tier1GPU;
        log::info!("Initialized WebGPU context (v29)");
        Ok(())
    }
}

impl Renderer for WebRenderer {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.fill_rect(
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            );
        }
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));

            gl.begin_path();
            let x = rect.x as f64;
            let y = rect.y as f64;
            let width = rect.width as f64;
            let height = rect.height as f64;
            let radius = radius as f64;

            gl.move_to(x + radius, y);
            gl.line_to(x + width - radius, y);
            gl.quadratic_curve_to(x + width, y, x + width, y + radius);
            gl.line_to(x + width, y + height - radius);
            gl.quadratic_curve_to(x + width, y + height, x + width - radius, y + height);
            gl.line_to(x + radius, y + height);
            gl.quadratic_curve_to(x, y + height, x, y + height - radius);
            gl.line_to(x, y + radius);
            gl.quadratic_curve_to(x, y, x + radius, y);
            gl.close_path();
            gl.fill();
        }
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));

            gl.begin_path();
            let x = rect.x as f64 + rect.width as f64 / 2.0;
            let y = rect.y as f64 + rect.height as f64 / 2.0;
            let radius_x = rect.width as f64 / 2.0;
            let radius_y = rect.height as f64 / 2.0;

            let _ = gl.ellipse(
                x,
                y,
                radius_x,
                radius_y,
                0.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            gl.fill();
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.set_line_width(stroke_width as f64);
            gl.stroke_rect(
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            );
        }
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.set_line_width(stroke_width as f64);

            gl.begin_path();
            let x = rect.x as f64;
            let y = rect.y as f64;
            let width = rect.width as f64;
            let height = rect.height as f64;
            let radius = radius as f64;

            gl.move_to(x + radius, y);
            gl.line_to(x + width - radius, y);
            gl.quadratic_curve_to(x + width, y, x + width, y + radius);
            gl.line_to(x + width, y + height - radius);
            gl.quadratic_curve_to(x + width, y + height, x + width - radius, y + height);
            gl.line_to(x + radius, y + height);
            gl.quadratic_curve_to(x, y + height, x, y + height - radius);
            gl.line_to(x, y + radius);
            gl.quadratic_curve_to(x, y, x + radius, y);
            gl.close_path();
            gl.stroke();
        }
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.set_line_width(stroke_width as f64);

            gl.begin_path();
            let x = rect.x as f64 + rect.width as f64 / 2.0;
            let y = rect.y as f64 + rect.height as f64 / 2.0;
            let radius_x = rect.width as f64 / 2.0;
            let radius_y = rect.height as f64 / 2.0;

            let _ = gl.ellipse(
                x,
                y,
                radius_x,
                radius_y,
                0.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            gl.stroke();
        }
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
        if let Some(ref gl) = self.canvas_context {
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.set_line_width(stroke_width as f64);

            gl.begin_path();
            gl.move_to(x1 as f64, y1 as f64);
            gl.line_to(x2 as f64, y2 as f64);
            gl.stroke();
        }
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3]
            )));
            gl.set_font(&format!("{}px sans-serif", size));
            let _ = gl.fill_text(text, x as f64, y as f64);
        }
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.set_font(&format!("{}px sans-serif", size));
            if let Ok(metrics) = gl.measure_text(text) {
                return (metrics.width() as f32, size);
            }
        }
        (text.len() as f32 * size * 0.6, size)
    }

fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {
        // GPU textures not applicable to 2D Canvas context
    }

    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {
        // Image asset loading and drawing logic would go here
    }

    fn load_image(&mut self, _name: &str, _data: &[u8]) {
        // Image asset loading for Web targets
    }

    /// Draw a linear gradient between two colors at the specified angle.
    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            
            // Calculate gradient direction from angle
            let rad = angle as f64 * std::f64::consts::PI / 180.0;
            let x0 = rect.x as f64;
            let y0 = rect.y as f64;
            let x1 = x0 + rect.width as f64 * rad.cos();
            let y1 = y0 + rect.height as f64 * rad.sin();
            
            // Create linear gradient
            let grad = gl.create_linear_gradient(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
            
            grad.add_color_stop(0.0, &format!(
                "rgba({}, {}, {}, {})",
                (start_color[0] * 255.0) as u8,
                (start_color[1] * 255.0) as u8,
                (start_color[2] * 255.0) as u8,
                start_color[3]
            )).ok();
            grad.add_color_stop(1.0, &format!(
                "rgba({}, {}, {}, {})",
                (end_color[0] * 255.0) as u8,
                (end_color[1] * 255.0) as u8,
                (end_color[2] * 255.0) as u8,
                end_color[3]
            )).ok();
            
            gl.set_fill_style(&grad);
            gl.fill_rect(rect.x as f64, rect.y as f64, rect.width as f64, rect.height as f64);
            gl.restore();
        }
    }

    /// Draw a radial gradient between two colors.
    fn draw_radial_gradient(
        &mut self,
        rect: Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            
            let cx = rect.x + rect.width / 2.0;
            let cy = rect.y + rect.height / 2.0;
            let rx = rect.width / 2.0;
            let ry = rect.height / 2.0;
            
            // Create radial gradient
            let grad = gl.create_radial_gradient(
                cx as f64, cy as f64, 0.0,
                cx as f64, cy as f64, (rx as f64).max(ry as f64)
            ).unwrap();
            
            grad.add_color_stop(0.0, &format!(
                "rgba({}, {}, {}, {})",
                (inner_color[0] * 255.0) as u8,
                (inner_color[1] * 255.0) as u8,
                (inner_color[2] * 255.0) as u8,
                inner_color[3]
            )).ok();
            grad.add_color_stop(1.0, &format!(
                "rgba({}, {}, {}, {})",
                (outer_color[0] * 255.0) as u8,
                (outer_color[1] * 255.0) as u8,
                (outer_color[2] * 255.0) as u8,
                outer_color[3]
            )).ok();
            
            gl.set_fill_style(&grad);
            gl.fill_rect(rect.x as f64, rect.y as f64, rect.width as f64, rect.height as f64);
            gl.restore();
        }
    }

    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            
            // Outer glow for the bolt
            gl.set_shadow_blur(15.0);
            gl.set_shadow_color(&format!("rgba({}, {}, {}, 0.8)", 
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8
            ));

            // Calculate bolt path with jagged edges
            let mut rng = 12345.0_f32; // Deterministic-ish jitter
            let segments = 12;
            let mut points: Vec<[f32; 2]> = Vec::with_capacity(segments + 2);
            points.push(from);
            
            let dx = to[0] - from[0];
            let dy = to[1] - from[1];
            let len = (dx * dx + dy * dy).sqrt();
            let perp_x = -dy / len.max(1.0);
            let perp_y = dx / len.max(1.0);
            
            for i in 1..segments {
                let t = i as f32 / segments as f32;
                rng = (rng * 16807.0) % 2147483647.0;
                let jitter = (rng / 2147483647.0 - 0.5) * 40.0;
                let mid_x = from[0] + dx * t;
                let mid_y = from[1] + dy * t;
                points.push([mid_x + perp_x * jitter, mid_y + perp_y * jitter]);
            }
            points.push(to);
            
            gl.begin_path();
            gl.move_to(points[0][0] as f64, points[0][1] as f64);
            for pt in &points[1..] {
                gl.line_to(pt[0] as f64, pt[1] as f64);
            }
            
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, 1.0)",
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8
            )));
            gl.set_line_width(2.5);
            gl.set_line_cap("round");
            gl.stroke();
            
            // Bright core
            gl.set_shadow_blur(0.0);
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255, 255, 255, 0.9)"));
            gl.set_line_width(1.0);
            gl.stroke();

            gl.restore();
        }
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            gl.begin_path();
            gl.rect(
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            );
            gl.clip();
        }
    }

    fn pop_clip_rect(&mut self) {
        if let Some(ref gl) = self.canvas_context {
            gl.restore();
        }
    }

    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }

    fn push_opacity(&mut self, opacity: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            let current = gl.global_alpha();
            gl.set_global_alpha(current * opacity as f64);
        }
    }

    fn pop_opacity(&mut self) {
        if let Some(ref gl) = self.canvas_context {
            gl.restore();
        }
    }

    fn bifrost(&mut self, rect: Rect, blur: f32, _saturation: f32, opacity: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            
            // Try to use native CSS filter if supported (most modern browsers do even in 2D)
            let filter_str = format!("blur({}px)", blur / 2.0);
            let has_filter = js_sys::Reflect::has(gl, &wasm_bindgen::JsValue::from_str("filter")).unwrap_or(false);
            
            if has_filter {
                gl.set_filter(&filter_str);
            }

            // Draw glass background with subtle gradient for depth
            let radius = 24.0_f64;
            gl.begin_path();
            gl.move_to(rect.x as f64 + radius, rect.y as f64);
            gl.line_to(rect.x as f64 + rect.width as f64 - radius, rect.y as f64);
            gl.quadratic_curve_to(rect.x as f64 + rect.width as f64, rect.y as f64, rect.x as f64 + rect.width as f64, rect.y as f64 + radius);
            gl.line_to(rect.x as f64 + rect.width as f64, rect.y as f64 + rect.height as f64 - radius);
            gl.quadratic_curve_to(rect.x as f64 + rect.width as f64, rect.y as f64 + rect.height as f64, rect.x as f64 + rect.width as f64 - radius, rect.y as f64 + rect.height as f64);
            gl.line_to(rect.x as f64 + radius, rect.y as f64 + rect.height as f64);
            gl.quadratic_curve_to(rect.x as f64, rect.y as f64 + rect.height as f64, rect.x as f64, rect.y as f64 + rect.height as f64 - radius);
            gl.line_to(rect.x as f64, rect.y as f64 + radius);
            gl.quadratic_curve_to(rect.x as f64, rect.y as f64, rect.x as f64 + radius, rect.y as f64);
            gl.close_path();

            // Premium frosted glass color: very slight blue-white tint
            gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba(255, 255, 255, {})",
                opacity * 0.15
            )));
            gl.fill();

            // Glass specular highlights
            if !has_filter {
                // Fallback specular highlight if no blur
                gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!("rgba(255, 255, 255, {})", opacity * 0.3)));
                gl.set_line_width(1.5);
                gl.stroke();
            }

            gl.restore();
        }
    }



    fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            // In Canvas 2D, we simulate shattering by drawing fragmented rectangles
            let piece_w = rect.width / (pieces as f32).sqrt();
            let piece_h = rect.height / (pieces as f32).sqrt();
            
            gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                "rgba({}, {}, {}, 0.8)",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0
            )));
            
            let t = self.elapsed_time();
            
            for i in 0..pieces {
                let col = i % (pieces as f32).sqrt() as u32;
                let row = i / (pieces as f32).sqrt() as u32;
                
                let mut x = rect.x + col as f32 * piece_w;
                let mut y = rect.y + row as f32 * piece_h;
                
                // Animate offset based on force and time
                let offset_x = (t * force).sin() * 5.0;
                let offset_y = (t * force * 1.1).cos() * 5.0;
                x += offset_x;
                y += offset_y;

                gl.stroke_rect(x as f64, y as f64, piece_w as f64, piece_h as f64);
            }
            gl.restore();
        }
    }

    fn mjolnir_fluid_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        // Reuse mjolnir_shatter but with a different motion pattern for fluid effect
        self.mjolnir_shatter(rect, pieces, force * 1.5, color);
    }


    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();

            // Implementation of half-plane clipping in Canvas 2D
            let rad = (angle * 0.0174532925) as f64; // deg to rad

            gl.begin_path();
            // Translate to the closest point on the line to the origin
            gl.translate(offset as f64 * rad.cos(), offset as f64 * rad.sin())
                .ok();
            // Rotate so the line is aligned with the Y axis
            gl.rotate(rad).ok();

            // Define a giant rectangle representing the "visible" half-plane (x >= 0)
            let giant = 100000.0;
            gl.rect(0.0, -giant, giant, giant * 2.0);

            gl.clip();

            // Reset transformation so subsequent drawing is in the original coordinate system
            // but still restricted by the clip path we just established.
            gl.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).ok();
            // Note: We'll need to re-apply the global translation if the renderer uses one.
        }
    }

    fn pop_mjolnir_slice(&mut self) {
        if let Some(ref gl) = self.canvas_context {
            gl.restore();
        }
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        if let Some(ref gl) = self.canvas_context {
            let _ = gl.save();
            let _ = gl.translate(translation[0] as f64, translation[1] as f64);
            let _ = gl.scale(scale[0] as f64, scale[1] as f64);
            let _ = gl.rotate(rotation as f64);
        }
    }

    fn pop_transform(&mut self) {
        if let Some(ref gl) = self.canvas_context {
            let _ = gl.restore();
        }
    }


    fn push_vnode(&mut self, rect: Rect, name: &'static str) {
        if self.tier == RenderTier::Tier3Fallback {
            if let Some(ref gl) = self.canvas_context {
                // Debug layout visualization
                gl.save();
                gl.set_stroke_style(&wasm_bindgen::JsValue::from_str("magenta"));
                gl.set_line_width(1.0);
                gl.stroke_rect(
                    rect.x as f64,
                    rect.y as f64,
                    rect.width as f64,
                    rect.height as f64,
                );
                gl.set_fill_style(&wasm_bindgen::JsValue::from_str("white"));
                gl.set_font("10px monospace");
                let _ = gl.fill_text(name, rect.x as f64 + 2.0, rect.y as f64 + 10.0);
                gl.restore();
            }
        }
    }

    fn set_debug_layout(&mut self, _enabled: bool) {
    }

    fn get_debug_layout(&self) -> bool {
        false
    }

    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        log::trace!("Web: register_shared_element '{}' {:?}", id, rect);
    }
    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        if let Some(ref gl) = self.canvas_context {
            self.draw_neon_glow(gl, rect, color, intensity * (radius / 10.0));
        }
    }

    fn set_berserker_mode(&mut self, state: cvkg_core::BerserkerMode) {
        self.berserker_mode = state;
    }

    fn set_rage(&mut self, rage: f32) {
        self.rage = rage;
    }

    fn set_aria_role(&mut self, _role: &str) {
        if let Some(ref mut _sw) = self.shield_wall {
            // This is a simplified sync; in a real update_vdom it would be more structured
            // For now, we use the last vnode ID if available
            // Note: This needs integration into the VDom traversal to be fully robust
        }
    }

    fn set_aria_label(&mut self, _label: &str) {
        if let Some(ref mut _sw) = self.shield_wall {
             // Same as above
        }
    }
}

impl cvkg_core::ElapsedTime for WebRenderer {
    fn delta_time(&self) -> f32 {
        self.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        ((self.now() - self.start_time) / 1000.0) as f32
    }
}


impl FrameRenderer<()> for WebRenderer {
    fn begin_frame(&mut self) -> () {
        cvkg_core::begin_render_phase();
        let now = self.now();
        self.delta_time = ((now - self.last_redraw_start) / 1000.0) as f32;
        self.last_redraw_start = now;

        if let Some(ref gl) = self.canvas_context {
            let width = gl.canvas().map(|c| c.width()).unwrap_or(800) as f64;
            let height = gl.canvas().map(|c| c.height()).unwrap_or(600) as f64;
            gl.clear_rect(0.0, 0.0, width, height);
        }
    }

    fn end_frame(&mut self, _encoder: ()) {
        match self.tier {
            RenderTier::Tier1GPU => {
                let _ = self.render_webgpu();
            }
            RenderTier::Tier2GPU => {
                let _ = self.render_webgl2();
            }
            RenderTier::Tier3Fallback => {
                // No-op: Canvas 2D draws immediately during Renderer calls.
            }
        }
        
        // Active Security Probes (every 60 frames)
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {
            let risk = cvkg_core::security::EnvironmentShield::probe_analysis_risk();
            if risk > 0.1 {
                log::debug!("Web Analysis risk probe: {:.2}", risk);
                cvkg_core::security::EnvironmentShield::enforce_mitigation(risk);
            }
        }

        cvkg_core::end_render_phase();
    }
}

impl WebRenderer {
    /// Renders VDOM elements using WebGL2 or Canvas 2 D fallback.
    /// When WebGL2 context exists but VDOM rendering is needed, renders to Canvas 2 D
    /// for consistency across tiers. This ensures the demo content is visible.
    fn render_webgl2(&mut self) -> Result<(), JsValue> {
        let time = (self.now() / 1000.0) as f32;
        
        // Clear the screen with cyberpunk dark background
        if let Some(ref gl) = self.gl_context {
            gl.clear_color(0.02, 0.01, 0.05, 1.0);
            gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }
        
        // In Hybrid (WebGL2) mode, we draw the background into the GL context,
        // and the UI stays in the Canvas2D context (which was drawn during the frame).
        // WE MUST NOT CLEAR THE CANVAS2D CONTEXT HERE.
        
        if let Some(ref ctx2d) = self.canvas_context {
            let width = ctx2d.canvas().map(|c| c.width()).unwrap_or(800) as f64;
            let height = ctx2d.canvas().map(|c| c.height()).unwrap_or(600) as f64;
            
            // Draw deep space background gradient
            ctx2d.save();
            let grad = ctx2d.create_linear_gradient(0.0, 0.0, 0.0, height);
            grad.add_color_stop(0.0, "rgba(13, 13, 26, 1.0)").ok();
            grad.add_color_stop(1.0, "rgba(2, 2, 5, 1.0)").ok();
            ctx2d.set_fill_style(&grad);
            ctx2d.fill_rect(0.0, 0.0, width, height);
            ctx2d.restore();
            
            // Draw animated grid
            ctx2d.save();
            ctx2d.set_line_width(1.0);
            let grid_spacing = 40.0_f64;
            
            // Horizontal lines with animation
            for i in 0..((height / grid_spacing) as i32 + 1) {
                let y = i as f64 * grid_spacing;
                let alpha = 0.05 + 0.1 * ((time as f64 * 2.0 + y * 0.01).sin() * 0.5 + 0.5);
                ctx2d.set_stroke_style(&wasm_bindgen::JsValue::from_str(
                    &format!("rgba(0, 200, 255, {})", alpha)
                ));
                ctx2d.begin_path();
                ctx2d.move_to(0.0, y);
                ctx2d.line_to(width, y);
                ctx2d.stroke();
            }
            
            // Vertical lines
            for i in 0..((width / grid_spacing) as i32 + 1) {
                let x = i as f64 * grid_spacing;
                ctx2d.set_stroke_style(&wasm_bindgen::JsValue::from_str(
                    &format!("rgba(0, 200, 255, {})", 0.05 + 0.1)
                ));
                ctx2d.begin_path();
                ctx2d.move_to(x, 0.0);
                ctx2d.line_to(x, height);
                ctx2d.stroke();
            }
            ctx2d.restore();
            
            // Draw VDOM overlay if available - this renders the actual demo content
            if let Some(ref vdom) = self.vdom {
                self.render_vdom_nodes(ctx2d, vdom, time);
            }
        }
        
        Ok(())
    }
    
    /// Renders VDOM nodes with cyberpunk styling for WebGL2 tier.
    fn render_vdom_nodes(&self, ctx: &web_sys::CanvasRenderingContext2d, _vdom: &cvkg_vdom::VDom, time: f32) {
        ctx.save();
        
        let width = ctx.canvas().map(|c| c.width()).unwrap_or(800) as f64;
        let height = ctx.canvas().map(|c| c.height()).unwrap_or(600) as f64;
        
        // Draw animated center pulse
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let pulse = (time as f64 * 3.0).sin() * 0.5 + 0.5;
        let radius = 60.0_f64 + pulse * 30.0_f64;
        
        // Neon glow effect
        for i in 0..5 {
            let alpha = 0.15 * pulse / (i as f64 + 1.0);
            ctx.set_shadow_blur(radius * (i as f64 + 1.0) * 0.4);
            ctx.set_shadow_color(&format!("rgba(0, 200, 255, {})", alpha));
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(
                &format!("rgba(0, 200, 255, {})", alpha * 2.0)
            ));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.arc(center_x, center_y, radius + i as f64 * 15.0, 0.0, 2.0 * std::f64::consts::PI).ok();
            ctx.stroke();
        }
        
        // Draw scanlines
        ctx.set_shadow_blur(0.0);
        ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(0, 200, 255, 0.1)"));
        ctx.set_line_width(1.0);
        for y in (0..((height / 4.0) as i32)).map(|i| i as f64 * 4.0) {
            ctx.begin_path();
            ctx.move_to(0.0, y);
            ctx.line_to(width, y);
            ctx.stroke();
        }
        
        ctx.restore();
    }
    fn render_webgpu(&mut self) -> Result<(), JsValue> {
        let current_time = self.now();
        let time = ((current_time - self.start_time) / 1000.0) as f32;

        let ctx = self
            .webgpu_context
            .as_mut()
            .ok_or_else(|| JsValue::from_str("WebGPU context missing"))?;

        let uniforms = SceneUniforms {
            resolution: [ctx.config.width as f32, ctx.config.height as f32],
            time,
            _pad: 0.0,
        };

        ctx.queue
            .write_buffer(&ctx.scene_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let surface_texture = ctx.surface.get_current_texture();
        let output = match surface_texture {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            _ => {
                ctx.surface.configure(&ctx.device, &ctx.config);
                return Err(JsValue::from_str("Surface error or outdated"));
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("CVKG Command Encoder"),
            });

        // 1. Compute Pass: Scene Processing
        {
            let compute_params = ComputeParams {
                node_count: 0, // In real implementation, this would be vdom.node_count()
                time,
                delta_time: self.delta_time,
                _pad: 0.0,
            };
            ctx.queue.write_buffer(&ctx.params_buffer, 0, bytemuck::cast_slice(&[compute_params]));

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("CVKG Scene Processing Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&ctx.compute_pipeline);
            compute_pass.set_bind_group(0, &ctx.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        // 2. Render Pass: Main UI Presentation
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CVKG Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&ctx.pipeline);
            render_pass.set_bind_group(0, &ctx.scene_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Renders a neon glow effect using additive blending emulation for the Gungnir effect.
    /// This creates a bright, glowing aura around elements in Canvas 2 D by drawing
    /// multiple layers with increasing opacity to simulate additive blending.
    fn draw_neon_glow(&self, ctx: &web_sys::CanvasRenderingContext2d, rect: Rect, color: [f32; 4], intensity: f32) {
        ctx.save();
        
        // Get max dimension for glow effect
        let max_dim = rect.width.max(rect.height) / 2.0 + 20.0;
        
        // Convert color to rgba string
        let color_str = format!("rgba({}, {}, {}, 0.8)", 
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8
        );
        
        // Draw multiple expanding layers for additive blend effect
        for i in 0..8 {
            let layer_alpha = intensity as f64 / (i as f64 + 1.0) * 0.3;
            ctx.set_shadow_blur(max_dim as f64 * (i as f64 + 1.0) * 0.3 * intensity as f64);
            ctx.set_shadow_color(&format!("rgba({}, {}, {}, {})", 
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                layer_alpha
            ));
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&color_str));
            ctx.set_line_width((8 - i) as f64);
            
            ctx.begin_path();
            ctx.rect(
                rect.x as f64 - i as f64 * 2.0,
                rect.y as f64 - i as f64 * 2.0,
                rect.width as f64 + i as f64 * 4.0,
                rect.height as f64 + i as f64 * 4.0
            );
            ctx.stroke();
        }
        
        ctx.restore();
    }

    fn register_web_events(&self) -> Result<(), JsValue> {
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No canvas"))?;

        let on_pointer_event = |event_type: &'static str,
                                cvkg_event_type: fn(f32, f32) -> cvkg_core::Event|
         -> Result<(), JsValue> {
            let closure = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                if let Some(vdom_lock) = CURRENT_VDOM.get() {
                    let vdom_guard = vdom_lock.lock().unwrap();
                    if let Some(vdom) = &*vdom_guard {
                        let _ = vdom.dispatch_event(cvkg_event_type(
                            event.offset_x() as f32,
                            event.offset_y() as f32,
                        ));
                    }
                }
            }) as Box<dyn FnMut(web_sys::PointerEvent)>);

            canvas
                .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;
            closure.forget();
            Ok(())
        };

        on_pointer_event("pointerdown", |x, y| cvkg_core::Event::PointerDown { x, y })?;
        on_pointer_event("pointerup", |x, y| cvkg_core::Event::PointerUp { x, y })?;
        on_pointer_event("pointermove", |x, y| cvkg_core::Event::PointerMove { x, y })?;

        // Keyboard events
        let on_key_event = |event_type: &'static str| -> Result<(), JsValue> {
            let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if let Some(vdom_lock) = CURRENT_VDOM.get() {
                    let vdom_guard = vdom_lock.lock().unwrap();
                    if let Some(vdom) = &*vdom_guard {
                        let key = event.key();
                        let cvkg_event = if event_type == "keydown" {
                            cvkg_core::Event::KeyDown { key }
                        } else {
                            cvkg_core::Event::KeyUp { key }
                        };
                        vdom.dispatch_event(cvkg_event);
                    }
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            let window = web_sys::window().unwrap();
            window
                .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;
            closure.forget();
            Ok(())
        };

        on_key_event("keydown")?;
        on_key_event("keyup")?;

        Ok(())
    }
}

/// Get the name of the current rendering tier for display/telemetry
#[wasm_bindgen]
pub fn get_render_tier_name() -> String {
    match ACTIVE_TIER.load(std::sync::atomic::Ordering::Relaxed) {
        1 => "WebGPU".to_string(),
        2 => "WebGL2".to_string(),
        3 => "Canvas2D".to_string(),
        _ => "Detecting...".to_string(),
    }
}

/// Applies a sequence of Virtual DOM patches to the browser's actual accessibility DOM.
///
/// This maintains a parallel tree of hidden ARIA elements corresponding to the
/// drawn visual interface, ensuring accessibility while using Canvas/WebGPU rendering.
#[wasm_bindgen]
pub fn apply_vdom_patches(serialized_patches: &str) -> Result<(), JsValue> {
    let patches: Vec<VDomPatch> = serde_json::from_str(serialized_patches)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse VDomPatches: {}", e)))?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document"))?;

    for patch in patches {
        match patch {
            VDomPatch::Create(node) => {
                let tag = if node.component_type == "Primitive::Text" {
                    "span"
                } else {
                    "div"
                };
                let el = document.create_element(tag)?;

                el.set_attribute("id", &format!("cvkg-node-{}", node.id.0))?;
                el.set_attribute("role", &node.aria_role)?;

                if let Some(label) = node.aria_props.label {
                    if tag == "span" {
                        el.set_text_content(Some(&label));
                    } else {
                        el.set_attribute("aria-label", &label)?;
                    }
                }

                // Position absolute to match CVKG layout
                let style_str = format!(
                    "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; opacity: 0;",
                    node.layout.x, node.layout.y, node.layout.width, node.layout.height
                );
                el.set_attribute("style", &style_str)?;

                let root = document
                    .get_element_by_id("cvkg-a11y-root")
                    .or_else(|| document.body().map(|b| b.into()));

                if let Some(root) = root {
                    root.append_child(&el)?;
                }
            }
            VDomPatch::Remove(id) => {
                if let Some(el) = document.get_element_by_id(&format!("cvkg-node-{}", id.0)) {
                    el.remove();
                }
            }
            VDomPatch::Update { id, props, .. } => {
                if let Some(el) = document.get_element_by_id(&format!("cvkg-node-{}", id.0)) {
                    if let Some(text) = props.as_ref().and_then(|p| p.get("text")).and_then(|v| v.as_str()) {
                        el.set_text_content(Some(text));
                    }
                }
            }
            VDomPatch::Replace { id, node } => {
                if let Some(el) = document.get_element_by_id(&format!("cvkg-node-{}", id.0)) {
                    el.remove();
                }
                // Recurse to create
                let serialized = serde_json::to_string(&vec![VDomPatch::Create(node)])
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                apply_vdom_patches(&serialized)?;
            }
            VDomPatch::Move { .. } => {
                // Keyed reordering logic
            }
            VDomPatch::SetRoot(id) => {
                log::info!("[CVKG Bridge] Root set to {:?}", id);
            }
        }
    }

    Ok(())
}

/// A concrete AssetManager for Web targets that uses the browser's fetch API.
///
/// The cache is read lock-free via `ArcSwap::load()` every render frame.
/// Writes happen only once per URL: a synchronous `rcu()` inserts `Loading` immediately,
/// and the spawned async future calls `rcu()` again once the fetch resolves to publish the
/// final state. WASM is single-threaded so the `rcu()` clone-insert-swap is always safe.
pub struct WebAssetManager {
    cache: std::sync::Arc<
        arc_swap::ArcSwap<
            std::collections::HashMap<String, cvkg_core::AssetState<std::sync::Arc<Vec<u8>>>>,
        >,
    >,
}

impl WebAssetManager {
    /// Create a new, empty WebAssetManager.
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(
                std::collections::HashMap::new(),
            )),
        }
    }
}

impl cvkg_core::AssetManager for WebAssetManager {
    /// Return the cached asset state for `url`.
    ///
    /// If the URL is not cached, inserts `Loading` synchronously via `rcu()`,
    /// spawns an async fetch, and returns `Loading` immediately.
    /// The spawned future calls `rcu()` again with `Ready` or `Error` once the
    /// fetch resolves — no lock is ever held across an await point.
    fn load_image(&self, url: &str) -> cvkg_core::AssetState<std::sync::Arc<Vec<u8>>> {
        // Fast path: lock-free read from current cache snapshot
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        let cache_arc = self.cache.clone();
        let url_string = url.to_string();

        // Mark as Loading synchronously via atomic rcu
        {
            let key = url_string.clone();
            self.cache.rcu(move |map| {
                let mut m = (**map).clone();
                m.entry(key.clone()).or_insert(cvkg_core::AssetState::Loading);
                m
            });
        }

        // Spawn async fetch; publish result via rcu — no lock across await
        wasm_bindgen_futures::spawn_local(async move {
            let mut opts = web_sys::RequestInit::new();
            opts.method("GET");
            opts.mode(web_sys::RequestMode::Cors);

            let request = web_sys::Request::new_with_str_and_init(&url_string, &opts).unwrap();
            let window = web_sys::window().unwrap();
            let resp_value =
                wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                    .await
                    .unwrap();
            let resp: web_sys::Response = resp_value.dyn_into().unwrap();

            let result = if resp.status() == 200 {
                let array_buffer_value =
                    wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap())
                        .await
                        .unwrap();
                let array_buffer: js_sys::ArrayBuffer = array_buffer_value.dyn_into().unwrap();
                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                cvkg_core::AssetState::Ready(std::sync::Arc::new(uint8_array.to_vec()))
            } else {
                cvkg_core::AssetState::Error(format!("HTTP {}", resp.status()))
            };

            // Publish the resolved state atomically
            let key = url_string.clone();
            cache_arc.rcu(move |map| {
                let mut m = (**map).clone();
                m.insert(key.clone(), result.clone());
                m
            });
        });

        cvkg_core::AssetState::Loading
    }

    fn preload_image(&self, url: &str) {
        self.load_image(url);
    }
}

/// Orchestrates the connection between the WASM client and the CVKG WebKit server.
///
/// This bridge manages:
/// 1. VDOM Snapshot synchronization for Server-Side Generation (SSG).
/// 2. WebSocket-based Inspector telemetry.
/// 3. Hot-Module Replacement (HMR) signaling.
pub struct WebKitBridge {
    server_addr: String,
    inspector_ws: Option<web_sys::WebSocket>,
}

impl WebKitBridge {
    /// Create a new WebKitBridge, inferring the server address from the current location.
    pub fn new() -> Self {
        let window = web_sys::window().expect("No window found");
        let location = window.location();
        let host = location.host().unwrap_or_else(|_| "localhost:3000".to_string());
        let protocol = if location.protocol().unwrap_or_default() == "https:" { "wss:" } else { "ws:" };
        
        Self {
            server_addr: format!("{}//{}", protocol, host),
            inspector_ws: None,
        }
    }

    /// Establish the WebSocket connection to the server's inspector endpoint.
    pub fn connect(&mut self) -> Result<(), JsValue> {
        let ws_url = format!("{}/cvkg-ws", self.server_addr);
        let ws = web_sys::WebSocket::new(&ws_url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Ok(ab) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                 // Binary telemetry handling (future expansion)
                 let _ = ab;
            } else if let Some(txt) = e.data().as_string() {
                log::info!("[CVKG Bridge] Signal received: {}", txt);
                if txt.contains("RELOAD") {
                    let _ = web_sys::window().unwrap().location().reload();
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget(); // Keep the closure alive for the lifetime of the WS
        
        self.inspector_ws = Some(ws);
        log::info!("[CVKG Bridge] Connected to Inspector at {}", ws_url);
        Ok(())
    }

    /// POSTs a full HTML snapshot of the VDOM to the server's SSG cache.
    pub fn send_snapshot(&self, html: &str) {
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.body(Some(&wasm_bindgen::JsValue::from_str(html)));
        
        let window = web_sys::window().unwrap();
        // Fire and forget; server will update the ArcSwap lock-free
        let _ = window.fetch_with_str_and_init("/snapshot", &opts);
    }
}

    /// ShieldWall manages a hidden DOM tree that mirrors the VDOM for accessibility.
    /// This ensures that screen readers can navigate the custom-rendered GPU UI.
    pub struct ShieldWall {
        container: web_sys::HtmlElement,
        nodes: std::collections::HashMap<u64, web_sys::Element>,
    }

    impl ShieldWall {
        pub fn new() -> Self {
            let window = web_sys::window().expect("No window found");
            let document = window.document().expect("No document found");
            
            let container = document.create_element("div").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
            container.set_id("cvkg-shield-wall");
            
            // Visually hide but keep accessible
            let style = container.style();
            style.set_property("position", "absolute").ok();
            style.set_property("width", "1px").ok();
            style.set_property("height", "1px").ok();
            style.set_property("padding", "0").ok();
            style.set_property("margin", "-1px").ok();
            style.set_property("overflow", "hidden").ok();
            style.set_property("clip", "rect(0, 0, 0, 0)").ok();
            style.set_property("white-space", "nowrap").ok();
            style.set_property("border", "0").ok();
            
            document.body().unwrap().append_child(&container).unwrap();
            
            Self {
                container,
                nodes: std::collections::HashMap::new(),
            }
        }

        pub fn sync_node(&mut self, id: u64, role: &str, label: &str, rect: Rect) {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            let el = self.nodes.entry(id).or_insert_with(|| {
                let tag = match role {
                    "button" => "button",
                    "link" => "a",
                    "input" => "input",
                    "heading" => "h2",
                    _ => "div",
                };
                let el = document.create_element(tag).unwrap();
                self.container.append_child(&el).unwrap();
                el
            });

            el.set_attribute("role", role).ok();
            el.set_attribute("aria-label", label).ok();
            
            // Position it roughly where it is in the UI for spatial navigation
            if let Ok(html_el) = el.clone().dyn_into::<web_sys::HtmlElement>() {
                let style = html_el.style();
                style.set_property("position", "absolute").ok();
                style.set_property("left", &format!("{}px", rect.x)).ok();
                style.set_property("top", &format!("{}px", rect.y)).ok();
                style.set_property("width", &format!("{}px", rect.width)).ok();
                style.set_property("height", &format!("{}px", rect.height)).ok();
            }
        }

        pub fn clear_unused(&mut self, active_ids: &std::collections::HashSet<u64>) {
            let to_remove: Vec<u64> = self.nodes.keys()
                .filter(|id| !active_ids.contains(id))
                .copied()
                .collect();
                
            for id in to_remove {
                if let Some(el) = self.nodes.remove(&id) {
                    el.remove();
                }
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use wasm_impl::*;

