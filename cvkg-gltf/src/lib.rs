//! cvkg-gltf — glTF 2.0 asset loader for CVKG.
//!
//! Loads `.glb` and `.gltf` files and converts them into CVKG's native
//! 3D types: `Mesh`, `Material3D`, `Transform3D`, and `Camera3D`.
//!
//! # Usage
//!
//! ```no_run
//! use cvkg_gltf::load_gltf;
//!
//! let scene = load_gltf("path/to/model.glb")
//!     .expect("Failed to load glTF file");
//!
//! for node in &scene.nodes {
//!     println!("Node: {} ({} children)", node.name, node.children.len());
//! }
//! ```

mod importer;
pub mod player;
mod types;

pub use importer::load_gltf;
pub use types::{
    Animation3D, AnimationChannel3D, AnimationProperty, LoadedMesh, LoadedTexture, Node3D, Scene3D,
    Skin3D, TextureFormat,
};
