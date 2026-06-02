/// Resource registry — maps `ResourceId` to actual GPU resources.
///
/// Tracks allocation lifetimes and provides the LRU eviction order for
/// `Frame`-lifetime transient resources at end of frame.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::kvasir::resource::{
    ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(pub u64);

pub enum ResourceKind {
    Image {
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_levels: u32,
    },
    Buffer {
        size: u64,
        usage: wgpu::BufferUsages,
    },
    Atlas,
}

pub enum ResourceLifetime {
    Frame,
    FrameContent,
    Persistent,
}

pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub label: &'static str,
    pub lifetime: ResourceLifetime,
}

pub struct ResourceIdGenerator(AtomicU64);

impl ResourceIdGenerator {
    pub const fn new() -> Self {
        Self(AtomicU64::new(1))
    }
    pub fn next(&self) -> ResourceId {
        ResourceId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ResourceIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps resource IDs to actual GPU textures and buffers.
pub struct ResourceRegistry {
    descriptors: HashMap<ResourceId, ResourceDescriptor>,
    gpu_images: HashMap<ResourceId, wgpu::Texture>,
    gpu_buffers: HashMap<ResourceId, wgpu::Buffer>,
    next_id: ResourceIdGenerator,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            gpu_images: HashMap::new(),
            gpu_buffers: HashMap::new(),
            next_id: ResourceIdGenerator::new(),
        }
    }

    /// Register a resource descriptor. Returns the assigned `ResourceId`.
    pub fn register(&mut self, mut desc: ResourceDescriptor) -> ResourceId {
        if desc.id.0 == 0 {
            desc.id = self.next_id.next();
        }
        let id = desc.id;
        self.descriptors.insert(id, desc);
        id
    }

    pub fn descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor> {
        self.descriptors.get(&id)
    }

    pub fn image(&self, id: ResourceId) -> Option<&wgpu::Texture> {
        self.gpu_images.get(&id)
    }

    pub fn image_view(
        &self,
        id: ResourceId,
    ) -> Option<&wgpu::TextureView> {
        // Texture views are created on-demand from the stored texture
        // For now, return the texture (views are cached in SurfaceContext)
        None // Placeholder — will be populated from SurfaceContext
    }

    /// Get or create a GPU image matching the descriptor.
    pub fn get_or_create_image(
        &mut self,
        desc: &ResourceDescriptor,
        device: &wgpu::Device,
    ) -> ResourceId {
        if let existing = self.descriptors.get(&desc.id) {
            if existing.label == desc.label {
                return desc.id;
            }
        }

        if let ResourceKind::Image {
            width,
            height,
            format,
            mip_levels,
        } = desc.kind
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(desc.label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let id = desc.id;
            self.gpu_images.insert(id, texture);
            self.descriptors.insert(id, desc.clone());
            id
        } else {
            panic!("get_or_create_image called with non-Image descriptor");
        }
    }

    /// Free all `Frame`-lifetime resources. Called at end of each frame.
    pub fn evict_frame_resources(&mut self) {
        let to_remove: Vec<ResourceId> = self
            .descriptors
            .iter()
            .filter(|(_, d)| matches!(d.lifetime, ResourceLifetime::Frame))
            .map(|(id, _)| *id)
            .collect();
        for id in to_remove {
            self.gpu_images.remove(&id);
            self.gpu_buffers.remove(&id);
            self.descriptors.remove(&id);
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
