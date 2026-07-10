use glam::{Mat4, Vec3};

/// Six frustum planes: [left, right, bottom, top, near, far].
/// Each plane is (nx, ny, nz, d) where dot(n, p) + d = 0.
/// Normal points inward (toward the visible region).
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [[f32; 4]; 6],
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix (column-major, right-handed).
    /// The VP matrix transforms world-space points to clip space.
    /// Planes are returned in left, right, bottom, top, near, far order.
    pub fn from_view_projection(vp: &Mat4) -> Self {
        let m = vp.to_cols_array_2d(); // m[row][col]

        // Left:   col3 + col0
        // Right:  col3 - col0
        // Bottom: col3 + col1
        // Top:    col3 - col1
        // Near:   col3 + col2
        // Far:    col3 - col2
        let left = [
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        ];
        let right = [
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        ];
        let bottom = [
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        ];
        let top = [
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        ];
        let near = [
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        ];
        let far = [
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        ];

        // Normalize planes (optional but useful for sphere test)
        let planes = [left, right, bottom, top, near, far].map(normalize_plane);

        Self { planes }
    }

    /// Test if an AABB intersects the frustum.
    /// Returns true if the box is potentially visible (inside or intersecting).
    pub fn intersects_aabb(&self, center: Vec3, half_extents: Vec3) -> bool {
        for plane in self.planes {
            let n = Vec3::new(plane[0], plane[1], plane[2]);
            let d = plane[3];

            // Find the positive vertex (furthest along plane normal)
            let p = Vec3::new(
                if n.x >= 0.0 {
                    center.x + half_extents.x
                } else {
                    center.x - half_extents.x
                },
                if n.y >= 0.0 {
                    center.y + half_extents.y
                } else {
                    center.y - half_extents.y
                },
                if n.z >= 0.0 {
                    center.z + half_extents.z
                } else {
                    center.z - half_extents.z
                },
            );

            // If positive vertex is outside (dot < 0), box is culled
            if n.dot(p) + d < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere intersects the frustum.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in self.planes {
            let n = Vec3::new(plane[0], plane[1], plane[2]);
            let d = plane[3];
            if n.dot(center) + d < -radius {
                return false;
            }
        }
        true
    }
}

fn normalize_plane(p: [f32; 4]) -> [f32; 4] {
    let n = Vec3::new(p[0], p[1], p[2]);
    let len = n.length();
    if len == 0.0 {
        return p;
    }
    [p[0] / len, p[1] / len, p[2] / len, p[3] / len]
}
