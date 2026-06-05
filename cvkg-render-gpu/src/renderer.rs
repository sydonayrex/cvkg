//! The main SurtrRenderer struct and core frame lifecycle.
use cvkg_core::Rect;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use cvkg_core::Renderer;
use bytemuck;
use crate::color_blindness::ColorBlindUniforms;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions,
    StrokeTessellator, VertexBuffers,
};
use crate::types::*;
use crate::vertex::*;
use crate::atlas::YggdrasilPacker;
use cvkg_core::{ColorTheme, SceneUniforms};
use crate::kvasir;
use crate::{WGSL_SRC, WGSL_OPAQUE, WGSL_GLASS, WGSL_PBR, WGSL_GRADIENT};
use crate::draw::{parse_svg_animations, usvg_to_lyon};



/// SurtrRenderer implements the high-performance GPU backend.
#[allow(dead_code)]
pub struct SurtrRenderer {
    pub(crate) instance: Arc<wgpu::Instance>,
    pub(crate) adapter: Arc<wgpu::Adapter>,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,

    // Kvasir resource registry — tracks GPU resource lifetimes


    // Multi-Window Surface Management
    pub(crate) surfaces: std::collections::HashMap<winit::window::WindowId, SurfaceContext>,
    pub(crate) current_window: Option<winit::window::WindowId>,
    pub headless_context: Option<HeadlessContext>,

    // Mega-Atlas (Shared across all windows)
    pub(crate) text_engine: cvkg_runic_text::RunicTextEngine,
    pub(crate) mega_atlas_tex: wgpu::Texture,
    pub(crate) mega_atlas_bind_group: wgpu::BindGroup,
    pub(crate) text_cache: LruCache<u64, (Rect, f32, f32)>,
    pub(crate) atlas_packer: YggdrasilPacker,
    pub(crate) image_uv_registry: LruCache<String, Rect>,
    pub(crate) texture_registry: LruCache<String, u32>,
    pub(crate) texture_views: Vec<wgpu::TextureView>,
    pub(crate) dummy_sampler: wgpu::Sampler,
    pub(crate) svg_cache: LruCache<String, SvgModel>,
    /// Parsed SVG trees for serialization and filter application.
    pub(crate) svg_trees: LruCache<String, usvg::Tree>,
    /// SVG filter evaluation engine.
    pub(crate) filter_engine: Option<cvkg_svg_filters::FilterEngine>,
    /// Pending filter batches accumulated during tessellation.
    pub(crate) filter_batches: Vec<cvkg_svg_filters::FilterNode>,

