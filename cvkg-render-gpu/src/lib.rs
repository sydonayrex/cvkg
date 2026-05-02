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
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! # Surtr Render Pipeline
//!
//! The "Fiery Giant" of the CVKG architecture. This is the authoritative GPU renderer
//! powered by `wgpu`. It manages the heat of the GPU to forge high-fidelity
//! "Berserker" aesthetics.
//!
//! - **The Flaming Sword**: Command submission and synchronization.
//! - **Muspelheim Passes**: Multi-pass Gaussian blur and bloom for Bifrost/Gungnir.

/// ShelfPacker — A simple shelf-based atlas packer for the Mega-Atlas.
#[derive(Debug, Clone)]
struct ShelfPacker {
    width: u32,
    height: u32,
    shelf_y: u32,
    shelf_height: u32,
    current_x: u32,
}

impl ShelfPacker {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height, shelf_y: 0, shelf_height: 0, current_x: 0 }
    }

    fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        // Bounds check for the item itself
        if w > self.width || h > self.height {
            return None;
        }

        // Shelf packing algorithm: simple and fast for real-time UI.
        if self.current_x + w > self.width {
            // New shelf
            self.shelf_y += self.shelf_height;
            self.current_x = 0;
            self.shelf_height = 0;
        }

        if self.shelf_y + h > self.height {
            return None; // Out of space
        }

        let pos = (self.current_x, self.shelf_y);
        self.current_x += w;
        if h > self.shelf_height {
            self.shelf_height = h;
        }
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_packer_basic() {
        let mut packer = ShelfPacker::new(100, 100);
        
        // Pack first item
        assert_eq!(packer.pack(10, 10), Some((0, 0)));
        assert_eq!(packer.current_x, 10);
        assert_eq!(packer.shelf_height, 10);
        
        // Pack second item on same shelf
        assert_eq!(packer.pack(20, 15), Some((10, 0)));
        assert_eq!(packer.current_x, 30);
        assert_eq!(packer.shelf_height, 15);
    }

    #[test]
    fn test_shelf_packer_wrap() {
        let mut packer = ShelfPacker::new(100, 100);
        packer.pack(60, 10);
        
        // This should trigger a new shelf
        assert_eq!(packer.pack(50, 20), Some((0, 10)));
        assert_eq!(packer.current_x, 50);
        assert_eq!(packer.shelf_y, 10);
        assert_eq!(packer.shelf_height, 20);
    }

    #[test]
    fn test_shelf_packer_full() {
        let mut packer = ShelfPacker::new(10, 10);
        assert_eq!(packer.pack(11, 5), None);
        assert_eq!(packer.pack(5, 11), None);
    }
}

use cvkg_core::{Mesh, Rect, Renderer, LAYOUT_DIRTY};
use std::sync::Arc;
use std::sync::atomic::Ordering;
include!(concat!(env!("OUT_DIR"), "/shader_spirv.rs"));


/// SvgModel — A collection of tessellated triangles representing a vector icon.
#[derive(Clone, Debug)]
pub struct SvgModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub view_box: Rect,
}

// ShieldWall — re-export AccessKit types so callers can build tree updates
// without depending on accesskit directly.
pub use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree,
    TreeId, TreeUpdate,
};
pub use accesskit_winit::Adapter as ShieldWallAdapter;

// Re-export ColorTheme and SceneUniforms for cvkg-render-gpu users
pub use cvkg_core::{
    ColorTheme, SceneUniforms,
};

use lyon::tessellation::{
    FillOptions, FillTessellator, FillVertex, FillVertexConstructor, VertexBuffers, BuffersBuilder
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub mode: u32,
    pub radius: f32,
    pub slice: [f32; 4],
    pub logical: [f32; 2],
    pub size: [f32; 2],
    pub screen: [f32; 2],
    pub clip: [f32; 4], // [x, y, width, height]
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub _pad: f32,
}



/// Represents a single batched GPU draw call.
/// Batches are broken whenever the active texture or primitive mode changes.
#[derive(Debug, Clone)]
struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: Option<Rect>,
    pub index_start: u32,
    pub index_count: u32,
    pub is_glass: bool,
    pub is_ui: bool,
}

#[derive(Debug, Clone, Copy)]
struct ShadowState {
    pub radius: f32,
    pub color: [f32; 4],
    pub _offset: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 14] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
        3 => Float32x4, // color
        4 => Uint32,    // mode
        5 => Float32,   // radius
        6 => Float32x4, // slice
        7 => Float32x2, // logical
        8 => Float32x2, // size
        9 => Float32x2, // screen
        10 => Float32x4, // clip
        11 => Float32x2, // translation
        12 => Float32x2, // scale
        13 => Float32    // rotation
    ];


    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// SurtrRenderer implements the high-performance GPU backend.
pub struct SurtrRenderer {
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Multi-Window Surface Management
    surfaces: std::collections::HashMap<winit::window::WindowId, SurfaceContext>,
    current_window: Option<winit::window::WindowId>,
    pub headless_context: Option<HeadlessContext>,

    // Mega-Atlas (Shared across all windows)
    text_engine: runic_text::RunicTextEngine,
    mega_atlas_tex: wgpu::Texture,
    #[allow(dead_code)]
    mega_atlas_view: wgpu::TextureView,
    _mega_atlas_sampler: wgpu::Sampler,
    mega_atlas_bind_group: wgpu::BindGroup,
    text_cache: std::collections::HashMap<runic_text::CacheKey, (Rect, f32, f32)>,
    atlas_packer: ShelfPacker,
    image_uv_registry: std::collections::HashMap<String, Rect>,
    texture_registry: std::collections::HashMap<String, u32>,
    svg_cache: std::collections::HashMap<String, SvgModel>,

    // Niflheim Resources (Shared)
    dummy_texture_bind_group: wgpu::BindGroup,
    dummy_env_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    shared_elements: std::collections::HashMap<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    draw_calls: Vec<DrawCall>,
    current_texture_id: Option<u32>,

    // Opacity & Clip Stacks
    opacity_stack: Vec<f32>,
    clip_stack: Vec<Rect>,
    slice_stack: Vec<(f32, f32)>,
    shadow_stack: Vec<ShadowState>,

    // The Forge's Heart (Shared Berserker State)
    theme_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,
    berserker_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    berserker_bind_group_layout: wgpu::BindGroupLayout,
    start_time: std::time::Instant,
    current_theme: ColorTheme,
    current_scene: SceneUniforms,
    current_z: f32,

    // Muspelheim Pipelines (Shared)
    pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    bloom_extract_pipeline: wgpu::RenderPipeline,
    blur_h_pipeline: wgpu::RenderPipeline,
    blur_v_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    env_bind_group_layout: wgpu::BindGroupLayout,
    
    // Telemetry
    pub telemetry: cvkg_core::TelemetryData,

