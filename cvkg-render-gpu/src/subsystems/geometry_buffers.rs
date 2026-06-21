//! P1-1 (phase 6): GeometryBuffers -- the three GPU draw buffers.
//!
//! Extracted from types.rs so the buffer management subsystem
//! has its own module. The SurtrRenderer holds a single
//! `GeometryBuffers` field; the three buffers are accessed via
//! `renderer.geometry_buffers.{vertex,index,instance}_buffer`.

use crate::vertex::{InstanceData, Vertex};

/// Group of three GPU buffers used for geometry rendering:
/// vertex, index, and instance. Owned by the renderer and used
/// for every draw call.
pub struct GeometryBuffers {
    /// Vertex buffer. Stores `Vertex` (position + normal + uv + color).
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer. Stores u32 indices into the vertex buffer.
    pub index_buffer: wgpu::Buffer,
    /// Instance buffer. Stores `InstanceData` for instanced rendering.
    pub instance_buffer: wgpu::Buffer,
    /// Capacity in vertices (used to size the vertex and instance buffers).
    pub max_vertices: usize,
    /// Capacity in indices (used to size the index buffer).
    pub max_indices: usize,
}

impl GeometryBuffers {
    /// Create the three geometry buffers on the given device with
    /// the given capacities. The buffers are immediately usable
    /// for COPY_DST writes.
    pub fn forge(
        device: &wgpu::Device,
        max_vertices: usize,
        max_indices: usize,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Vertex Anvil"),
            size: (max_vertices * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Index Anvil"),
            size: (max_indices * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Instance Anvil"),
            size: (max_vertices / 4 * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            max_vertices,
            max_indices,
        }
    }

    /// Total VRAM cost of the three buffers in bytes.
    pub fn vram_bytes(&self) -> u64 {
        let vertex_bytes = self.max_vertices * std::mem::size_of::<Vertex>();
        let index_bytes = self.max_indices * std::mem::size_of::<u32>();
        let instance_bytes =
            (self.max_vertices / 4) * std::mem::size_of::<InstanceData>();
        (vertex_bytes + index_bytes + instance_bytes) as u64
    }

    /// Grow the vertex buffer to accommodate at least `min_capacity`
    /// vertices. Returns true if the buffer was actually
    /// reallocated (i.e., the previous capacity was too small).
    /// Returns false if the buffer was already large enough,
    /// avoiding the expensive `device.create_buffer()` call.
    pub fn grow_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        min_capacity: usize,
        max_capacity: usize,
    ) -> bool {
        let current = self.vertex_buffer.size() as usize
            / std::mem::size_of::<Vertex>();
        if min_capacity <= current {
            return false;
        }
        let new_size = (min_capacity.min(max_capacity)) * std::mem::size_of::<Vertex>();
        self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer (Grown)"),
            size: new_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        true
    }

    /// Grow the index buffer to accommodate at least `min_capacity`
    /// indices. Returns true if the buffer was actually
    /// reallocated. Returns false if the buffer was already
    /// large enough.
    pub fn grow_index_buffer(
        &mut self,
        device: &wgpu::Device,
        min_capacity: usize,
        max_capacity: usize,
    ) -> bool {
        let current = self.index_buffer.size() as usize
            / std::mem::size_of::<u32>();
        if min_capacity <= current {
            return false;
        }
        let new_size = (min_capacity.min(max_capacity)) * std::mem::size_of::<u32>();
        self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer (Grown)"),
            size: new_size as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        true
    }
}
