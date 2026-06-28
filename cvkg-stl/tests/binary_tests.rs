//! Binary STL parser tests — P0

/// Generate a binary STL file for a unit cube centered at origin.
/// 12 triangles (2 per face × 6 faces), 8 unique corners.
fn generate_unit_cube_binary() -> Vec<u8> {
    let mut data = Vec::new();

    // 80-byte header
    let mut header = vec![0u8; 80];
    header[..6].copy_from_slice(b"solid ");
    data.extend_from_slice(&header);

    // Number of triangles: 12
    data.extend_from_slice(&12u32.to_le_bytes());

    // 8 corners of unit cube [-0.5, 0.5]³
    let corners = [
        [-0.5, -0.5, -0.5], // 0
        [ 0.5, -0.5, -0.5], // 1
        [ 0.5,  0.5, -0.5], // 2
        [-0.5,  0.5, -0.5], // 3
        [-0.5, -0.5,  0.5], // 4
        [ 0.5, -0.5,  0.5], // 5
        [ 0.5,  0.5,  0.5], // 6
        [-0.5,  0.5,  0.5], // 7
    ];

    // 12 triangles (CCW winding when viewed from outside)
    let triangles: [[u32; 3]; 12] = [
        // Front face (z = +0.5): 4,5,6 and 4,6,7
        [4, 5, 6], [4, 6, 7],
        // Back face (z = -0.5): 1,0,3 and 1,3,2
        [1, 0, 3], [1, 3, 2],
        // Top face (y = +0.5): 3,7,6 and 3,6,2
        [3, 7, 6], [3, 6, 2],
        // Bottom face (y = -0.5): 0,4,5 and 0,5,1
        [0, 4, 5], [0, 5, 1],
        // Right face (x = +0.5): 5,6,2 and 5,2,1
        [5, 6, 2], [5, 2, 1],
        // Left face (x = -0.5): 0,3,7 and 0,7,4
        [0, 3, 7], [0, 7, 4],
    ];

    let normals: [[f32; 3]; 12] = [
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],   // front
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],  // back
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],    // top
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],  // bottom
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],    // right
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],  // left
    ];

    for (i, tri) in triangles.iter().enumerate() {
        // Normal (3 × f32)
        for &comp in &normals[i] {
            data.extend_from_slice(&comp.to_le_bytes());
        }
        // 3 vertices (9 × f32)
        for &vi in tri {
            for &comp in &corners[vi as usize] {
                data.extend_from_slice(&(comp as f32).to_le_bytes());
            }
        }
        // Attribute byte count (u16, always 0)
        data.extend_from_slice(&0u16.to_le_bytes());
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_triangle_count_matches_header() {
        let data = generate_unit_cube_binary();
        let mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");
        assert_eq!(mesh.indices.len(), 12 * 3, "expected 12 triangles × 3 indices");
    }

    #[test]
    fn test_binary_vertex_count_with_dedup() {
        let data = generate_unit_cube_binary();
        let mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");
        // With dedup: 8 unique corners
        assert_eq!(mesh.vertices.len(), 8, "deduped cube should have 8 vertices");
    }

    #[test]
    fn test_binary_known_cube_vertices() {
        let data = generate_unit_cube_binary();
        let mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");

        // Check that vertex (0.5, 0.5, 0.5) exists
        let has_corner = mesh.vertices.iter().any(|v| {
            (v[0] - 0.5).abs() < 0.001 && (v[1] - 0.5).abs() < 0.001 && (v[2] - 0.5).abs() < 0.001
        });
        assert!(has_corner, "missing corner (0.5, 0.5, 0.5)");
    }

    #[test]
    fn test_binary_truncated_header_errors() {
        let data = vec![0u8; 80]; // Only header, no triangle count
        let result = cvkg_stl::parse_bytes(&data);
        assert!(result.is_err(), "truncated header should error");
    }

    #[test]
    fn test_binary_too_short_errors() {
        let data = vec![0u8; 40]; // Way too short
        let result = cvkg_stl::parse_bytes(&data);
        assert!(result.is_err(), "too-short data should error");
    }

    #[test]
    fn test_binary_zero_triangles_ok() {
        let mut data = vec![0u8; 80]; // Header
        data.extend_from_slice(&0u32.to_le_bytes()); // 0 triangles
        let mesh = cvkg_stl::parse_bytes(&data).expect("zero triangles should succeed");
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }
}
