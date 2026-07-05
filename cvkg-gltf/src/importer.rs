//! Converts glTF 2.0 assets into CVKG scene data.
//!
//! The entry point is [`load_gltf`], which reads a `.glb` or `.gltf` file
//! and returns a [`Scene3D`] containing all meshes, materials, textures,
//! cameras, and the node hierarchy.

use std::path::Path;

use anyhow::{Context, Result};
use cvkg_core::mesh::{Camera3D, Material3D, Mesh, Transform3D};
use glam::{Quat, Vec3};
use gltf::scene::Transform as GltfTransform;

use crate::types::{LoadedMesh, LoadedTexture, Node3D, Scene3D, TextureFormat};

/// Load a glTF 2.0 file (`.glb` or `.gltf`) and convert it to a CVKG [`Scene3D`].
///
/// # Errors
/// Returns an error if the file cannot be read, is not valid glTF 2.0, or
/// contains unsupported features (e.g. morph targets with non-standard
/// accessor types).
pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Scene3D> {
    let path = path.as_ref();
    let (document, buffers, images) = gltf::import(path)
        .with_context(|| format!("Failed to load glTF file: {}", path.display()))?;

    // ── 1. Convert materials ────────────────────────────────────────────
    let materials: Vec<Material3D> = document.materials().map(convert_material).collect();

    // ── 2. Convert textures ─────────────────────────────────────────────
    let textures: Vec<LoadedTexture> = images
        .iter()
        .enumerate()
        .map(|(i, img)| convert_image(i, img))
        .collect::<Result<_>>()?;

    // ── 3. Convert cameras ──────────────────────────────────────────────
    let cameras: Vec<Camera3D> = document.cameras().map(convert_camera).collect();

    // ── 4. Convert meshes — one LoadedMesh per primitive ────────────────
    //     Build an offset table so we can map (gltf_mesh_index, primitive_index)
    //     to a flat offset in the meshes vector.
    let mut mesh_offset_per_gltf_mesh: Vec<(usize, usize)> = Vec::new(); // (gltf_mesh_idx, primitive_count)
    for gltf_mesh in document.meshes() {
        let count = gltf_mesh.primitives().len();
        mesh_offset_per_gltf_mesh.push((gltf_mesh.index(), count));
    }
    // Build cumulative offset: mesh_prim_start[gltf_mesh_idx] = first primitive index
    let mut mesh_prim_offset = Vec::with_capacity(mesh_offset_per_gltf_mesh.len() + 1);
    mesh_prim_offset.push(0usize);
    for &(_, count) in &mesh_offset_per_gltf_mesh {
        let next = mesh_prim_offset.last().copied().unwrap_or(0) + count;
        mesh_prim_offset.push(next);
    }
    let total_primitives = mesh_prim_offset.last().copied().unwrap_or(0);

    let mut meshes = Vec::with_capacity(total_primitives);
    for gltf_mesh in document.meshes() {
        for (prim_idx, primitive) in gltf_mesh.primitives().enumerate() {
            let loaded = convert_primitive(gltf_mesh.name(), prim_idx, primitive, &buffers)?;
            meshes.push(loaded);
        }
    }

    // ── 5. Build node hierarchy from the default scene ─────────────────
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .context("glTF file has no scenes")?;

    let nodes = build_hierarchy(scene, mesh_prim_offset.as_slice(), &cameras);

    Ok(Scene3D {
        nodes,
        meshes,
        materials,
        textures,
        cameras,
    })
}

// ── Material conversion ─────────────────────────────────────────────────────

fn convert_material(gltf_mat: gltf::Material<'_>) -> Material3D {
    let pbr = gltf_mat.pbr_metallic_roughness();
    let base_color: [f32; 4] = pbr.base_color_factor().into();
    let base_color_texture: Option<String> = pbr
        .base_color_texture()
        .map(|info| info.texture().name().unwrap_or("base_color").to_string());
    let normal_map_texture: Option<String> = gltf_mat
        .normal_texture()
        .map(|info| info.texture().name().unwrap_or("normal_map").to_string());
    let metallic_roughness_texture: Option<String> = pbr
        .metallic_roughness_texture()
        .map(|info| info.texture().name().unwrap_or("orm").to_string());
    let emissive: [f32; 3] = gltf_mat.emissive_factor().into();

    Material3D {
        base_color,
        base_color_texture,
        normal_map_texture,
        metallic_roughness_texture,
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        emissive,
        opacity: base_color[3],
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
    }
}

// ── Texture conversion ──────────────────────────────────────────────────────

