//! Scene flattener — converts hierarchical scene data into flat GPU-ready instances.
//!
//! The `SceneFlattener` walks a scene graph (either from `cvkg-gltf` or `cvkg-scene`)
//! and produces a list of [`FlatMeshInstance`] with absolute transforms and AABBs,
//! ready to be submitted to the GPU via [`GpuRenderer::submit_mesh_3d`].

use cvkg_core::mesh::Mesh;
#[cfg(any(feature = "gltf", feature = "scene"))]
use cvkg_spatial::frustum::Frustum;
use glam::Mat4;
use glam::Vec3;
#[cfg(feature = "scene")]
use glam::Quat;

/// A mesh instance with absolute world-space transform and precomputed AABB.
///
/// This is the output type of the scene flattener — ready to be consumed
/// by `GpuRenderer::submit_mesh_3d` for GPU upload.
#[derive(Debug, Clone)]
pub struct FlatMeshInstance {
    /// Source mesh data (vertices, normals, indices, etc.).
    pub mesh: Mesh,
    /// PBR material index into the scene's material array.
    pub material_index: Option<usize>,
    /// Absolute world-space transform matrix.
    pub transform: Mat4,
    /// Axis-aligned bounding box center (world space).
    pub aabb_center: Vec3,
    /// Axis-aligned bounding box half-extents (world space).
    pub aabb_half_extents: Vec3,
}

/// Flattened scene — all meshes with absolute transforms, ready for GPU submission.
#[derive(Debug, Default)]
pub struct FlatScene {
    /// Mesh instances with absolute transforms.
    pub instances: Vec<FlatMeshInstance>,
    /// Scene lights (passed through unchanged).
    pub lights: Vec<cvkg_render_gpu::passes::shadow::DirectionalLight>,
}

/// Converts hierarchical scene data into flat GPU-ready instances.
///
/// # Usage
///
/// ```ignore
/// let flattener = SceneFlattener::new();
/// let flat = flattener.flatten_gltf(&scene, &frustum_culler);
/// for instance in &flat.instances {
///     renderer.submit_mesh_3d(&instance.mesh, instance.transform);
/// }
/// ```
pub struct SceneFlattener;

impl SceneFlattener {
    /// Create a new flattener.
    pub fn new() -> Self {
        Self
    }

    /// Flatten a glTF scene into flat mesh instances.
    ///
    /// Walks the `Node3D` tree, computes absolute transforms by multiplying
    /// down the hierarchy, and collects meshes with their world-space AABBs.
    ///
    /// If `frustum` is provided, only instances whose AABB intersects the
    /// frustum are included in the output.
    #[cfg(feature = "gltf")]
    pub fn flatten_gltf(
        &self,
        scene: &cvkg_gltf::Scene3D,
        frustum: Option<&Frustum>,
    ) -> FlatScene {
        let mut instances = Vec::new();

        // Build absolute transforms for all nodes.
        let mut node_transforms = vec![Mat4::IDENTITY; scene.nodes.len()];

        // Topological pass: process nodes in index order (parents before children).
        for (i, node) in scene.nodes.iter().enumerate() {
            let local = node.transform.to_mat4();
            let parent_transform = node
                .parent
                .map(|p| node_transforms[p])
                .unwrap_or(Mat4::IDENTITY);
            node_transforms[i] = parent_transform * local;
        }

        // Collect meshes with absolute transforms.
        for (i, node) in scene.nodes.iter().enumerate() {
            let Some(mesh_idx) = node.mesh_index else {
                continue;
            };
            let Some(loaded_mesh) = scene.meshes.get(mesh_idx) else {
                tracing::warn!(
                    "SceneFlattener: node '{}' references invalid mesh index {}",
                    node.name,
                    mesh_idx
                );
                continue;
            };

            let transform = node_transforms[i];
            let (local_center, local_half) = loaded_mesh.mesh.aabb();

            // Transform AABB to world space: world_half = |R| * local_half + |S| * local_center
            // where R = rotation part, S = scale part of the matrix
            let world_center = transform.transform_point3(local_center);
            // Extract rotation+scale part (upper 3x3), take absolute values
            let col0 = transform.col(0).truncate().abs();
            let col1 = transform.col(1).truncate().abs();
            let col2 = transform.col(2).truncate().abs();
            let world_half = col0 * local_half.x + col1 * local_half.y + col2 * local_half.z;

            // Frustum cull if provided.
            if let Some(frustum) = frustum {
                if !frustum.intersects_aabb(world_center, world_half) {
                    continue;
                }
            }

            instances.push(FlatMeshInstance {
                mesh: loaded_mesh.mesh.clone(),
                material_index: loaded_mesh.material_index,
                transform,
                aabb_center: world_center,
                aabb_half_extents: world_half,
            });
        }

        FlatScene {
            instances,
            lights: Vec::new(),
        }
    }

