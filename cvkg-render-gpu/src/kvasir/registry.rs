use super::resource::{ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime};
use std::collections::HashMap;

pub struct TextureResource {
    pub texture: Option<wgpu::Texture>,
    pub view: wgpu::TextureView,
    pub lifetime: ResourceLifetime,
}

#[derive(Default)]
pub struct ResourceRegistry {
    textures: HashMap<ResourceId, TextureResource>,
    pool: HashMap<(wgpu::TextureFormat, u32, u32), Vec<TextureResource>>,
    next_id: u32,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            pool: HashMap::new(),
            next_id: 10000,
        }
    }

    pub fn register_external_texture(
        &mut self,
        id: ResourceId,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
    ) {
        self.textures.insert(
            id,
            TextureResource {
                texture: Some(texture),
                view,
                lifetime: ResourceLifetime::Frame,
            },
        );
    }

    pub fn alias(&mut self, alias_id: ResourceId, actual_id: ResourceId) {
        if let Some(res) = self.textures.get(&actual_id) {
            let cloned = TextureResource {
                texture: res.texture.clone(),
                view: res.view.clone(),
                lifetime: ResourceLifetime::Frame,
            };
            self.textures.insert(alias_id, cloned);
        }
    }

    pub fn alias_view(&mut self, alias_id: ResourceId, view: wgpu::TextureView) {
        self.textures.insert(
            alias_id,
            TextureResource {
                texture: None,
                view,
                lifetime: ResourceLifetime::Frame,
            },
        );
    }

    pub fn allocate_offscreen(
        &mut self,
        device: &wgpu::Device,
        target_id: u64,
        size: [u32; 2],
    ) -> ResourceId {
        let desc = ResourceDescriptor {
            label: Some(format!("Offscreen {}", target_id)),
            kind: ResourceKind::Image {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: size[0].max(1),
                height: size[1].max(1),
                mip_level_count: 1,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            },
            lifetime: ResourceLifetime::Frame,
        };
        let id = ResourceId(1000 + target_id as u32);

        if let ResourceKind::Image {
            format,
            width,
            height,
            mip_level_count,
            usage,
        } = &desc.kind
        {
            let pool_key = (*format, *width, *height);
            let pooled_tex = if let Some(pool_list) = self.pool.get_mut(&pool_key) {
                pool_list.pop()
            } else {
                None
            };

            let tex_res = if let Some(mut t) = pooled_tex {
                t.lifetime = desc.lifetime;
                t
            } else {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: desc.label.as_deref(),
                    size: wgpu::Extent3d {
                        width: *width,
                        height: *height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: *mip_level_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: *format,
                    usage: *usage,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                TextureResource {
                    texture: Some(texture),
                    view,
                    lifetime: desc.lifetime,
                }
            };

            self.textures.insert(id, tex_res);
        }
        id
    }

    pub fn allocate_image(
        &mut self,
        device: &wgpu::Device,
        desc: &ResourceDescriptor,
    ) -> ResourceId {
        let id = ResourceId(self.next_id);
        self.next_id += 1;

        if let ResourceKind::Image {
            format,
            width,
            height,
            mip_level_count,
            usage,
        } = &desc.kind
        {
            let pool_key = (*format, *width, *height);
            let pooled_tex = if let Some(pool_list) = self.pool.get_mut(&pool_key) {
                pool_list.pop()
            } else {
                None
            };

            let tex_res = if let Some(mut t) = pooled_tex {
                t.lifetime = desc.lifetime;
                t
            } else {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: desc.label.as_deref(),
                    size: wgpu::Extent3d {
                        width: *width,
                        height: *height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: *mip_level_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: *format,
                    usage: *usage,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                TextureResource {
                    texture: Some(texture),
                    view,
                    lifetime: desc.lifetime,
                }
            };

            self.textures.insert(id, tex_res);
        } else {
            panic!("allocate_image called with non-Image descriptor");
        }
        id
    }

    pub fn get_texture_view(&self, id: ResourceId) -> Option<wgpu::TextureView> {
        self.textures.get(&id).map(|res| res.view.clone())
    }

    pub fn get_texture(&self, id: ResourceId) -> Option<wgpu::Texture> {
        self.textures.get(&id).and_then(|res| res.texture.clone())
    }

    pub fn remove_image(&mut self, id: ResourceId) {
        self.textures.remove(&id);
    }

    pub fn evict_frame_resources(&mut self) {
        // Move transient frame resources back into the pool instead of destroying them
        let mut to_remove = Vec::new();
        for (id, res) in &self.textures {
            if res.lifetime == ResourceLifetime::Frame {
                to_remove.push(*id);
            }
        }
        for id in to_remove {
            if let Some(res) = self.textures.remove(&id) {
                if let Some(tex) = res.texture {
                    let size = tex.size();
                    let format = tex.format();
                    let pool_key = (format, size.width, size.height);
                    self.pool
                        .entry(pool_key)
                        .or_default()
                        .push(TextureResource {
                            texture: Some(tex),
                            view: res.view,
                            lifetime: res.lifetime,
                        });
                }
            }
        }
    }
}
