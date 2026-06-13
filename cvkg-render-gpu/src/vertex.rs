//! Vertex layouts, instance data, and tessellation vertex constructors.
use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub material_id: u32,
    pub radius: f32,
    pub slice: [f32; 4],
    pub logical: [f32; 2],
    pub size: [f32; 2],
    pub clip: [f32; 4], // [x, y, width, height]
    pub tex_index: u32,
}

/// Per-instance data for instanced rendering.
/// Stores transform data previously duplicated across all vertices of a path/quad.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub blur_radius: f32,
    /// Per-instance Index of Refraction (IOR) override for custom glass thickness refraction.
    pub ior_override: f32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
            blur_radius: 0.0,
            ior_override: 0.0,
        }
    }
}

impl InstanceData {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        11 => Float32x2, // translation
        12 => Float32x2, // scale
        13 => Float32,   // rotation
        14 => Float32,   // blur_radius
        15 => Float32,   // ior_override
    ];

    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 11] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
        3 => Float32x4, // color
        4 => Uint32,    // mode
        5 => Float32,   // radius
        6 => Float32x4, // slice
        7 => Float32x2, // logical
        8 => Float32x2, // size
        9 => Float32x4, // clip
        10 => Uint32,   // tex_index
    ];

    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub(crate) struct SceneVertexConstructor {
    pub(crate) color: [f32; 4],
}

/// Vertex constructor for stroke tessellation -- includes clip for transform.
pub(crate) struct CustomStrokeVertexConstructor {
    pub(crate) color: [f32; 4],
    pub(crate) clip: [f32; 4],
}

impl StrokeVertexConstructor<Vertex> for CustomStrokeVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        let pos = vertex.position();
        Vertex {
            position: [pos.x, pos.y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: self.color,
            material_id: 0,
            radius: 0.0,
            slice: [0.0, 0.0, 0.0, 1.0],
            logical: [pos.x, pos.y],
            size: [1.0, 1.0],
            clip: self.clip,
            tex_index: 0,
        }
    }
}

impl FillVertexConstructor<Vertex> for SceneVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: self.color,
            material_id: 0,
            radius: 0.0,
            slice: [0.0, 0.0, 0.0, 1.0],
            logical: [vertex.position().x, vertex.position().y],
            size: [1.0, 1.0],
            clip: [-10000.0, -10000.0, 20000.0, 20000.0],
            tex_index: 0,
        }
    }
}