    /// Configuration for render-loop frame timing and degradation strategies.
    pub frame_budget: cvkg_core::FrameBudget,
    /// Instant at the start of the last redraw, used for measuring frame timings.
    pub last_redraw_start: std::time::Instant,
    /// Instant at the start of the last frame, used for frame_time_ms calculation.
    pub last_frame_start: std::time::Instant,

    // VRAM Tracking (Bytes)
    vram_buffers_bytes: u64,
    vram_textures_bytes: u64,

    // Debugging
    _debug_layout: bool,

    // Transform Stack
    transform_stack: Vec<([f32; 2], [f32; 2], f32)>,
    /// Whether a redraw has been requested for the next frame.
    pub redraw_requested: bool,
}


struct SurfaceContext {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    scene_texture: wgpu::TextureView,
    scene_bind_group: wgpu::BindGroup,
    scene_texture_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    blur_texture_a: wgpu::TextureView,
    blur_texture_b: wgpu::TextureView,
    blur_bind_group_a: wgpu::BindGroup,
    blur_bind_group_b: wgpu::BindGroup,
    blur_env_bind_group_a: wgpu::BindGroup,
    scale_factor: f32,
    sampler: wgpu::Sampler,
}

/// HeadlessContext — A rendering target for surface-less execution.
pub struct HeadlessContext {
    pub scene_texture: wgpu::TextureView,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_texture_bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,
    pub blur_texture_a: wgpu::TextureView,
    pub blur_texture_b: wgpu::TextureView,
    pub blur_bind_group_a: wgpu::BindGroup,
    pub blur_bind_group_b: wgpu::BindGroup,
    pub blur_env_bind_group_a: wgpu::BindGroup,
    pub scale_factor: f32,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
}

const MAX_VERTICES: usize = 100_000;
const MAX_INDICES: usize = 150_000;

impl SurtrRenderer {
    /// forge — Initializes the Surtr GPU renderer from a winit window.
    ///
    /// This method performs the following:
    /// 1. Negotiates a wgpu surface and adapter.
    /// 2. Forges the Muspelheim multi-pass pipeline layouts.
    /// 3. Initializes the Berserker state buffers and texture registries.
    pub async fn forge(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        println!("[GPU] Requesting HighPerformance adapter...");
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok();

        if adapter.is_none() {
            println!("[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            println!("[GPU] Hardware adapters failed, trying Software fallback...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: true,
                })
                .await
                .ok();
        }

