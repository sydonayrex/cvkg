#![cfg(target_arch = "wasm32")]
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

use cvkg_core::{FrameRenderer, Rect, Renderer, View, RenderTier};
use wasm_bindgen::prelude::*;

use cvkg_vdom::VDomPatch;
use web_sys::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneUniforms {
    resolution: [f32; 2],
    time: f32,
    _pad: f32,
}

static CURRENT_VDOM: std::sync::OnceLock<std::sync::Mutex<Option<cvkg_vdom::VDom>>> =
    std::sync::OnceLock::new();

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_buffer: wgpu::Buffer,
}

/// Web renderer backend implementing the CvkgRenderer trait
pub struct WebRenderer {
    canvas: Option<web_sys::HtmlCanvasElement>,
    canvas_context: Option<web_sys::CanvasRenderingContext2d>,
    gl_context: Option<web_sys::WebGl2RenderingContext>,
    #[allow(dead_code)]
    webgpu_context: Option<GpuContext>,
    tier: RenderTier,
    vdom: Option<cvkg_vdom::VDom>,
    previous_vdom: Option<cvkg_vdom::VDom>,
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
}

// WebRenderer is only used on a single thread in WASM, but Renderer trait requires Send.
unsafe impl Send for WebRenderer {}

impl WebRenderer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        Self {
            canvas: None,
            canvas_context: None,
            gl_context: None,
            webgpu_context: None,
            tier: RenderTier::Tier3Fallback,
            vdom: Some(cvkg_vdom::VDom::new()),
            previous_vdom: None,
            start_time: now,
            asset_manager: std::sync::Arc::new(WebAssetManager::new()),
            telemetry: cvkg_core::TelemetryData::default(),
            frame_budget: cvkg_core::FrameBudget::default(),
            last_redraw_start: now,
            delta_time: 0.016,
            redraw_requested: false,
        }
    }

    /// Get real-time performance telemetry.
    fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
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
            self.init_canvas()
        }
    }

    #[doc(hidden)]
    pub async fn init_async(&mut self) -> Result<(), JsValue> {
        #[cfg(feature = "webgpu")]
        {
            match self.init_webgpu_async().await {
                Ok(_) => {
                    self.use_webgpu = true;
                    log::info!("Initialized WebGPU context");
                }
                Err(e) => {
                    log::warn!("WebGPU initialization failed: {:?}", e);
                }
            }
        }
        self.init_canvas()?;

        // Register AssetManager in the environment
        cvkg_core::env::insert::<cvkg_core::AssetKey>(self.asset_manager.clone());

        self.register_web_events()?;
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
        let new_vdom = cvkg_vdom::VDom::build(&view, rect);

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
        self.init_base_canvas()?;

        // 1. Try WebGPU
        #[cfg(feature = "webgpu")]
        {
            log::info!("Attempting WebGPU initialization...");
            if let Ok(_) = self.init_webgpu_async().await {
                self.tier = RenderTier::Tier1GPU;
                log::info!("Forge Success: WebGPU tier active.");
                return Ok(self.tier);
            }
        }

        // 2. Try WebGL2
        log::info!("Attempting WebGL2 fallback...");
        if let Ok(_) = self.init_webgl2() {
            self.tier = RenderTier::Tier2GPU;
            log::info!("Forge Success: WebGL2 tier active.");
            return Ok(self.tier);
        }

        // 3. Fallback to Canvas 2D
        log::info!("Attempting Canvas 2D final fallback...");
        self.init_canvas_2d()?;
        self.tier = RenderTier::Tier3Fallback;
        log::info!("Forge Success: Canvas 2D tier active (Degraded).");
        Ok(self.tier)
    }

    fn init_base_canvas(&mut self) -> Result<(), JsValue> {
        if self.canvas.is_none() {
            let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
            let document = window
                .document()
                .ok_or_else(|| JsValue::from_str("No document found"))?;
            let canvas = document
                .create_element("canvas")?
                .dyn_into::<web_sys::HtmlCanvasElement>()?;
            canvas.set_width(window.inner_width()?.as_f64().unwrap_or(800.0) as u32);
            canvas.set_height(window.inner_height()?.as_f64().unwrap_or(600.0) as u32);
            self.canvas = Some(canvas);
        }
        Ok(())
    }

    fn init_webgl2(&mut self) -> Result<(), JsValue> {
        let canvas = self.canvas.as_ref().unwrap();
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 not supported"))?
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;
        self.gl_context = Some(context);
        Ok(())
    }

    fn init_canvas_2d(&mut self) -> Result<(), JsValue> {
        let canvas = self.canvas.as_ref().unwrap();
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D context not supported"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        self.canvas_context = Some(context);
        Ok(())
    }

    #[cfg(feature = "webgpu")]
    async fn init_webgpu_async(&mut self) -> Result<(), JsValue> {
        // Create WebGPU instance with explicit fields (no Default in v29)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        // Get the canvas element
        let canvas = self
            .canvas
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Canvas not initialized"))?;

        // Create surface from canvas (modern API returns Result)
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {}", e)))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to request adapter: {}", e)))?;

        // Request device and queue with modern Descriptor
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("CVKG WebGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
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
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas_width,
            height: canvas_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                bind_group_layouts: &[&scene_bind_group_layout],
                push_constant_ranges: &[],
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
            multiview: None,
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
        });

        self.use_webgpu = true;
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
            match self.tier {
                RenderTier::Tier1GPU | RenderTier::Tier2GPU => {
                    // High/Mid Tier: Apply CSS blur filter approximation
                    gl.set_filter(&format!("blur({}px)", blur / 4.0));
                    gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                        "rgba(255, 255, 255, {})",
                        opacity * 0.2
                    )));
                }
                RenderTier::Tier3Fallback => {
                    // Low Tier: Degrade to simple semi-transparent white (glassmorphism -> transparency)
                    gl.set_fill_style(&wasm_bindgen::JsValue::from_str(&format!(
                        "rgba(255, 255, 255, {})",
                        opacity * 0.4
                    )));
                }
            }
            gl.fill_rect(
                rect.x as f64,
                rect.y as f64,
                rect.width as f64,
                rect.height as f64,
            );
            gl.restore();
        }
    }

    fn gungnir(&mut self, rect: Rect, glow_radius: f32, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            match self.tier {
                RenderTier::Tier1GPU | RenderTier::Tier2GPU => {
                    gl.set_shadow_blur(glow_radius as f64);
                    gl.set_shadow_color(&format!(
                        "rgba({}, {}, {}, {})",
                        color[0] * 255.0,
                        color[1] * 255.0,
                        color[2] * 255.0,
                        color[3]
                    ));
                }
                RenderTier::Tier3Fallback => {
                    // No glow on fallback tier
                }
            }
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
            gl.restore();
        }
    }

    fn mjolnir_shatter(&mut self, rect: Rect, _pieces: u32, _force: f32, color: [f32; 4]) {
        if let Some(ref gl) = self.canvas_context {
            gl.save();
            match self.tier {
                RenderTier::Tier1GPU | RenderTier::Tier2GPU => {
                    // Approximate shatter with dashed lines
                    gl.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(255, 255, 255, 0.8)"));
                    let _ = gl.set_line_dash(&wasm_bindgen::JsValue::from_str("[5, 2]"));
                    gl.stroke_rect(
                        rect.x as f64,
                        rect.y as f64,
                        rect.width as f64,
                        rect.height as f64,
                    );
                }
                RenderTier::Tier3Fallback => {
                    // Degrade to simple outline
                    gl.set_stroke_style(&wasm_bindgen::JsValue::from_str(&format!(
                        "rgba({}, {}, {}, 0.5)",
                        color[0] * 255.0,
                        color[1] * 255.0,
                        color[2] * 255.0
                    )));
                    gl.stroke_rect(
                        rect.x as f64,
                        rect.y as f64,
                        rect.width as f64,
                        rect.height as f64,
                    );
                }
            }
            gl.restore();
        }
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
                if self.telemetry.debug_layout {
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
    }

    fn set_debug_layout(&mut self, enabled: bool) {
        self.telemetry.debug_layout = enabled;
    }

    fn get_debug_layout(&self) -> bool {
        self.telemetry.debug_layout
    }

    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        log::trace!("Web: register_shared_element '{}' {:?}", id, rect);
    }

    fn delta_time(&self) -> f32 {
        self.delta_time
    }

    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }
}

