use std::collections::HashMap;
use crate::types::FilterError;
use crate::graph::{FilterGraph, FilterInput};

/// A reusable texture buffer in the pool.
#[derive(Debug)]
struct PooledTexture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    /// Whether this buffer is currently in use.
    in_use: bool,
}

/// Pool of reusable textures for filter intermediate results.
pub struct TransientFilterPool {
    available: Vec<PooledTexture>,
    total_allocated: usize,
    total_reused: usize,
}

impl TransientFilterPool {
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            total_allocated: 0,
            total_reused: 0,
        }
    }

    /// Acquire a texture from the pool, or create a new one if none match.
    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (&wgpu::Texture, &wgpu::TextureView) {
        if let Some(pos) = self.available.iter().position(|p| {
            !p.in_use && p.width == width && p.height == height
        }) {
            let pooled = &mut self.available[pos];
            pooled.in_use = true;
            self.total_reused += 1;
            return (&pooled.texture, &pooled.view);
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TransientFilter"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.available.push(PooledTexture {
            width,
            height,
            texture,
            view,
            in_use: true,
        });
        self.total_allocated += 1;
        let last = self.available.last().unwrap();
        (&last.texture, &last.view)
    }

    /// Release a texture back to the pool for reuse.
    pub fn release(&mut self, width: u32, height: u32) {
        if let Some(pooled) = self.available.iter_mut().find(|p| {
            p.in_use && p.width == width && p.height == height
        }) {
            pooled.in_use = false;
        }
    }

    /// Reset the pool (mark all textures as available).
    pub fn reset(&mut self) {
        for pooled in &mut self.available {
            pooled.in_use = false;
        }
    }

    pub fn len(&self) -> usize {
        self.available.len()
    }

    pub fn is_empty(&self) -> bool {
        self.available.is_empty()
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.total_allocated, self.total_reused)
    }
}

impl Default for TransientFilterPool {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// P1-29: Filter Resources as First-Class Graph Resources
// =============================================================================

/// A first-class filter resource that can be shared across filter nodes.
#[derive(Debug, Clone)]
pub struct FilterResource {
    pub name: String,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub ref_count: usize,
    pub computed: bool,
}

impl FilterResource {
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            pixels: vec![0; (width * height * 4) as usize],
            width,
            height,
            ref_count: 0,
            computed: false,
        }
    }

    pub fn increment_ref(&mut self) {
        self.ref_count += 1;
    }

    pub fn decrement_ref(&mut self) {
        self.ref_count = self.ref_count.saturating_sub(1);
    }

    pub fn is_shared(&self) -> bool {
        self.ref_count > 1
    }
}

// =============================================================================
// P1-30: Filter Planner
// =============================================================================

#[derive(Debug, Clone)]
pub struct FilterResourcePlan {
    pub resources: HashMap<String, FilterResource>,
    pub execution_order: Vec<usize>,
}

pub struct FilterPlanner;

impl FilterPlanner {
    pub fn plan(graph: &FilterGraph) -> Result<FilterResourcePlan, FilterError> {
        let mut resources: HashMap<String, FilterResource> = HashMap::new();
        let mut ref_counts: HashMap<String, usize> = HashMap::new();

        for node in graph.nodes() {
            for input in &node.inputs {
                if let FilterInput::Reference(name) = input {
                    *ref_counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        for node in graph.nodes() {
            if !node.result_name.is_empty() {
                let count = ref_counts.get(&node.result_name).copied().unwrap_or(0);
                let mut resource = FilterResource::new(&node.result_name, 0, 0);
                resource.ref_count = count;
                resources.insert(node.result_name.clone(), resource);
            }
        }

        let execution_order: Vec<usize> = (0..graph.nodes().len()).collect();

        Ok(FilterResourcePlan {
            resources,
            execution_order,
        })
    }

    pub fn shared_resource_count(plan: &FilterResourcePlan) -> usize {
        plan.resources.values().filter(|r| r.is_shared()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_is_empty() {
        let pool = TransientFilterPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn stats_start_at_zero() {
        let pool = TransientFilterPool::new();
        let (allocated, reused) = pool.stats();
        assert_eq!(allocated, 0);
        assert_eq!(reused, 0);
    }

    #[test]
    fn filter_resource_new() {
        let r = FilterResource::new("blur", 100, 100);
        assert_eq!(r.name, "blur");
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 100);
        assert_eq!(r.ref_count, 0);
        assert!(!r.computed);
    }

    #[test]
    fn ref_counting() {
        let mut r = FilterResource::new("blur", 100, 100);
        r.increment_ref();
        r.increment_ref();
        assert_eq!(r.ref_count, 2);
        assert!(r.is_shared());
        r.decrement_ref();
        assert_eq!(r.ref_count, 1);
        assert!(!r.is_shared());
    }

    #[test]
    fn decrement_saturates_at_zero() {
        let mut r = FilterResource::new("blur", 100, 100);
        r.decrement_ref();
        assert_eq!(r.ref_count, 0);
    }
}
