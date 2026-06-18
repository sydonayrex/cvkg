/// Opaque handle to a GPU resource (Texture, Buffer, etc.) managed by the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

/// P1-20 fix: resource access type for hazard analysis.
///
/// Each pass declares which resources it reads and which it writes.
/// The graph planner can then detect:
///   - Write-after-write (WAW): two passes write the same resource
///     without a barrier between them
///   - Read-after-write (RAW): a pass reads a resource that was
///     written by an earlier pass without a barrier
///   - Write-after-read (WAR): a pass writes a resource that is
///     still being read by an earlier pass
///
/// Hazards are detected at graph compilation time, not at runtime,
/// so they don't add frame-time overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    Read,
    Write,
}

impl ResourceAccess {
    /// Returns true if this access conflicts with another in the
    /// sense that they cannot be reordered. The caller is responsible
    /// for inserting the appropriate barrier (wgpu render pass
    /// boundary, compute pass boundary, or queue.submit) between
    /// the two accesses.
    pub fn conflicts_with(self, other: ResourceAccess) -> bool {
        // The conflict table:
        //   Read  + Read  = false (parallel reads are fine)
        //   Read  + Write = true  (WAR hazard)
        //   Write + Read  = true  (RAW hazard)
        //   Write + Write = true  (WAW hazard)
        match (self, other) {
            (ResourceAccess::Read, ResourceAccess::Read) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Image {
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
    },
    Buffer {
        size: u64,
        usage: wgpu::BufferUsages,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLifetime {
    /// Destroyed automatically at the end of the frame.
    Frame,
    /// Lives until explicitly destroyed or window is closed.
    Persistent,
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub label: Option<String>,
    pub kind: ResourceKind,
    pub lifetime: ResourceLifetime,
}

#[cfg(test)]
mod p1_20_hazard_tracking_tests {
    use super::ResourceAccess;

    #[test]
    fn read_read_does_not_conflict() {
        // P1-20: two parallel reads of the same resource are safe.
        assert!(!ResourceAccess::Read.conflicts_with(ResourceAccess::Read));
    }

    #[test]
    fn read_write_conflicts_war() {
        // P1-20: write-after-read is a hazard (the write could
        // clobber the read result if not synchronized).
        assert!(ResourceAccess::Read.conflicts_with(ResourceAccess::Write));
    }

    #[test]
    fn write_read_conflicts_raw() {
        // P1-20: read-after-write is a hazard (the read could
        // see stale data if not synchronized).
        assert!(ResourceAccess::Write.conflicts_with(ResourceAccess::Read));
    }

    #[test]
    fn write_write_conflicts_waw() {
        // P1-20: write-after-write is a hazard (one write could
        // overwrite the other's result).
        assert!(ResourceAccess::Write.conflicts_with(ResourceAccess::Write));
    }

    #[test]
    fn conflict_table_is_symmetric() {
        // P1-20: the conflict relation must be symmetric. If A
        // conflicts with B, B must conflict with A.
        for a in [ResourceAccess::Read, ResourceAccess::Write] {
            for b in [ResourceAccess::Read, ResourceAccess::Write] {
                assert_eq!(
                    a.conflicts_with(b),
                    b.conflicts_with(a),
                    "conflict table not symmetric for {a:?} vs {b:?}"
                );
            }
        }
    }
}

// =========================================================================
// P1-25: CPU/Shader Material ID drift detection
// =========================================================================

#[cfg(test)]
mod p1_25_material_id_consistency_tests {
    // Reference values from the Rust `material_id` module in
    // cvkg-render-gpu/src/renderer.rs. If you change the Rust
    // constants, you MUST also update the WGSL shader files (and
    // these expected values).
    const RUST_GLASS: u32 = 7;
    const RUST_DROP_SHADOW: u32 = 18;
    const RUST_MESH_3D: u32 = 21;

    /// Scan a WGSL file for `Nu` literal patterns and return the
    /// set of material_id values it references.
    fn scan_wgsl_for_material_ids(source: &str) -> std::collections::HashSet<u32> {
        let mut ids = std::collections::HashSet::new();
        let bytes = source.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_digit() {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str = std::str::from_utf8(&bytes[start..i]).unwrap();
                if let Ok(n) = num_str.parse::<u32>() {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j] == b'u' {
                        ids.insert(n);
                    }
                }
            } else {
                i += 1;
            }
        }
        ids
    }

    #[test]
    fn glass_id_appears_in_wgsl() {
        let source = include_str!("../shaders/material_opaque.wgsl");
        let ids = scan_wgsl_for_material_ids(source);
        assert!(
            ids.contains(&RUST_GLASS),
            "WGSL must reference material_id {RUST_GLASS} (GLASS)"
        );
    }

    #[test]
    fn drop_shadow_id_appears_in_wgsl() {
        let source = include_str!("../shaders/material_opaque.wgsl");
        let ids = scan_wgsl_for_material_ids(source);
        assert!(
            ids.contains(&RUST_DROP_SHADOW),
            "WGSL must reference material_id {RUST_DROP_SHADOW} (DROP_SHADOW)"
        );
    }

    #[test]
    fn mesh_3d_id_appears_in_wgsl() {
        let source = include_str!("../shaders/material_opaque.wgsl");
        let ids = scan_wgsl_for_material_ids(source);
        assert!(
            ids.contains(&RUST_MESH_3D),
            "WGSL must reference material_id {RUST_MESH_3D} (MESH_3D)"
        );
    }

    #[test]
    fn scanner_works() {
        let src = "if (in.material_id == 18u) { } else if (in.material_id == 21u) { }";
        let ids = scan_wgsl_for_material_ids(src);
        assert!(ids.contains(&18));
        assert!(ids.contains(&21));
        let src2 = "let x = 18;";
        let ids2 = scan_wgsl_for_material_ids(src2);
        assert!(!ids2.contains(&18));
    }
}
