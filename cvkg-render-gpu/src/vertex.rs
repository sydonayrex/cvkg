//! Vertex layouts, instance data, and tessellation vertex constructors.
use lyon::tessellation::{FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor};

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
    pub screen: [f32; 2],
    pub clip: [f32; 4], // [x, y, width, height]
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub tex_index: u32,
}

/// Per-instance data for instanced rendering.
/// Stores transform data previously duplicated across all 4 vertices of a quad.
/// With instancing, 4 vertices + 1 instance draw a quad, reducing vertex bandwidth ~4x.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub _pad: f32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self { translation: [0.0, 0.0], scale: [1.0, 1.0], rotation: 0.0, _pad: 0.0 }
    }
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 15] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
        3 => Float32x4, // color
        4 => Uint32,    // mode
        5 => Float32,   // radius
        6 => Float32x4, // slice
        7 => Float32x2, // logical
        8 => Float32x2, // size
        9 => Float32x2, // screen
        10 => Float32x4, // clip
        11 => Float32x2, // translation
        12 => Float32x2, // scale
        13 => Float32,   // rotation
        14 => Uint32     // tex_index
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
    pub(crate) translation: [f32; 2],
    pub(crate) scale: [f32; 2],
    pub(crate) rotation: f32,
}

/// Vertex constructor for stroke tessellation -- includes screen and clip for transform.
pub(crate) struct CustomStrokeVertexConstructor {
    pub(crate) color: [f32; 4],
    pub(crate) translation: [f32; 2],
    pub(crate) scale: [f32; 2],
    pub(crate) rotation: f32,
    pub(crate) screen: [f32; 2],
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
            screen: self.screen,
            clip: self.clip,
            translation: self.translation,
            scale: self.scale,
            rotation: self.rotation,
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
            screen: [0.0, 0.0],
            clip: [-10000.0, -10000.0, 20000.0, 20000.0],
            translation: self.translation,
            scale: self.scale,
            rotation: self.rotation,
            tex_index: 0,
        }
    }
}

