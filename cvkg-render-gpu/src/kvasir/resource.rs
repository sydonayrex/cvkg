/// Resource identity and descriptor types.
///
/// Every GPU resource (texture, buffer, atlas) managed by the renderer is
/// assigned a `ResourceId` and described by a `ResourceDescriptor`. The
/// `ResourceRegistry` maps these to actual `wgpu::Texture` / `wgpu::Buffer`
/// handles and tracks their allocation lifetimes.

use std::sync::atomic::{AtomicU64, Ordering};

/// Opaque identifier for a named resource in the render graph.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(pub u64);

/// What kind of data a resource carries — used by the planner to infer
/// correct barrier types and valid access modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Texture atlas (persistent across frames).
    Atlas,
}

/// Controls when the GPU backing for a resource is freed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLifetime {
    /// Both allocation and contents are transient — freed at end of frame.
    Frame,
    /// Allocation persists across frames, but contents are rebuilt each frame.
    /// Use this for ping-pong blur textures: no realloc cost, fresh pixels.
    FrameContent,
    /// Lives across frames. Loaded assets, atlases, retained scene geometry.
    Persistent,
}

/// Full description of one named resource.
#[derive(Clone, Debug)]
pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub label: &'static str,
    pub lifetime: ResourceLifetime,
}

/// Generates monotonically increasing `ResourceId`s.
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
