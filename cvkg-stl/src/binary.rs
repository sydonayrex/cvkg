// Binary STL parser with vertex deduplication and normal averaging.
use crate::error::{StlError, StlMesh};
use std::collections::HashMap;
use std::io::Read;

/// Maximum number of triangles allowed in a binary STL file.
/// Prevents OOM from malicious files with huge triangle counts.
const MAX_STL_TRIANGLES: u32 = 10_000_000;

/// Validate that an f32 is finite (not NaN or ±Inf).
fn validate_f32(value: f32, context: &str) -> Result<f32, StlError> {
    if value.is_nan() || value.is_infinite() {
        Err(StlError::InvalidFloat {
            value,
            context: context.to_string(),
        })
    } else {
        Ok(value)
    }
}

/// Parse a binary STL file.
///
/// Format:
///   80 bytes: header
///   4 bytes:  u32 LE number of triangles
///   50 bytes per triangle:
///     12 bytes: normal vector (3 × f32 LE)
///     36 bytes: 3 vertices (9 × f32 LE)
///     2 bytes:  attribute byte count (u16 LE, usually 0)
pub fn parse<R: Read>(mut reader: R) -> Result<StlMesh, StlError> {
    // Read 80-byte header
    let mut header = [0u8; 80];
    reader.read_exact(&mut header)?;

    // Read triangle count
    let mut count_buf = [0u8; 4];
    reader.read_exact(&mut count_buf)?;
    let num_triangles = u32::from_le_bytes(count_buf);

    if num_triangles > MAX_STL_TRIANGLES {
        return Err(StlError::TooManyTriangles {
            count: num_triangles,
            max: MAX_STL_TRIANGLES,
        });
    }

    if num_triangles == 0 {
        return Ok(StlMesh::new());
    }

    // Temporary storage: raw per-face data before dedup
    let mut raw_normals: Vec<[f32; 3]> = Vec::with_capacity(num_triangles as usize);
    let mut raw_vertices: Vec<[f32; 3]> = Vec::with_capacity(num_triangles as usize * 3);

    let mut triangle_buf = [0u8; 50];

    for tri_idx in 0..num_triangles {
        reader.read_exact(&mut triangle_buf)?;

        // Parse normal (bytes 0-11)
        let nx = validate_f32(
            f32::from_le_bytes([triangle_buf[0], triangle_buf[1], triangle_buf[2], triangle_buf[3]]),
            &format!("triangle {tri_idx} normal.x"),
        )?;
        let ny = validate_f32(
            f32::from_le_bytes([triangle_buf[4], triangle_buf[5], triangle_buf[6], triangle_buf[7]]),
            &format!("triangle {tri_idx} normal.y"),
        )?;
        let nz = validate_f32(
            f32::from_le_bytes([triangle_buf[8], triangle_buf[9], triangle_buf[10], triangle_buf[11]]),
            &format!("triangle {tri_idx} normal.z"),
        )?;
        let normal = [nx, ny, nz];

        // Parse 3 vertices (bytes 12-47)
        let mut base = 12;
        for vert_idx in 0..3 {
            let vx = validate_f32(
                f32::from_le_bytes([
                    triangle_buf[base],
                    triangle_buf[base + 1],
                    triangle_buf[base + 2],
                    triangle_buf[base + 3],
                ]),
                &format!("triangle {tri_idx} vertex[{vert_idx}].x"),
            )?;
            let vy = validate_f32(
                f32::from_le_bytes([
                    triangle_buf[base + 4],
                    triangle_buf[base + 5],
                    triangle_buf[base + 6],
                    triangle_buf[base + 7],
                ]),
                &format!("triangle {tri_idx} vertex[{vert_idx}].y"),
            )?;
            let vz = validate_f32(
                f32::from_le_bytes([
                    triangle_buf[base + 8],
                    triangle_buf[base + 9],
                    triangle_buf[base + 10],
                    triangle_buf[base + 11],
                ]),
                &format!("triangle {tri_idx} vertex[{vert_idx}].z"),
            )?;
            raw_normals.push(normal);
            raw_vertices.push([vx, vy, vz]);
            base += 12;
        }
        // Attribute byte count (bytes 48-49) — discarded
    }

    // Deduplicate vertices using raw byte keys
    let mut vertex_map: HashMap<[u8; 12], u32> = HashMap::new();
    let mut mesh = StlMesh::new();
    mesh.vertices.reserve(raw_vertices.len() / 2); // estimate
    mesh.normals.reserve(raw_vertices.len() / 2);
    mesh.indices.reserve(raw_vertices.len());

    for (i, vertex) in raw_vertices.iter().enumerate() {
        let key = [
            vertex[0].to_le_bytes()[0], vertex[0].to_le_bytes()[1],
            vertex[0].to_le_bytes()[2], vertex[0].to_le_bytes()[3],
            vertex[1].to_le_bytes()[0], vertex[1].to_le_bytes()[1],
            vertex[1].to_le_bytes()[2], vertex[1].to_le_bytes()[3],
            vertex[2].to_le_bytes()[0], vertex[2].to_le_bytes()[1],
            vertex[2].to_le_bytes()[2], vertex[2].to_le_bytes()[3],
        ];

        let idx = match vertex_map.get(&key) {
            Some(&existing_idx) => {
                // Accumulate normal for existing vertex
                let n = &raw_normals[i];
                mesh.normals[existing_idx as usize][0] += n[0];
                mesh.normals[existing_idx as usize][1] += n[1];
                mesh.normals[existing_idx as usize][2] += n[2];
                existing_idx
            }
            None => {
                let new_idx = mesh.vertices.len() as u32;
                vertex_map.insert(key, new_idx);
                mesh.vertices.push(*vertex);
                let n = &raw_normals[i];
                mesh.normals.push([n[0], n[1], n[2]]);
                new_idx
            }
        };
        mesh.indices.push(idx);
    }

    // Normalize accumulated normals
    for normal in mesh.normals.iter_mut() {
        let mag = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if mag > 1e-15 {
            normal[0] /= mag;
            normal[1] /= mag;
            normal[2] /= mag;
        }
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal binary STL file with the given triangle count and data.
    fn build_stl(num_triangles: u32, triangles: &[[u8; 50]]) -> Vec<u8> {
        let mut data = Vec::with_capacity(80 + 4 + triangles.len() * 50);
        // 80-byte header
        data.extend_from_slice(&[0u8; 80]);
        // triangle count
        data.extend_from_slice(&num_triangles.to_le_bytes());
        // triangle data
        for tri in triangles {
            data.extend_from_slice(tri);
        }
        data
    }

    /// Build a single triangle with given normal and vertices.
    fn make_triangle(normal: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [u8; 50] {
        let mut buf = [0u8; 50];
        // normal
        buf[0..4].copy_from_slice(&normal[0].to_le_bytes());
        buf[4..8].copy_from_slice(&normal[1].to_le_bytes());
        buf[8..12].copy_from_slice(&normal[2].to_le_bytes());
        // vertex 0
        buf[12..16].copy_from_slice(&v0[0].to_le_bytes());
        buf[16..20].copy_from_slice(&v0[1].to_le_bytes());
        buf[20..24].copy_from_slice(&v0[2].to_le_bytes());
        // vertex 1
        buf[24..28].copy_from_slice(&v1[0].to_le_bytes());
        buf[28..32].copy_from_slice(&v1[1].to_le_bytes());
        buf[32..36].copy_from_slice(&v1[2].to_le_bytes());
        // vertex 2
        buf[36..40].copy_from_slice(&v2[0].to_le_bytes());
        buf[40..44].copy_from_slice(&v2[1].to_le_bytes());
        buf[44..48].copy_from_slice(&v2[2].to_le_bytes());
        // attribute byte count (2 bytes, usually 0)
        buf[48..50].copy_from_slice(&0u16.to_le_bytes());
        buf
    }

    #[test]
    fn test_stl_rejects_excessive_triangle_count() {
        // Build a file that claims u32::MAX triangles but has no actual data.
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 80]); // header
        data.extend_from_slice(&u32::MAX.to_le_bytes()); // triangle count

        let result = parse(&data[..]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            StlError::TooManyTriangles { count, max } => {
                assert_eq!(count, u32::MAX);
                assert_eq!(max, MAX_STL_TRIANGLES);
            }
            other => panic!("Expected TooManyTriangles, got {:?}", other),
        }
    }

    #[test]
    fn test_stl_rejects_nan_values() {
        // Build a file with 1 triangle where vertex[0].x is NaN
        let nan = f32::NAN;
        let tri = make_triangle(
            [0.0, 0.0, 1.0],
            [nan, 0.0, 0.0], // NaN in vertex
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let data = build_stl(1, &[tri]);

        let result = parse(&data[..]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            StlError::InvalidFloat { value, context } => {
                assert!(value.is_nan());
                assert!(context.contains("vertex"));
            }
            other => panic!("Expected InvalidFloat, got {:?}", other),
        }
    }

    #[test]
    fn test_stl_rejects_inf_values() {
        // Build a file with 1 triangle where normal.y is +Inf
        let inf = f32::INFINITY;
        let tri = make_triangle(
            [0.0, inf, 0.0], // Inf in normal
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let data = build_stl(1, &[tri]);

        let result = parse(&data[..]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            StlError::InvalidFloat { value, context } => {
                assert!(value.is_infinite());
                assert!(context.contains("normal"));
            }
            other => panic!("Expected InvalidFloat, got {:?}", other),
        }
    }

    #[test]
    fn test_stl_accepts_valid_triangle() {
        let tri = make_triangle(
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let data = build_stl(1, &[tri]);

        let result = parse(&data[..]);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }
}