fn convert_image(index: usize, img: &gltf::image::Data) -> Result<LoadedTexture> {
    let (data, format) = match img.format {
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity((img.width * img.height * 4) as usize);
            for chunk in img.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            (rgba, TextureFormat::Rgba8Srgb)
        }
        gltf::image::Format::R8G8B8A8 => (img.pixels.clone(), TextureFormat::Rgba8Srgb),
        other => {
            anyhow::bail!("Unsupported glTF image format: {other:?} (index {index})");
        }
    };
    Ok(LoadedTexture {
        name: format!("texture_{index}"),
        data,
        width: img.width,
        height: img.height,
        format,
    })
}

// ── Primitive conversion ────────────────────────────────────────────────────

fn convert_primitive(
    mesh_name: Option<&str>,
    prim_idx: usize,
    primitive: gltf::Primitive<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<LoadedMesh> {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // Positions (required)
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .context("Primitive missing POSITION accessor")?
        .collect();

    // Normals (fall back to upward default)
    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);

    // Tex coords — channel 0 only
    let tex_coords: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|t| t.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    // Indices (required for indexed rendering)
    let indices: Vec<u32> = reader
        .read_indices()
        .context("Primitive missing indices")?
        .into_u32()
        .collect();

    // Material
    let material_index = primitive.material().index();

    // Display name
    let name = match mesh_name {
        Some(n) if !n.is_empty() => format!("{n}/prim_{prim_idx}"),
        _ => format!("prim_{prim_idx}"),
    };

    Ok(LoadedMesh {
        name,
        mesh: {
            let mut m = Mesh {
                vertices: positions,
                normals,
                tex_coords,
                indices,
                tangents: Vec::new(),
            };
            m.tangents = m.compute_tangents();
            m
        },
        material_index,
    })
}

// ── Camera conversion ──────────────────────────────────────────────────────

fn convert_camera(camera: gltf::Camera<'_>) -> Camera3D {
    let (fov_y, near, far, aspect, perspective) = match camera.projection() {
        gltf::camera::Projection::Perspective(p) => (
            p.yfov(),
            p.znear(),
            p.zfar().unwrap_or(1000.0),
            p.aspect_ratio().unwrap_or(1.6),
            true,
        ),
        gltf::camera::Projection::Orthographic(_) => {
            (std::f32::consts::FRAC_PI_4, 0.01, 1000.0, 1.6, false)
        }
    };

    Camera3D {
        position: Vec3::ZERO,
        target: Vec3::NEG_Z,
        up: Vec3::Y,
        fov_y,
        near,
        far,
        perspective,
        aspect,
    }
}

// ── Node hierarchy ─────────────────────────────────────────────────────────

fn build_hierarchy(
    scene: gltf::Scene<'_>,
    mesh_prim_offset: &[usize],
    cameras: &[Camera3D],
) -> Vec<Node3D> {
    let mut nodes = Vec::new();

    for root_node in scene.nodes() {
        flatten_node(root_node, None, mesh_prim_offset, cameras, &mut nodes);
    }

    nodes
}

fn flatten_node(
    node: gltf::Node<'_>,
    parent: Option<usize>,
    mesh_prim_offset: &[usize],
    cameras: &[Camera3D],
    nodes: &mut Vec<Node3D>,
) -> usize {
    let index = nodes.len();

    // Convert transform
    let transform = match node.transform() {
        GltfTransform::Matrix { matrix } => {
            let m = glam::Mat4::from_cols_array_2d(&matrix);
            let (scale, rotation, translation) = m.to_scale_rotation_translation();
            Transform3D {
                position: translation,
                rotation,
                scale,
            }
        }
        GltfTransform::Decomposed {
            translation,
            rotation,
            scale,
        } => Transform3D {
            position: Vec3::from(translation),
            rotation: Quat::from_array(rotation),
            scale: Vec3::from(scale),
        },
    };

    // Map glTF mesh to CVKG mesh index
    let mesh_index = node.mesh().map(|m| {
        let gltf_idx = m.index();
        // The first primitive of this glTF mesh starts at mesh_prim_offset[gltf_idx]
        mesh_prim_offset.get(gltf_idx).copied().unwrap_or(0)
    });

    // Map glTF camera to CVKG camera index
    let camera_index = node.camera().map(|c| {
        // The glTF camera has an index in the document's camera list
        c.index()
    });

    let name = node.name().unwrap_or("").to_string();

    let children: Vec<usize> = node
        .children()
        .map(|child| flatten_node(child, Some(index), mesh_prim_offset, cameras, nodes))
        .collect();

    nodes.push(Node3D {
        index,
        parent,
        children,
        name,
        transform,
        mesh_index,
        camera_index,
    });

    index
}
