use super::resource::{ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct TextureResource {
    pub texture: Option<wgpu::Texture>,
    pub view: wgpu::TextureView,
    pub lifetime: ResourceLifetime,
}

#[derive(Default)]
pub struct ResourceRegistry {
    textures: RefCell<HashMap<ResourceId, TextureResource>>,
    pool: RefCell<HashMap<(wgpu::TextureFormat, u32, u32), Vec<TextureResource>>>,
    next_id: std::cell::Cell<u32>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            textures: RefCell::new(HashMap::new()),
            pool: RefCell::new(HashMap::new()),
            next_id: std::cell::Cell::new(10000),
        }
    }

    pub fn register_external_texture(
        &self,
        id: ResourceId,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
    ) {
        self.textures.borrow_mut().insert(
            id,
            TextureResource {
                texture: Some(texture),
                view,
                lifetime: ResourceLifetime::Frame,
            },
        );
    }

    pub fn alias(&self, alias_id: ResourceId, actual_id: ResourceId) {
        let textures = self.textures.borrow();
        if let Some(res) = textures.get(&actual_id) {
            let cloned = TextureResource {
                texture: res.texture.clone(),
                view: res.view.clone(),
                lifetime: ResourceLifetime::Frame,
            };
            drop(textures);
            self.textures.borrow_mut().insert(alias_id, cloned);
        }
    }

    pub fn alias_view(&self, alias_id: ResourceId, view: wgpu::TextureView) {
        self.textures.borrow_mut().insert(
            alias_id,
            TextureResource {
                texture: None,
                view,
                lifetime: ResourceLifetime::Frame,
            },
        );
    }

    pub fn allocate_offscreen(
        &self,
        device: &wgpu::Device,
        target_id: u64,
        size: [u32; 2],
    ) -> ResourceId {
        let id = ResourceId(1000 + target_id as u32);
        let pool_key = (
            wgpu::TextureFormat::Bgra8UnormSrgb,
            size[0].max(1),
            size[1].max(1),
        );

        let tex_res = {
            // Scope the pool borrow so it's released before texture creation (if needed)
            let mut pool = self.pool.borrow_mut();
            if let Some(mut t) = pool.get_mut(&pool_key).and_then(|list| list.pop()) {
                t.lifetime = ResourceLifetime::Frame;
                return id;
            }
            // Pool borrow released here — now create the texture
            drop(pool);
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Offscreen {}", target_id)),
                size: wgpu::Extent3d {
                    width: size[0].max(1),
                    height: size[1].max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            TextureResource {
                texture: Some(texture),
                view,
                lifetime: ResourceLifetime::Frame,
            }
        };

        self.textures.borrow_mut().insert(id, tex_res);
        id
    }

    pub fn allocate_image(
        &self,
        device: &wgpu::Device,
        desc: &ResourceDescriptor,
    ) -> ResourceId {
        let id = ResourceId(self.next_id.get());
        self.next_id.set(self.next_id.get() + 1);

        if let ResourceKind::Image {
            format,
            width,
            height,
            mip_level_count,
            usage,
        } = &desc.kind
        {
            let pool_key = (*format, *width, *height);
            let tex_res = {
                let mut pool = self.pool.borrow_mut();
                if let Some(mut t) = pool.get_mut(&pool_key).and_then(|list| list.pop()) {
                    t.lifetime = desc.lifetime;
                    t
                } else {
                    drop(pool);
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
                }
            };
            self.textures.borrow_mut().insert(id, tex_res);
        } else {
            panic!("allocate_image called with non-Image descriptor");
        }
        id
    }

    pub fn get_texture_view(&self, id: ResourceId) -> Option<wgpu::TextureView> {
        self.textures.borrow().get(&id).map(|res| res.view.clone())
    }

    pub fn get_texture(&self, id: ResourceId) -> Option<wgpu::Texture> {
        self.textures
            .borrow()
            .get(&id)
            .and_then(|res| res.texture.clone())
    }

    pub fn remove_image(&self, id: ResourceId) {
        self.textures.borrow_mut().remove(&id);
    }

    pub fn evict_frame_resources(&self) {
        // Move transient frame resources back into the pool instead of destroying them
        let mut to_remove = Vec::new();
        {
            let textures = self.textures.borrow();
            for (id, res) in textures.iter() {
                if res.lifetime == ResourceLifetime::Frame {
                    to_remove.push(*id);
                }
            }
        } // drop textures borrow

        for id in to_remove {
            let res = self.textures.borrow_mut().remove(&id);
            if let Some(res) = res
                && let Some(tex) = res.texture
            {
                let size = tex.size();
                let format = tex.format();
                let pool_key = (format, size.width, size.height);
                let mut pool = self.pool.borrow_mut();
                let pool_list = pool.entry(pool_key).or_default();
                if pool_list.len() < 4 {
                    pool_list.push(TextureResource {
                        texture: Some(tex),
                        view: res.view,
                        lifetime: res.lifetime,
                    });
                }
            }
        }
    }
}
