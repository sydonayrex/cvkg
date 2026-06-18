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