impl FrameRenderer<()> for WebRenderer {
    fn begin_frame(&mut self) -> () {
        cvkg_core::begin_render_phase();
        let now = web_sys::window().unwrap().performance().unwrap().now();
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
        cvkg_core::end_render_phase();
    }
}

impl WebRenderer {
    fn render_webgl2(&mut self) -> Result<(), JsValue> {
        let gl = self
            .gl_context
            .as_ref()
            .ok_or_else(|| JsValue::from_str("WebGL2 context missing"))?;
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        Ok(())
    }

    fn render_webgpu(&mut self) -> Result<(), JsValue> {
        let ctx = self
            .webgpu_context
            .as_mut()
            .ok_or_else(|| JsValue::from_str("WebGPU context missing"))?;

        let current_time = web_sys::window().unwrap().performance().unwrap().now();
        let time = ((current_time - self.start_time) / 1000.0) as f32;

        let uniforms = SceneUniforms {
            resolution: [ctx.config.width as f32, ctx.config.height as f32],
            time,
            _pad: 0.0,
        };

        ctx.queue
            .write_buffer(&ctx.scene_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = match ctx.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
                ctx.surface.configure(&ctx.device, &ctx.config);
                return Err(JsValue::from_str("Surface outdated, reconfigured"));
            }
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Failed to get surface texture: {}",
                    e
                )));
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
            });

            render_pass.set_pipeline(&ctx.pipeline);
            render_pass.set_bind_group(0, &ctx.scene_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
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
            VDomPatch::Update { id, props } => {
                if let Some(el) = document.get_element_by_id(&format!("cvkg-node-{}", id.0)) {
                    if let Some(text) = props.get("text").and_then(|v| v.as_str()) {
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
