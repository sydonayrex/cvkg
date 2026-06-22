// =============================================================================
// KVASIR IDENTITY -- Platform-wide unique identifier (crosscrate.md Finding #2)
// =============================================================================

/// Platform-wide unique identifier used by every CVKG graph layer.
///
/// # Why this exists
/// The crosscrate audit (Finding #2) identified that each crate maintained its own
/// incompatible `NodeId(u64)` newtype, causing type-level friction whenever two
/// layers needed to reference the same object (e.g., VDOM ↔ Scene sync).
///
/// # Contract
/// - Every `KvasirId` produced by [`KvasirId::new`] is globally unique within
///   a single process lifetime (backed by a monotonic atomic counter).
/// - IDs are sequential and cache-friendly in `HashMap` / `BTreeMap` keys.
/// - `KvasirId(0)` is **reserved as the null/invalid sentinel** — never returned
///   by `new()`.
/// - `Serialize`/`Deserialize` round-trips through the inner `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct KvasirId(pub u64);

impl KvasirId {
    /// The null sentinel value. Never allocated by [`KvasirId::new`].
    pub const NULL: KvasirId = KvasirId(0);

    /// Allocate a new process-unique `KvasirId`.
    ///
    /// Uses a relaxed atomic increment — order does not matter because IDs
    /// only need to be distinct, not sequentially ordered relative to other
    /// memory operations.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        KvasirId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns `true` if this is the null sentinel value.
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for KvasirId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KvasirId({})", self.0)
    }
}

/// Lossless conversion from a raw `u64` into a `KvasirId`.
///
/// # Why this exists
/// The crosscrate audit (Phase 1 of the implementation plan) unifies identity
/// across `cvkg-scene`, `cvkg-vdom`, and `cvkg-flow` by making each crate's
/// `NodeId` a type alias for `KvasirId`. Existing call sites that constructed
/// `NodeId(some_u64)` need a way to migrate without touching every literal.
///
/// # Contract
/// - `u64` -> `KvasirId` is infallible (any `u64` is a valid id; 0 maps to NULL).
/// - `KvasirId` -> `u64` is infallible (trivially the inner value).
///
/// # Note
/// Allocating ids should still go through `KvasirId::new()` so that the
/// atomic counter is respected. `From<u64>` is for *existing* ids that came
/// from serialized data or stable test fixtures.
impl From<u64> for KvasirId {
    fn from(value: u64) -> Self {
        KvasirId(value)
    }
}

impl From<KvasirId> for u64 {
    fn from(id: KvasirId) -> Self {
        id.0
    }
}

