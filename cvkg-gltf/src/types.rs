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
    /// Skeletal and transform animations.
    pub animations: Vec<Animation3D>,
    /// Skins containing joint/weight hierarchies.
    pub skins: Vec<Skin3D>,
}

/// A skeletal/node animation channel.
#[derive(Debug, Clone)]
pub struct Animation3D {
    /// Human-readable name of the animation sequence.
    pub name: String,
    /// Targeted node channels (T/R/S).
    pub channels: Vec<AnimationChannel3D>,
}

/// Target channels for keyframe transformations.
#[derive(Debug, Clone)]
pub struct AnimationChannel3D {
    /// Index of the target Node3D being animated.
    pub target_node: usize,
    /// Property path to animate (Translation, Rotation, Scale, Weights).
    pub property: AnimationProperty,
    /// Timestamps in seconds.
    pub keyframes: Vec<f32>,
    /// Raw transform data matching the keyframe timestamps.
    pub values: Vec<f32>,
}

/// Transform properties animated by glTF channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationProperty {
    /// Position offset.
    Translation,
    /// Rotation quaternion.
    Rotation,
    /// Scale factor.
    Scale,
    /// Morph target weights.
    MorphWeights,
}

/// A skin containing bind matrices and joint node references.
#[derive(Debug, Clone)]
pub struct Skin3D {
    /// Human-readable name of the skin.
    pub name: String,
    /// Node index of the skeletal hierarchy root joint.
    pub skeleton_root: Option<usize>,
    /// Indices of nodes acting as joints in the skeleton.
    pub joints: Vec<usize>,
    /// Inverse bind matrices aligning joints to mesh space.
    pub inverse_bind_matrices: Vec<glam::Mat4>,
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
    /// Index into `Scene3D::skins` if this node has an associated skeletal skin.
    pub skin_index: Option<usize>,
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