    /// Flatten a `cvkg-scene` SceneGraph's 3D nodes into flat mesh instances.
    ///
    /// Only nodes with `is_3d == true` are included. Nodes without mesh data
    /// (pure transform nodes) are traversed for hierarchy but skipped in output.
    ///
    /// The `mesh_lookup` closure maps a `component_type` string to an optional
    /// `(Mesh, Option<usize>)` — the mesh data and optional material index.
    /// Returns `None` for nodes that don't have mesh data.
    #[cfg(feature = "scene")]
    pub fn flatten_scene_graph<F>(
        &self,
        scene: &cvkg_scene::SceneGraph,
        mesh_lookup: F,
        frustum: Option<&Frustum>,
    ) -> FlatScene
    where
        F: Fn(&str) -> Option<(Mesh, Option<usize>)>,
    {
        use cvkg_scene::NodeId;
        use std::collections::HashMap;

        let mut instances = Vec::new();

        // Build parent map and compute absolute transforms.
        let mut node_transforms: HashMap<NodeId, Mat4> = HashMap::new();

        // First pass: compute transforms for all 3D nodes.
        for (id, node) in scene.nodes.iter() {
            if !node.is_3d {
                continue;
            }

            let position = Vec3::from(node.position_3d);
            let rotation = Quat::from_xyzw(
                node.rotation_3d[0],
                node.rotation_3d[1],
                node.rotation_3d[2],
                node.rotation_3d[3],
            );
            let scale = Vec3::from(node.scale_3d);

            let local = Mat4::from_scale_rotation_translation(scale, rotation, position);

            // Find parent's absolute transform by walking up.
            let parent_transform = self.find_parent_transform(scene, *id, &node_transforms);

            node_transforms.insert(*id, parent_transform * local);
        }

        // Second pass: collect mesh instances.
        for (id, node) in scene.nodes.iter() {
            if !node.is_3d {
                continue;
            }

            let Some((mesh, material_index)) = mesh_lookup(&node.component_type) else {
                continue;
            };

            let transform = node_transforms.get(id).copied().unwrap_or(Mat4::IDENTITY);
            let (local_center, local_half) = mesh.aabb();

            let world_center = transform.transform_point3(local_center);
            let col0 = transform.col(0).truncate().abs();
            let col1 = transform.col(1).truncate().abs();
            let col2 = transform.col(2).truncate().abs();
            let world_half = col0 * local_half.x + col1 * local_half.y + col2 * local_half.z;

            if let Some(frustum) = frustum {
                if !frustum.intersects_aabb(world_center, world_half) {
                    continue;
                }
            }

            instances.push(FlatMeshInstance {
                mesh,
                material_index,
                transform,
                aabb_center: world_center,
                aabb_half_extents: world_half,
            });
        }

        FlatScene {
            instances,
            lights: Vec::new(),
        }
    }

    /// Submit a flattened scene to the GPU renderer.
    ///
    /// Iterates over all [`FlatMeshInstance`] in the scene and submits each
    /// one to the renderer using `submit_mesh_3d_matrix`. Materials are
    /// looked up by index from the provided `materials` slice; nodes without
    /// a material index or with an out-of-range index use a default white material.
    #[cfg(feature = "gltf")]
    pub fn submit_scene(
        &self,
        renderer: &mut cvkg_render_gpu::GpuRenderer,
        scene: &FlatScene,
        materials: &[cvkg_core::Material3D],
    ) {
        let default_material = cvkg_core::Material3D::default();

        for instance in &scene.instances {
            let material = instance
                .material_index
                .and_then(|i| materials.get(i))
                .unwrap_or(&default_material);

            renderer.submit_mesh_3d_matrix(&instance.mesh, material, &instance.transform);
        }
    }

    /// Find the parent's absolute transform by scanning the scene graph.
    ///
    /// Since `SceneGraph` doesn't store parent IDs directly, we find the parent
    /// by checking which node lists this ID in its children.
    #[cfg(feature = "scene")]
    fn find_parent_transform(
        &self,
        scene: &cvkg_scene::SceneGraph,
        child_id: cvkg_scene::NodeId,
        transforms: &std::collections::HashMap<cvkg_scene::NodeId, Mat4>,
    ) -> Mat4 {
        for (_id, node) in scene.nodes.iter() {
            if node.children.contains(&child_id) {
                return transforms.get(&_id).copied().unwrap_or(Mat4::IDENTITY);
            }
        }
        Mat4::IDENTITY
    }
}

impl Default for SceneFlattener {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;

