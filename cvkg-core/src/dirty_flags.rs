// =============================================================================
// INVALIDATION MODEL -- Platform-wide dirty flag system (crosscrate.md Finding #3)
// =============================================================================

use crate::identity::KvasirId;
use serde::{Deserialize, Serialize};

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
        Self {
            id,
            flags: DirtyFlags::ALL,
        }
    }
}