    // Niflheim Resources (Shared)
    pub(crate) dummy_texture_bind_group: wgpu::BindGroup,
    pub(crate) dummy_env_bind_group: wgpu::BindGroup,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) shared_elements: LruCache<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) instance_data: Vec<InstanceData>,
    pub(crate) staging_belt: wgpu::util::StagingBelt,
    pub(crate) staging_command_buffers: Vec<wgpu::CommandBuffer>,
    pub(crate) draw_calls: Vec<DrawCall>,
    pub(crate) current_texture_id: Option<u32>,

    // Opacity & Clip Stacks
    pub(crate) opacity_stack: Vec<f32>,
    pub(crate) clip_stack: Vec<Rect>,
    pub(crate) slice_stack: Vec<(f32, f32)>,
    pub(crate) shadow_stack: Vec<ShadowState>,

    // The Forge's Heart (Shared Berserker State)
    pub(crate) theme_buffer: wgpu::Buffer,
    pub(crate) scene_buffer: wgpu::Buffer,
    pub(crate) berserker_bind_group: wgpu::BindGroup,
    pub(crate) berserker_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) start_time: std::time::Instant,
    pub(crate) current_theme: ColorTheme,
    pub(crate) current_scene: SceneUniforms,
    pub(crate) current_z: f32,

    // Muspelheim Pipelines (Shared)
    pub(crate) pipeline: wgpu::RenderPipeline,
    /// Specialized opaque/2D material pipeline (modes 0-20 excluding 7,13-15,18,21).
    pub(crate) opaque_pipeline: wgpu::RenderPipeline,
    /// Specialized glass material pipeline (mode 7 only, ~150 lines of complex math).
    pub(crate) glass_pipeline: wgpu::RenderPipeline,
    /// Specialized 3D PBR pipeline (modes 13, 14, 21 — raymarching).
    pub(crate) pbr_pipeline: wgpu::RenderPipeline,
    /// Specialized gradient/shadow pipeline (modes 15, 18).
    pub(crate) gradient_pipeline: wgpu::RenderPipeline,
    pub(crate) background_pipeline: wgpu::RenderPipeline,
    pub(crate) bloom_extract_pipeline: wgpu::RenderPipeline,
    /// Identity copy pipeline for Pass 2 backdrop blur (all pixels, no luminance gate).
    pub(crate) copy_pipeline: wgpu::RenderPipeline,
    pub(crate) blur_h_pipeline: wgpu::RenderPipeline,
    pub(crate) blur_v_pipeline: wgpu::RenderPipeline,
    pub(crate) composite_pipeline: wgpu::RenderPipeline,
    /// Color blindness simulation pipeline (fullscreen triangle).
    pub(crate) color_blind_pipeline: wgpu::RenderPipeline,
    /// Kawase blur pyramid downsample pipeline (separate shader module).
    pub(crate) kawase_down_pipeline: wgpu::RenderPipeline,
    /// Kawase blur pyramid upsample pipeline (separate shader module).
    pub(crate) kawase_up_pipeline: wgpu::RenderPipeline,
    /// Kawase blur bind group layout (uniform + texture + sampler).
    pub(crate) kawase_bind_group_layout: wgpu::BindGroupLayout,
    /// Environment bind group layout (texture + sampler).
    pub(crate) env_bind_group_layout: wgpu::BindGroupLayout,

    // Telemetry
    pub telemetry: cvkg_core::TelemetryData,

    /// Configuration for render-loop frame timing and degradation strategies.
    pub frame_budget: cvkg_core::FrameBudget,
    /// Staging buffer for windowed frame capture.
    pub(crate) capture_staging_buffer: Option<wgpu::Buffer>,
    /// Instant at the start of the last redraw, used for measuring frame timings.
    pub last_redraw_start: std::time::Instant,
    /// Instant at the start of the last frame, used for frame_time_ms calculation.
    pub last_frame_start: std::time::Instant,

    // VRAM Tracking (Bytes)
    pub(crate) vram_buffers_bytes: u64,
    pub(crate) vram_textures_bytes: u64,

    // Debugging
    pub(crate) _debug_layout: bool,

    // Transform Stack — stores full affine matrices for correct SVG transform composition.
    pub(crate) transform_stack: Vec<glam::Mat3>,
    /// Whether a redraw has been requested for the next frame.
    pub redraw_requested: bool,
    /// Cursor for compositor draw call submission tracking.
    pub(crate) compositor_index_cursor: u32,

    /// Bloom post-processing enabled flag.
    pub bloom_enabled: bool,
    /// Color blindness bind group layout (texture + sampler + uniform).
    pub(crate) color_blind_bind_group_layout: wgpu::BindGroupLayout,
    /// Color blindness uniform buffer (updated each frame when mode changes).
    pub(crate) color_blind_uniform_buffer: wgpu::Buffer,
    /// Color blindness simulation mode (Normal = disabled).
    pub color_blind_mode: crate::color_blindness::ColorBlindMode,
    /// Color blindness effect intensity (0.0–1.0).
    pub color_blind_intensity: f32,
    /// Sampler for the color blindness pass (reused from main pipeline).
    pub(crate) sampler: wgpu::Sampler,

    // Timestamp Queries (Norse: Skuld = future/time/debt)
    pub(crate) skuld_queries: Option<wgpu::QuerySet>,
    pub(crate) skuld_buffer: Option<wgpu::Buffer>,
    pub(crate) skuld_read_buffer: Option<wgpu::Buffer>,
    pub(crate) skuld_period: f32,
    pub last_gpu_time_ns: u64,

    // VDOM node stack for hierarchy tracking
    pub(crate) vnode_stack: Vec<(Rect, &'static str)>,

    /// Event handlers registered during render passes.
    /// Maps "event_type" -> list of handlers.
    pub(crate) event_handlers: std::collections::HashMap<
        String,
        Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>,
    >,

    /// Bind group layout for reading blur output in glass composite pass.
    pub(crate) glass_output_bind_group_layout: wgpu::BindGroupLayout,
    /// Current material state — draw calls are tagged with this material.
    pub(crate) current_draw_material: cvkg_core::DrawMaterial,

    /// Memoization cache for frame-level render skipping.
    /// Tracks (id, data_hash) -> skip_render for deduplication.
    pub(crate) memo_cache: std::collections::HashMap<u64, u64>,
}

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

        let mut adapter = None;

        // Manual override for driver/adapter selection (e.g. forcing amdgpu-pro over RADV)
        if let Ok(filter) = std::env::var("WGPU_ADAPTER_NAME") {
            let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
            println!("[GPU] Available adapters:");
            for a in &adapters {
                let info = a.get_info();
                println!(
                    "  - Name: '{}' | Driver: '{}' | Backend: {:?}",
                    info.name, info.driver, info.backend
                );
            }

            adapter = adapters.into_iter().find(|a| {
                let info = a.get_info();
                let match_found = info.name.to_lowercase().contains(&filter.to_lowercase())
                    || info.driver.to_lowercase().contains(&filter.to_lowercase());
                if match_found {
                    println!(
                        "[GPU] Manual selection match: {} | Driver: {}",
                        info.name, info.driver
                    );
                }
                match_found
            });

            if adapter.is_some() {
                println!(
                    "[GPU] Forced adapter selection via WGPU_ADAPTER_NAME='{}'",
                    filter
                );
            } else {
                println!(
                    "[GPU] WGPU_ADAPTER_NAME='{}' provided but no matching adapter found. Falling back...",
                    filter
                );
            }
        }

        if adapter.is_none() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            println!(
                "[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower..."
            );
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
        let info = adapter.get_info();
        println!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name, info.device_type, info.backend
        );
        println!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let mut required_features =
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY;
        if supports_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Forge"),
                required_features,
                required_limits: wgpu::Limits {
                    max_bindings_per_bind_group: 256,
                    max_binding_array_elements_per_shader_stage: 256,
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);

        device.on_uncaptured_error(Arc::new(|error| {
            log::error!(
                "[GPU] Uncaptured device error (Device Lost or Panic): {:?}",
                error
            );
            // In a full recovery scenario, we would signal the event loop to rebuild the GPU context
        }));

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
        // Ensure we have valid dimensions - Wayland may return 0 for not-yet-committed surfaces
        let width = if size.width > 0 { size.width } else { 1280 };
        let height = if size.height > 0 { size.height } else { 720 };
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = if surface_caps.formats.is_empty() {
            log::error!("[GPU] CRITICAL: No compatible surface formats found for this adapter!");
            log::error!(
                "[GPU] Adapter: {} | Backend: {:?}",
                adapter.get_info().name,
                adapter.get_info().backend
            );
            // Fallback to a common format to avoid immediate panic, though configuration may still fail
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0])
        };

        // Dynamic capability selection for robust Wayland/X11 rendering
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            log::warn!("[GPU] Mailbox not supported, falling back to Fifo (V-Sync)");
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        log::info!(
            "[GPU] Configuring surface: {}x{} | {:?} | {:?}",
            width,
            height,
            present_mode,
            alpha_mode
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("[GPU] Surface configuration successful.");

        let renderer = Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            Some((window, surface, config)),
            None,
        )
        .await;
        log::info!("[GPU] Forge internal complete.");
        renderer
    }

    /// Internal rendering pipeline constructor.
    /// This function spans ~600 lines because it is responsible for forging the entire wgpu state machine.
    ///
    /// ## Structure:
    /// 1. Formats & Timestamp query resolution buffers
    /// 2. Bind Group Layouts (Uniforms, Environment, Blur, Color Blindness)
    /// 3. Pipeline compilation (Opaque, Glass, Text, SVG paths)
    /// 4. Global Mega Atlas and Dummy Texture initialization
    /// 5. Staging belt & Telemetry scaffolding
    pub(crate) async fn forge_internal(
        instance: Arc<wgpu::Instance>,
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_info: Option<(
            Arc<winit::window::Window>,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
        )>,
        headless_info: Option<(u32, u32, wgpu::TextureFormat)>,
    ) -> Self {
        let format = if let Some((_, _, ref config)) = surface_info {
            config.format
        } else if let Some((_, _, f)) = headless_info {
            f
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let skuld_period = queue.get_timestamp_period();
        let (skuld_queries, skuld_buffer, skuld_read_buffer) = if supports_timestamps {
            let q = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Skuld Timestamp Queries"),
                count: 2,
                ty: wgpu::QueryType::Timestamp,
            });
            let b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Skuld Query Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let rb = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Skuld Read Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            (Some(q), Some(b), Some(rb))
        } else {
            (None, None, None)
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Main Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_SRC)),
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
                        count: std::num::NonZeroU32::new(256),
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
                    format,
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
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // ── Specialized Material Pipelines ─────────────────────────────────────
        let opaque_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Opaque"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_OPAQUE)),
        });
        let glass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Glass"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_GLASS)),
        });
        let pbr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim PBR"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_PBR)),
        });
        let gradient_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Gradient"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_GRADIENT)),
        });

        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Opaque"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &opaque_shader, entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &opaque_shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            multiview_mask: None, cache: None,
        });
        let glass_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Glass"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &glass_shader, entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &glass_shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            multiview_mask: None, cache: None,
        });
        let pbr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim PBR"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pbr_shader, entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pbr_shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            multiview_mask: None, cache: None,
        });
        let gradient_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Gradient"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gradient_shader, entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gradient_shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            multiview_mask: None, cache: None,
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
                        format,
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

        // Muspelheim Copy Pipeline (identity copy for backdrop blur Pass 2)
        let copy_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Muspelheim Copy"),
                layout: Some(&post_process_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_fullscreen"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_copy"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
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
        // NOTE: No blending - blur is a full-screen filter that replaces the destination
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
                    format,
                    blend: None, // Full-screen filter - replace, not blend
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
                    format,
                    blend: None, // Full-screen filter - replace, not blend
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

        // Kawase blur pyramid pipelines (separate shader module — conflicting bindings)
        // NOTE: Compiled separately because blur_pyramid.wgsl defines its own
        // @group(0) bindings (BlurUniforms + texture + sampler) that conflict
        // with the main WGSL_SRC pipeline layout.
        let kawase_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kawase Blur Pyramid"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                include_str!("shaders/blur_pyramid.wgsl"),
            )),
        });
        let kawase_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Kawase Blur BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let kawase_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Kawase Pipeline Layout"),
            bind_group_layouts: &[Some(&kawase_bgl)],
            immediate_size: 0,
        });
        let kawase_down_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kawase Downsample"),
            layout: Some(&kawase_layout),
            vertex: wgpu::VertexState {
                module: &kawase_shader, entry_point: Some("vs_blur"),
                buffers: &[], compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &kawase_shader, entry_point: Some("fs_kawase_down"),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            multiview_mask: None, cache: None,
        });
        let kawase_up_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kawase Upsample"),
            layout: Some(&kawase_layout),
            vertex: wgpu::VertexState {
                module: &kawase_shader, entry_point: Some("vs_blur"),
                buffers: &[], compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &kawase_shader, entry_point: Some("fs_kawase_up"),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            multiview_mask: None, cache: None,
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
                    format,
                    // Additive blend: src + dst — glow lights up the scene
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mega_atlas_view_obj =
            mega_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, // Use linear for images
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

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

        let mut texture_views_list: Vec<wgpu::TextureView> =
            (0..256).map(|_| dummy_view.clone()).collect();
        texture_views_list[0] = mega_atlas_view_obj.clone();

        let views_refs: Vec<&wgpu::TextureView> = texture_views_list.iter().collect();
        let mega_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&text_sampler),
                },
            ],
            label: Some("Mega-Atlas Bind Group"),
        });

        let dummy_views_refs: Vec<&wgpu::TextureView> = (0..256).map(|_| &dummy_view).collect();
        let dummy_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&dummy_views_refs),
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
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Instance Anvil"),
            size: (MAX_VERTICES / 4 * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
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

        let (width, height, scale_factor) = if let Some((ref window, _, ref config)) = surface_info
        {
            (config.width, config.height, window.scale_factor() as f32)
        } else if let Some((w, h, _)) = headless_info {
            (w, h, 1.0)
        } else {
            (1280, 720, 1.0)
        };

        let mut current_scene =
            SceneUniforms::new(width as f32 / scale_factor, height as f32 / scale_factor);
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

        let staging_belt = wgpu::util::StagingBelt::new((*device).clone(), 1024 * 1024);

        let glass_output_bind_group_layout = env_bind_group_layout.clone();

        // Color blindness pipeline layout (1 bind group: texture + sampler + uniform)
        let color_blind_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Blind Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<crate::color_blindness::ColorBlindUniforms>() as u64,
                        ),
                    },
                    count: None,
                },
            ],
        });
        let color_blind_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Blind Pipeline Layout"),
            bind_group_layouts: &[Some(&color_blind_bgl)],
            immediate_size: 0,
        });

        // Color blindness shader module and pipeline (separate from main shader)
        let color_blind_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Color Blind Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                crate::color_blindness::shader_source(),
            )),
        });
        let color_blind_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Color Blindness"),
            layout: Some(&color_blind_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_blind_shader,
                entry_point: Some("fs_main_vs"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_blind_shader,
                entry_point: Some("fs_color_blind"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        // Color blindness uniform buffer (updated each frame when mode is active)
        let color_blind_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color Blind Uniforms"),
            size: std::mem::size_of::<crate::color_blindness::ColorBlindUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Sampler for the color blindness pass (and other post-process passes)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),

            surfaces,
            current_window,
            headless_context,
            pipeline,
            opaque_pipeline,
            glass_pipeline,
            pbr_pipeline,
            gradient_pipeline,
            bloom_extract_pipeline,
            copy_pipeline,
            blur_h_pipeline,
            blur_v_pipeline,
            composite_pipeline,
            env_bind_group_layout,
            text_engine: cvkg_runic_text::RunicTextEngine::default(),
            mega_atlas_tex,
            mega_atlas_bind_group,
            text_cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
            atlas_packer: YggdrasilPacker::new(4096, 4096),
            image_uv_registry: LruCache::new(NonZeroUsize::new(256).unwrap()),
            texture_registry: LruCache::new(NonZeroUsize::new(255).unwrap()),
            texture_views: texture_views_list,
            dummy_sampler,
            svg_cache: LruCache::new(NonZeroUsize::new(128).unwrap()),
            svg_trees: LruCache::new(NonZeroUsize::new(128).unwrap()),
            filter_engine: Some(cvkg_svg_filters::FilterEngine::new(
                cvkg_svg_filters::GpuContext {
                    device: device.clone(),
                    queue: queue.clone(),
                },
            ).expect("Failed to create SVG filter engine")),
            filter_batches: Vec::new(),
            dummy_texture_bind_group,
            dummy_env_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
            shared_elements: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            vertex_buffer,
            index_buffer,
            instance_buffer,
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
            instance_data: Vec::with_capacity(MAX_VERTICES / 4),
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
            capture_staging_buffer: None,
            compositor_index_cursor: 0,
            vram_buffers_bytes: 0,
            vram_textures_bytes: 0,
            _debug_layout: false,
            transform_stack: Vec::new(),
            redraw_requested: false,
            skuld_queries,
            skuld_buffer,
            skuld_read_buffer,
            skuld_period,
            last_gpu_time_ns: 0,
            vnode_stack: Vec::new(),
            event_handlers: std::collections::HashMap::new(),
            staging_belt,
            staging_command_buffers: Vec::new(),
            glass_output_bind_group_layout,
            current_draw_material: cvkg_core::DrawMaterial::Opaque,
            memo_cache: std::collections::HashMap::new(),
            bloom_enabled: true,
            color_blind_mode: crate::color_blindness::ColorBlindMode::Normal,
            color_blind_intensity: 1.0,
            color_blind_pipeline,
            color_blind_bind_group_layout: color_blind_bgl,
            color_blind_uniform_buffer,
            sampler,
            kawase_down_pipeline,
            kawase_up_pipeline,
            kawase_bind_group_layout: kawase_bgl,
        }
    }

    pub(crate) fn rebuild_texture_array_bind_group(&mut self) {
        let views: Vec<&wgpu::TextureView> = self.texture_views.iter().collect();
        self.mega_atlas_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.dummy_sampler),
                },
            ],
            label: Some("Surtr Texture Array Bind Group"),
        });
    }

    /// Update VRAM telemetry based on currently allocated resources.
    pub(crate) fn update_vram_telemetry(&mut self) {
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
            texture_bytes += surface_bytes * 3; // scene (1x), depth (1x), blur a/b (0.5x), bloom a/b (0.5x)
        }

        self.vram_textures_bytes = texture_bytes;

        self.telemetry.vram_buffers_mb = buffer_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_textures_mb = texture_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_pipelines_mb = 0.0;
        self.telemetry.vram_usage_mb =
            self.telemetry.vram_buffers_mb + self.telemetry.vram_textures_mb;
    }

    /// Get real-time performance telemetry.
    pub fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
    }

    /// resize — Reconfigures a specific surface and its internal textures.
    pub fn resize(
        &mut self,
        window_id: winit::window::WindowId,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) {
        if width > 0
            && height > 0
            && let Some(ctx) = self.surfaces.get_mut(&window_id)
        {
            ctx.config.width = width;
            ctx.config.height = height;
            ctx.scale_factor = scale_factor;
            ctx.surface.configure(&self.device, &ctx.config);

            // Re-create Muspelheim textures for this surface
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
                format: ctx.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
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
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ctx.scene_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Scene Bind Group Resize"),
            });

            let scene_views: Vec<&wgpu::TextureView> =
                (0..256).map(|_| &ctx.scene_texture).collect();
            ctx.scene_texture_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                        },
                    ],
                    label: Some("Scene Texture Bind Group Resize"),
                });

            let blur_views_a: Vec<&wgpu::TextureView> =
                (0..256).map(|_| &ctx.blur_texture_a).collect();
            ctx.blur_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Bind Group A Resize"),
            });

            let blur_views_b: Vec<&wgpu::TextureView> =
                (0..256).map(|_| &ctx.blur_texture_b).collect();
            ctx.blur_bind_group_b = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Bind Group B Resize"),
            });

            ctx.blur_env_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.env_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ctx.blur_texture_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Env Bind Group A Resize"),
            });

            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            ctx.depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    /// begin_frame_headless — Strike the flaming sword to begin a new GPU frame for headless rendering.
    pub fn begin_frame_headless(&mut self) -> wgpu::CommandEncoder {
        self.current_window = None;
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.filter_batches.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.current_z = 0.0;
        self.compositor_index_cursor = self.indices.len() as u32;
        self.vnode_stack.clear();
        self.event_handlers.clear();

        // Clear memoization cache at the start of each frame
        self.memo_cache.clear();

        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

        // Recall staging belt buffers so they can be reused for vertex upload
        self.staging_belt.recall();

        let ctx = self
            .headless_context
            .as_ref()
            .expect("Headless context not initialized");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.width as f32 / ctx.scale_factor;
        let logical_h = ctx.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Headless Command Encoder"),
            })
    }

    /// begin_frame — Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        // Skuld: Read the timestamps from the previous frame
        if let Some(rb) = &self.skuld_read_buffer {
            let slice = rb.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });

            // Poll to ensure mapping is complete
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .unwrap();

            if rx.recv().is_ok() {
                let data = slice.get_mapped_range();
                let timestamps: [u64; 2] = bytemuck::cast_slice(&data).try_into().unwrap_or([0, 0]);
                drop(data);
                rb.unmap();

                if timestamps[1] > timestamps[0] {
                    let diff_ticks = timestamps[1] - timestamps[0];
                    self.last_gpu_time_ns = (diff_ticks as f64 * self.skuld_period as f64) as u64;
                    // println!("[Skuld] GPU Time: {} ms", self.last_gpu_time_ns as f64 / 1_000_000.0);
                }
            }
        }

        self.staging_belt.recall();
        self.current_window = Some(window_id);
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.filter_batches.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.current_z = 0.0;
        self.vnode_stack.clear();
        self.event_handlers.clear();
        
        // Clear memoization cache at the start of each frame
        self.memo_cache.clear();

        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

        let ctx = self
            .surfaces
            .get(&window_id)
            .expect("Window not registered");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.config.width as f32 / ctx.scale_factor;
        let logical_h = ctx.config.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Command Encoder"),
            })
    }

    /// register_window — Attaches a new OS window to the shared GPU context.
    pub fn register_window(&mut self, window: Arc<winit::window::Window>) {
        let size = window.inner_size();
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps.formats[0];

        // Dynamic present mode selection — Mailbox not available on all platforms (e.g. Wayland)
        let present_mode = if caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            log::warn!("[GPU] Mailbox not supported, falling back to Fifo (V-Sync)");
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            caps.alpha_modes[0]
        };

        log::info!(
            "[GPU] Configuring surface: {}x{} | {:?} | {:?}",
            size.width, size.height, present_mode, alpha_mode
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
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

    pub(crate) fn create_headless_context(
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_width = (width / 2).max(1);
        let blur_height = (height / 2).max(1);
        let blur_texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Blur Texture"),
            size: wgpu::Extent3d {
                width: blur_width,
                height: blur_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 5,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let blur_tex_a = device.create_texture(&blur_texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&blur_texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Create dedicated bloom textures (full resolution for proper bloom, separate from backdrop blur)
        let bloom_tex_a = device.create_texture(&blur_texture_desc);
        let bloom_texture_a = bloom_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let bloom_tex_b = device.create_texture(&blur_texture_desc);
        let bloom_texture_b = bloom_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

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

        let scene_views: Vec<&wgpu::TextureView> = (0..256).map(|_| &scene_texture).collect();
        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Scene Texture Bind Group"),
        });

        let blur_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_a).collect();
        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Bind Group A"),
        });

        let blur_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_b).collect();
        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
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

        // Bloom bind groups (dedicated textures to avoid clobbering backdrop blur)
        let bloom_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &bloom_texture_a).collect();
        let bloom_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&bloom_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Bloom Bind Group A"),
        });

        let bloom_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &bloom_texture_b).collect();
        let bloom_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&bloom_views_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Bloom Bind Group B"),
        });

        let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Bloom Env Bind Group A"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
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
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        HeadlessContext {
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_tex_a,
            blur_texture_a,
            blur_tex_b,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            bloom_tex_a,
            bloom_texture_a,
            bloom_tex_b,
            bloom_texture_b,
            bloom_bind_group_a,
            bloom_bind_group_b,
            bloom_env_bind_group_a,
            scale_factor: 1.0,
            sampler,
            width,
            height,
            output_texture,
            output_view,
        }
    }

    pub(crate) fn create_surface_context(
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

        let blur_width = (width / 2).max(1);
        let blur_height = (height / 2).max(1);
        let blur_texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Blur Texture"),
            size: wgpu::Extent3d {
                width: blur_width,
                height: blur_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 5,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let blur_tex_a = device.create_texture(&blur_texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&blur_texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Create dedicated bloom textures (full resolution for proper bloom, separate from backdrop blur)
        let bloom_tex_a = device.create_texture(&blur_texture_desc);
        let bloom_texture_a = bloom_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let bloom_tex_b = device.create_texture(&blur_texture_desc);
        let bloom_texture_b = bloom_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

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

        let scene_views: Vec<&wgpu::TextureView> = (0..256).map(|_| &scene_texture).collect();
        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Scene Texture Bind Group"),
        });

        let blur_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_a).collect();
        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Bind Group A"),
        });

        let blur_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_b).collect();
        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
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
            ],
            label: Some("Blur Env Bind Group A"),
        });

        // Bloom bind groups (dedicated textures to avoid clobbering backdrop blur)
        let bloom_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &bloom_texture_a).collect();
        let bloom_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&bloom_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Bloom Bind Group A"),
        });

        let bloom_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &bloom_texture_b).collect();
        let bloom_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&bloom_views_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Bloom Bind Group B"),
        });

        let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Bloom Env Bind Group A"),
        });

        SurfaceContext {
            surface,
            config,
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_tex_a,
            blur_texture_a,
            blur_tex_b,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            bloom_tex_a,
            bloom_texture_a,
            bloom_tex_b,
            bloom_texture_b,
            bloom_bind_group_a,
            bloom_bind_group_b,
            bloom_env_bind_group_a,
            scale_factor,
            sampler,
        }
    }

    pub fn reset_time(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    /// reclaim_vram — Atomic recycling of the Mega-Atlas and all associated caches.
    /// This prevents OOM and silent failures by quenching the atlas when full.
    pub fn reclaim_vram(&mut self) {
        log::warn!("[GPU] Yggdrasil Compaction: Compacting Mega-Atlas...");

        let new_mega_atlas_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Yggdrasil Mega-Atlas (Compacted)"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mut new_packer = YggdrasilPacker::new(4096, 4096);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Atlas Compaction Encoder"),
            });

        let image_entries: Vec<(String, Rect)> = self
            .image_uv_registry
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (name, old_uv) in image_entries {
            if let Some(&tex_idx) = self.texture_registry.get(&name)
                && tex_idx == 0
            {
                let w_px = (old_uv.width * 4096.0).round() as u32;
                let h_px = (old_uv.height * 4096.0).round() as u32;
                let old_x_px = (old_uv.x * 4096.0).round() as u32;
                let old_y_px = (old_uv.y * 4096.0).round() as u32;

                if let Some((new_x, new_y)) = new_packer.pack(w_px, h_px) {
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_atlas_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: old_x_px,
                                y: old_y_px,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: &new_mega_atlas_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: new_x,
                                y: new_y,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: w_px,
                            height: h_px,
                            depth_or_array_layers: 1,
                        },
                    );

                    let new_uv = Rect {
                        x: new_x as f32 / 4096.0,
                        y: new_y as f32 / 4096.0,
                        width: old_uv.width,
                        height: old_uv.height,
                    };
                    self.image_uv_registry.put(name.clone(), new_uv);
                }
            }
        }

        let text_entries: Vec<(u64, (Rect, f32, f32))> =
            self.text_cache.iter().map(|(k, v)| (*k, *v)).collect();
        for (hash, (old_uv, w_f, h_f)) in text_entries {
            let w_px = (old_uv.width * 4096.0).round() as u32;
            let h_px = (old_uv.height * 4096.0).round() as u32;
            let old_x_px = (old_uv.x * 4096.0).round() as u32;
            let old_y_px = (old_uv.y * 4096.0).round() as u32;

            if let Some((new_x, new_y)) = new_packer.pack(w_px, h_px) {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.mega_atlas_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: old_x_px,
                            y: old_y_px,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &new_mega_atlas_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: new_x,
                            y: new_y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: w_px,
                        height: h_px,
                        depth_or_array_layers: 1,
                    },
                );

                let new_uv = Rect {
                    x: new_x as f32 / 4096.0,
                    y: new_y as f32 / 4096.0,
                    width: old_uv.width,
                    height: old_uv.height,
                };
                self.text_cache.put(hash, (new_uv, w_f, h_f));
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.mega_atlas_tex = new_mega_atlas_tex;
        let mega_atlas_view_obj = self
            .mega_atlas_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.texture_views[0] = mega_atlas_view_obj.clone();

        self.rebuild_texture_array_bind_group();

        if !self.texture_bind_groups.is_empty() {
            self.texture_bind_groups[0] = self.mega_atlas_bind_group.clone();
        }

        self.atlas_packer = new_packer;
        self.telemetry.vram_exhausted = false;
    }

    pub(crate) fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        material_id: u32,
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

                self.fill_rect_with_full_params(shard_rect, c, material_id, None, force, uv);
            }
        }
    }

    pub(crate) fn recursive_bolt(&mut self, from: [f32; 2], to: [f32; 2], depth: u32, color: [f32; 4]) {
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

    pub(crate) fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
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

    pub(crate) fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        material_id: u32,
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
                material: if material_id == 7 {
                    cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
                } else if material_id == 6 {
                    cvkg_core::DrawMaterial::TopUI
                } else {
                    cvkg_core::DrawMaterial::Opaque
                },
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

            let (translation, scale_transform, rotation, _, _) = self.current_transform();
            self.vertices.push(Vertex {
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: uvs[i],
                color, material_id, radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                screen,
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
                translation,
                scale: scale_transform,
                rotation,
                tex_index: 0,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }
    pub(crate) fn get_texture_id(&mut self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode — Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color, material_id, texture_id,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    pub(crate) fn fill_rect_with_full_params(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
    ) {
        // If a shadow is active, draw it first
        if let Some(shadow) = self.shadow_stack.last().copied()
            && shadow.color[3] > 0.001
        {
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
            rect, color, material_id, texture_id, radius, uv_rect, slice,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_rect_with_full_params_and_slice(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
    ) {
        let scissor = self.clip_stack.last().copied();

        let material = if material_id == 7 {
            cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
        } else if material_id == 6 {
            cvkg_core::DrawMaterial::TopUI
        } else if material_id == 0 {
            cvkg_core::DrawMaterial::Opaque
        } else {
            self.current_draw_material
        };

        // Batching: check if we need to start a new DrawCall
        // With Texture Array, we no longer need to break batches when the texture changes,
        // as long as they are all part of the same array bind group (Group 0).
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = Some(0); // All textures are now in the binding array at Group 0
            self.draw_calls.push(DrawCall {
                texture_id: self.current_texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                material,
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

        let (translation, scale_transform, rotation, _, _) = self.current_transform();

        let tex_index = texture_id.unwrap_or(0);

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color, material_id, radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color, material_id, radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color, material_id, radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color, material_id, radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
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


    // ═══════════════════════════════════════════════════════════════════════════
    // Kvasir pass encoding methods
    // ═══════════════════════════════════════════════════════════════════════════
    // Each method encodes one render pass into the provided command encoder.
    // Called from end_frame() which assembles the graph-driven pass sequence.

    /// Pass 1: Clear scene+depth, draw atmosphere, draw opaque geometry.
    pub(crate) fn execute_pass_geometry(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx_scene_texture: &wgpu::TextureView,
        ctx_depth_texture_view: &wgpu::TextureView,
        _scale: f32,
    ) {
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P1 Opaque Background"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx_scene_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }),
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
            timestamp_writes: self.skuld_queries.as_ref().map(|q| {
                wgpu::RenderPassTimestampWrites {
                    query_set: q,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: None,
                }
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if self.current_scene.scene_type == cvkg_core::SCENE_AURORA {
            p.set_pipeline(&self.background_pipeline);
            p.set_bind_group(0, &self.dummy_texture_bind_group, &[]);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..3, 0..1);
        }

        if !self.draw_calls.is_empty() {
            p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);

            for call in self.draw_calls.iter().filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Opaque)) {
                p.set_pipeline(&self.opaque_pipeline);
                let bg = if let Some(id) = call.texture_id {
                    if id == 0 { &self.mega_atlas_bind_group }
                    else {
                        self.texture_bind_groups.get(id as usize)
                            .unwrap_or(&self.dummy_texture_bind_group)
                    }
                } else { &self.dummy_texture_bind_group };
                p.set_bind_group(0, bg, &[]);
                p.draw_indexed(call.index_start..call.index_start + call.index_count, 0, 0..1);
                self.telemetry.draw_calls += 1;
                self.telemetry.vertices += call.index_count;
            }
        }
    }

    /// Pass 2: Identity copy scene → blur texture (all pixels).
    pub(crate) fn execute_pass_backdrop_copy(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_texture: &wgpu::Texture,
        source_bind_group: &wgpu::BindGroup,
    ) {
        let target_view = target_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("backdrop_copy_mip0"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Backdrop Copy"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });
        p.set_pipeline(&self.copy_pipeline);
        p.set_bind_group(0, source_bind_group, &[]);
        p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
        p.set_bind_group(2, &self.berserker_bind_group, &[]);
        p.draw(0..3, 0..1);
    }

    /// Pass 3: Kawase blur pyramid on backdrop texture.
    /// Downsamples from mip 0 → mip 4, then upsamples back 4 → 0.
    /// Each pass uses the Kawase shader with a diagonal cross kernel.
    pub(crate) fn execute_pass_backdrop_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        blur_tex: &wgpu::Texture,
        blur_width: u32,
        blur_height: u32,
    ) {
        // Kawase blur pyramid: downsample 0→4, then upsample 4→0
        // Each pass uses the Kawase shader with a diagonal 4-tap kernel.
        //
        // The uniform buffer provides: [texture_size.xy, mip_level, kernel_width]
        // per the BlurUniforms struct in blur_pyramid.wgsl.

        // Create a uniform buffer for the Kawase params.
        // Each downsample iteration uses kernel_width = iteration_index (0,1,2,3)
        // Each upsample iteration uses the same pattern in reverse.
        let _uniform_data: [[f32; 4]; 2] = [
            [blur_width as f32, blur_height as f32, 0.0, 0.0], // params.xy = size, params.z = mip, params.w = kernel_width
            [0.0, 0.0, 0.0, 0.0], // padding to 32 bytes (min_binding_size)
        ];
        // Use queue.write_buffer to upload uniforms each iteration.
        // For simplicity, create one buffer and re-write it per pass.
        let kawase_uniform = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kawase Uniform"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create per-mip views of the blur texture.
        let mip_views: Vec<wgpu::TextureView> = (0..5)
            .map(|mip| {
                blur_tex.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("blur_mip_{}", mip)),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        // Create bind groups: each mip gets a bind group with the source texture view,
        // the sampler, and the uniform buffer.
        let kawase_bind_groups: Vec<wgpu::BindGroup> = (0..5)
            .map(|mip| {
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("kawase_bg_{}", mip)),
                    layout: &self.kawase_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                wgpu::BufferBinding {
                                    buffer: &kawase_uniform,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(32),
                                },
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&mip_views[mip as usize]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                })
            })
            .collect();

        let mip_scales = [
            (blur_width as f32, blur_height as f32, 1.0_f32),       // mip 0: full res
            (blur_width as f32 / 2.0, blur_height as f32 / 2.0, 2.0), // mip 1: half
            (blur_width as f32 / 4.0, blur_height as f32 / 4.0, 3.0), // mip 2: quarter
            (blur_width as f32 / 8.0, blur_height as f32 / 8.0, 4.0), // mip 3: eighth
            (blur_width as f32 / 16.0, blur_height as f32 / 16.0, 5.0), // mip 4: sixteenth
        ];

        // Downsample chain: read from mip N-1, write to mip N
        for mip in 1..5 {
            let kernel_width = mip_scales[mip as usize].2;
            // Update uniform buffer
            let uniform_data: [f32; 8] = [
                mip_scales[(mip - 1) as usize].0, mip_scales[(mip - 1) as usize].1,
                (mip - 1) as f32, kernel_width,
                0.0, 0.0, 0.0, 0.0,
            ];
            self.queue.write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data[..8]));

            let w = mip_scales[mip as usize].0.max(1.0) as u32;
            let h = mip_scales[mip as usize].1.max(1.0) as u32;

            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Down {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[mip as usize],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&self.kawase_down_pipeline);
            p.set_bind_group(0, &kawase_bind_groups[(mip - 1) as usize], &[]);
            p.draw(0..3, 0..1);
        }

        // Upsample chain: read from mip N, write to mip N-1
        for mip in (1..5).rev() {
            let kernel_width = mip_scales[mip as usize].2;
            let uniform_data: [f32; 8] = [
                mip_scales[mip as usize].0, mip_scales[mip as usize].1,
                mip as f32, kernel_width,
                0.0, 0.0, 0.0, 0.0,
            ];
            self.queue.write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data[..8]));

            let w = mip_scales[(mip - 1) as usize].0.max(1.0) as u32;
            let h = mip_scales[(mip - 1) as usize].1.max(1.0) as u32;

            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Up {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[(mip - 1) as usize],
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&self.kawase_up_pipeline);
            p.set_bind_group(0, &kawase_bind_groups[mip as usize], &[]);
            p.draw(0..3, 0..1);
        }

        log::trace!("[Kvasir] backdrop_blur: Kawase pyramid ({}x{})", blur_width, blur_height);
    }

    /// Pass 4: Glass panels with backdrop blur sampling.
    pub(crate) fn execute_pass_glass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx_scene_texture: &wgpu::TextureView,
        ctx_depth_texture_view: &wgpu::TextureView,
        ctx_blur_env_bind_group_a: &wgpu::BindGroup,
        scale: f32,
    ) {
        let rt_w = self.current_width() as i32;
        let rt_h = self.current_height() as i32;
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P3 Liquid Glass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx_scene_texture,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: ctx_depth_texture_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        p.set_pipeline(&self.glass_pipeline);
        p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]);
        p.set_bind_group(2, &self.berserker_bind_group, &[]);
        for call in self.draw_calls.iter().filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. })) {
            let bg = if let Some(id) = call.texture_id {
                if id == 0 { &self.mega_atlas_bind_group }
                else { self.texture_bind_groups.get(id as usize).unwrap_or(&self.dummy_texture_bind_group) }
            } else { &self.dummy_texture_bind_group };
            p.set_bind_group(0, bg, &[]);
            if let Some(rect) = call.scissor_rect {
                if rt_w > 0 && rt_h > 0 {
                    let x1 = (rect.x * scale).round() as i32;
                    let y1 = (rect.y * scale).round() as i32;
                    let x2 = ((rect.x + rect.width) * scale).round() as i32;
                    let y2 = ((rect.y + rect.height) * scale).round() as i32;
                    let w = (x2 - x1).clamp(0, rt_w);
                    let h = (y2 - y1).clamp(0, rt_h);
                    if w > 0 && h > 0 { p.set_scissor_rect(x1 as u32, y1 as u32, w as u32, h as u32); }
                    else { p.set_scissor_rect(0, 0, 1, 1); }
                }
            }
            p.draw_indexed(call.index_start..call.index_start + call.index_count, 0, 0..1);
            self.telemetry.draw_calls += 1;
            self.telemetry.vertices += call.index_count;
        }
    }

    /// Pass 5: UI overlay.
    pub(crate) fn execute_pass_ui(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx_scene_texture: &wgpu::TextureView,
        ctx_depth_texture_view: &wgpu::TextureView,
        scale: f32,
    ) {
        let rt_w = self.current_width() as i32;
        let rt_h = self.current_height() as i32;
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P4 UI Layer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx_scene_texture,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: ctx_depth_texture_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        p.set_pipeline(&self.opaque_pipeline);
        p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
        p.set_bind_group(2, &self.berserker_bind_group, &[]);
        for call in self.draw_calls.iter().filter(|c| matches!(c.material, cvkg_core::DrawMaterial::TopUI)) {
            let bg = if let Some(id) = call.texture_id {
                if id == 0 { &self.mega_atlas_bind_group }
                else { self.texture_bind_groups.get(id as usize).unwrap_or(&self.dummy_texture_bind_group) }
            } else { &self.dummy_texture_bind_group };
            p.set_bind_group(0, bg, &[]);
            if let Some(rect) = call.scissor_rect {
                if rt_w > 0 && rt_h > 0 {
                    let x1 = (rect.x * scale).round() as i32;
                    let y1 = (rect.y * scale).round() as i32;
                    let x2 = ((rect.x + rect.width) * scale).round() as i32;
                    let y2 = ((rect.y + rect.height) * scale).round() as i32;
                    let w = (x2 - x1).clamp(0, rt_w);
                    let h = (y2 - y1).clamp(0, rt_h);
                    if w > 0 && h > 0 { p.set_scissor_rect(x1 as u32, y1 as u32, w as u32, h as u32); }
                    else { p.set_scissor_rect(0, 0, 1, 1); }
                }
            }
            p.draw_indexed(call.index_start..call.index_start + call.index_count, 0, 0..1);
            self.telemetry.draw_calls += 1;
            self.telemetry.vertices += call.index_count;
        }
    }

    /// Pass 6: Bloom extract (luminance-gated).
    pub(crate) fn execute_pass_bloom_extract(
        &mut self,
        post_encoder: &mut wgpu::CommandEncoder,
        _ctx_scene_texture: &wgpu::TextureView,
        ctx_scene_texture_bind_group: &wgpu::BindGroup,
        bloom_texture: &wgpu::Texture,
    ) {
        // Create a single-mip view for the render pass (mip 0 only)
        let bloom_view = bloom_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("bloom_extract_mip0"),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Bloom Extract"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &bloom_view,
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
        p.draw(0..3, 0..1);
    }

    /// Pass 7: Bloom blur using Kawase pyramid (2 iterations).
    /// Uses the same Kawase pipelines as backdrop blur but on bloom textures.
    pub(crate) fn execute_pass_bloom_blur(
        &mut self,
        post_encoder: &mut wgpu::CommandEncoder,
        bloom_tex: &wgpu::Texture,
        bloom_width: u32,
        bloom_height: u32,
    ) {
        // Create uniform buffer for Kawase params
        let kawase_uniform = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kawase Bloom Uniform"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create per-mip views of the bloom texture
        let mip_views: Vec<wgpu::TextureView> = (0..5)
            .map(|mip| {
                bloom_tex.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("bloom_mip_{}", mip)),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let mip_scales = [
            (bloom_width as f32, bloom_height as f32, 1.0_f32),
            (bloom_width as f32 / 2.0, bloom_height as f32 / 2.0, 2.0),
            (bloom_width as f32 / 4.0, bloom_height as f32 / 4.0, 3.0),
            (bloom_width as f32 / 8.0, bloom_height as f32 / 8.0, 4.0),
            (bloom_width as f32 / 16.0, bloom_height as f32 / 16.0, 5.0),
        ];

        // Downsample chain
        for mip in 1..5 {
            let kernel_width = mip_scales[mip as usize].2;
            let uniform_data: [f32; 8] = [
                mip_scales[(mip - 1) as usize].0, mip_scales[(mip - 1) as usize].1,
                (mip - 1) as f32, kernel_width,
                0.0, 0.0, 0.0, 0.0
            ];
            self.queue.write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

            let w = mip_scales[mip as usize].0.max(1.0) as u32;
            let h = mip_scales[mip as usize].1.max(1.0) as u32;

            // Re-create bind group for this mip level
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("kawase_bloom_bg_{}", mip)),
                layout: &self.kawase_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &kawase_uniform, offset: 0, size: wgpu::BufferSize::new(32),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mip_views[(mip - 1) as usize]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Bloom Down {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[mip as usize],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&self.kawase_down_pipeline);
            p.set_bind_group(0, &bg, &[]);
            p.draw(0..3, 0..1);
        }

        // Upsample chain
        for mip in (1..5).rev() {
            let kernel_width = mip_scales[mip as usize].2;
            let uniform_data: [f32; 8] = [
                mip_scales[mip as usize].0, mip_scales[mip as usize].1,
                mip as f32, kernel_width,
                0.0, 0.0, 0.0, 0.0
            ];
            self.queue.write_buffer(&kawase_uniform, 0, bytemuck::cast_slice(&uniform_data));

            let w = mip_scales[(mip - 1) as usize].0.max(1.0) as u32;
            let h = mip_scales[(mip - 1) as usize].1.max(1.0) as u32;

            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("kawase_bloom_up_{}", mip)),
                layout: &self.kawase_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &kawase_uniform, offset: 0, size: wgpu::BufferSize::new(32),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mip_views[mip as usize]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Kawase Bloom Up {}", mip)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip_views[(mip - 1) as usize],
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
            p.set_pipeline(&self.kawase_up_pipeline);
            p.set_bind_group(0, &bg, &[]);
            p.draw(0..3, 0..1);
        }

        log::trace!("[Kvasir] bloom_blur: Kawase pyramid ({}x{})", bloom_width, bloom_height);
    }

    /// Pass 8: Composite scene+bloom → swapchain.
    pub(crate) fn execute_pass_composite(
        &mut self,
        post_encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        _scene_texture: &wgpu::TextureView,
        scene_texture_bind_group: &wgpu::BindGroup,
        _bloom_texture_a: &wgpu::TextureView,
        bloom_env_bind_group_a: &wgpu::BindGroup,
    ) {
        let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr P7 Composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: self.skuld_queries.as_ref().map(|q| {
                wgpu::RenderPassTimestampWrites {
                    query_set: q,
                    beginning_of_pass_write_index: None,
                    end_of_pass_write_index: Some(1),
                }
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        p.set_pipeline(&self.composite_pipeline);
        p.set_bind_group(0, scene_texture_bind_group, &[]);
        p.set_bind_group(1, bloom_env_bind_group_a, &[]);
        p.set_bind_group(2, &self.berserker_bind_group, &[]);
        p.draw(0..3, 0..1);
    }

    /// Pass 9: Accessibility (color blindness transform).
    /// Applies Brettel/Viénot Daltonization matrix in linear RGB space.
    /// Runs after composite and before present when color_blind_mode != Normal.
    pub(crate) fn execute_pass_accessibility(
        &mut self,
        post_encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        scene_texture: &wgpu::TextureView,
        _scene_texture_bind_group: &wgpu::BindGroup,
    ) {
        // Skip if mode is Normal (identity transform)
        if self.color_blind_mode.is_identity() {
            return;
        }

        // Update uniform buffer with current mode/intensity
        let uniforms = ColorBlindUniforms::new(self.color_blind_mode, self.color_blind_intensity);
        self.queue.write_buffer(
            &self.color_blind_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create bind group at draw time with the actual scene texture
        let color_blind_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Blind Bind Group"),
            layout: &self.color_blind_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.color_blind_uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(
                            std::mem::size_of::<ColorBlindUniforms>() as u64,
                        ),
                    }),
                },
            ],
        });

        let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Accessibility"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        p.set_pipeline(&self.color_blind_pipeline);
        p.set_bind_group(0, &color_blind_bind_group, &[]);
        p.draw(0..3, 0..1);
    }

    /// end_frame — Quench the blade by submitting the full Muspelheim multi-pass effect.
    ///
    /// Since the Renderer 3.0 migration, the pass sequence is driven by a Kvasir
    /// dependency graph rather than hardcoded ordering. The graph is built each
    /// frame (cheap — just node/edge allocation), validated (cycle detection,
    /// input satisfiability), then executed. Conditional passes (glass, bloom,
    /// accessibility) are automatically eliminated when not needed.
    pub fn end_frame(&mut self, mut encoder: wgpu::CommandEncoder) {
        struct ActiveFrameResources {
            surface_texture: Option<wgpu::SurfaceTexture>,
            target_view: wgpu::TextureView,
            scene_texture: wgpu::TextureView,
            depth_texture_view: wgpu::TextureView,
            scene_texture_bind_group: wgpu::BindGroup,
            blur_tex_a: wgpu::Texture,
            blur_texture_a: wgpu::TextureView,
            blur_env_bind_group_a: wgpu::BindGroup,
            bloom_tex_a: wgpu::Texture,
            bloom_texture_a: wgpu::TextureView,
            bloom_env_bind_group_a: wgpu::BindGroup,
            scale_factor: f32,
        }

        let res = if let Some(window_id) = self.current_window {
            let Some(ctx) = self.surfaces.get(&window_id) else {
                log::error!("[GPU] Missing surface context for end_frame");
                return;
            };
            let frame = match ctx.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                    ctx.surface.configure(&self.device, &ctx.config);
                    t
                }
                other => {
                    log::warn!("[GPU] Surface texture acquisition failed ({:?}), reconfiguring surface", other);
                    ctx.surface.configure(&self.device, &ctx.config);
                    self.queue.submit(std::iter::once(encoder.finish()));
                    return;
                }
            };
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            ActiveFrameResources {
                surface_texture: Some(frame),
                target_view: view,
                scene_texture: ctx.scene_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                scene_texture_bind_group: ctx.scene_texture_bind_group.clone(),
                blur_tex_a: ctx.blur_tex_a.clone(),
                blur_texture_a: ctx.blur_texture_a.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                bloom_tex_a: ctx.bloom_tex_a.clone(),
                bloom_texture_a: ctx.bloom_texture_a.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                scale_factor: ctx.scale_factor,
            }
        } else {
            let Some(ctx) = self.headless_context.as_ref() else {
                log::error!("[GPU] No headless context for end_frame");
                return;
            };
            ActiveFrameResources {
                surface_texture: None,
                target_view: ctx.output_view.clone(),
                scene_texture: ctx.scene_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                scene_texture_bind_group: ctx.scene_texture_bind_group.clone(),
                blur_tex_a: ctx.blur_tex_a.clone(),
                blur_texture_a: ctx.blur_texture_a.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                bloom_tex_a: ctx.bloom_tex_a.clone(),
                bloom_texture_a: ctx.bloom_texture_a.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                scale_factor: self.current_scale_factor(),
            }
        };

        // ── Build and execute the Kvasir frame graph ─────────────────────────────
        let has_glass = self.draw_calls.iter().any(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }));
        let has_bloom = self.bloom_enabled;
        let has_accessibility = self.color_blind_mode != crate::color_blindness::ColorBlindMode::Normal;

        let mut post_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Surtr Post-Process Encoder"),
        });

        // Build the frame graph using the Kvasir helper for correct pass ordering.
        // Conditional passes (glass, bloom, accessibility) are included/excluded based on frame state.
        // This replaces the hardcoded if/else pass dispatch with a data-driven approach:
        // the graph declares which passes exist and their ordering, and we execute only enabled ones.
        //
        // NOTE: Geometry is uploaded by render_frame() via StagingBelt into staging_command_buffers.
        // Those staging commands must be submitted before the render pass encoders below, which is
        // guaranteed by inserting the render encoders after the existing staging entries (see submit block).

        let pass_nodes = kvasir::nodes::build_pass_sequence(has_glass, has_bloom, has_accessibility);

        // Execute each enabled pass in dependency order
        for node in &pass_nodes {
            if !node.enabled { continue; }
            match node.id {
                kvasir::nodes::PassId::Geometry => self.execute_pass_geometry(&mut encoder, &res.scene_texture, &res.depth_texture_view, res.scale_factor),
                kvasir::nodes::PassId::BackdropCopy => self.execute_pass_backdrop_copy(&mut encoder, &res.blur_tex_a, &res.scene_texture_bind_group),
                kvasir::nodes::PassId::BackdropBlur => self.execute_pass_backdrop_blur(&mut encoder, &res.blur_tex_a, self.current_width() / 2, self.current_height() / 2),
                kvasir::nodes::PassId::Glass => self.execute_pass_glass(&mut encoder, &res.scene_texture, &res.depth_texture_view, &res.blur_env_bind_group_a, res.scale_factor),
                kvasir::nodes::PassId::UI => self.execute_pass_ui(&mut encoder, &res.scene_texture, &res.depth_texture_view, res.scale_factor),
                kvasir::nodes::PassId::BloomExtract => self.execute_pass_bloom_extract(&mut post_encoder, &res.scene_texture, &res.scene_texture_bind_group, &res.bloom_tex_a),
                kvasir::nodes::PassId::BloomBlur => self.execute_pass_bloom_blur(&mut post_encoder, &res.bloom_tex_a, self.current_width() / 2, self.current_height() / 2),
                kvasir::nodes::PassId::Composite => self.execute_pass_composite(&mut post_encoder, &res.target_view, &res.scene_texture, &res.scene_texture_bind_group, &res.bloom_texture_a, &res.bloom_env_bind_group_a),
                kvasir::nodes::PassId::Accessibility => self.execute_pass_accessibility(&mut post_encoder, &res.target_view, &res.scene_texture, &res.scene_texture_bind_group),
                kvasir::nodes::PassId::Present => { /* swapchain present happens after submit */ }
            }
        }

        // ── Submit ─────────────────────────────────────────────────────────────
        // staging_command_buffers already contains the geometry upload encoder from
        // render_frame() (StagingBelt). The render pass encoders must come AFTER it
        // so the GPU sees vertex/index data before the draw calls that reference it.
        self.staging_command_buffers.push(encoder.finish());
        self.staging_command_buffers.push(post_encoder.finish());

        // Skuld: Resolve timestamps (preserved from original)
        if let (Some(q), Some(b), Some(rb)) = (
            &self.skuld_queries,
            &self.skuld_buffer,
            &self.skuld_read_buffer,
        ) {
            let mut resolve_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Skuld Resolve Encoder") });
            resolve_encoder.resolve_query_set(q, 0..2, b, 0);
            resolve_encoder.copy_buffer_to_buffer(b, 0, rb, 0, 16);
            self.staging_command_buffers.push(resolve_encoder.finish());
        }

        let cmds = std::mem::take(&mut self.staging_command_buffers);
        self.queue.submit(cmds);
        self.telemetry.frame_time_ms = self.last_frame_start.elapsed().as_secs_f32() * 1000.0;
        self.update_vram_telemetry();

        if let Some(f) = res.surface_texture {
            f.present();
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

impl SurtrRenderer {
    /// Submit pre-routed draw command buckets from the cvkg-compositor.
    ///
    /// Accepts `CommandBuckets` produced by `CompositorEngine::flatten_and_route()`
    /// and submits draw calls in the correct pass order for the Backdrop Capture
    /// Architecture:
    /// 1. Scene commands (opaque) → Scene Capture pass
    /// 2. Glass commands → Material Composite pass (samples blur pyramid)
    /// 3. Overlay commands → Top-Level Foreground pass
    pub fn submit_buckets(&mut self, buckets: &cvkg_compositor::CommandBuckets) {
        // Scene pass — opaque draw calls
        for routed in &buckets.scene_commands {
            self.set_material(cvkg_core::DrawMaterial::Opaque);
            self.submit_routed(routed);
        }

        // Glass pass — glassmorphism draw calls sampling blur pyramid
        for routed in &buckets.glass_commands {
            let core_material = match routed.material {
                cvkg_compositor::Material::Opaque => cvkg_core::DrawMaterial::Opaque,
                cvkg_compositor::Material::Glass {
                    blur_radius,
                    depth_index: _,
                } => cvkg_core::DrawMaterial::Glass { blur_radius },
                cvkg_compositor::Material::Overlay => cvkg_core::DrawMaterial::TopUI,
                _ => cvkg_core::DrawMaterial::Opaque,
            };
            self.set_material(core_material);
            self.submit_routed(routed);
        }

        // Overlay pass — foreground UI (crisp text, icons, edge lighting)
        for routed in &buckets.overlay_commands {
            self.set_material(cvkg_core::DrawMaterial::TopUI);
            self.submit_routed(routed);
        }
    }

    /// Submit a single routed draw command through the internal pipeline.
    pub(crate) fn submit_routed(&mut self, routed: &cvkg_compositor::RoutedDrawCommand) {
        let cmd = &routed.command;
        let current_tail = self.indices.len() as u32;
        let index_count = current_tail - self.compositor_index_cursor;
        if index_count == 0 { return; }
        let material = match routed.material {
            cvkg_compositor::Material::Glass { blur_radius, .. } => cvkg_core::DrawMaterial::Glass { blur_radius },
            cvkg_compositor::Material::Overlay => cvkg_core::DrawMaterial::TopUI,
            _ => cvkg_core::DrawMaterial::Opaque,
        };
        self.draw_calls.push(DrawCall {
            texture_id: cmd.texture_id,
            scissor_rect: cmd.scissor_rect,
            index_start: self.compositor_index_cursor,
            index_count,
            material,
        });
        self.compositor_index_cursor = current_tail;
    }
}

impl SurtrRenderer {
    /// Returns the current effective opacity (product of all stacked values).
    pub(crate) fn apply_opacity(&self, mut color: [f32; 4]) -> [f32; 4] {
        if let Some(&alpha) = self.opacity_stack.last() {
            color[3] *= alpha;
        }
        color
    }

    /// load_svg — Parses an SVG file and tessellates its paths into GPU triangles.
    pub fn load_svg(&mut self, name: &str, data: &[u8]) {
        let opt = usvg::Options::default();
        let tree = match usvg::Tree::from_data(data, &opt) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to parse SVG '{}': {:?}, skipping load", name, e);
                return;
            }
        };

        let view_box = Rect {
            x: 0.0,
            y: 0.0,
            width: tree.size().width(),
            height: tree.size().height(),
        };

        let parsed_animations = parse_svg_animations(data);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();
        let mut finalized_animations = Vec::new();

        for child in tree.root().children() {
            self.tessellate_node(
                child,
                &mut fill_tessellator,
                &mut stroke_tessellator,
                &mut vertices,
                &mut indices,
                &parsed_animations,
                &mut finalized_animations,
            );
        }

        self.svg_cache.put(
            name.to_string(),
            SvgModel {
                vertices,
                indices,
                view_box,
                animations: finalized_animations,
            },
        );
        self.svg_trees.put(name.to_string(), tree);
    }

    pub(crate) fn tessellate_node(
        &self,
        node: &usvg::Node,
        fill_tessellator: &mut FillTessellator,
        stroke_tessellator: &mut StrokeTessellator,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        parsed_animations: &[SvgAnimation],
        finalized_animations: &mut Vec<SvgAnimation>,
    ) {
        let start_idx = vertices.len();
        let node_id = match node {
            usvg::Node::Group(g) => g.id().to_string(),
            usvg::Node::Path(p) => p.id().to_string(),
            _ => String::new(),
        };

        if let usvg::Node::Group(ref group) = *node {
            for child in group.children() {
                self.tessellate_node(
                    child,
                    fill_tessellator,
                    stroke_tessellator,
                    vertices,
                    indices,
                    parsed_animations,
                    finalized_animations,
                );
            }
        } else if let usvg::Node::Path(ref path) = *node {
            let has_fill = path.fill().is_some();
            let has_stroke = path.stroke().is_some();

            // If neither fill nor stroke, log and skip
            if !has_fill && !has_stroke {
                log::debug!("SVG path '{}' has no fill or stroke, skipping", node_id);
                return;
            }

            let lyon_path = usvg_to_lyon(path);
            let screen = [4096.0, 4096.0]; // Placeholder, will be overridden if needed
            let clip = [-10000.0, -10000.0, 20000.0, 20000.0]; // Default clip

            // Tessellate fill if present
            if has_fill {
                if let Some(fill) = path.fill() {
                    let color = match fill.paint() {
                        usvg::Paint::Color(c) => [
                            c.red as f32 / 255.0,
                            c.green as f32 / 255.0,
                            c.blue as f32 / 255.0,
                            fill.opacity().get(),
                        ],
                        usvg::Paint::LinearGradient(_) | usvg::Paint::RadialGradient(_) | usvg::Paint::Pattern(_) => {
                            log::warn!("SVG path '{}' uses gradient/pattern fill which is not supported, using white fallback", node_id);
                            [1.0, 1.0, 1.0, 1.0]
                        }
                    };

                    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
                    let base_index_idx = indices.len() as u32;

                    if let Err(e) = fill_tessellator.tessellate_path(
                        &lyon_path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(
                            &mut buffers,
                            SceneVertexConstructor {
                                color,
                                translation: [0.0, 0.0],
                                scale: [1.0, 1.0],
                                rotation: 0.0,
                            },
                        ),
                    ) {
                        log::warn!("SVG fill tessellation failed for path '{}': {:?}, skipping", node_id, e);
                        return;
                    }

                    vertices.extend(buffers.vertices);
                    for idx in buffers.indices {
                        indices.push(base_index_idx + idx);
                    }
                }
            }

            // Tessellate stroke if present
            if has_stroke {
                if let Some(stroke) = path.stroke() {
                    let stroke_index_idx = indices.len() as u32; // New base for stroke indices
                    let stroke_width = stroke.width().get(); // Direct float value
                    let color = match stroke.paint() {
                        usvg::Paint::Color(c) => [
                            c.red as f32 / 255.0,
                            c.green as f32 / 255.0,
                            c.blue as f32 / 255.0,
                            stroke.opacity().get(),
                        ],
                        usvg::Paint::LinearGradient(_) | usvg::Paint::RadialGradient(_) | usvg::Paint::Pattern(_) => {
                            log::warn!("SVG path '{}' uses gradient/pattern stroke which is not supported, using white fallback", node_id);
                            [1.0, 1.0, 1.0, 1.0]
                        }
                    };

                    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();

                    if let Err(e) = stroke_tessellator.tessellate_path(
                        &lyon_path,
                        &StrokeOptions::default().with_line_width(stroke_width),
                        &mut BuffersBuilder::new(
                            &mut buffers,
                            CustomStrokeVertexConstructor {
                                color,
                                translation: [0.0, 0.0],
                                scale: [1.0, 1.0],
                                rotation: 0.0,
                                screen,
                                clip,
                            },
                        ),
                    ) {
                        log::warn!("SVG stroke tessellation failed for path '{}': {:?}, skipping", node_id, e);
                        return;
                    }

                    vertices.extend(buffers.vertices);
                    for idx in buffers.indices {
                        indices.push(stroke_index_idx + idx);
                    }
                }
            }
        }

        let end_idx = vertices.len();
        if !node_id.is_empty() && start_idx < end_idx {
            for anim in parsed_animations {
                if anim.target_id == node_id {
                    let mut final_anim = anim.clone();
                    final_anim.vertex_range = start_idx..end_idx;
                    finalized_animations.push(final_anim);
                }
            }
        }
    }

    /// draw_svg — Renders a pre-loaded SVG icon at the specified logical rect.
    pub fn draw_svg(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, material_id: u32) {
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

        let mut local_vertices = model.vertices.clone();
        for anim in &model.animations {
            let t = (self.current_scene.time % anim.duration) / anim.duration;
            let val = anim.from_val + (anim.to_val - anim.from_val) * t;

            if anim.attribute_name == "transform" {
                // assume rotation
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for i in anim.vertex_range.clone() {
                    let p = local_vertices[i].position;
                    if p[0] < min_x {
                        min_x = p[0];
                    }
                    if p[1] < min_y {
                        min_y = p[1];
                    }
                    if p[0] > max_x {
                        max_x = p[0];
                    }
                    if p[1] > max_y {
                        max_y = p[1];
                    }
                }
                let cx = (min_x + max_x) * 0.5;
                let cy = (min_y + max_y) * 0.5;

                let c = val.to_radians().cos();
                let s = val.to_radians().sin();

                for i in anim.vertex_range.clone() {
                    let p = local_vertices[i].position;
                    let dx = p[0] - cx;
                    let dy = p[1] - cy;
                    local_vertices[i].position[0] = cx + dx * c - dy * s;
                    local_vertices[i].position[1] = cy + dx * s + dy * c;
                }
            } else if anim.attribute_name == "opacity" {
                for i in anim.vertex_range.clone() {
                    local_vertices[i].color[3] = val;
                }
            }
        }

        for mut v in local_vertices {
            let rel_x = (v.position[0] - model.view_box.x) / model.view_box.width;
            let rel_y = (v.position[1] - model.view_box.y) / model.view_box.height;

            v.position[0] = snap(rect.x + rel_x * rect.width);
            v.position[1] = snap(rect.y + rel_y * rect.height);
            v.position[2] = self.current_z;
            v.logical = [v.position[0], v.position[1]];
            v.screen = screen;
            v.clip = clip;
            v.material_id = material_id;

            if let Some(override_color) = color {
                let mut c = override_color;
                c[3] *= v.color[3]; // preserve animated opacity
                v.color = self.apply_opacity(c);
            } else {
                v.color = self.apply_opacity(v.color);
            }
            self.vertices.push(v);
        }

        for idx in &model.indices {
            self.indices.push(base_idx + *idx);
        }

        let material = match material_id {
            7 => cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 },
            0 => cvkg_core::DrawMaterial::Opaque,
            _ => cvkg_core::DrawMaterial::TopUI,
        };
        let tid = self.get_texture_id("__mega_atlas");

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != tid
            || last_call.unwrap().scissor_rect != self.clip_stack.last().copied()
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = tid;
            self.draw_calls.push(DrawCall {
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() - model.indices.len()) as u32,
                index_count: 0,
                material,
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
            println!(
                "[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower..."
            );
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
        let info = adapter.get_info();
        println!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name, info.device_type, info.backend
        );
        println!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let required_features = adapter.features()
            & (wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Headless Forge"),
                required_features,
                required_limits: wgpu::Limits {
                    max_bindings_per_bind_group: adapter
                        .limits()
                        .max_bindings_per_bind_group
                        .min(256),
                    max_binding_array_elements_per_shader_stage: adapter
                        .limits()
                        .max_binding_array_elements_per_shader_stage
                        .min(256),
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);

        device.on_uncaptured_error(Arc::new(|error| {
            log::error!(
                "[GPU] Uncaptured device error (Device Lost or Panic): {:?}",
                error
            );
        }));

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            None,
            Some((width, height, wgpu::TextureFormat::Rgba8UnormSrgb)),
        )
        .await
    }

    /// capture_frame — Read back the rendered frame as a byte buffer (RGBA8).
    pub async fn capture_frame(&self) -> Result<Vec<u8>, String> {
        let ctx = self
            .headless_context
            .as_ref()
            .ok_or("Headless context required for capture")?;
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                width,
                height,
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

            println!("Capture frame: data len={}, first 4 bytes={:?}", data.len(), &data[0..4.min(data.len())]);

            drop(data);
            output_buffer.unmap();
            Ok(result)
        } else {
            Err("Failed to capture frame".to_string())
        }
    }

    pub(crate) fn current_width(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).map(|s| s.config.width).unwrap_or(1)
        } else {
            self.headless_context.as_ref().map(|h| h.width).unwrap_or(1)
        }
    }

    pub(crate) fn current_height(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).map(|s| s.config.height).unwrap_or(1)
        } else {
            self.headless_context.as_ref().map(|h| h.height).unwrap_or(1)
        }
    }

    pub(crate) fn current_scale_factor(&self) -> f32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).map(|s| s.scale_factor).unwrap_or(1.0)
        } else {
            self.headless_context.as_ref().map(|h| h.scale_factor).unwrap_or(1.0)
        }
    }

    /// Find a filter by ID in the SVG tree's filter list.
    pub(crate) fn find_filter<'a>(tree: &'a usvg::Tree, filter_id: &str) -> Option<&'a usvg::filter::Filter> {
        tree.filters()
            .iter()
            .find(|f| f.id() == filter_id)
            .map(|arc| arc.as_ref())
    }
}
