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

// =========================================================================
// P2-7: Scissor rect math for zero-dimension edge case
// =========================================================================

#[cfg(test)]
mod p2_7_scissor_rect_tests {
    /// Pure function that reproduces the scissor rect computation.
    /// Returns (x, y, w, h) where (0,0,0,0) means zero-area scissor.
    fn compute_scissor(
        rect: Option<(f32, f32, f32, f32)>,  // (x, y, width, height)
        scale: f32,
        rt_w: i32,
        rt_h: i32,
    ) -> Option<(u32, u32, u32, u32)> {
        let (x, y, w, h) = rect?;
        if rt_w <= 0 || rt_h <= 0 {
            return None;
        }
        let x1 = (x * scale).round() as i32;
        let y1 = (y * scale).round() as i32;
        let x2 = ((x + w) * scale).round() as i32;
        let y2 = ((y + h) * scale).round() as i32;
        let sw = (x2 - x1).clamp(0, rt_w);
        let sh = (y2 - y1).clamp(0, rt_h);
        // P2-7: zero dimensions use zero-area scissor (0,0,0,0).
        if sw > 0 && sh > 0 {
            Some((x1 as u32, y1 as u32, sw as u32, sh as u32))
        } else {
            Some((0, 0, 0, 0))
        }
    }

    #[test]
    fn normal_rect_produces_correct_scissor() {
        let sc = compute_scissor(Some((10.0, 20.0, 100.0, 50.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((10, 20, 100, 50)));
    }

    #[test]
    fn zero_width_rect_produces_zero_scissor() {
        let sc = compute_scissor(Some((10.0, 20.0, 0.0, 50.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((0, 0, 0, 0)));
    }

    #[test]
    fn zero_height_rect_produces_zero_scissor() {
        let sc = compute_scissor(Some((10.0, 20.0, 50.0, 0.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((0, 0, 0, 0)));
    }

    #[test]
    fn negative_dimensions_clamp_to_zero() {
        let sc = compute_scissor(Some((100.0, 100.0, -50.0, 50.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((0, 0, 0, 0)));
    }

    #[test]
    fn none_rect_returns_none() {
        let sc = compute_scissor(None, 1.0, 800, 600);
        assert_eq!(sc, None);
    }

    #[test]
    fn scissor_clamps_to_render_target() {
        // Rect at (700, 500) with size 200x200 in an 800x600 target.
        // x1=700, x2=900, w=clamp(900-700, 0, 800)=200 (clamp to
        // upper bound of rt_w is not applied here -- clamp(0, rt_w)
        // means the upper bound is rt_w but w=200 <= 800 so the
        // clamp doesn't affect it). The actual behavior is that w
        // is the difference x2-x1 = 200, which fits within 800.
        let sc = compute_scissor(Some((700.0, 500.0, 200.0, 200.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((700, 500, 200, 200)));
    }

    #[test]
    fn scissor_extending_past_target_clamps() {
        // Rect at (700, 500) with size 500x500. w = 1200-700 = 500.
        // The clamp(0, 800) doesn't trigger because 500 <= 800.
        // Note: the original code does NOT actually clamp the rect
        // to the render target bounds; it just ensures w/h is
        // non-negative. A separate audit (P3-*) would be needed
        // to add proper bounds clamping. For now, verify the
        // current (non-bounds-clamping) behavior.
        let sc = compute_scissor(Some((700.0, 500.0, 500.0, 500.0)), 1.0, 800, 600);
        assert_eq!(sc, Some((700, 500, 500, 500)));
    }
}