    fn sample_mesh() -> Mesh {
        Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            indices: vec![0, 1, 2],
            tex_coords: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            joint_indices: vec![],
            joint_weights: vec![],
        }
    }

    #[test]
    fn test_flatten_empty_scene() {
        let flattener = SceneFlattener::new();
        let scene = cvkg_gltf::Scene3D {
            nodes: vec![],
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            cameras: vec![],
            animations: vec![],
            skins: vec![],
        };
        let flat = flattener.flatten_gltf(&scene, None);
        assert!(flat.instances.is_empty());
    }

    #[test]
    fn test_flatten_single_node_with_mesh() {
        use cvkg_core::mesh::Transform3D;

        let flattener = SceneFlattener::new();
        let scene = cvkg_gltf::Scene3D {
            nodes: vec![cvkg_gltf::Node3D {
                index: 0,
                parent: None,
                children: vec![],
                name: "test".to_string(),
                transform: Transform3D {
                    position: Vec3::new(1.0, 2.0, 3.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                mesh_index: Some(0),
                camera_index: None,
                skin_index: None,
            }],
            meshes: vec![cvkg_gltf::LoadedMesh {
                name: "test_mesh".to_string(),
                mesh: sample_mesh(),
                material_index: None,
            }],
            materials: vec![],
            textures: vec![],
            cameras: vec![],
            animations: vec![],
            skins: vec![],
        };

        let flat = flattener.flatten_gltf(&scene, None);
        assert_eq!(flat.instances.len(), 1);

        let inst = &flat.instances[0];
        // Transform should be translation (1, 2, 3)
        let pos = inst.transform.col(3).truncate();
        assert!((pos.x - 1.0).abs() < 1e-5);
        assert!((pos.y - 2.0).abs() < 1e-5);
        assert!((pos.z - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_flatten_child_inherits_parent_transform() {
        use cvkg_core::mesh::Transform3D;

        let flattener = SceneFlattener::new();
        let scene = cvkg_gltf::Scene3D {
            nodes: vec![
                cvkg_gltf::Node3D {
                    index: 0,
                    parent: None,
                    children: vec![1],
                    name: "parent".to_string(),
                    transform: Transform3D {
                        position: Vec3::new(10.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    mesh_index: None,
                    camera_index: None,
                    skin_index: None,
                },
                cvkg_gltf::Node3D {
                    index: 1,
                    parent: Some(0),
                    children: vec![],
                    name: "child".to_string(),
                    transform: Transform3D {
                        position: Vec3::new(1.0, 0.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    mesh_index: Some(0),
                    camera_index: None,
                    skin_index: None,
                },
            ],
            meshes: vec![cvkg_gltf::LoadedMesh {
                name: "child_mesh".to_string(),
                mesh: sample_mesh(),
                material_index: None,
            }],
            materials: vec![],
            textures: vec![],
            cameras: vec![],
            animations: vec![],
            skins: vec![],
        };

        let flat = flattener.flatten_gltf(&scene, None);
        assert_eq!(flat.instances.len(), 1);

        let pos = flat.instances[0].transform.col(3).truncate();
        // Child at (1,0,0) relative to parent at (10,0,0) = world (11,0,0)
        assert!((pos.x - 11.0).abs() < 1e-5);
    }

    #[test]
    fn test_flatten_frustum_cull() {
        use cvkg_core::mesh::Transform3D;
        use cvkg_spatial::frustum::Frustum;

        let flattener = SceneFlattener::new();
        let scene = cvkg_gltf::Scene3D {
            nodes: vec![
                cvkg_gltf::Node3D {
                    index: 0,
                    parent: None,
                    children: vec![],
                    name: "visible".to_string(),
                    transform: Transform3D {
                        position: Vec3::new(0.0, 0.0, -5.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    mesh_index: Some(0),
                    camera_index: None,
                    skin_index: None,
                },
                cvkg_gltf::Node3D {
                    index: 1,
                    parent: None,
                    children: vec![],
                    name: "behind_camera".to_string(),
                    transform: Transform3D {
                        position: Vec3::new(0.0, 0.0, 100.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    mesh_index: Some(0),
                    camera_index: None,
                    skin_index: None,
                },
            ],
            meshes: vec![cvkg_gltf::LoadedMesh {
                name: "test_mesh".to_string(),
                mesh: sample_mesh(),
                material_index: None,
            }],
            materials: vec![],
            textures: vec![],
            cameras: vec![],
            animations: vec![],
            skins: vec![],
        };

        // Create a frustum looking down -Z from origin.
        let view = Mat4::look_to_lh(Vec3::ZERO, -Vec3::Z, Vec3::Y);
        let proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 50.0);
        let frustum = Frustum::from_view_projection(&(proj * view));

        let flat = flattener.flatten_gltf(&scene, Some(&frustum));
        // Only the visible node (at z=-5) should pass; behind_camera (at z=100) should be culled.
        assert_eq!(flat.instances.len(), 1);
        assert_eq!(flat.instances[0].aabb_center.z, -5.0);
    }
}
