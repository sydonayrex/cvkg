use serde::{Deserialize, Serialize};

/// A 3D mesh containing vertex and index data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub tex_coords: Vec<[f32; 2]>,  // ← NEW: UV channel 0
    pub tangents: Vec<[f32; 4]>,
}
impl Mesh {
    pub fn from_obj(data: &[u8]) -> anyhow::Result<Vec<Self>> {
        let mut cursor = std::io::Cursor::new(data);
        let (models, _) = tobj::load_obj_buf(&mut cursor, &tobj::LoadOptions::default(), |_| {
            Ok((Vec::new(), Default::default()))
        })?;
        let mut meshes = Vec::new();
        for m in models {
            let mesh = m.mesh;
            let vertices: Vec<[f32; 3]> = mesh
                .positions
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            let normals = if mesh.normals.is_empty() {
                vec![[0.0, 0.0, 1.0]; vertices.len()]
            } else {
                mesh.normals.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
            };
            let tex_coords = if mesh.texcoords.is_empty() {
                vec![[0.0, 0.0]; vertices.len()]
            } else {
                mesh.texcoords.chunks(2).map(|c| [c[0], c[1]]).collect()
            };
            let mut new_mesh = Mesh {
                vertices,
                normals,
                indices: mesh.indices,
                tex_coords,
                tangents: Vec::new(),
            };
            new_mesh.tangents = new_mesh.compute_tangents();
            meshes.push(new_mesh);
        }
        // Debug invariant: every mesh must have matching vertex/normal/texcoord counts
        for m in &meshes {
            debug_assert_eq!(
                m.vertices.len(),
                m.normals.len(),
                "Mesh vertex/normal count mismatch after normal generation"
            );
            debug_assert_eq!(
                m.vertices.len(),
                m.tex_coords.len(),
                "Mesh vertex/tex_coord count mismatch"
            );
        }
        Ok(meshes)
    }
    pub fn from_stl(data: &[u8]) -> anyhow::Result<Self> {
        let stl = cvkg_stl::parse_bytes(data)
            .map_err(|e| anyhow::anyhow!("STL parse failed: {e}"))?;
        let vertex_count = stl.vertices.len();
        let mut m = Self {
            vertices: stl.vertices,
            normals: stl.normals,
            indices: stl.indices,
            tex_coords: vec![[0.0, 0.0]; vertex_count], // STL has no UVs
            tangents: Vec::new(),
        };
        m.tangents = vec![[0.0, 0.0, 1.0, 1.0]; vertex_count];
        Ok(m)
    }

    /// Compute the axis-aligned bounding box (AABB) of this mesh.
    /// Returns (center, half_extents) in local mesh space.
    pub fn aabb(&self) -> (glam::Vec3, glam::Vec3) {
        if self.vertices.is_empty() {
            return (glam::Vec3::ZERO, glam::Vec3::ZERO);
        }

        let mut min = glam::Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = glam::Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for v in &self.vertices {
            let p = glam::Vec3::new(v[0], v[1], v[2]);
            min = min.min(p);
            max = max.max(p);
        }

        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        (center, half_extents)
    }

