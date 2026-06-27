/// A 3D mesh containing vertex and index data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
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
            meshes.push(Mesh {
                vertices,
                normals,
                indices: mesh.indices,
            });
        }
        Ok(meshes)
    }
    pub fn from_stl(data: &[u8]) -> anyhow::Result<Self> {
        let mut cursor = std::io::Cursor::new(data);
        let stl = stl_io::read_stl(&mut cursor)?;
        let vertices: Vec<[f32; 3]> = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let mut indices = Vec::new();
        for face in stl.faces {
            indices.push(face.vertices[0] as u32);
            indices.push(face.vertices[1] as u32);
            indices.push(face.vertices[2] as u32);
        }
        let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];
        Ok(Mesh {
            vertices,
            normals,
            indices,
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3D TYPES -- Phase 1: Camera, Transform, and 2.5D layer support
// ══════════════════════════════════════════════════════════════════════════

/// A 3D transform: position, rotation (quaternion), and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Convert this transform to a 4x4 model matrix.
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material3D {
    /// Base color (RGBA).
    pub base_color: [f32; 4],
    /// Metallic factor (0 = dielectric, 1 = metallic).
    pub metallic: f32,
    /// Roughness factor (0 = mirror, 1 = fully diffuse).
    pub roughness: f32,
    /// Emissive color (RGB) for self-illumination.
    pub emissive: [f32; 3],
    /// Opacity (0 = transparent, 1 = opaque).
    pub opacity: f32,
}

impl Default for Material3D {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            opacity: 1.0,
        }
    }
}

impl Material3D {
    /// Create a simple unlit material with just a color.
    pub fn unlit(color: [f32; 4]) -> Self {
        Self {
            base_color: color,
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
        }
    }

    /// Create a metallic material.
    pub fn metallic(color: [f32; 4], roughness: f32) -> Self {
        Self {
            base_color: color,
            metallic: 1.0,
            roughness: roughness.clamp(0.0, 1.0),
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
        }
    }
}