        let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");
        println!("[GPU] Selected adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Forge"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
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
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            Some((window, surface, config)),
            None
        ).await
    }

    async fn forge_internal(
        instance: Arc<wgpu::Instance>,
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_info: Option<(Arc<winit::window::Window>, wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
        headless_info: Option<(u32, u32, wgpu::TextureFormat)>,
    ) -> Self {
        let format = if let Some((_, _, ref config)) = surface_info {
            config.format
        } else if let Some((_, _, f)) = headless_info {
            f
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Main Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(SPIRV)),
        });

        // Niflheim Bind Group Layout (for textures/samplers)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Niflheim Texture Bind Group Layout"),
            });

        // Environment Bind Group Layout (for blurred background / Bifrost)
        // Environment Bind Group Layout (for blurred background / Bifrost)
        let env_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Surtr Environment Bind Group Layout"),
            });

        let berserker_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Surtr Berserker Bind Group Layout"),
            });

        // Pipeline setup
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surtr Main Pipeline Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        // Specialized layout for post-processing (Bloom Extract, Blur) which only need Group 0 + Globals
        let post_process_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Muspelheim Post Process Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        // Specialized layout for composite (Blur + Scene)
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Muspelheim Composite Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_background"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Muspelheim Bloom Extract Pipeline
        let bloom_extract_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Muspelheim Bloom Extract"),
                layout: Some(&post_process_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_fullscreen"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_bloom_extract"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        // Muspelheim Blur Pipelines (H and V)
        let blur_h_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Horizontal Blur"),
            layout: Some(&post_process_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blur_h"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let blur_v_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Vertical Blur"),
            layout: Some(&post_process_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blur_v"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Muspelheim Composite Pipeline (additive blend onto screen)
        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Composite"),
            layout: Some(&composite_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_composite"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    // Additive blend: src + dst — glow lights up the scene
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Forge the Mega-Atlas (4096x4096 RGBA for production batching)
        let mega_atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Mega-Atlas"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mega_atlas_view_obj = mega_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, // Use linear for images
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mega_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&mega_atlas_view_obj),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&text_sampler),
                },
            ],
            label: Some("Mega-Atlas Bind Group"),
        });

        // Clear the mega atlas to transparency initially
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &mega_atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &vec![0u8; 4096 * 4096 * 4],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4096 * 4),
                rows_per_image: Some(4096),
            },
            wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
        );

        // Forge the Niflheim Dummy Texture (1x1 White)
        let dummy_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Niflheim Dummy Texture"),
            size: dummy_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &dummy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0, 0, 0, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            dummy_size,
        );

        let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let dummy_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_sampler),
                },
            ],
            label: Some("Dummy Texture Bind Group"),
        });

        let dummy_env_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_sampler),
                },
            ],
            label: Some("Dummy Env Bind Group"),
        });

        let mut texture_registry = std::collections::HashMap::new();
        let mut texture_bind_groups = Vec::new();

        texture_registry.insert("__mega_atlas".to_string(), 0);
        texture_bind_groups.push(mega_atlas_bind_group.clone());

        // Forge the Anvil (Buffers)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Vertex Anvil"),
            size: (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Index Anvil"),
            size: (MAX_INDICES * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Forge the Heart (Berserker Uniforms)
        let current_theme = ColorTheme::default();
        use wgpu::util::DeviceExt;
        let theme_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr Theme Buffer"),
            contents: bytemuck::bytes_of(&current_theme),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (width, height, scale_factor) = if let Some((ref window, _, ref config)) = surface_info {
            (config.width, config.height, window.scale_factor() as f32)
        } else if let Some((w, h, _)) = headless_info {
            (w, h, 1.0)
        } else {
            (1280, 720, 1.0)
        };

        let mut current_scene = SceneUniforms::new(
            width as f32 / scale_factor,
            height as f32 / scale_factor,
        );
        current_scene.scale_factor = scale_factor;
        let scene_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr Scene Buffer"),
            contents: bytemuck::bytes_of(&current_scene),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let berserker_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &berserker_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: theme_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene_buffer.as_entire_binding(),
                },
            ],
            label: Some("Surtr Berserker Bind Group"),
        });

        let mut surfaces = std::collections::HashMap::new();
        let mut current_window = None;
        let mut headless_context = None;

        if let Some((window, surface, config)) = surface_info {
            let window_id = window.id();
            let ctx = Self::create_surface_context(
                &device,
                surface,
                config,
                &env_bind_group_layout,
                &texture_bind_group_layout,
                scale_factor,
            );
            surfaces.insert(window_id, ctx);
            current_window = Some(window_id);
        } else if let Some((w, h, f)) = headless_info {
            headless_context = Some(Self::create_headless_context(
                &device,
                w,
                h,
                f,
                &env_bind_group_layout,
                &texture_bind_group_layout,
            ));
        }

        Self {
            instance,
            adapter,
            device,
            queue,
            surfaces,
            current_window,
            headless_context,
            pipeline,
            bloom_extract_pipeline,
            blur_h_pipeline,
            blur_v_pipeline,
            composite_pipeline,
            env_bind_group_layout,
            text_engine: runic_text::RunicTextEngine::default(),
            mega_atlas_tex,
            mega_atlas_view: mega_atlas_view_obj,
            _mega_atlas_sampler: text_sampler,
            mega_atlas_bind_group,
            text_cache: std::collections::HashMap::new(),
            atlas_packer: ShelfPacker::new(4096, 4096),
            image_uv_registry: std::collections::HashMap::new(),
            svg_cache: std::collections::HashMap::new(),
            dummy_texture_bind_group,
            dummy_env_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
            texture_registry,
            shared_elements: std::collections::HashMap::new(),
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
            draw_calls: Vec::new(),
            current_texture_id: None,
            opacity_stack: vec![1.0],
            clip_stack: Vec::new(),
            slice_stack: Vec::new(),
            shadow_stack: Vec::new(),
            theme_buffer,
            scene_buffer,
            berserker_bind_group,
            berserker_bind_group_layout,
            start_time: std::time::Instant::now(),
            current_theme,
            current_scene,
            background_pipeline,
            current_z: 0.0,
            telemetry: cvkg_core::TelemetryData::default(),
            last_frame_start: std::time::Instant::now(),
            last_redraw_start: std::time::Instant::now(),
            frame_budget: cvkg_core::FrameBudget::default(),
            vram_buffers_bytes: 0,
            vram_textures_bytes: 0,
            _debug_layout: false,
            transform_stack: Vec::new(),
            redraw_requested: false,
        }
    }

    /// Update VRAM telemetry based on currently allocated resources.
    fn update_vram_telemetry(&mut self) {
        // Calculate Buffer VRAM
        let mut buffer_bytes = 0;
        buffer_bytes += (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64;
        buffer_bytes += (MAX_INDICES * std::mem::size_of::<u32>()) as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::ColorTheme>() as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::SceneUniforms>() as u64;
        self.vram_buffers_bytes = buffer_bytes;

        // Calculate Texture VRAM
        let mut texture_bytes = 0;
        texture_bytes += 4096 * 4096 * 4; // Mega Atlas (RGBA8)
        texture_bytes += 4; // Dummy (RGBA8)
        
        for ctx in self.surfaces.values() {
            let bpp = 4; 
            let surface_bytes = (ctx.config.width * ctx.config.height * bpp) as u64;
            texture_bytes += surface_bytes * 3; // scene, blur_a, blur_b
            texture_bytes += (ctx.config.width * ctx.config.height * 4) as u64; // depth (Depth32Float)
        }
        
        self.vram_textures_bytes = texture_bytes;
        
        self.telemetry.vram_buffers_mb = buffer_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_textures_mb = texture_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_pipelines_mb = 0.0; 
        self.telemetry.vram_usage_mb = self.telemetry.vram_buffers_mb + self.telemetry.vram_textures_mb;
    }

    /// Get real-time performance telemetry.
    pub fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
    }

    /// resize — Reconfigures a specific surface and its internal textures.
    pub fn resize(&mut self, window_id: winit::window::WindowId, width: u32, height: u32, scale_factor: f32) {
        if width > 0 && height > 0
            && let Some(ctx) = self.surfaces.get_mut(&window_id) {
                ctx.config.width = width;
                ctx.config.height = height;
                ctx.scale_factor = scale_factor;
                ctx.surface.configure(&self.device, &ctx.config);

                // Re-create Muspelheim textures for this surface
                let texture_desc = wgpu::TextureDescriptor {
                    label: Some("Surtr Scene Texture"),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: ctx.config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                };

                let scene_tex = self.device.create_texture(&texture_desc);
                ctx.scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

                let blur_tex_a = self.device.create_texture(&texture_desc);
                ctx.blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

                let blur_tex_b = self.device.create_texture(&texture_desc);
                ctx.blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

                // Re-create bind groups for this surface
                ctx.scene_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.env_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&ctx.scene_texture) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&ctx.sampler) },
                    ],
                    label: Some("Scene Bind Group Resize"),
                });

                ctx.scene_texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&ctx.scene_texture) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&ctx.sampler) },
                    ],
                    label: Some("Scene Texture Bind Group Resize"),
                });

                ctx.blur_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&ctx.blur_texture_a) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&ctx.sampler) },
                    ],
                    label: Some("Blur Bind Group A Resize"),
                });

                ctx.blur_bind_group_b = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&ctx.blur_texture_b) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&ctx.sampler) },
                    ],
                    label: Some("Blur Bind Group B Resize"),
                });

                ctx.blur_env_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.env_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&ctx.blur_texture_a) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&ctx.sampler) },
                    ],
                    label: Some("Blur Env Bind Group A Resize"),
                });

                let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Surtr Depth Texture"),
                    size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                ctx.depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    /// begin_frame_headless — Strike the flaming sword to begin a new GPU frame for headless rendering.
    pub fn begin_frame_headless(&mut self) -> wgpu::CommandEncoder {
        self.current_window = None;
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.current_z = 0.0;
        
        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

        let ctx = self.headless_context.as_ref().expect("Headless context not initialized");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.width as f32 / ctx.scale_factor;
        let logical_h = ctx.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj = glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surtr Headless Command Encoder"),
        })
    }

    /// begin_frame — Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        self.current_window = Some(window_id);
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.current_z = 0.0;
        
        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

        let ctx = self.surfaces.get(&window_id).expect("Window not registered");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.config.width as f32 / ctx.scale_factor;
        let logical_h = ctx.config.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj = glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surtr Command Encoder"),
        })
    }




    /// register_window — Attaches a new OS window to the shared GPU context.
    pub fn register_window(&mut self, window: Arc<winit::window::Window>) {
        let size = window.inner_size();
        let surface = self.instance.create_surface(window.clone()).expect("Failed to create surface");
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.device, &config);

        let ctx = Self::create_surface_context(
            &self.device,
            surface,
            config,
            &self.env_bind_group_layout,
            &self.texture_bind_group_layout,
            window.scale_factor() as f32,
        );

        self.surfaces.insert(window.id(), ctx);
    }

    fn create_headless_context(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> HeadlessContext {
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Headless Scene Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_a = device.create_texture(&texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Scene Bind Group"),
        });

        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Scene Texture Bind Group"),
        });

        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Bind Group A"),
        });

        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Bind Group B"),
        });

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Env Bind Group A"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Output Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        HeadlessContext {
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_texture_a,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            scale_factor: 1.0,
            sampler,
            width,
            height,
            output_texture,
            output_view,
        }
    }

    fn create_surface_context(
        device: &wgpu::Device,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        scale_factor: f32,
    ) -> SurfaceContext {
        let width = config.width;
        let height = config.height;

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Scene Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_a = device.create_texture(&texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Scene Bind Group"),
        });

        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Scene Texture Bind Group"),
        });

        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Bind Group A"),
        });

        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Bind Group B"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                },
            ],
            label: Some("Blur Env Bind Group A"),
        });

        SurfaceContext {
            surface,
            config,
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_texture_a,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            scale_factor,
            sampler,
        }
    }

    /// Reset the internal clock (for interactive effects)
    pub fn reset_time(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        mode: u32,
    ) {
        // High-Fidelity Variable Particle Density
        let count = (pieces as f32).sqrt().ceil() as u32;
        let dw = rect.width / count as f32;
        let dh = rect.height / count as f32;

        let c = self.apply_opacity(color);

        for y in 0..count {
            for x in 0..count {
                let shard_rect = Rect {
                    x: rect.x + x as f32 * dw,
                    y: rect.y + y as f32 * dh,
                    width: dw,
                    height: dh,
                };

                let uv = Rect {
                    x: x as f32 / count as f32,
                    y: y as f32 / count as f32,
                    width: 1.0 / count as f32,
                    height: 1.0 / count as f32,
                };

                self.fill_rect_with_full_params(shard_rect, c, mode, None, force, uv);
            }
        }
    }

    fn recursive_bolt(&mut self, from: [f32; 2], to: [f32; 2], depth: u32, color: [f32; 4]) {
        if depth == 0 {
            self.draw_lightning_segment(from, to, color);
            return;
        }

        let mid_x = (from[0] + to[0]) * 0.5;
        let mid_y = (from[1] + to[1]) * 0.5;

        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();

        // Perpendicular offset for jaggedness
        let offset_scale = len * 0.15;
        let seed = (from[0] * 12.9898 + from[1] * 78.233 + (depth as f32) * 37.11)
            .sin()
            .fract();
        let offset_x = -dy / len * (seed - 0.5) * offset_scale;
        let offset_y = dx / len * (seed - 0.5) * offset_scale;

        let mid = [mid_x + offset_x, mid_y + offset_y];

        self.recursive_bolt(from, mid, depth - 1, color);
        self.recursive_bolt(mid, to, depth - 1, color);

        // 20% chance of a secondary branch
        if depth > 2 && seed > 0.8 {
            let branch_to = [
                mid[0] + offset_x * 2.0 + (seed * 100.0).sin() * 50.0,
                mid[1] + offset_y * 2.0 + (seed * 100.0).cos() * 50.0,
            ];
            self.recursive_bolt(mid, branch_to, depth - 2, color);
        }
    }

    fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        let glow_width = 32.0;
        let core_width = 4.0;
        let c = self.apply_opacity(color);

        // 1. Render Volumetric Glow (Cyan)
        let gnx = -dy / len * glow_width * 0.5;
        let gny = dx / len * glow_width * 0.5;
        let gp1 = [from[0] + gnx, from[1] + gny];
        let gp2 = [to[0] + gnx, to[1] + gny];
        let gp3 = [to[0] - gnx, to[1] - gny];
        let gp4 = [from[0] - gnx, from[1] - gny];
        self.push_oriented_quad(
            [gp1, gp2, gp3, gp4],
            c,
            9,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );

        // 2. Render Blinding Core (White)
        let cnx = -dy / len * core_width * 0.5;
        let cny = dx / len * core_width * 0.5;
        let cp1 = [from[0] + cnx, from[1] + cny];
        let cp2 = [to[0] + cnx, to[1] + cny];
        let cp3 = [to[0] - cnx, to[1] - cny];
        let cp4 = [from[0] - cnx, from[1] - cny];
        self.push_oriented_quad(
            [cp1, cp2, cp3, cp4],
            [1.0, 1.0, 1.0, c[3]],
            0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        mode: u32,
        uv_rect: Rect,
    ) {
        let scissor = self.clip_stack.last().copied();
        let texture_id = None; // Oriented quads like lightning don't use textures yet

        if self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || self.draw_calls.last().unwrap().scissor_rect != scissor
        {
            self.current_texture_id = texture_id;
            self.draw_calls.push(DrawCall {
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                is_glass: mode == 7,
                is_ui: mode == 6,
            });
        }

        let uvs = [
            [uv_rect.x, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            [uv_rect.x, uv_rect.y + uv_rect.height],
        ];

        let screen = [self.current_width() as f32, self.current_height() as f32];
        let rect = Rect {
            x: points[0][0],
            y: points[0][1],
            width: 1.0,
            height: 1.0,
        };

        for i in 0..4 {
            let px = points[i][0];
            let py = points[i][1];

            let (translation, scale_transform, rotation) = self.get_current_transform();
            self.vertices.push(Vertex {
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: uvs[i],
                color,
                mode,
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                screen,
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
                translation,
                scale: scale_transform,
                rotation,
                _pad: 0.0,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }
    fn get_texture_id(&self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode — Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color,
            mode,
            texture_id,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn fill_rect_with_full_params(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
    ) {
        // If a shadow is active, draw it first
        if let Some(shadow) = self.shadow_stack.last().copied() 
            && shadow.color[3] > 0.001 {
            Renderer::draw_drop_shadow(
                self,
                rect,
                radius,
                shadow.color,
                shadow.radius,
                0.0, // Spread
            );
        }

        let slice = self
            .slice_stack
            .last()
            .copied()
            .map(|(a, o)| [a, o, 1.0, 1.0])
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        self.fill_rect_with_full_params_and_slice(
            rect, color, mode, texture_id, radius, uv_rect, slice,
        );
    }

    fn fill_rect_with_full_params_and_slice(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
    ) {
        let scissor = self.clip_stack.last().copied();

        let is_glass = mode == 7;
        let is_ui = !is_glass;

        // Batching: check if we need to start a new DrawCall
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().is_glass != is_glass
            || last_call.unwrap().is_ui != is_ui;

        if needs_new_call {
            self.current_texture_id = texture_id;
            self.draw_calls.push(DrawCall {
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                is_glass,
                is_ui,
            });
        }

        let scale = self.current_scale_factor();
        let snap = |v: f32| (v * scale).round() / scale;

        let base_idx = self.vertices.len() as u32;
        let x1 = snap(rect.x);
        let y1 = snap(rect.y);
        let x2 = snap(rect.x + rect.width);
        let y2 = snap(rect.y + rect.height);
        let z = self.current_z;
        let normal = [0.0, 0.0, 1.0];
        let screen = [self.current_width() as f32, self.current_height() as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        let (translation, scale_transform, rotation) = self.get_current_transform();

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color,
            mode,
            radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            _pad: 0.0,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color,
            mode,
            radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            _pad: 0.0,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color,
            mode,
            radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            _pad: 0.0,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color,
            mode,
            radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            _pad: 0.0,
        });

        self.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }

    /// end_frame — Quench the blade by submitting the full Muspelheim multi-pass effect.
    pub fn end_frame(&mut self, mut encoder: wgpu::CommandEncoder) {
        // Visual Lint: If layout was dirtied during the render phase (layout thrashing),
        // draw a 10px red border as a warning flash.
        if LAYOUT_DIRTY.swap(false, Ordering::AcqRel)
            && let Some(window_id) = self.current_window
            && let Some(surface_ctx) = self.surfaces.get(&window_id) {
                let w = surface_ctx.config.width as f32;
                let h = surface_ctx.config.height as f32;
                let border_rect = Rect { x: 0.0, y: 0.0, width: w, height: h };
                // Draw a thick red border to signal layout-thrashing
                self.stroke_rect(border_rect, [1.0, 0.0, 0.0, 1.0], 10.0);
        }

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        let (surface_texture, target_view, ctx_scene_texture, ctx_depth_texture_view, ctx_blur_env_bind_group_a, ctx_scene_texture_bind_group, ctx_blur_texture_a, ctx_blur_texture_b, _ctx_sampler, ctx_blur_bind_group_a, ctx_blur_bind_group_b) = if let Some(window_id) = self.current_window {
            let ctx = self.surfaces.get(&window_id).expect("Missing surface context");
            let frame = match ctx.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                    ctx.surface.configure(&self.device, &ctx.config);
                    t
                }
                _ => return, // Silent failure for window issues
            };
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(frame), view, &ctx.scene_texture, &ctx.depth_texture_view, &ctx.blur_env_bind_group_a, &ctx.scene_texture_bind_group, &ctx.blur_texture_a, &ctx.blur_texture_b, &ctx.sampler, &ctx.blur_bind_group_a, &ctx.blur_bind_group_b)
        } else {
            let ctx = self.headless_context.as_ref().expect("No headless context for end_frame");
            (None, ctx.output_view.clone(), &ctx.scene_texture, &ctx.depth_texture_view, &ctx.blur_env_bind_group_a, &ctx.scene_texture_bind_group, &ctx.blur_texture_a, &ctx.blur_texture_b, &ctx.sampler, &ctx.blur_bind_group_a, &ctx.blur_bind_group_b)
        };

        // ── Pass 1: Opaque Background & Atmosphere ──────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P1 Opaque Background"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_scene_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: ctx_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // 1a. Background Atmosphere
            p.set_pipeline(&self.background_pipeline);
            p.set_bind_group(0, &self.dummy_texture_bind_group, &[]);
            p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]); // Use previous frame's blur for background depth
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);

            // 1b. Opaque Main Elements (non-glass, non-ui)
            if !self.draw_calls.is_empty() {
                p.set_pipeline(&self.pipeline);
                p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);

                for call in self.draw_calls.iter().filter(|c| !c.is_glass && !c.is_ui) {
                    let bg = if let Some(id) = call.texture_id {
                        if id == 0 {
                            &self.mega_atlas_bind_group
                        } else {
                            self.texture_bind_groups
                                .get(id as usize)
                                .unwrap_or(&self.dummy_texture_bind_group)
                        }
                    } else {
                        &self.dummy_texture_bind_group
                    };
                    p.set_bind_group(0, bg, &[]);
                    p.draw_indexed(
                        call.index_start..call.index_start + call.index_count,
                        0,
                        0..1,
                    );
                    self.telemetry.draw_calls += 1;
                    self.telemetry.vertices += call.index_count;
                }
            }
        }

        // ── Pass 2: Backdrop Blur (Bifrost) ──────────────────────────────────
        // Capture the background into blur_texture_b
        {
            // First extract into texture_a
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr Blur Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline); // Use extract as a direct copy for now
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
        }

        let blur_iters: u32 = 4;
        for _i in 0..blur_iters {
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur H"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_b, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
        }

        // ── Pass 3: Liquid Glass Elements ───────────────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P3 Liquid Glass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_scene_texture, // RENDER OVER THE OPAQUE BACKGROUND
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: ctx_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            p.set_pipeline(&self.pipeline);
            p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]); // Sample the freshly blurred backdrop
            p.set_bind_group(2, &self.berserker_bind_group, &[]);

            for call in self.draw_calls.iter().filter(|c| c.is_glass) {
                let bg = if let Some(id) = call.texture_id {
                    if id == 0 {
                        &self.mega_atlas_bind_group
                    } else {
                        self.texture_bind_groups
                            .get(id as usize)
                            .unwrap_or(&self.dummy_texture_bind_group)
                    }
                } else {
                    &self.dummy_texture_bind_group
                };
                p.set_bind_group(0, bg, &[]);
                p.draw_indexed(
                    call.index_start..call.index_start + call.index_count,
                    0,
                    0..1,
                );
                self.telemetry.draw_calls += 1;
                self.telemetry.vertices += call.index_count;
            }
        }

        // ── Pass 4: UI & Text Overlay ──────────────────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P4 UI Layer"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_scene_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: ctx_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            p.set_pipeline(&self.pipeline);
            p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);

            for call in self.draw_calls.iter().filter(|c| c.is_ui) {
                let bg = if let Some(id) = call.texture_id {
                    if id == 0 {
                        &self.mega_atlas_bind_group
                    } else {
                        self.texture_bind_groups
                            .get(id as usize)
                            .unwrap_or(&self.dummy_texture_bind_group)
                    }
                } else {
                    &self.dummy_texture_bind_group
                };
                p.set_bind_group(0, bg, &[]);
                p.draw_indexed(
                    call.index_start..call.index_start + call.index_count,
                    0,
                    0..1,
                );
                self.telemetry.draw_calls += 1;
                self.telemetry.vertices += call.index_count;
            }
        }

        // ── Pass 5: Bloom Extract (Complete Scene) ──────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr Bloom Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline);
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
        }

        // ── Pass 6: Blur Bloom ──────────────────────────────────────────────
        for _ in 0..2 {
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Bloom Blur H"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Bloom Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_b, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
        }

        // ── Pass 7: Composite & Tone Map ────────────────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P7 Composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.composite_pipeline);
            // Headless doesn't use these bind groups in composite yet, or does it?
            // Wait, we need to use the headless versions.
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]); // Bloom overlay
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
            self.telemetry.draw_calls += 1;
        }

        self.telemetry.frame_time_ms = self.last_frame_start.elapsed().as_secs_f32() * 1000.0;
        self.update_vram_telemetry();
        self.queue.submit(Some(encoder.finish()));
        if let Some(f) = surface_texture {
            f.present();
        }
    }
}