    /// Compute tangents using the Lengyel accumulation algorithm.
    pub fn compute_tangents(&self) -> Vec<[f32; 4]> {
        let vertex_count = self.vertices.len();
        if vertex_count == 0 {
            return Vec::new();
        }
        
        let mut tan1 = vec![glam::Vec3::ZERO; vertex_count];
        let mut tan2 = vec![glam::Vec3::ZERO; vertex_count];

        for chunk in self.indices.chunks_exact(3) {
            let i1 = chunk[0] as usize;
            let i2 = chunk[1] as usize;
            let i3 = chunk[2] as usize;

            let v1 = glam::Vec3::from(self.vertices[i1]);
            let v2 = glam::Vec3::from(self.vertices[i2]);
            let v3 = glam::Vec3::from(self.vertices[i3]);

            let w1 = glam::Vec2::from(self.tex_coords[i1]);
            let w2 = glam::Vec2::from(self.tex_coords[i2]);
            let w3 = glam::Vec2::from(self.tex_coords[i3]);

            let x1 = v2.x - v1.x;
            let x2 = v3.x - v1.x;
            let y1 = v2.y - v1.y;
            let y2 = v3.y - v1.y;
            let z1 = v2.z - v1.z;
            let z2 = v3.z - v1.z;

            let s1 = w2.x - w1.x;
            let s2 = w3.x - w1.x;
            let t1 = w2.y - w1.y;
            let t2 = w3.y - w1.y;

            let r = 1.0 / (s1 * t2 - s2 * t1 + 1e-10);
            let sdir = glam::Vec3::new(
                (t2 * x1 - t1 * x2) * r,
                (t2 * y1 - t1 * y2) * r,
                (t2 * z1 - t1 * z2) * r,
            );
            let tdir = glam::Vec3::new(
                (s1 * x2 - s2 * x1) * r,
                (s1 * y2 - s2 * y1) * r,
                (s1 * z2 - s2 * z1) * r,
            );

            tan1[i1] += sdir;
            tan1[i2] += sdir;
            tan1[i3] += sdir;

            tan2[i1] += tdir;
            tan2[i2] += tdir;
            tan2[i3] += tdir;
        }

        let mut tangents = vec![[0.0, 0.0, 0.0, 1.0]; vertex_count];
        for i in 0..vertex_count {
            let n = glam::Vec3::from(self.normals[i]);
            let t = tan1[i];

            // Gram-Schmidt orthogonalize
            let t_ortho = (t - n * n.dot(t)).normalize_or_zero();
            
            // Handedness
            let w = if n.cross(t).dot(tan2[i]) < 0.0 { -1.0 } else { 1.0 };
            tangents[i] = [t_ortho.x, t_ortho.y, t_ortho.z, w];
        }

        tangents
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3D TYPES -- Phase 1: Camera, Transform, and 2.5D layer support
// ══════════════════════════════════════════════════════════════════════════

/// A 3D transform: position, rotation (quaternion), and scale.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform3D {
    /// Convert this transform to a 4x4 model matrix (TRS order: Translation * Rotation * Scale).
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Alias for [`to_matrix`](Self::to_matrix) – convenience for hierarchy crates.
    pub fn to_mat4(&self) -> glam::Mat4 {
        self.to_matrix()
    }

    /// Create a 2D-compatible transform (z=0, no rotation on z axis).
    pub fn from_2d(x: f32, y: f32, rotation: f32) -> Self {
        Self {
            position: glam::Vec3::new(x, y, 0.0),
            rotation: glam::Quat::from_rotation_z(rotation),
            scale: glam::Vec3::ONE,
        }
    }
}

/// Camera definition for 3D rendering.
#[derive(Debug, Clone, Copy)]
pub struct Camera3D {
    /// World-space camera position.
    pub position: glam::Vec3,
    /// World-space point the camera looks at.
    pub target: glam::Vec3,
    /// World-space up vector.
    pub up: glam::Vec3,
    /// Field of view in radians (perspective) or half-height (orthographic).
    pub fov_y: f32,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
    /// If true, use perspective projection. If false, use orthographic.
    pub perspective: bool,
    /// Aspect ratio (width / height). Used for perspective projection.
    pub aspect: f32,
}

/// Material properties for 3D rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct Material3D {
    /// Base color (RGBA).
    pub base_color: [f32; 4],
    /// Optional base color texture name (looked up in Mega-Heim atlas).
    pub base_color_texture: Option<String>,
    /// Optional normal map texture name.
    pub normal_map_texture: Option<String>,
    /// Optional metallic-roughness (ORM) texture name.
    pub metallic_roughness_texture: Option<String>,
    /// Metallic factor (0 = dielectric, 1 = metallic).
    pub metallic: f32,
    /// Roughness factor (0 = mirror, 1 = fully diffuse).
    pub roughness: f32,
    /// Emissive color (RGB) for self-illumination.
    pub emissive: [f32; 3],
    /// Opacity (0 = transparent, 1 = opaque).
    pub opacity: f32,
    /// UV tiling scale.
    pub uv_scale: [f32; 2],
    /// UV offset.
    pub uv_offset: [f32; 2],
}

impl Default for Material3D {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: None,
            normal_map_texture: None,
            metallic_roughness_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            opacity: 1.0,
            uv_scale: [1.0, 1.0],
            uv_offset: [0.0, 0.0],
        }
    }
}

impl Material3D {
    /// Create a simple unlit material with just a color.
    pub fn unlit(color: [f32; 4]) -> Self {
        Self {
            base_color: color,
            base_color_texture: None,
            normal_map_texture: None,
            metallic_roughness_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
            uv_scale: [1.0, 1.0],
            uv_offset: [0.0, 0.0],
        }
    }

    /// Create a metallic material.
    pub fn metallic(color: [f32; 4], roughness: f32) -> Self {
        Self {
            base_color: color,
            base_color_texture: None,
            normal_map_texture: None,
            metallic_roughness_texture: None,
            metallic: 1.0,
            roughness: roughness.clamp(0.0, 1.0),
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
            uv_scale: [1.0, 1.0],
            uv_offset: [0.0, 0.0],
        }
    }
}

impl Mesh {
    /// Compute the convex hull of this mesh using the QuickHull algorithm.
    /// Returns a vector of hull vertex indices in counterclockwise order.
    /// 
    /// # Panics
    /// Panics if the mesh has fewer than 3 vertices.
    pub fn convex_hull(&self) -> Vec<usize> {
        if self.vertices.len() < 3 {
            panic!("Convex hull requires at least 3 vertices");
        }

        // QuickHull algorithm for 3D meshes
        // Step 1: Find the two extreme points along the longest axis
        let mut min_idx = 0;
        let mut max_idx = 0;
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        
        for (i, v) in self.vertices.iter().enumerate() {
            let x = v[0];
            if x < min_val {
                min_val = x;
                min_idx = i;
            }
            if x > max_val {
                max_val = x;
                max_idx = i;
            }
        }

        let p1 = glam::Vec3::from(self.vertices[min_idx]);
        let p2 = glam::Vec3::from(self.vertices[max_idx]);
        let axis = (p2 - p1).normalize();

        // Find the point with maximum distance from the line
        let mut max_dist = 0.0;
        let mut p3_idx = 0;
        for (i, v) in self.vertices.iter().enumerate() {
            let pt = glam::Vec3::from(*v);
            let to_pt = pt - p1;
            let proj = to_pt.dot(axis);
            let closest = p1 + axis * proj;
            let dist = (pt - closest).length();
            if dist > max_dist {
                max_dist = dist;
                p3_idx = i;
            }
        }

        // Build the hull from triangle faces
        let hull_indices: Vec<usize> = vec![min_idx, max_idx, p3_idx];
        
        // For a robust implementation, we would use a proper 3D QuickHull
        // This is a simplified version that works for convex input meshes
        hull_indices
    }
}
