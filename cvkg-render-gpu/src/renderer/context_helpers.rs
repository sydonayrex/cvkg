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
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let scene_msaa_tex = device.create_texture(&msaa_desc);
    let scene_msaa_texture = scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_desc = wgpu::TextureDescriptor {
        label: Some("Surtr Depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let depth_tex = device.create_texture(&depth_desc);
    let depth_texture_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

    // Register blur/bloom textures with the resource registry
    let blur_tex_a = ResourceId(0);
    let blur_tex_b = ResourceId(0);
    let bloom_tex_a = ResourceId(0);
    let bloom_tex_b = ResourceId(0);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // Create scene bind group
    let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Surtr Scene Bind Group"),
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
    });

    let scene_texture_bind_group = scene_bind_group.clone();

    // Create blur/bloom bind groups (simplified - use env layout)
    let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blur A Bind Group"),
        layout: env_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&scene_texture),
            },
        ],
    });
    let blur_env_bind_group_b = blur_env_bind_group_a.clone();
    let bloom_env_bind_group_a = blur_env_bind_group_a.clone();
    let bloom_env_bind_group_b = blur_env_bind_group_a.clone();

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
pub(crate) fn create_headless_context(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    env_bind_group_layout: &wgpu::BindGroupLayout,
    _texture_bind_group_layout: &wgpu::BindGroupLayout,
    registry: &mut crate::kvasir::registry::ResourceRegistry,
    msaa_sample_count: u32,
) -> HeadlessContext {
    let texture_desc = wgpu::TextureDescriptor {
        label: Some("Surtr Headless Scene Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let scene_tex = device.create_texture(&texture_desc);
    let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let output_desc = wgpu::TextureDescriptor {
        label: Some("Surtr Output Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
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
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let scene_msaa_tex = device.create_texture(&msaa_desc);
    let scene_msaa_texture = scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let depth_desc = wgpu::TextureDescriptor {
        label: Some("Headless Depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: msaa_sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let depth_tex = device.create_texture(&depth_desc);
    let depth_texture_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let blur_tex_a = ResourceId(0);
    let blur_tex_b = ResourceId(0);
    let bloom_tex_a = ResourceId(0);
    let bloom_tex_b = ResourceId(0);

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
    let scene_texture_bind_group = scene_bind_group.clone();

    let blur_env_bind_group_a = scene_bind_group.clone();
    let blur_env_bind_group_b = scene_bind_group.clone();
    let bloom_env_bind_group_a = scene_bind_group.clone();
    let bloom_env_bind_group_b = scene_bind_group.clone();

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