impl cvkg_core::ElapsedTime for SurtrRenderer {
    fn delta_time(&self) -> f32 {
        self.current_scene.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl cvkg_core::Renderer for SurtrRenderer {
    /// fill_rect — Standard rectangle drawing method.
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_mode(rect, self.apply_opacity(color), 0, None);
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            3,
            None,
            radius,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            4,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn bifrost(&mut self, rect: Rect, blur: f32, _saturation: f32, opacity: f32) {
        // Calculate screen-space UVs for high-fidelity global refraction
        let screen_uv = Rect {
            x: rect.x / self.current_width() as f32,
            y: rect.y / self.current_height() as f32,
            width: rect.width / self.current_width() as f32,
            height: rect.height / self.current_height() as f32,
        };
        // Use mode 7 for high-fidelity background blur sampling
        // Use the blur parameter as corner radius for the glass panel
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, opacity], 7, None, blur, screen_uv);
    }

    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        // Create neon glow effect using additive blending
        // This renders a glowing aura around the element
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        let max_dim = rect.width.max(rect.height) * 0.5 + radius;
        
        // Draw expanding glow layers
        for i in 0..8 {
            let alpha = intensity / (i as f32 + 1.0) * 0.3;
            let glow_color = [color[0], color[1], color[2], alpha];
            self.fill_rect_with_mode(
                Rect {
                    x: center_x - max_dim - i as f32 * 2.0,
                    y: center_y - max_dim - i as f32 * 2.0,
                    width: max_dim * 2.0 + i as f32 * 4.0,
                    height: max_dim * 2.0 + i as f32 * 4.0,
                },
                glow_color,
                8, // Mode for additive blending
                None,
            );
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let c = self.apply_opacity(color);
        let hw = stroke_width;
        // Top, bottom, left, right edge bars
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y + rect.height - hw,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x + rect.width - hw,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            17,
            None,
            radius,
            Rect {
                x: stroke_width,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _stroke_width: f32) {
        // Future: Implement stroked SDFs.
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(start_color),
            15,
            None,
            0.0,
            Rect {
                x: angle,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            end_color,
        );
    }

    fn draw_radial_gradient(&mut self, rect: Rect, inner_color: [f32; 4], outer_color: [f32; 4]) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(inner_color),
            16,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            outer_color,
        );
    }

    fn draw_drop_shadow(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        blur: f32,
        spread: f32,
    ) {
        let margin = blur + spread;
        let inflated = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + margin * 2.0,
            height: rect.height + margin * 2.0,
        };
        // uv.x = total margin (for SDF offset), uv.y = blur width (for falloff)
        self.fill_rect_with_full_params(
            inflated,
            self.apply_opacity(color),
            18,
            None,
            radius,
            Rect {
                x: margin,
                y: blur,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_dashed_rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        width: f32,
        dash: f32,
        gap: f32,
    ) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            19,
            None,
            radius,
            Rect {
                x: width,
                y: dash,
                width: gap,
                height: 0.0,
            },
        );
    }

    fn draw_9slice(
        &mut self,
        image_name: &str,
        rect: Rect,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) {
        let c = self.apply_opacity([1.0, 1.0, 1.0, 1.0]);
        let tid = self.get_texture_id(image_name);
        self.fill_rect_with_full_params(
            rect,
            c,
            20,
            tid,
            bottom,
            Rect {
                x: left,
                y: top,
                width: right,
                height: 0.0,
            },
        );
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
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        let c = self.apply_opacity(color);
        let tid = self.get_texture_id("__mega_atlas");

        self.fill_rect_with_mode(
            Rect {
                x: (x1 + x2) / 2.0 - len / 2.0,
                y: (y1 + y2) / 2.0 - stroke_width / 2.0,
                width: len,
                height: stroke_width,
            },
            c,
            1, // Gungnir Mode for glowing lines
            tid,
        );
    }

    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        let tid = self.get_texture_id("__mega_atlas");
        let uv_rect = self.image_uv_registry.get(image_name).copied().unwrap_or(Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 });
        self.fill_rect_with_full_params(
            rect,
            [1.0, 1.0, 1.0, 1.0],
            2,
            tid,
            0.0,
            uv_rect,
        );
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // High-DPI: Shape and rasterize at the physical scale factor for maximum sharpness.
        let scaled_size = size * self.current_scale_factor();
        let shaped = self.text_engine.shape(text, "sans-serif", scaled_size);
        let c = self.apply_opacity(color);

        for glyph in shaped.glyphs {
            let cache_key = glyph.cache_key;

            let (uv_rect, w, h) = if let Some(info) = self.text_cache.get(&cache_key) {
                *info
            } else {
                if let Some(image) = self.text_engine.rasterize(cache_key) {
                    let gw = image.width;
                    let gh = image.height;

                    if let Some((nx, ny)) = self.atlas_packer.pack(gw, gh) {
                        let mut rgba_data = Vec::with_capacity((gw * gh * 4) as usize);
                        for alpha in &image.data {
                            rgba_data.push(255);
                            rgba_data.push(255);
                            rgba_data.push(255);
                            rgba_data.push(*alpha);
                        }

                        self.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &self.mega_atlas_tex,
                                mip_level: 0,
                                origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                                aspect: wgpu::TextureAspect::All,
                            },
                            &rgba_data,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(gw * 4),
                                rows_per_image: Some(gh),
                            },
                            wgpu::Extent3d {
                                width: gw,
                                height: gh,
                                depth_or_array_layers: 1,
                            },
                        );

                        let info = (
                            Rect {
                                x: nx as f32 / 4096.0,
                                y: ny as f32 / 4096.0,
                                width: gw as f32 / 4096.0,
                                height: gh as f32 / 4096.0,
                            },
                            gw as f32,
                            gh as f32,
                        );
                        self.text_cache.insert(cache_key, info);
                        info
                    } else {
                        (Rect::zero(), 0.0, 0.0)
                    }
                } else {
                    (Rect::zero(), 0.0, 0.0)
                }
            };

            if w > 0.0 {
                // Map physical glyph dimensions and positions back to logical units
                // so the logical orthographic projection matrix places them correctly.
                let glyph_rect = Rect {
                    x: x + glyph.x / self.current_scale_factor(),
                    y: y + glyph.y / self.current_scale_factor(),
                    width: w / self.current_scale_factor(),
                    height: h / self.current_scale_factor(),
                };
                let tid = self.get_texture_id("__mega_atlas");
                self.fill_rect_with_full_params(glyph_rect, c, 6, tid, 0.0, uv_rect);
            }
        }
    }

    /// measure_text — Calculates the dimensions of a text string without rendering.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let shaped = self.text_engine.shape(text, "sans-serif", size);
        (shaped.width, shaped.height)
    }

    fn draw_texture(&mut self, texture_id: u32, rect: Rect) {
        self.fill_rect_with_full_params(
            rect,
            [1.0, 1.0, 1.0, 1.0],
            2,
            Some(texture_id),
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    /// load_image — Proactively pushes a raw asset into the Mega-Atlas.
    fn load_image(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains_key(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                eprintln!("Failed to load image {}: {}", name, e);
                image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]))
            }
        };
        let (width, height) = img.dimensions();
        
        // Pack into Mega-Atlas
        if let Some((x, y)) = self.atlas_packer.pack(width, height) {
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.mega_atlas_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );

            // Store UV rect (logical 0-1)
            let uv_rect = Rect {
                x: x as f32 / 4096.0,
                y: y as f32 / 4096.0,
                width: width as f32 / 4096.0,
                height: height as f32 / 4096.0,
            };
            self.image_uv_registry.insert(name.to_string(), uv_rect);
            self.texture_registry.insert(name.to_string(), 0);
        } else {
            eprintln!("Mega-Atlas is FULL! Could not pack image: {}", name);
        }
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    fn push_opacity(&mut self, opacity: f32) {
        let current = self.opacity_stack.last().copied().unwrap_or(1.0);
        self.opacity_stack.push(current * opacity);
    }

    fn pop_opacity(&mut self) {
        self.opacity_stack.pop();
    }

    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.shadow_stack.push(ShadowState {
            radius,
            color,
            _offset: offset,
        });
    }

    fn pop_shadow(&mut self) {
        self.shadow_stack.pop();
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        let (current_t, current_s, current_r) = self.transform_stack.last().copied().unwrap_or(([0.0, 0.0], [1.0, 1.0], 0.0));
        
        // Combine transforms (simplified: this doesn't handle full matrix multiplication yet,
        // but for basic UI nesting it's often sufficient to just add translation and multiply scale).
        // A full implementation would use mat3x3.
        let new_t = [current_t[0] + translation[0] * current_s[0], current_t[1] + translation[1] * current_s[1]];
        let new_s = [current_s[0] * scale[0], current_s[1] * scale[1]];
        let new_r = current_r + rotation;
        
        self.transform_stack.push((new_t, new_s, new_r));
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn set_theme(&mut self, theme: ColorTheme) {

        self.current_theme = theme;
        self.queue
            .write_buffer(&self.theme_buffer, 0, bytemuck::bytes_of(&theme));
    }

    fn set_rage(&mut self, rage: f32) {
        self.current_scene.berzerker_rage = rage;
        // scene_buffer is updated every frame in begin_frame, so no need to write here
    }

    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.current_scene.shatter_origin = origin;
        self.current_scene.shatter_time = self.current_scene.time;
        self.current_scene.shatter_force = force;
    }

    /// push_mjolnir_slice — Pushes a geometric clipping plane onto the stack.
    /// All subsequent draw calls will be sliced by this plane until it is popped.
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.slice_stack.push((angle, offset));
    }

    /// pop_mjolnir_slice — Removes the top-most geometric clipping plane from the stack.
    fn pop_mjolnir_slice(&mut self) {
        self.slice_stack.pop();
    }

    fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 8);
    }

    fn mjolnir_fluid_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 11);
    }

    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.recursive_bolt(from, to, 4, color);
    }

    fn upload_data_texture(&mut self, id: &str, data: &[f32], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(id),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(id),
        });
        self.texture_bind_groups.push(bind_group);
        let tid = (self.texture_bind_groups.len() - 1) as u32;
        self.texture_registry.insert(id.to_string(), tid);
    }

    fn draw_heatmap(&mut self, texture_id: &str, rect: Rect, _palette: &str) {
        let tid = self.get_texture_id(texture_id);
        self.fill_rect_with_mode(rect, [1.0, 1.0, 1.0, 1.0], 12, tid);
    }

    fn draw_mesh(&mut self, mesh: &Mesh, color: [f32; 4], transform: glam::Mat4) {
        let base_idx = self.vertices.len() as u32;
        let screen = [self.current_width() as f32, self.current_height() as f32];

        for i in 0..mesh.vertices.len() {
            let pos = transform.transform_point3(glam::Vec3::from(mesh.vertices[i]));
            let norm = transform.transform_vector3(glam::Vec3::from(mesh.normals[i]));

            let (translation, scale_transform, rotation) = self.get_current_transform();
            self.vertices.push(Vertex {
                position: pos.to_array(),
                normal: norm.to_array(),
                uv: [0.0, 0.0],
                color,
                mode: 13, // Mode 13: 3D Surface
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [0.0, 0.0],
                size: [0.0, 0.0],
                screen,
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
                translation,
                scale: scale_transform,
                rotation,
                _pad: 0.0,
            });
        }

        for idx in &mesh.indices {
            self.indices.push(base_idx + idx);
        }

        if self.draw_calls.is_empty() || self.current_texture_id.is_some() {
            self.current_texture_id = None;
            self.draw_calls.push(DrawCall {
                texture_id: None,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
                index_count: mesh.indices.len() as u32,
                is_glass: false,
                is_ui: false,
            });
        } else {
            self.draw_calls.last_mut().unwrap().index_count += mesh.indices.len() as u32;
        }
    }

    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        self.shared_elements.insert(id.to_string(), rect);
    }

    fn set_z_index(&mut self, z: f32) {
        self.current_z = z;
    }

    fn get_z_index(&self) -> f32 {
        self.current_z
    }


    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }
}

