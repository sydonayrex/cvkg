use std::fmt;

/// Errors that can occur when parsing STL files.
#[derive(Debug, Clone, PartialEq)]
pub enum StlError {
    /// I/O error from the underlying reader.
    Io(String),
    /// File is too truncated to contain valid STL data.
    Truncated,
    /// Data is not valid ASCII STL (should try binary fallback).
    NotAscii,
    /// Invalid or unparseable ASCII content.
    InvalidAscii(String),
    /// Face with != 3 vertices encountered.
    NonTriangleFace,
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::Truncated => write!(f, "unexpected end of file"),
            Self::NotAscii => write!(f, "not ASCII STL"),
            Self::InvalidAscii(msg) => write!(f, "invalid ASCII STL: {msg}"),
            Self::NonTriangleFace => write!(f, "face with != 3 vertices"),
        }
    }
}

impl std::error::Error for StlError {}

impl From<std::io::Error> for StlError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// STL format variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFormat {
    /// ASCII STL format.
    Ascii,
    /// Binary STL format.
    Binary,
}

/// A parsed STL mesh with shared vertices and normals.
#[derive(Debug, Clone, PartialEq)]
pub struct StlMesh {
    /// Deduplicated vertices (unique positions).
    pub vertices: Vec<[f32; 3]>,
    /// Per-vertex normals (either from file or computed from faces).
    pub normals: Vec<[f32; 3]>,
    /// Triangle vertex indices (every 3 = one face).
    pub indices: Vec<u32>,
}

impl StlMesh {
    /// Creates an empty mesh.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Compute per-vertex normals from face geometry, replacing file normals.
    ///
    /// For each triangle, computes the cross product of two edges and averages
    /// the resulting normal across all shared vertices.
    pub fn compute_normals(&mut self) {
        self.normals = vec![[0.0, 0.0, 0.0]; self.vertices.len()];
        let mut counts = vec![0u32; self.vertices.len()];

        for tri in self.indices.chunks_exact(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;
            let v0 = self.vertices[i0];
            let v1 = self.vertices[i1];
            let v2 = self.vertices[i2];

            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            let mag = (nx * nx + ny * ny + nz * nz).sqrt();
            let (nx, ny, nz) = if mag < 1e-15 {
                (0.0, 0.0, 0.0)
            } else {
                (nx / mag, ny / mag, nz / mag)
            };

            for &i in &[i0, i1, i2] {
                self.normals[i][0] += nx;
                self.normals[i][1] += ny;
                self.normals[i][2] += nz;
                counts[i] += 1;
            }
        }

        // Average accumulated normals
        for (i, n) in self.normals.iter_mut().enumerate() {
            if counts[i] > 0 {
                n[0] /= counts[i] as f32;
                n[1] /= counts[i] as f32;
                n[2] /= counts[i] as f32;
                let mag = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if mag > 1e-15 {
                    n[0] /= mag;
                    n[1] /= mag;
                    n[2] /= mag;
                }
            }
        }
    }
}

impl Default for StlMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect STL format from the first 80 bytes.
///
/// Returns `Some(StlFormat::Ascii)` if the header starts with "solid"
/// and contains only ASCII bytes, `Some(StlFormat::Binary)` otherwise,
/// or `None` if fewer than 80 bytes are available.
pub fn detect_format(header: &[u8]) -> Option<StlFormat> {
    if header.len() < 80 {
        return None;
    }
    let starts_with_solid = header[..5].eq_ignore_ascii_case(b"solid");
    let all_ascii = header[..80].iter().all(|&b| b < 128);
    if starts_with_solid && all_ascii {
        Some(StlFormat::Ascii)
    } else {
        Some(StlFormat::Binary)
    }
}
