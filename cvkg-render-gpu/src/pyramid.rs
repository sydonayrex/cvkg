use crate::kvasir::registry::ResourceRegistry;
use crate::kvasir::resource::{ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime};
use wgpu;

pub struct ImagePyramid {
    pub mips: Vec<ResourceId>,
    pub luminance: ResourceId,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
}

impl ImagePyramid {
    pub fn new(
        registry: &mut ResourceRegistry,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        levels: u32,
    ) -> Self {
        let mut mips = Vec::new();
        let mut mip_w = width;
        let mut mip_h = height;

        for i in 0..levels {
            let desc = ResourceDescriptor {
                label: Some(format!("pyramid_mip_{}", i)),
                kind: ResourceKind::Image {
                    width: mip_w,
                    height: mip_h,
                    format: wgpu::TextureFormat::Rgba16Float,
                    mip_level_count: 1,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST,
                },
                lifetime: ResourceLifetime::Frame,
            };
            mips.push(registry.allocate_image(device, &desc));

            mip_w = (mip_w / 2).max(1);
            mip_h = (mip_h / 2).max(1);
        }

        // Luminance texture matches the last allocated mip level dimensions
        let last_mip_w = (mip_w * 2).min(width);
        let last_mip_h = (mip_h * 2).min(height);
        let luminance = registry.allocate_image(
            device,
            &ResourceDescriptor {
                label: Some("pyramid_luminance".to_string()),
                kind: ResourceKind::Image {
                    width: last_mip_w,
                    height: last_mip_h,
                    format: wgpu::TextureFormat::R8Unorm,
                    mip_level_count: 1,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                },
                lifetime: ResourceLifetime::Frame,
            },
        );

        Self {
            mips,
            luminance,
            width,
            height,
            levels,
        }
    }

    pub fn sample_at_blur_radius(&self, radius: f32) -> ResourceId {
        // Simple mapping: radius -> mip level.
        // e.g., radius 0 -> mip 0
        // radius 4 -> mip 1, etc.
        let mip = (radius.log2().max(0.0) as usize).min(self.levels.saturating_sub(1) as usize);
        self.mips[mip]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniforms {
    pub params: [f32; 4], // xy = src_texture_size, z = mip_level, w = offset
    pub mode: u32,        // 0=down, 1=up, 2=composite
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}