// --- SVG Helpers ---

fn usvg_to_lyon(path: &usvg::Path) -> lyon::path::Path {
    let mut builder = lyon::path::Path::builder();
    for segment in path.data().segments() {
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                builder.begin(lyon::math::point(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                builder.line_to(lyon::math::point(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p) => {
                builder.quadratic_bezier_to(
                    lyon::math::point(p1.x, p1.y),
                    lyon::math::point(p.x, p.y),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p) => {
                builder.cubic_bezier_to(
                    lyon::math::point(p1.x, p1.y),
                    lyon::math::point(p2.x, p2.y),
                    lyon::math::point(p.x, p.y),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                builder.end(true);
            }
        }
    }
    builder.build()
}

impl SurtrRenderer {
    fn get_current_transform(&self) -> ([f32; 2], [f32; 2], f32) {
        self.transform_stack
            .last()
            .cloned()
            .unwrap_or(([0.0, 0.0], [1.0, 1.0], 0.0))
    }
}

struct SceneVertexConstructor {
    color: [f32; 4],
    translation: [f32; 2],
    scale: [f32; 2],
    rotation: f32,
}

impl FillVertexConstructor<Vertex> for SceneVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: self.color,
            mode: 0,
            radius: 0.0,
            slice: [0.0, 0.0, 0.0, 1.0],
            logical: [vertex.position().x, vertex.position().y],
            size: [1.0, 1.0],
            screen: [0.0, 0.0],
            clip: [-10000.0, -10000.0, 20000.0, 20000.0],
            translation: self.translation,
            scale: self.scale,
            rotation: self.rotation,
            _pad: 0.0,
        }
    }
}

impl Drop for SurtrRenderer {
    fn drop(&mut self) {
        // Ensure GPU is idle before dropping to avoid Swapchain semaphore panics
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
}

impl cvkg_core::FrameRenderer<wgpu::CommandEncoder> for SurtrRenderer {
    fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        cvkg_core::begin_render_phase();
        let id = self.current_window.expect("No target window set for frame. Call set_target_window first.");
        self.begin_frame(id)
    }

    fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
        self.end_frame(encoder);
        cvkg_core::end_render_phase();
    }
}

