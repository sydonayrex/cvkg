use crate::heim::SkylinePacker;
use crate::renderer::context_helpers::{
    compute_mip_levels, create_headless_context, create_surface_context,
    load_pipeline_cache_with_integrity_check,
};
use crate::renderer::pipelines::compile_render_pipelines;
use crate::renderer::{GpuRenderer, QualityLevel};
use crate::types::{
    GpuParticle, HeadlessContext, MAX_INDICES, MAX_PARTICLES, MAX_VERTICES, ParticleUniforms,
    SurfaceContext,
};
use crate::{
    WGSL_BIFROST, WGSL_BLOOM, WGSL_COLOR_BLIND, WGSL_COMMON, WGSL_MATERIAL_GLASS,
    WGSL_MATERIAL_OPAQUE, WGSL_SHAPES,
};
use cvkg_core::{ColorTheme, Rect, SceneUniforms};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

impl GpuRenderer {
    /// forge -- Initializes the Surtr GPU renderer from a winit window.
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

        log::info!("[Surtr] Renderer backend: GpuRenderer (wgpu)");

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        log::info!("[GPU] Requesting HighPerformance adapter...");

        let mut adapter = None;

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(filter) = std::env::var("WGPU_ADAPTER_NAME") {
            let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
            log::info!("[GPU] Available adapters:");
            for a in &adapters {
                let info = a.get_info();
                log::info!(
                    "  - Name: '{}' | Driver: '{}' | Backend: {:?}",
                    info.name,
                    info.driver,
                    info.backend
                );
            }

            adapter = adapters.into_iter().find(|a| {
                let info = a.get_info();
                let match_found = info.name.to_lowercase().contains(&filter.to_lowercase())
                    || info.driver.to_lowercase().contains(&filter.to_lowercase());
                if match_found {
                    log::info!(
                        "[GPU] Manual selection match: {} | Driver: {}",
                        info.name,
                        info.driver
                    );
                }
                match_found
            });

            if adapter.is_some() {
                log::info!(
                    "[GPU] Forced adapter selection via WGPU_ADAPTER_NAME='{}'",
                    filter
                );
            } else {
                log::warn!(
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
            log::warn!(
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
            log::warn!("[GPU] Hardware adapters failed, trying Software fallback...");
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
        // P1-26: detect GPU vendor for logging and future
        // capability-based shader selection.
        let caps =
            crate::subsystems::GpuCapabilities::detect(&info.name, format!("{:?}", info.backend));
        log::info!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?} -- detected as {}",
            info.name,
            info.device_type,
            info.backend,
            caps.vendor
        );
        log::info!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let supports_pipeline_cache = adapter.features().contains(wgpu::Features::PIPELINE_CACHE);
        #[cfg(not(target_arch = "wasm32"))]
        let mut required_features =
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY;

        #[cfg(target_arch = "wasm32")]
        let mut required_features = wgpu::Features::empty(); // Fallbacks for WebGL
        if supports_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
        }
        if supports_pipeline_cache {
            required_features |= wgpu::Features::PIPELINE_CACHE;
        }
        // Enable validation layer in debug builds for better error reporting
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        {
            log::info!("[GPU] Validation layer enabled (debug build)");
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
        }));

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
        // Ensure we have valid dimensions - Wayland may return 0 for not-yet-committed surfaces
        let width = if size.width > 0 { size.width } else { 1280 };
        let height = if size.height > 0 { size.height } else { 720 };
        let surface_caps = surface.get_capabilities(&adapter);
        // HDR/Display P3 surface format selection:
        // WHY: Tahoe requires wide-gamut Display P3 or HDR (Rgba16Float) color spaces when available.
        // CONTRACT: Uses select_best_surface_format to safely fall back on mobile/legacy GPUs.
        let surface_format = Self::select_best_surface_format(&surface_caps.formats);

        log::info!(
            "[GPU] Available present modes: {:?}",
            surface_caps.present_modes
        );
        log::info!(
            "[GPU] Adapter: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            log::info!("[GPU] Selected: Immediate (no vsync, uncapped)");
            wgpu::PresentMode::Immediate
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            log::info!("[GPU] Selected: Mailbox (no vsync)");
            wgpu::PresentMode::Mailbox
        } else {
            log::info!("[GPU] Selected: Fifo (V-Sync capped at compositor rate)");
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
            desired_maximum_frame_latency: 1,
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

        let pipeline_cache = if device.features().contains(wgpu::Features::PIPELINE_CACHE) {
            let cache_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("pipeline_cache")))
                .unwrap_or_else(|| std::env::temp_dir().join("cvkg_pipeline_cache"));
            let _ = std::fs::create_dir_all(&cache_dir);
            let cache_path = cache_dir.join("cvkg_render_gpu.bin");
            let cache_data = match load_pipeline_cache_with_integrity_check(&cache_path) {
                Ok(data) => data,
                Err(reason) => {
                    log::warn!(
                        "[GPU] pipeline cache integrity check failed: {reason}; using empty cache"
                    );
                    None
                }
            };
            Some(unsafe {
                device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: Some("CVKG Pipeline Cache"),
                    data: cache_data.as_deref(),
                    fallback: true,
                })
            })
        } else {
            log::debug!(
                "[GPU] device does not expose PIPELINE_CACHE; compiling pipelines without cache"
            );
            None
        };
        let materials_generated = crate::material::generate_builtins_wgsl();

        let wgsl_src = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_SHAPES,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );
        let wgsl_opaque = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_MATERIAL_OPAQUE,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );
        let wgsl_glass = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_MATERIAL_GLASS,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Main Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_src)),
        });

        #[cfg(target_arch = "wasm32")]
        let texture_array_count: Option<std::num::NonZeroU32> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let texture_array_count: Option<std::num::NonZeroU32> = std::num::NonZeroU32::new(32);

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
                        count: texture_array_count,
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

        let gradient_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("Surtr Gradient Bind Group Layout"),
            });

        let pipes = compile_render_pipelines(
            &device,
            format,
            pipeline_cache.as_ref(),
            &texture_bind_group_layout,
            &env_bind_group_layout,
            &berserker_bind_group_layout,
            &gradient_bind_group_layout,
            &shader,
            wgsl_opaque.as_str(),
            wgsl_glass.as_str(),
            &queue,
        );

        // Forge the Mega-Heim (4096x4096 RGBA for production batching)
        let mega_heim_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Mega-Heim"),
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
        let mega_heim_view_obj = mega_heim_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
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

        // Non-filtering sampler required by the gradient bind group layout.
        let gradient_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Gradient bind group: requires non-filterable texture + non-filtering sampler.
        // The gradient layout expects Float { filterable: false } texture.
        let gradient_dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gradient Dummy Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gradient_dummy_view = gradient_dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gradient_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gradient_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gradient_dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gradient_sampler),
                },
            ],
            label: Some("Gradient Dummy Bind Group"),
        });
        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Non-filtering sampler required by the gradient bind group layout.
        let gradient_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let mut texture_views_list: Vec<wgpu::TextureView> =
            (0..32).map(|_| dummy_view.clone()).collect();
        texture_views_list[0] = mega_heim_view_obj.clone();

        let views_refs: Vec<&wgpu::TextureView> = texture_views_list.iter().collect();
        let mega_heim_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            label: Some("Mega-Heim Bind Group"),
        });

        let dummy_views_refs: Vec<&wgpu::TextureView> = (0..32).map(|_| &dummy_view).collect();
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
        let dummy_depth_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Dummy Depth Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_depth_view = dummy_depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let dummy_depth_tex_msaa = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Dummy Depth Texture MSAA"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_depth_view_msaa =
            dummy_depth_tex_msaa.create_view(&wgpu::TextureViewDescriptor::default());

        let mut texture_registry = LruCache::new(NonZeroUsize::new(31).unwrap());
        let mut texture_bind_groups = Vec::new();

        // Index 0 is permanently reserved for the Mega-Heim atlas. Loaded images start at 1.
        texture_registry.put("__mega_heim".to_string(), 0);
        texture_bind_groups.push(mega_heim_bind_group.clone());

        let geometry_buffers =
            crate::types::GeometryBuffers::forge(&device, MAX_VERTICES, MAX_INDICES);

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
        let msaa_sample_count = QualityLevel::default().msaa_sample_count();
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

        let mut registry = crate::kvasir::registry::ResourceRegistry::new();
        let mut surfaces = std::collections::HashMap::new();
        let mut current_window = None;
        let mut headless_context = None;

        if let Some((window, surface, config)) = surface_info {
            let window_id = window.id();
            let ctx = create_surface_context(
                &device,
                surface,
                config,
                &env_bind_group_layout,
                &texture_bind_group_layout,
                scale_factor,
                msaa_sample_count,
                &mut registry,
            );
            surfaces.insert(window_id, ctx);
            current_window = Some(window_id);
        } else if let Some((w, h, f)) = headless_info {
            headless_context = Some(create_headless_context(
                &device,
                w,
                h,
                f,
                &env_bind_group_layout,
                &texture_bind_group_layout,
                &mut registry,
                msaa_sample_count,
            ));
        }

        let staging_belt = wgpu::util::StagingBelt::new((*device).clone(), 1024 * 1024);

        let glass_output_bind_group_layout = env_bind_group_layout.clone();

        Self {
            registry,
            ai_material_rx: None,
            active_offscreens: Vec::new(),
            effect_pipelines: std::collections::HashMap::new(),
            effect_params_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dummy Effect Buffer"),
                size: 256,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            effect_params_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Dummy Effect Bind Group"),
                layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[],
                }),
                entries: &[],
            }),
            linear_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Linear Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }),
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),

            surfaces,
            current_window,
            headless_context,
            pipeline: pipes.pipeline,
            opaque_pipeline: pipes.opaque_pipeline,
            ui_pipeline: pipes.ui_pipeline,
            glass_pipeline: pipes.glass_pipeline,
            bloom_extract_pipeline: pipes.bloom_extract_pipeline,
            copy_pipeline: pipes.copy_pipeline,
            composite_pipeline: pipes.composite_pipeline,
            env_bind_group_layout,
            mega_heim_tex,
            mega_heim_bind_group,
            config: crate::subsystems::RendererConfig::default(),
            text: crate::types::TextSubsystem::forge(NonZeroUsize::new(8192).unwrap()),
            heim_packer: SkylinePacker::new(4096, 4096),
            image_uv_registry: {
                let mut cache = LruCache::new(NonZeroUsize::new(256).unwrap());
                cache.put(
                    "__mega_heim".to_string(),
                    cvkg_core::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 1.0,
                        height: 1.0,
                    },
                );
                cache
            },
            texture_registry,
            texture_views: texture_views_list,
            dummy_sampler,
            dummy_depth_view,
            dummy_depth_view_msaa,
            svg: crate::types::SvgSubsystem::forge(
                &device,
                &queue,
                NonZeroUsize::new(512).unwrap(),
                NonZeroUsize::new(512).unwrap(),
            ),
            dummy_texture_bind_group,
            gradient_stop_texture: dummy_texture.clone(),
            gradient_stop_texture_view: dummy_view.clone(),
            gradient_bind_group,
            gradient_texture_cache: std::collections::HashMap::new(),
            gradient_stops_hash: 0,
            gradient_bind_group_layout,
            dummy_env_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
            shared_elements: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            geometry_buffers,
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
            background_pipeline: pipes.background_pipeline,
            current_z: 0.0,
            default_background_color: [0.02, 0.02, 0.05, 1.0],
            app_drew_background: false,
            frame_rendered: false,
            current_draw_order: 0,
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
            particle_compute_pipeline: pipes.particle_compute_pipeline,
            particle_compute_bgl: pipes.particle_compute_bgl,
            particle_buffer: pipes.particle_buffer,
            particle_uniform_buffer: pipes.particle_uniform_buffer,
            particles: crate::types::ParticleSubsystem::forge(),
            particle_render_pipeline: pipes.particle_render_pipeline,
            particle_render_bgl: pipes.particle_render_bgl,
            particle_render_bind_group: None,
            particle_compute_bind_group: None,
            vnode_stack: Vec::new(),
            event_handlers: std::collections::HashMap::new(),
            staging_belt,
            staging_command_buffers: Vec::new(),
            glass_output_bind_group_layout,
            current_draw_material: cvkg_core::DrawMaterial::Opaque,
            portal_regions: std::collections::VecDeque::new(),
            cached_graph_plan: None,
            material_compilation_hash: 0,
            memo_cache: std::collections::HashMap::new(),
            frame_generation: 0,
            quality_level: QualityLevel::default(),
            pipeline_cache,
            bloom_enabled: true,
            volumetric_enabled: false,
            path_geometry_cache: lru::LruCache::new(NonZeroUsize::new(64).unwrap()),
            color_blind_mode: crate::color_blindness::ColorBlindMode::Normal,
            color_blind_intensity: 1.0,
            color_blind_pipeline: pipes.color_blind_pipeline,
            volumetric_pipeline: pipes.volumetric_pipeline,
            volumetric_bind_group_layout: pipes.volumetric_bind_group_layout,
            volumetric_uniform_buffer: pipes.volumetric_uniform_buffer,
            volumetric_depth_sampler: pipes.volumetric_depth_sampler,
            hologram_instances: Vec::new(),
            color_blind_bind_group_layout: pipes.color_blind_bind_group_layout,
            color_blind_uniform_buffer: pipes.color_blind_uniform_buffer,
            sampler: pipes.sampler,
            kawase_down_pipeline: pipes.kawase_down_pipeline,
            kawase_up_pipeline: pipes.kawase_up_pipeline,
            kawase_bind_group_layout: pipes.kawase_bind_group_layout,
            kawase_uniform: pipes.kawase_uniform,
            kawase_uniform_buffers: pipes.kawase_uniform_buffers,
            bind_group_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            texture_view_cache: std::sync::Mutex::new(std::collections::HashMap::new()),

            // SVG Filter Engine Resources (initialized lazily on first use)
            blur_pipeline: None,
            blur_uniform: None,
            blur_bind_group_layout: None,
            blend_pipeline: None,
            blend_bind_group_layout: None,
            flood_pipeline: None,
            copy_bind_group_layout: None,

            // Error tracking
            render_error_count: 0,
            has_fatal_error: false,
        }
    }
}
