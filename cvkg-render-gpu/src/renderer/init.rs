use crate::heim::SkylinePacker;
use crate::renderer::context_helpers::{
    compute_mip_levels, create_headless_context, create_surface_context,
    load_pipeline_cache_with_integrity_check,
};
use crate::renderer::pipelines::compile_render_pipelines;
use crate::renderer::{GpuRenderer, QualityLevel};
use crate::types::{
    GpuParticle, HeadlessContext, ParticleUniforms, SurfaceContext,
};
use crate::{
    WGSL_BIFROST, WGSL_BLOOM, WGSL_COLOR_BLIND, WGSL_COMMON, WGSL_MATERIAL_GLASS,
    WGSL_MATERIAL_OPAQUE, WGSL_MATERIAL_PBR, WGSL_MATERIAL_SHADOW, WGSL_SHAPES,
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
        let _ = env_logger::try_init();
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

        tracing::info!("[Surtr] Renderer backend: GpuRenderer (wgpu)");

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        tracing::info!("[GPU] Requesting HighPerformance adapter...");

        let mut adapter = None;

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(filter) = std::env::var("WGPU_ADAPTER_NAME") {
            let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
            tracing::info!("[GPU] Available adapters:");
            for a in &adapters {
                let info = a.get_info();
                tracing::info!(
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
                    tracing::info!(
                        "[GPU] Manual selection match: {} | Driver: {}",
                        info.name,
                        info.driver
                    );
                }
                match_found
            });

            if adapter.is_some() {
                tracing::info!(
                    "[GPU] Forced adapter selection via WGPU_ADAPTER_NAME='{}'",
                    filter
                );
            } else {
                tracing::warn!(
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
            tracing::warn!(
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
            tracing::warn!("[GPU] Hardware adapters failed, trying Software fallback...");
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
        tracing::info!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?} -- detected as {}",
            info.name,
            info.device_type,
            info.backend,
            caps.vendor
        );
        tracing::info!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
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
            tracing::info!("[GPU] Validation layer enabled (debug build)");
        }

        let tier = crate::renderer::GpuCapabilityTier::detect(&adapter);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Forge"),
                required_features,
                required_limits: tier.required_limits.clone(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| {
                tracing::error!("[GPU] Failed to create device with tier {:?}: {e}", tier);
                e
            })
            .expect("Failed to create Surtr device even after capability detection");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);

        device.on_uncaptured_error(Arc::new(|error| {
            tracing::error!(
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
        // Surface format selection. HDR (Rgba16Float) is OPT-IN via the config's
        // `prefer_hdr` flag (default false): an HDR float swapchain needs OS-level
        // HDR display configuration, and on a machine without it wgpu presents with
        // no validation error but the whole window shows wrong/shifted colors.
        // Forge time has no user config yet, so default to LDR (prefer_hdr = false);
        // the config-driven choice is applied in register_window/resize.
        let surface_format = Self::select_best_surface_format(&surface_caps.formats, false);

        tracing::info!(
            "[GPU] Available present modes: {:?}",
            surface_caps.present_modes
        );
        tracing::info!(
            "[GPU] Adapter: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            tracing::info!("[GPU] Selected: Immediate (no vsync, uncapped)");
            wgpu::PresentMode::Immediate
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            tracing::info!("[GPU] Selected: Mailbox (no vsync)");
            wgpu::PresentMode::Mailbox
        } else {
            tracing::info!("[GPU] Selected: Fifo (V-Sync capped at compositor rate)");
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

        tracing::info!(
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
        tracing::info!("[GPU] Surface configuration successful.");

        let renderer = Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            tier,
            Some((window, surface, config)),
            None,
        )
        .await;
        tracing::info!("[GPU] Forge internal complete.");
        renderer
    }

    /// Internal rendering pipeline constructor.
    pub(crate) async fn forge_internal(
        instance: Arc<wgpu::Instance>,
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        tier: crate::renderer::GpuCapabilityTier,
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
                    tracing::warn!(
                        "[GPU] pipeline cache integrity check failed: {reason}; using empty cache"
                    );
                    None
                }
            };
            // SAFETY: create_pipeline_cache is marked unsafe in wgpu but only reads the
            // data slice and does not require lifetime guarantees beyond the descriptor.
            // The cache_data is a Vec<u8> that lives for the duration of this call.
            Some(unsafe {
                device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: Some("CVKG Pipeline Cache"),
                    data: cache_data.as_deref(),
                    fallback: true,
                })
            })
        } else {
            tracing::debug!(
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
        let wgsl_pbr = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_MATERIAL_PBR,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );
        let wgsl_shadow = format!(
            "{}{}{}",
            WGSL_COMMON, WGSL_MATERIAL_SHADOW, materials_generated
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Main Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_src)),
        });

        #[cfg(target_arch = "wasm32")]
        let texture_array_count: Option<std::num::NonZeroU32> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let texture_array_count: Option<std::num::NonZeroU32> =
            std::num::NonZeroU32::new(tier.texture_array_count);

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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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
                    // Folded GI entries (folded into group 3 to respect WebGPU's
                    // max_bind_groups=4 limit). common.wgsl declares these at
                    // @group(3) bindings 2,3.
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Surtr Gradient Bind Group Layout"),
            });

        // Create GI bind group layout. Two entries:
//   binding 0: GiHeader uniform buffer (volume params, ≤64 bytes)
//   binding 1: read-only storage buffer for the 4096 SH probe coefficients.
// We split them because the probe grid is 4096 * (4 * vec3<f32>) = 256 KB,
// which exceeds the default 64 KB uniform-buffer limit in wgpu/WebGPU.
// Every real GI implementation uses a storage buffer for irradiance volumes.
let gi_bind_group_layout =
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
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("Surtr GI Bind Group Layout"),
    });

        let pbr_material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Binding 0: Shadow Map Texture Atlas (single 2D depth)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    // Binding 1: Shadow Sampler (comparison)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                    // Binding 8: IBL Texture (standard 2D)
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Binding 9: IBL Sampler (filtering)
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Binding 6: Normal Map Texture (standard 2D)
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Binding 7: Normal Map Sampler (filtering)
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Folded GI entries (folded into group 3 to respect WebGPU's
                    // max_bind_groups=4 limit). common.wgsl declares these at
                    // @group(3) bindings 2,3.
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Surtr PBR Material Bind Group Layout"),
            });

        let msaa_sample_count = QualityLevel::default().msaa_sample_count();
        tracing::info!(
            "[GPU] Forge internal: quality_level={:?}, msaa_sample_count={}, surface_size={}x{}",
            QualityLevel::default(),
            msaa_sample_count,
            surface_info.as_ref().map(|(_, _, c)| c.width).unwrap_or(0),
            surface_info.as_ref().map(|(_, _, c)| c.height).unwrap_or(0),
        );
        let pipes = compile_render_pipelines(
            &device,
            &tier,
            format,
            pipeline_cache.as_ref(),
            &texture_bind_group_layout,
            &env_bind_group_layout,
            &berserker_bind_group_layout,
            &gradient_bind_group_layout,
            &pbr_material_bind_group_layout,
            &gi_bind_group_layout,
            &shader,
            wgsl_opaque.as_str(),
            wgsl_glass.as_str(),
            wgsl_pbr.as_str(),
            wgsl_shadow.as_str(),
            &queue,
            msaa_sample_count,
        );

        // Forge the Mega-Heim (size from detected GPU tier; 4096x4096 RGBA on full-tier,
        // 2048x2048 on reduced-tier integrated GPUs)
        let mega_heim_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Mega-Heim"),
            size: wgpu::Extent3d {
                width: tier.mega_heim_size,
                height: tier.mega_heim_size,
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
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gradient_dummy_view =
            gradient_dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // gradient_dummy_texture, gradient_dummy_view, gradient_sampler are
        // declared above (line ~753). The actual gradient_bind_group creation is
        // deferred until after gi_header_buffer / gi_probe_buffer exist, since
        // the GI bindings (2, 3) were folded into gradient_bind_group_layout.
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
            (0..tier.texture_array_count).map(|_| dummy_view.clone()).collect();
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

        let dummy_views_refs: Vec<&wgpu::TextureView> = (0..tier.texture_array_count).map(|_| &dummy_view).collect();
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

        let shadow_map_size = 1024;
        let shadow_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr CSM Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: shadow_map_size * 2, // 2x2 grid double resolution
                height: shadow_map_size * 2,
                depth_or_array_layers: 1, // Single 2D texture atlas
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_map_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Surtr CSM Shadow Map View"),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..wgpu::TextureViewDescriptor::default()
        });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Surtr CSM Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..wgpu::SamplerDescriptor::default()
        });

        let mut texture_registry =
            LruCache::new(NonZeroUsize::new(tier.texture_registry_capacity).unwrap());
        let mut texture_bind_groups = Vec::new();

        // Index 0 is permanently reserved for the Mega-Heim atlas. Loaded images start at 1.
        texture_registry.put("__mega_heim".to_string(), 0);
        texture_bind_groups.push(mega_heim_bind_group.clone());

        let geometry_buffers =
            crate::types::GeometryBuffers::forge(&device, tier.max_vertices, tier.max_indices);

        let instance_buffer_3d = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr 3D Instance Buffer"),
            size: (tier.max_vertices / 4 * std::mem::size_of::<crate::vertex::InstanceData3D>())
                as u64,
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

        let csm_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr CSM Buffer"),
            contents: bytemuck::bytes_of(&cvkg_core::render_tier::CsmUniforms::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Forge the GI binding resources. We split GI into two buffers:
        //   1) gi_header_buffer — small uniform (32 B), holds volume params.
        //   2) gi_probe_buffer — read-only storage buffer holding the
        //      4096-probe SH coefficient grid (192 KB).
        // The CPU-side `GiUniforms` is still used by the GI node for
        // bookkeeping; we just split the backing storage so the
        // 196 KB buffer does NOT exceed wgpu's 64 KB uniform-buffer limit
        // (uniform buffers exceed that limit silently drop the layout).
        let gi_defaults = cvkg_core::GiUniforms::default();
        // Header layout (must match GiHeader in common.wgsl/deferred_lighting.wgsl).
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct GiHeaderGpu {
            volume_origin: [f32; 3],
            _pad0: f32,
            volume_spacing: [f32; 3],
            _pad1: f32,
            probe_dimensions: [u32; 3],
            _pad2: u32,
        }
        let gi_header_init = GiHeaderGpu {
            volume_origin: gi_defaults.volume_origin,
            _pad0: gi_defaults._pad0,
            volume_spacing: gi_defaults.volume_spacing,
            _pad1: gi_defaults._pad1,
            probe_dimensions: gi_defaults.probe_dimensions,
            _pad2: gi_defaults._pad2,
        };

        let gi_header_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr GI Header Buffer"),
            contents: bytemuck::bytes_of(&gi_header_init),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Probe coefficients: tier.gi_probe_count * (4 * 3) floats. Lives in a
        // storage buffer (read-only). mn is `manifest_n` addressing;
        // shader declares `array<array<vec3<f32>, 4>>` (runtime-sized), so
        // reducing the buffer size on the reduced tier just gives the GPU
        // fewer probes to read -- no WGSL changes needed.
        let probe_byte_len = (tier.gi_probe_count * 12 * std::mem::size_of::<f32>()) as u64;
        let probe_init_len = tier
            .gi_probe_count
            .min(gi_defaults.probe_coefficients.len());
        let gi_probe_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr GI Probe Buffer"),
            contents: bytemuck::cast_slice(
                &gi_defaults.probe_coefficients[..probe_init_len],
            ),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        tracing::info!(
            "[GPU] gi_probe_buffer created: {} bytes ({} probes, tier.gi_probe_count={})",
            probe_byte_len,
            tier.gi_probe_count,
            tier.gi_probe_count,
        );

        // gradient_bind_group is built here (after gi_header_buffer/gi_probe_buffer
        // exist), because GI bindings (2, 3) were folded into gradient_bind_group_layout
        // to respect WebGPU's max_bind_groups=4 limit. common.wgsl declares the
        // gi uniform + gi_probes storage at @group(3) bindings 2 and 3.
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gi_header_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gi_probe_buffer.as_entire_binding(),
                },
            ],
            label: Some("Gradient Dummy Bind Group"),
        });

        // A standalone GI bind group is still required for the deferred lighting
        // pass (deferred_layout puts GI at group(2), using gi_bind_group_layout
        // declared at index 2). The 2D pipeline layouts no longer reference
        // gi_bind_group_layout directly — they use gradient_bind_group_layout
        // for group 3 with GI folded in.
        let gi_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gi_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gi_header_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gi_probe_buffer.as_entire_binding(),
                },
            ],
            label: Some("Surtr GI Bind Group"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: csm_buffer.as_entire_binding(),
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
                tier.texture_array_count,
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
                tier.texture_array_count,
                &mut registry,
                msaa_sample_count,
            ));
        }

        let staging_belt = wgpu::util::StagingBelt::new((*device).clone(), 1024 * 1024);

        let glass_output_bind_group_layout = env_bind_group_layout.clone();

        // Skinning buffers are now managed per-mesh in skinning_buffer_pairs (Vec<(Buffer, Buffer)>)
        // No persistent src/dst buffers needed — each submit_mesh_3d creates its own pair.

        let skinning_joint_matrices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinning Joint Matrices Buffer"),
            size: (256 * std::mem::size_of::<glam::Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let skinning_morph_positions = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinning Morph Positions Buffer"),
            size: (65536 * 2 * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let skinning_morph_weights = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinning Morph Weights Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            skinning_compute_pipeline: pipes.skinning_compute_pipeline.clone(),
            skinning_bgl0: pipes.skinning_bgl0.clone(),
            skinning_bgl1: pipes.skinning_bgl1.clone(),
            skinning_bgl2: pipes.skinning_bgl2.clone(),
            skinning_buffer_pairs: Vec::new(),
            skinning_joint_matrices,
            skinning_morph_positions,
            skinning_morph_weights,
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
            pbr_pipeline: pipes.pbr_pipeline,
            transparent_pipeline: pipes.transparent_pipeline,
            shadow_pipeline: pipes.shadow_pipeline,
            gbuffer_pipeline: pipes.gbuffer_pipeline,
            deferred_lighting_pipeline: pipes.deferred_lighting_pipeline,
            ssao_pipeline: pipes.ssao_pipeline,
            bloom_extract_pipeline: pipes.bloom_extract_pipeline,
            copy_pipeline: pipes.copy_pipeline,
            composite_pipeline: pipes.composite_pipeline,
            env_bind_group_layout,
            mega_heim_tex,
            mega_heim_bind_group,
            config: crate::subsystems::RendererConfig::default(),
            capability_tier: tier.clone(),
            text: crate::types::TextSubsystem::forge(NonZeroUsize::new(8192).unwrap()),
            heim_packer: SkylinePacker::new(tier.mega_heim_size, tier.mega_heim_size),
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
            text_sampler,
            dummy_view: dummy_view.clone(),
            dummy_depth_view: dummy_depth_view.clone(),
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
            vertices: Vec::with_capacity(tier.max_vertices),
            indices: Vec::with_capacity(tier.max_indices),
            instance_data: Vec::with_capacity(tier.max_vertices / 4),
            instance_data_3d: Vec::with_capacity(tier.max_vertices / 4),
            instance_buffer_3d: Some(instance_buffer_3d),
            draw_calls: Vec::new(),
            current_texture_id: None,
            current_panel_id: None,
            panel_stack: Vec::new(),
            panel_vdoms: HashMap::new(),
            world_space_panels: Vec::new(),
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
            transform_stack_3d: Vec::new(),
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
            // Volumetric raymarching is opt-in; it is only meaningful when a
            // hologram rect is registered (see api::register_hologram), which
            // sets this flag true. Default false so a fresh renderer does not
            // wipe the scene with a placeholder fullscreen pass.
            volumetric_enabled: false,
            deferred_enabled: true,
            frame_counter: 0,
            path_geometry_cache: lru::LruCache::new(NonZeroUsize::new(64).unwrap()),
            color_blind_mode: crate::color_blindness::ColorBlindMode::Normal,
            color_blind_intensity: 1.0,
            color_blind_pipeline: pipes.color_blind_pipeline,
            volumetric_pipeline: pipes.volumetric_pipeline,
            volumetric_bind_group_layout: pipes.volumetric_bind_group_layout,
            volumetric_uniform_buffer: pipes.volumetric_uniform_buffer,
            csm_buffer,
            pbr_material_bind_group_layout,
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

            // Deferred rendering bind groups
            // Create bind groups before assigning layouts
            deferred_bind_group: {
                // For initial state, bind empty/default textures
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Deferred G-Buffer Bind Group"),
                    layout: &pipes.deferred_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&dummy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&dummy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(&dummy_depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::TextureView(&dummy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                    ],
                })
            },
            ssao_bind_group: {
                // Create SSAO bind group with depth/normal textures
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("SSAO Bind Group"),
                    layout: &pipes.ssao_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&dummy_depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&dummy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor::default())),
                        },
                    ],
                })
            },
            deferred_bgl: pipes.deferred_bgl,
            ssao_bgl: pipes.ssao_bgl,

            // GI mapping resources (header uniform + probe storage)
            gi_header_buffer,
            gi_probe_buffer,
            gi_bind_group,
            gi_bind_group_layout,

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

            // Shadow map resources
            shadow_map_texture: Some(shadow_map_texture),
            shadow_map_view: Some(shadow_map_view),
            shadow_sampler: Some(shadow_sampler),
            shadow_light_vp: glam::Mat4::IDENTITY,
            shadow_map_size: 1024,
            shadow_bias: 0.005,

            // 3D mesh staging
            pending_directional_light: None,
            pending_mesh_instances_3d: Vec::new(),
            pending_transparent_instances_3d: Vec::new(),
            pending_scene_radius: 100.0,

            theme_stack: Vec::new(),
            portal_theme_stack: Vec::new(),
        }
    }
}