impl SurtrRenderer {
    /// Returns the current effective opacity (product of all stacked values).
    fn apply_opacity(&self, mut color: [f32; 4]) -> [f32; 4] {
        if let Some(&alpha) = self.opacity_stack.last() {
            color[3] *= alpha;
        }
        color
    }

    /// load_svg — Parses an SVG file and tessellates its paths into GPU triangles.
    pub fn load_svg(&mut self, name: &str, data: &[u8]) {
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(data, &opt).expect("Failed to parse SVG");
        
        let view_box = Rect {
            x: 0.0,
            y: 0.0,
            width: tree.size().width(),
            height: tree.size().height(),
        };

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut tessellator = FillTessellator::new();

        for child in tree.root().children() {
            self.tessellate_node(child, &mut tessellator, &mut vertices, &mut indices);
        }

        self.svg_cache.insert(name.to_string(), SvgModel {
            vertices,
            indices,
            view_box,
        });
    }

    fn tessellate_node(&self, node: &usvg::Node, tessellator: &mut FillTessellator, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        if let usvg::Node::Group(ref group) = *node {
            for child in group.children() {
                self.tessellate_node(child, tessellator, vertices, indices);
            }
        } else if let usvg::Node::Path(ref path) = *node
            && let Some(fill) = path.fill() {
                let color = match fill.paint() {
                    usvg::Paint::Color(c) => [
                        c.red as f32 / 255.0,
                        c.green as f32 / 255.0,
                        c.blue as f32 / 255.0,
                        fill.opacity().get(),
                    ],
                    _ => [1.0, 1.0, 1.0, 1.0],
                };

                let lyon_path = usvg_to_lyon(path);
                let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
                let base_vertex_idx = vertices.len() as u32;

                tessellator.tessellate_path(
                    &lyon_path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut buffers, SceneVertexConstructor {
                        color,
                        translation: [0.0, 0.0],
                        scale: [1.0, 1.0],
                        rotation: 0.0,
                    }),
                ).unwrap();

                vertices.extend(buffers.vertices);
                for idx in buffers.indices {
                    indices.push(base_vertex_idx + idx);
                }
        }
    }

    /// draw_svg — Renders a pre-loaded SVG icon at the specified logical rect.
    pub fn draw_svg(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, mode: u32) {
        let model = if let Some(m) = self.svg_cache.get(name) {
            m.clone()
        } else {
            return;
        };

        let _scale_x = rect.width / model.view_box.width;
        let _scale_y = rect.height / model.view_box.height;
        let base_idx = self.vertices.len() as u32;
        let screen = [self.current_width() as f32, self.current_height() as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];
        let scale = self.current_scale_factor();
        let snap = |v: f32| (v * scale).round() / scale;

        for v in &model.vertices {
            let mut v = *v;
            let rel_x = (v.position[0] - model.view_box.x) / model.view_box.width;
            let rel_y = (v.position[1] - model.view_box.y) / model.view_box.height;
            
            v.position[0] = snap(rect.x + rel_x * rect.width);
            v.position[1] = snap(rect.y + rel_y * rect.height);
            v.position[2] = self.current_z;
            v.logical = [v.position[0], v.position[1]];
            v.screen = screen;
            v.clip = clip;
            v.mode = mode;
            
            if let Some(override_color) = color {
                v.color = self.apply_opacity(override_color);
            } else {
                v.color = self.apply_opacity(v.color);
            }
            self.vertices.push(v);
        }

        for idx in &model.indices {
            self.indices.push(base_idx + *idx);
        }

        let is_ui = true;
        let is_glass = mode == 7;
        let tid = self.get_texture_id("__mega_atlas");

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != tid
            || last_call.unwrap().scissor_rect != self.clip_stack.last().copied()
            || last_call.unwrap().is_glass != is_glass
            || last_call.unwrap().is_ui != is_ui;

        if needs_new_call {
            self.current_texture_id = tid;
            self.draw_calls.push(DrawCall {
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() - model.indices.len()) as u32,
                index_count: 0,
                is_glass,
                is_ui,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += model.indices.len() as u32;
        }
    }

    /// forge_headless — Initializes Surtr without a window for visual regression testing.
    pub async fn forge_headless(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        println!("[GPU] Requesting HighPerformance adapter...");
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok();

        if adapter.is_none() {
            println!("[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            println!("[GPU] Hardware adapters failed, trying Software fallback...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: true,
                })
                .await
                .ok();
        }

        let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");
        println!("[GPU] Selected adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Headless Forge"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            None,
            Some((width, height, wgpu::TextureFormat::Rgba8UnormSrgb))
        ).await
    }

    /// capture_frame — Read back the rendered frame as a byte buffer (RGBA8).
    pub async fn capture_frame(&self) -> Vec<u8> {
        let ctx = self.headless_context.as_ref().expect("Headless context required for capture");
        let u32_size = std::mem::size_of::<u32>() as u32;
        let width = ctx.width;
        let height = ctx.height;
        let bytes_per_row = width * u32_size;
        let padding = (256 - (bytes_per_row % 256)) % 256;
        let padded_bytes_per_row = bytes_per_row + padding;
        
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Capture Buffer"),
            size: (padded_bytes_per_row as u64 * height as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Capture Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &ctx.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = sender.send(v);
        });

        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        if let Ok(Ok(_)) = receiver.await {
            let data = buffer_slice.get_mapped_range();
            let mut result = Vec::with_capacity((width * height * 4) as usize);
            
            for y in 0..height {
                let start = (y * padded_bytes_per_row) as usize;
                let end = start + bytes_per_row as usize;
                result.extend_from_slice(&data[start..end]);
            }
            
            drop(data);
            output_buffer.unmap();
            result
        } else {
            panic!("Failed to capture frame")
        }
    }

    fn current_width(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().config.width
        } else {
            self.headless_context.as_ref().unwrap().width
        }
    }

    fn current_height(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().config.height
        } else {
            self.headless_context.as_ref().unwrap().height
        }
    }

    fn current_scale_factor(&self) -> f32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().scale_factor
        } else {
            self.headless_context.as_ref().unwrap().scale_factor
        }
    }
}

