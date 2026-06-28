// Binary STL parser with vertex deduplication and normal averaging.
use crate::error::{StlError, StlMesh};
use std::collections::HashMap;
use std::io::Read;

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

    if num_triangles == 0 {
        return Ok(StlMesh::new());
    }

    // Temporary storage: raw per-face data before dedup
    let mut raw_normals: Vec<[f32; 3]> = Vec::with_capacity(num_triangles as usize);
    let mut raw_vertices: Vec<[f32; 3]> = Vec::with_capacity(num_triangles as usize * 3);

    let mut triangle_buf = [0u8; 50];

    for _ in 0..num_triangles {
        reader.read_exact(&mut triangle_buf)?;

        // Parse normal (bytes 0-11)
        let nx = f32::from_le_bytes([triangle_buf[0], triangle_buf[1], triangle_buf[2], triangle_buf[3]]);
        let ny = f32::from_le_bytes([triangle_buf[4], triangle_buf[5], triangle_buf[6], triangle_buf[7]]);
        let nz = f32::from_le_bytes([triangle_buf[8], triangle_buf[9], triangle_buf[10], triangle_buf[11]]);
        let normal = [nx, ny, nz];

        // Parse 3 vertices (bytes 12-47)
        let mut base = 12;
        for _ in 0..3 {
            let vx = f32::from_le_bytes([
                triangle_buf[base],
                triangle_buf[base + 1],
                triangle_buf[base + 2],
                triangle_buf[base + 3],
            ]);
            let vy = f32::from_le_bytes([
                triangle_buf[base + 4],
                triangle_buf[base + 5],
                triangle_buf[base + 6],
                triangle_buf[base + 7],
            ]);
            let vz = f32::from_le_bytes([
                triangle_buf[base + 8],
                triangle_buf[base + 9],
                triangle_buf[base + 10],
                triangle_buf[base + 11],
            ]);
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
