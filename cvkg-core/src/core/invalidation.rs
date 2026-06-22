// =============================================================================
// INVALIDATION MODEL -- Platform-wide dirty flag system (crosscrate.md Finding #3)
// =============================================================================

/// Bitmask encoding which pipeline layers are dirty for a given object.
///
/// # Why this exists
/// The crosscrate audit (Finding #3) identified that each crate had its own
/// `is_dirty: bool` field with no shared semantic. Without a unified model,
/// updates propagate as full-tree redraws instead of targeted passes, leading
/// to performance collapse at scale.
///
/// # Layers (in pipeline order)
/// - `STATE`     — application-level data changed (triggers LAYOUT + PAINT + COMPOSITE)
/// - `LAYOUT`    — size or position changed (triggers PAINT + COMPOSITE)
/// - `PAINT`     — visual appearance changed (triggers COMPOSITE only)
/// - `COMPOSITE` — compositing properties changed (e.g. opacity, transform, blur)
///
/// # Contract
/// A crate that dirtifies a layer MUST also dirtify all downstream layers.
/// Use the helper constants [`DirtyFlags::from_state_change`] etc. rather
/// than setting bits manually to ensure the invariant is maintained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DirtyFlags(pub u8);

impl DirtyFlags {
    /// No layers are dirty.
    pub const CLEAN: DirtyFlags = DirtyFlags(0b0000_0000);
    /// Application state changed — propagates to all downstream layers.
    pub const STATE: DirtyFlags = DirtyFlags(0b0000_1111);
    /// Layout (size/position) changed — propagates to paint + composite.
    pub const LAYOUT: DirtyFlags = DirtyFlags(0b0000_0111);
    /// Paint (visual) changed — propagates to composite.
    pub const PAINT: DirtyFlags = DirtyFlags(0b0000_0011);
    /// Compositing properties changed (opacity, clip, backdrop).
    pub const COMPOSITE: DirtyFlags = DirtyFlags(0b0000_0001);
    /// All layers dirty (equivalent to STATE).
    pub const ALL: DirtyFlags = DirtyFlags(0b0000_1111);

    /// Returns `true` if any dirty bits are set.
    #[inline]
    pub fn is_dirty(self) -> bool {
        self.0 != 0
    }

    /// Returns `true` if the composite layer needs reprocessing.
    #[inline]
    pub fn needs_composite(self) -> bool {
        self.0 & 0b0000_0001 != 0
    }

    /// Returns `true` if the paint layer needs reprocessing.
    #[inline]
    pub fn needs_paint(self) -> bool {
        self.0 & 0b0000_0010 != 0
    }

    /// Returns `true` if layout needs reprocessing.
    #[inline]
    pub fn needs_layout(self) -> bool {
        self.0 & 0b0000_0100 != 0
    }

    /// Returns `true` if application state has changed.
    #[inline]
    pub fn needs_state(self) -> bool {
        self.0 & 0b0000_1000 != 0
    }

    /// Merge another set of flags into this one (bitwise OR).
    #[inline]
    pub fn merge(self, other: DirtyFlags) -> DirtyFlags {
        DirtyFlags(self.0 | other.0)
    }

    /// Clear all dirty flags, marking this object as clean.
    #[inline]
    pub fn clear(self) -> DirtyFlags {
        DirtyFlags::CLEAN
    }
}

impl std::ops::BitOr for DirtyFlags {
    type Output = DirtyFlags;
    fn bitor(self, rhs: DirtyFlags) -> DirtyFlags {
        DirtyFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DirtyFlags {
    fn bitor_assign(&mut self, rhs: DirtyFlags) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for DirtyFlags {
    type Output = DirtyFlags;
    fn bitand(self, rhs: DirtyFlags) -> DirtyFlags {
        DirtyFlags(self.0 & rhs.0)
    }
}

/// A single invalidation record associating a `KvasirId` with its dirty layers.
///
/// # Contract
/// Invalidation records are produced by any system that mutates state and
/// consumed by the scheduler to determine what work must be done next frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationRecord {
    /// The object that was mutated.
    pub id: KvasirId,
    /// Which pipeline layers need reprocessing.
    pub flags: DirtyFlags,
}

impl InvalidationRecord {
    /// Create a new invalidation record.
    pub fn new(id: KvasirId, flags: DirtyFlags) -> Self {
        Self { id, flags }
    }

    /// Create a record indicating the object's full pipeline needs rebuilding.
    pub fn full(id: KvasirId) -> Self {
        Self { id, flags: DirtyFlags::ALL }
    }
}

#[cfg(test)]
mod kvasir_identity_tests {
    use super::*;

    #[test]
    fn kvasir_id_new_is_non_zero() {
        // Contract: KvasirId::new() must never return the null sentinel.
        let id = KvasirId::new();
        assert!(!id.is_null(), "KvasirId::new() returned null sentinel");
    }

    #[test]
    fn kvasir_id_new_is_unique() {
        // Each call must produce a distinct ID.
        let a = KvasirId::new();
        let b = KvasirId::new();
        let c = KvasirId::new();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn kvasir_id_null_sentinel() {
        assert!(KvasirId::NULL.is_null());
        assert!(!KvasirId::new().is_null());
    }

    #[test]
    fn kvasir_id_serde_roundtrip() {
        let id = KvasirId(42);
        let json = serde_json::to_string(&id).unwrap();
        let decoded: KvasirId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn dirty_flags_clean_is_not_dirty() {
        assert!(!DirtyFlags::CLEAN.is_dirty());
    }

    #[test]
    fn dirty_flags_all_implies_all_layers() {
        let f = DirtyFlags::ALL;
        assert!(f.needs_state());
        assert!(f.needs_layout());
        assert!(f.needs_paint());
        assert!(f.needs_composite());
    }

    #[test]
    fn dirty_flags_composite_only() {
        let f = DirtyFlags::COMPOSITE;
        assert!(!f.needs_state());
        assert!(!f.needs_layout());
        assert!(!f.needs_paint());
        assert!(f.needs_composite());
    }

    #[test]
    fn dirty_flags_merge() {
        let a = DirtyFlags::COMPOSITE;
        let b = DirtyFlags::PAINT;
        let merged = a.merge(b);
        assert!(merged.needs_composite());
        assert!(merged.needs_paint());
        assert!(!merged.needs_layout());
    }

    #[test]
    fn dirty_flags_bitor() {
        let combined = DirtyFlags::PAINT | DirtyFlags::COMPOSITE;
        assert!(combined.needs_paint());
        assert!(combined.needs_composite());
    }

    #[test]
    fn dirty_flags_clear() {
        let dirty = DirtyFlags::ALL;
        let clean = dirty.clear();
        assert!(!clean.is_dirty());
    }

    #[test]
    fn dirty_flags_serde_roundtrip() {
        let f = DirtyFlags::LAYOUT;
        let json = serde_json::to_string(&f).unwrap();
        let decoded: DirtyFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(f, decoded);
    }

    #[test]
    fn invalidation_record_full() {
        let id = KvasirId::new();
        let rec = InvalidationRecord::full(id);
        assert_eq!(rec.id, id);
        assert!(rec.flags.needs_state());
        assert!(rec.flags.needs_layout());
    }
}

