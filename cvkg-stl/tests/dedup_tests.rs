//! Vertex deduplication and normal computation tests — P3

#[cfg(test)]
mod tests {
    /// Generate binary STL for a unit cube (12 triangles, should dedup to 8 vertices).
    fn generate_unit_cube_binary() -> Vec<u8> {
        let mut data = Vec::new();
        let mut header = vec![0u8; 80];
        header[..6].copy_from_slice(b"solid ");
        data.extend_from_slice(&header);
        data.extend_from_slice(&12u32.to_le_bytes());

        let corners = [
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ];

        let triangles: [[u32; 3]; 12] = [
            [4, 5, 6], [4, 6, 7], // front
            [1, 0, 3], [1, 3, 2], // back
            [3, 7, 6], [3, 6, 2], // top
            [0, 4, 5], [0, 5, 1], // bottom
            [5, 6, 2], [5, 2, 1], // right
            [0, 3, 7], [0, 7, 4], // left
        ];

        let normals: [[f32; 3]; 12] = [
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        ];

        for (i, tri) in triangles.iter().enumerate() {
            for &comp in &normals[i] {
                data.extend_from_slice(&comp.to_le_bytes());
            }
            for &vi in tri {
                for &comp in &corners[vi as usize] {
                    data.extend_from_slice(&(comp as f32).to_le_bytes());
                }
            }
            data.extend_from_slice(&0u16.to_le_bytes());
        }

        data
    }

    #[test]
    fn test_cube_dedups_to_8_vertices() {
        let data = generate_unit_cube_binary();
        let mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");
        assert_eq!(mesh.vertices.len(), 8, "cube should have 8 unique corners");
        assert_eq!(mesh.indices.len(), 36, "12 triangles × 3 indices");
    }

    #[test]
    fn test_compute_normals_from_geometry() {
        // Generate a cube with zero normals in the file
        let mut data = Vec::new();
        let mut header = vec![0u8; 80];
        header[..6].copy_from_slice(b"solid ");
        data.extend_from_slice(&header);
        data.extend_from_slice(&2u32.to_le_bytes()); // 2 triangles

        // Two triangles forming a quad in XY plane (normal should be +Z)
        for _ in 0..2 {
            // Zero normal
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            // Vertices
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            // Attribute
            data.extend_from_slice(&0u16.to_le_bytes());
        }

        let mut mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");
        mesh.compute_normals();

        // All normals should be [0, 0, 1] or [0, 0, -1]
        for n in &mesh.normals {
            assert!(
                n[0].abs() < 0.001 && n[1].abs() < 0.001 && n[2].abs() > 0.99,
                "expected normal along Z axis, got {:?}",
                n
            );
        }
    }

    #[test]
    fn test_file_normals_averaged_across_shared_vertices() {
        let data = generate_unit_cube_binary();
        let mesh = cvkg_stl::parse_bytes(&data).expect("parse should succeed");

        // After dedup, every vertex normal should be normalized (length ≈ 1)
        for (i, n) in mesh.normals.iter().enumerate() {
            let mag = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(
                (mag - 1.0).abs() < 0.01,
                "vertex {} normal should be normalized, got mag={:?}",
                i,
                mag
            );
        }

        // Corner vertex (shared by 3 faces) should have a diagonal normal
        // rather than any single face normal
        let corner = mesh.vertices.iter().position(|v| {
            (v[0] - 0.5).abs() < 0.001 && (v[1] - 0.5).abs() < 0.001 && (v[2] - 0.5).abs() < 0.001
        });
        if let Some(idx) = corner {
            let n = mesh.normals[idx];
            // Corner normal should be roughly [0.577, 0.577, 0.577] (normalized [1,1,1])
            assert!(
                n[0] > 0.3 && n[1] > 0.3 && n[2] > 0.3,
                "corner normal should be diagonal, got {:?}",
                n
            );
        }
    }
}
