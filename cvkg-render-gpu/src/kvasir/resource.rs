/// Opaque handle to a GPU resource (Texture, Buffer, etc.) managed by the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Image {
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
    },
    Buffer {
        size: u64,
        usage: wgpu::BufferUsages,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifetime {
    /// Destroyed automatically at the end of the frame.
    Frame,
    /// Lives until explicitly destroyed or window is closed.
    Persistent,
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub label: Option<String>,
    pub kind: ResourceKind,
    pub lifetime: ResourceLifetime,
}
