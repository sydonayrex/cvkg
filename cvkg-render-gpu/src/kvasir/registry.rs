/// Resource registry — maps `ResourceId` to actual GPU resources.

use std::collections::HashMap;

use super::resource::{
    ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime,
};

/// Maps resource IDs to actual GPU textures and buffers.
/// Tracks allocation lifetimes and provides LRU eviction for transient resources.
pub struct ResourceRegistry {
    descriptors: HashMap<ResourceId, ResourceDescriptor>,
    gpu_images: HashMap<ResourceId, wgpu::Texture>,
    gpu_buffers: HashMap<ResourceId, wgpu::Buffer>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            gpu_images: HashMap::new(),
            gpu_buffers: HashMap::new(),
        }
    }

    /// Register a resource descriptor. Returns the assigned `ResourceId`.
    pub fn register(&mut self, desc: ResourceDescriptor) -> ResourceId {
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

    /// Get or create a GPU image matching the descriptor.
    pub fn get_or_create_image(
        &mut self,
        desc: &ResourceDescriptor,
        device: &wgpu::Device,
    ) -> ResourceId {
        if self.descriptors.contains_key(&desc.id) {
            return desc.id;
        }

        if let ResourceKind::Image { width, height, format, mip_levels } = desc.kind {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(desc.label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.gpu_images.insert(desc.id, texture);
            self.descriptors.insert(desc.id, desc.clone());
            desc.id
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
