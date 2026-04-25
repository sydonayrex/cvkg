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

use cvkg_core::{ColorTheme, Mesh, Rect, SceneUniforms};
use std::sync::Arc;

// ShieldWall — re-export AccessKit types so callers can build tree updates
// without depending on accesskit directly.
pub use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree,
    TreeId, TreeUpdate,
};
pub use accesskit_winit::Adapter as ShieldWallAdapter;

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

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 11] = wgpu::vertex_attr_array![
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
        10 => Float32x4 // clip
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
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    // Text Forge
    #[allow(dead_code)]
    font_system: cosmic_text::FontSystem,
    #[allow(dead_code)]
    swash_cache: cosmic_text::SwashCache,
    text_atlas_tex: wgpu::Texture,
    #[allow(dead_code)]
    text_atlas_view: wgpu::TextureView,
    #[allow(dead_code)]
    text_sampler: wgpu::Sampler,
    text_cache: std::collections::HashMap<cosmic_text::CacheKey, (Rect, f32, f32)>,
    text_atlas_pos: (u32, u32),

    // Niflheim Resources
    dummy_texture_bind_group: wgpu::BindGroup,
    dummy_env_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    texture_registry: std::collections::HashMap<String, u32>,
    shared_elements: std::collections::HashMap<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    draw_calls: Vec<DrawCall>,
    current_texture_id: Option<u32>,

    // Opacity stack: each push multiplies into the current effective alpha.
    opacity_stack: Vec<f32>,
    // Clip rect stack: used for batched scissoring.
    clip_stack: Vec<Rect>,
    // Mjolnir Slice stack: (angle, offset)
    slice_stack: Vec<(f32, f32)>,

    // The Forge's Heart (Berserker State)
    theme_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,
    berserker_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    berserker_bind_group_layout: wgpu::BindGroupLayout,
    start_time: std::time::Instant,
    current_theme: ColorTheme,
    current_scene: SceneUniforms,

    // Muspelheim Pipelines
    pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    bloom_extract_pipeline: wgpu::RenderPipeline,
    blur_h_pipeline: wgpu::RenderPipeline,
    blur_v_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,

    /// Muspelheim Textures & Bind Groups
    blur_texture_a: wgpu::TextureView,
    blur_texture_b: wgpu::TextureView,
    blur_bind_group_a: wgpu::BindGroup,
    blur_bind_group_b: wgpu::BindGroup,
    blur_env_bind_group_a: wgpu::BindGroup,
    scene_texture: wgpu::TextureView,
    scene_bind_group: wgpu::BindGroup,
    scene_texture_bind_group: wgpu::BindGroup,
    env_bind_group_layout: wgpu::BindGroupLayout,
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

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU for Surtr");

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

        // Load the Muspelheim Shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
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
                Some(&texture_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        // Specialized layout for composite (Blur + Scene)
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Muspelheim Composite Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&texture_bind_group_layout),
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
                    format: config.format,
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
                    format: config.format,
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
                        format: config.format,
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
                    format: config.format,
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
                    format: config.format,
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
                    format: config.format,
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

        // Muspelheim Intermediate Textures
        let blur_tex_desc = wgpu::TextureDescriptor {
            label: Some("Muspelheim Intermediate"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let blur_texture_a_obj = device.create_texture(&blur_tex_desc);
        let blur_texture_b_obj = device.create_texture(&blur_tex_desc);
        let blur_texture_a =
            blur_texture_a_obj.create_view(&wgpu::TextureViewDescriptor::default());
        let blur_texture_b =
            blur_texture_b_obj.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
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
            layout: &texture_bind_group_layout,
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

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &env_bind_group_layout,
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
            label: Some("Blur Env Bind Group A"),
        });

        // Forge the Scene Capture Texture
        let scene_texture_obj = device.create_texture(&blur_tex_desc);
        let scene_texture = scene_texture_obj.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &env_bind_group_layout,
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
            label: Some("Scene Capture Bind Group"),
        });

        // Forge the Text Atlas (1024x1024 Alpha-only for speed)
        let text_atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Text Atlas"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let text_atlas = text_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Clear the text atlas to transparency initially
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &text_atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &vec![0u8; 1024 * 1024],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: Some(1024),
            },
            wgpu::Extent3d {
                width: 1024,
                height: 1024,
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
            &[255, 255, 255, 255],
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

        let text_atlas_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&text_atlas),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&text_sampler),
                },
            ],
            label: Some("Text Atlas Bind Group"),
        });
        texture_registry.insert("__text_atlas".to_string(), texture_bind_groups.len() as u32);
        texture_bind_groups.push(text_atlas_bg);

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

        // Register atlas

        // Texture registry and bind groups already initialized above.

        // Forge the Heart (Berserker Uniforms)
        let current_theme = ColorTheme::default();
        use wgpu::util::DeviceExt;
        let theme_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr Theme Buffer"),
            contents: bytemuck::bytes_of(&current_theme),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let current_scene = SceneUniforms::new(config.width as f32, config.height as f32);
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

        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
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
            label: Some("Scene Texture Bind Group (Group 0)"),
        });

        Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            bloom_extract_pipeline,
            blur_h_pipeline,
            blur_v_pipeline,
            composite_pipeline,
            blur_texture_a,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            env_bind_group_layout,
            font_system: cosmic_text::FontSystem::new(),
            swash_cache: cosmic_text::SwashCache::new(),
            text_atlas_tex,
            text_atlas_view: text_atlas,
            text_sampler,
            text_cache: std::collections::HashMap::new(),
            text_atlas_pos: (0, 0),
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
            theme_buffer,
            scene_buffer,
            berserker_bind_group,
            berserker_bind_group_layout,
            start_time: std::time::Instant::now(),
            current_theme,
            current_scene,
            background_pipeline,
        }
    }

    /// resize — Reconfigures the surface and internal textures for a new resolution.
    ///
    /// This is a non-trivial algorithm that must ensure all intermediate Muspelheim textures
    /// (Bloom, Blur, Scene Capture) are recreated with matching dimensions to prevent
    /// coordinate drift in the composite pass.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Re-create Muspelheim textures
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
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };

            let scene_tex = self.device.create_texture(&texture_desc);
            self.scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_tex_a = self.device.create_texture(&texture_desc);
            self.blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_tex_b = self.device.create_texture(&texture_desc);
            self.blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

            // Re-create bind groups (using existing layouts)
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            self.scene_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.env_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.scene_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("Surtr Scene Bind Group Resize"),
            });

            self.blur_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.blur_texture_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("Surtr Blur Bind Group A Resize"),
            });

            self.blur_bind_group_b = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.blur_texture_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("Surtr Blur Bind Group B Resize"),
            });

            self.scene_texture_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.scene_texture),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: Some("Scene Texture Bind Group Resize"),
                });

            self.blur_env_bind_group_a =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.env_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.blur_texture_a),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: Some("Blur Env Bind Group A Resize"),
                });

            self.current_scene.resolution = [width as f32, height as f32];
        }
    }

    /// begin_frame — Strike the flaming sword to begin a new GPU frame.
    pub fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.shared_elements.clear(); // Clear registry for the new frame
        self.current_texture_id = None;

        let time = self.start_time.elapsed().as_secs_f32();
        let (width, height) = (self.config.width as f32, self.config.height as f32);
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [width, height];
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, width, height, 0.0, -100.0, 100.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr's Flaming Sword"),
            })
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

        let screen = [self.config.width as f32, self.config.height as f32];
        let rect = Rect {
            x: points[0][0],
            y: points[0][1],
            width: 1.0,
            height: 1.0,
        };

        for i in 0..4 {
            let px = points[i][0];
            let py = points[i][1];

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
        let is_ui = mode == 6;

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

        let base_idx = self.vertices.len() as u32;
        let x1 = rect.x;
        let y1 = rect.y;
        let x2 = rect.x + rect.width;
        let y2 = rect.y + rect.height;
        let z = 0.0;
        let normal = [0.0, 0.0, 1.0];
        let screen = [self.config.width as f32, self.config.height as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

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
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(&self.device, &self.config);
                t
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        let screen = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // ── Pass 1: Opaque Background & Atmosphere ──────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P1 Opaque Background"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.scene_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // 1a. Background Atmosphere
            p.set_pipeline(&self.background_pipeline);
            p.set_bind_group(0, &self.dummy_texture_bind_group, &[]);
            p.set_bind_group(1, &self.blur_env_bind_group_a, &[]); // Use previous frame's blur for background depth
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
                        self.texture_bind_groups
                            .get(id as usize)
                            .unwrap_or(&self.dummy_texture_bind_group)
                    } else {
                        &self.dummy_texture_bind_group
                    };
                    p.set_bind_group(0, bg, &[]);
                    p.draw_indexed(
                        call.index_start..call.index_start + call.index_count,
                        0,
                        0..1,
                    );
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
                    view: &self.blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline); // Use extract as a direct copy for now
            p.set_bind_group(0, &self.scene_texture_bind_group, &[]);
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
                        view: &self.blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, &self.blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, &self.blur_bind_group_b, &[]);
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
                    view: &self.scene_texture, // RENDER OVER THE OPAQUE BACKGROUND
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            p.set_pipeline(&self.pipeline);
            p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            p.set_bind_group(1, &self.blur_env_bind_group_a, &[]); // Sample the freshly blurred backdrop
            p.set_bind_group(2, &self.berserker_bind_group, &[]);

            for call in self.draw_calls.iter().filter(|c| c.is_glass) {
                let bg = if let Some(id) = call.texture_id {
                    self.texture_bind_groups
                        .get(id as usize)
                        .unwrap_or(&self.dummy_texture_bind_group)
                } else {
                    &self.dummy_texture_bind_group
                };
                p.set_bind_group(0, bg, &[]);
                p.draw_indexed(
                    call.index_start..call.index_start + call.index_count,
                    0,
                    0..1,
                );
            }
        }

        // ── Pass 4: UI & Text Overlay ──────────────────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P4 UI Layer"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.scene_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            p.set_pipeline(&self.pipeline);
            p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);

            for call in self.draw_calls.iter().filter(|c| c.is_ui) {
                let bg = if let Some(id) = call.texture_id {
                    self.texture_bind_groups
                        .get(id as usize)
                        .unwrap_or(&self.dummy_texture_bind_group)
                } else {
                    &self.dummy_texture_bind_group
                };
                p.set_bind_group(0, bg, &[]);
                p.draw_indexed(
                    call.index_start..call.index_start + call.index_count,
                    0,
                    0..1,
                );
            }
        }

        // ── Pass 5: Bloom Extract (Complete Scene) ──────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr Bloom Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline);
            p.set_bind_group(0, &self.scene_texture_bind_group, &[]);
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
                        view: &self.blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, &self.blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Bloom Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, &self.blur_bind_group_b, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
        }

        // ── Pass 7: Final Composite ─────────────────────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P7 Final Composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &screen,
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
            p.set_bind_group(0, &self.scene_texture_bind_group, &[]); // Main Scene (Group 0 -> t_diffuse)
            p.set_bind_group(1, &self.blur_bind_group_a, &[]); // Bloom (Group 1 -> t_env)
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
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
            x: rect.x / self.config.width as f32,
            y: rect.y / self.config.height as f32,
            width: rect.width / self.config.width as f32,
            height: rect.height / self.config.height as f32,
        };
        // Use mode 7 for high-fidelity background blur sampling
        // Use the blur parameter as corner radius for the glass panel
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, opacity], 7, None, blur, screen_uv);
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

        let _angle = dy.atan2(dx).to_degrees();
        let c = self.apply_opacity(color);

        // Push an oriented quad by using the Mjolnir Slice infrastructure or
        // by calculating rotated vertices. For now, we use a simple rotation push.
        // In a future pass, we will add 'rotation' to the Vertex struct for batching.
        // For now, we use the centered rect logic.
        self.fill_rect_with_mode(
            Rect {
                x: (x1 + x2) / 2.0 - len / 2.0,
                y: (y1 + y2) / 2.0 - stroke_width / 2.0,
                width: len,
                height: stroke_width,
            },
            c,
            1, // Gungnir Mode for glowing lines
            None,
        );
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // High-Fidelity Text Forge: Layout -> Rasterize -> Atlas -> Quad
        let mut buffer =
            cosmic_text::Buffer::new(&mut self.font_system, cosmic_text::Metrics::new(size, size));
        // Use Basic shaping for 'SYSTEM' readouts to avoid aggressive kerning pull-in
        buffer.set_text(
            &mut self.font_system,
            text,
            &cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let c = self.apply_opacity(color);

        let mut glyph_idx = 0;
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                // Adaptive Berserker Tracking: More space for small text, tight for headers
                let tracking = (2.8 - 0.22 * (size - 16.0)).max(0.6);
                let x_offset = x + (glyph_idx as f32 * tracking);

                let physical_glyph = glyph.physical((x_offset, y), 1.0);
                let cache_key = physical_glyph.cache_key;

                // Check cache or rasterize (Keyed by the full CacheKey to support multiple sizes)
                let (uv_rect, w, h) = if let Some(info) = self.text_cache.get(&cache_key) {
                    *info
                } else {
                    // Rasterize new glyph
                    if let Some(image) =
                        self.swash_cache.get_image(&mut self.font_system, cache_key)
                    {
                        let (gx, _gy) = self.text_atlas_pos;
                        let gw = image.placement.width;
                        let gh = image.placement.height;

                        // Simple grid packing (Phase 2 Forge)
                        if gx + gw > 1024 {
                            self.text_atlas_pos.0 = 0;
                            self.text_atlas_pos.1 += 64; // Max glyph height
                        }
                        let (nx, ny) = self.text_atlas_pos;

                        self.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &self.text_atlas_tex,
                                mip_level: 0,
                                origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                                aspect: wgpu::TextureAspect::All,
                            },
                            &image.data,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(gw),
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
                                x: nx as f32 / 1024.0,
                                y: ny as f32 / 1024.0,
                                width: gw as f32 / 1024.0,
                                height: gh as f32 / 1024.0,
                            },
                            gw as f32,
                            gh as f32,
                        );
                        self.text_cache.insert(cache_key, info);
                        self.text_atlas_pos.0 += gw + 2;
                        info
                    } else {
                        (Rect::zero(), 0.0, 0.0)
                    }
                };

                if w > 0.0 {
                    let glyph_rect = Rect {
                        x: physical_glyph.x as f32,
                        y: physical_glyph.y as f32,
                        width: w,
                        height: h,
                    };
                    let tid = self.get_texture_id("__text_atlas");
                    self.fill_rect_with_full_params(glyph_rect, c, 6, tid, 0.0, uv_rect); // Mode 6 = Text
                }
                glyph_idx += 1;
            }
        }
    }

    /// measure_text — Calculates the dimensions of a text string without rendering.
    /// This is essential for layout engines to perform pre-pass calculations.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let mut buffer =
            cosmic_text::Buffer::new(&mut self.font_system, cosmic_text::Metrics::new(size, size));
        buffer.set_text(
            &mut self.font_system,
            text,
            &cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height += size;
        }

        (width, height)
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

    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        let tid = self.get_texture_id(image_name);
        self.fill_rect_with_full_params(
            rect,
            [1.0, 1.0, 1.0, 1.0],
            2,
            tid,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    /// load_image — Proactively pushes a raw asset into the GPU texture registry.
    /// This method is idempotent; it checks the cache to prevent redundant allocations.
    fn load_image(&mut self, name: &str, data: &[u8]) {
        if self.texture_registry.contains_key(name) {
            return;
        }
        let img = image::load_from_memory(data)
            .expect("Failed to load image")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            &img,
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
            label: Some(name),
        });
        self.texture_bind_groups.push(bind_group);
        let id = (self.texture_bind_groups.len() - 1) as u32;
        self.texture_registry.insert(name.to_string(), id);
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
        let screen = [self.config.width as f32, self.config.height as f32];

        for i in 0..mesh.vertices.len() {
            let pos = transform.transform_point3(glam::Vec3::from(mesh.vertices[i]));
            let norm = transform.transform_vector3(glam::Vec3::from(mesh.normals[i]));

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
}

impl Drop for SurtrRenderer {
    fn drop(&mut self) {
        // Ensure GPU is idle before dropping to avoid Swapchain semaphore panics
        let _ = self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    }
}

impl cvkg_core::FrameRenderer<wgpu::CommandEncoder> for SurtrRenderer {
    fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        self.begin_frame()
    }

    fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
        self.end_frame(encoder)
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
}
