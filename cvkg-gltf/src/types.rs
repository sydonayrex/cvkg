//! Types for glTF-imported 3D scenes.
//!
//! These types represent the result of loading a glTF file — a flat scene
//! graph with meshes, materials, textures, and cameras ready for CVKG's
//! rendering pipeline.

use cvkg_core::mesh::{Camera3D, Material3D, Transform3D};

/// The complete result of loading a glTF 2.0 asset.
///
/// Nodes are stored in a flat array — parent/child links use indices.
/// Meshes and materials reference each other via optional indices.
pub struct Scene3D {
    /// Flat node list (parents appear before children).
    pub nodes: Vec<Node3D>,
    /// Loaded meshes (one per glTF primitive).
    pub meshes: Vec<LoadedMesh>,
    /// Loaded materials.
    pub materials: Vec<Material3D>,
    /// Loaded texture pixel data.
    pub textures: Vec<LoadedTexture>,
    /// Cameras defined in the scene.
    pub cameras: Vec<Camera3D>,
}

/// A single node in a flat scene tree.
pub struct Node3D {
    /// Index of this node within `Scene3D::nodes`.
    pub index: usize,
    /// Index of the parent node, or `None` for root nodes.
    pub parent: Option<usize>,
    /// Indices of child nodes.
    pub children: Vec<usize>,
    /// Human-readable name from glTF (may be empty).
    pub name: String,
    /// Local-space transform.
    pub transform: Transform3D,
    /// Index into `Scene3D::meshes` if this node has a mesh.
    pub mesh_index: Option<usize>,
    /// Index into `Scene3D::cameras` if this node has a camera.
    pub camera_index: Option<usize>,
}

/// A mesh produced from one glTF primitive.
pub struct LoadedMesh {
    /// Display name (glTF mesh name + primitive index).
    pub name: String,
    /// The actual index/vertex data.
    pub mesh: cvkg_core::mesh::Mesh,
    /// Index into `Scene3D::materials`, or `None` if the primitive
    /// used no material (should fall back to a default).
    pub material_index: Option<usize>,
}

/// Loaded texture pixel data.
pub struct LoadedTexture {
    /// Texture name from glTF (may be empty).
    pub name: String,
    /// Raw pixel data (uncompressed).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Format of the pixel data.
    pub format: TextureFormat,
}

/// Pixel format for loaded texture data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    /// 8-bit RGBA, non-linear sRGB space.
    Rgba8Srgb,
    /// 8-bit RGBA, linear space (normals, ORM).
    Rgba8Unorm,
}
