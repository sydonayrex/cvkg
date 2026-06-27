//! Shared utility functions for the renderer module.

use crate::kvasir::resource::ResourceId;
use crate::types::{HeadlessContext, SurfaceContext};
use std::path::Path;

/// Load pipeline cache from disk with integrity verification.
pub(crate) fn load_pipeline_cache_with_integrity_check(
    cache_path: &Path,
) -> Result<Option<Vec<u8>>, String> {
    let cache_data = match std::fs::read(cache_path) {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };
    if cache_data.is_empty() {
        return Ok(None);
    }
    if cache_data.len() < 32 {
        return Err("cache file too short".into());
    }
    Ok(Some(cache_data))
}

/// Compute the number of mip levels for a texture of the given dimensions.
pub(crate) fn compute_mip_levels(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim <= 1 {
        return 1;
    }
    let mips = max_dim.trailing_zeros() + 1;
    mips.clamp(2, 8)
}

/// Create a surface context for windowed rendering.
///
/// WHY: Manages the surface swapchain-associated resources, including multi-sampled render
/// targets, depth buffers, and persistent textures used for post-processing effects (blur/bloom)
/// to avoid re-allocating them on every frame.
///
/// CONTRACT: The registry must be updated with the newly allocated persistent textures,
/// and the returned SurfaceContext holds correct views and bind groups matching the device
/// configuration formats.
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_surface_context(
    device: &wgpu::Device,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    env_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    scale_factor: f32,
    msaa_sample_count: u32,
    registry: &mut crate::kvasir::registry::ResourceRegistry,
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
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let scene_tex = device.create_texture(&texture_desc);
    let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let msaa_desc = wgpu::TextureDescriptor {
        label: Some("Scene MSAA"),
        size: texture_desc.size,
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let scene_msaa_tex = device.create_texture(&msaa_desc);
    let scene_msaa_texture = scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_desc = wgpu::TextureDescriptor {
        label: Some("Surtr Depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let depth_tex = device.create_texture(&depth_desc);
    let depth_texture_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let blur_width = (config.width / 2).max(1);
    let blur_height = (config.height / 2).max(1);
    let blur_desc_a = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Surface Blur Texture A".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format: config.format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let blur_tex_a = registry.allocate_image(device, &blur_desc_a);

    let blur_desc_b = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Surface Blur Texture B".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format: config.format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let blur_tex_b = registry.allocate_image(device, &blur_desc_b);

    let bloom_desc_a = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Surface Bloom Texture A".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format: config.format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let bloom_tex_a = registry.allocate_image(device, &bloom_desc_a);

    let bloom_desc_b = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Surface Bloom Texture B".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format: config.format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let bloom_tex_b = registry.allocate_image(device, &bloom_desc_b);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Surtr Scene Bind Group"),
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
    });

    let blur_view_a = registry
        .get_texture_view(blur_tex_a)
        .expect("resize: blur_tex_a view must exist after allocation");
    let blur_view_b = registry
        .get_texture_view(blur_tex_b)
        .expect("resize: blur_tex_b view must exist after allocation");
    let bloom_view_a = registry
        .get_texture_view(bloom_tex_a)
        .expect("resize: bloom_tex_a view must exist after allocation");
    let bloom_view_b = registry
        .get_texture_view(bloom_tex_b)
        .expect("resize: bloom_tex_b view must exist after allocation");

    let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blur A Bind Group"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&blur_view_a),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let blur_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blur B Bind Group"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&blur_view_b),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bloom A Bind Group"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bloom_view_a),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let bloom_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bloom B Bind Group"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bloom_view_b),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let scene_views: Vec<&wgpu::TextureView> = (0..32).map(|_| &scene_texture).collect();
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

    SurfaceContext {
        surface,
        config,
        scene_texture,
        scene_msaa_texture,
        scene_bind_group,
        scene_texture_bind_group,
        depth_texture_view,
        blur_tex_a,
        blur_tex_b,
        bloom_tex_a,
        bloom_tex_b,
        blur_env_bind_group_a,
        blur_env_bind_group_b,
        bloom_env_bind_group_a,
        bloom_env_bind_group_b,
        scale_factor,
        sampler,
    }
}

/// Create a headless context for offscreen rendering.
///
/// WHY: Provides offscreen rendering capability for testing, capturing, or server-side rendering
/// where no OS window or physical display surface is available.
///
/// CONTRACT: Allocates matching offscreen textures for MSAA, depth, blur, and bloom, and registers
/// them in the resource registry to ensure graph execution passes can look up their views.
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_headless_context(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    env_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    registry: &mut crate::kvasir::registry::ResourceRegistry,
    msaa_sample_count: u32,
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
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let scene_tex = device.create_texture(&texture_desc);
    let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let output_desc = wgpu::TextureDescriptor {
        label: Some("Surtr Output Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let output_texture = device.create_texture(&output_desc);
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let msaa_desc = wgpu::TextureDescriptor {
        label: Some("Headless MSAA"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let scene_msaa_tex = device.create_texture(&msaa_desc);
    let scene_msaa_texture = scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_desc = wgpu::TextureDescriptor {
        label: Some("Headless Depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let depth_tex = device.create_texture(&depth_desc);
    let depth_texture_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let blur_width = (width / 2).max(1);
    let blur_height = (height / 2).max(1);
    let blur_desc_a = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Headless Blur Texture A".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let blur_tex_a = registry.allocate_image(device, &blur_desc_a);

    let blur_desc_b = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Headless Blur Texture B".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let blur_tex_b = registry.allocate_image(device, &blur_desc_b);

    let bloom_desc_a = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Headless Bloom Texture A".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let bloom_tex_a = registry.allocate_image(device, &bloom_desc_a);

    let bloom_desc_b = crate::kvasir::resource::ResourceDescriptor {
        label: Some("Headless Bloom Texture B".into()),
        kind: crate::kvasir::resource::ResourceKind::Image {
            format,
            width: blur_width,
            height: blur_height,
            mip_level_count: compute_mip_levels(blur_width, blur_height),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        },
        lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
    };
    let bloom_tex_b = registry.allocate_image(device, &bloom_desc_b);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Headless Scene Bind Group"),
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
    });

    let blur_view_a = registry
        .get_texture_view(blur_tex_a)
        .expect("headless: blur_tex_a view must exist after allocation");
    let blur_view_b = registry
        .get_texture_view(blur_tex_b)
        .expect("headless: blur_tex_b view must exist after allocation");
    let bloom_view_a = registry
        .get_texture_view(bloom_tex_a)
        .expect("headless: bloom_tex_a view must exist after allocation");
    let bloom_view_b = registry
        .get_texture_view(bloom_tex_b)
        .expect("headless: bloom_tex_b view must exist after allocation");

    let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Headless Blur Env Bind Group A"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&blur_view_a),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let blur_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Headless Blur Env Bind Group B"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&blur_view_b),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Headless Bloom Env Bind Group A"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bloom_view_a),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let bloom_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Headless Bloom Env Bind Group B"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bloom_view_b),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let scene_views: Vec<&wgpu::TextureView> = (0..32).map(|_| &scene_texture).collect();
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

    HeadlessContext {
        scene_texture,
        scene_msaa_texture,
        scene_bind_group,
        scene_texture_bind_group,
        depth_texture_view,
        blur_tex_a,
        blur_tex_b,
        bloom_tex_a,
        bloom_tex_b,
        blur_env_bind_group_a,
        blur_env_bind_group_b,
        bloom_env_bind_group_a,
        bloom_env_bind_group_b,
        scale_factor: 1.0,
        sampler,
        width,
        height,
        output_texture,
        output_view,
    }
}
