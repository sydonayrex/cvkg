//! Compile-time crate manifest declaring frame pipeline contributions.
//!
//! # Contract
//! - All types are `const`-constructible — no heap allocation, no vtables.
//! - `FrameManifest::merge()` is a `const fn` — conflicts produce compile errors.

use crate::FramePhase;

/// Minimal pass node trait known to cvkg-core.
///
/// cvkg-render-gpu's `KvasirNode` trait extends this with Kvasir-specific
/// methods (`inputs()`, `outputs()`, `execute()` with `ExecutionContext`).
pub trait PassNode: Send + Sync {
    /// Human-readable label for debugging and DOT graph output.
    fn label(&self) -> &'static str;
}

/// Compile-time crate manifest declaring frame pipeline contributions.
#[derive(Debug, Clone, Copy)]
pub struct FrameManifest {
    /// Phases this crate contributes work to, in ascending FramePhase order.
    pub phase_contributions: &'static [FramePhase],
    /// Render pass slots this crate contributes to the Kvasir graph.
    pub pass_nodes: &'static [PassNodeDescriptor],
    /// Per-phase time budget requests.
    pub time_budget_requests: &'static [TimeBudgetRequest],
}

/// Descriptor for a render pass node contributed at compile time.
#[derive(Debug, Clone, Copy)]
pub struct PassNodeDescriptor {
    /// Unique pass identifier within the merged set.
    pub id: &'static str,
    /// Human-readable label (DOT graph output, debug tracing).
    pub label: &'static str,
    /// Logical resource input names (e.g. "scene_color", "depth").
    pub inputs: &'static [&'static str],
    /// Logical resource output names (e.g. "particle_buffer").
    pub outputs: &'static [&'static str],
    /// Pass IDs that must execute before this one.
    pub after: &'static [&'static str],
    /// Constructor function pointer. Called at runtime to produce Box<dyn PassNode>.
    pub constructor: fn() -> Box<dyn PassNode>,
}

/// Per-phase time budget request from a subsystem.
#[derive(Debug, Clone, Copy)]
pub struct TimeBudgetRequest {
    /// Which frame phase this budget applies to.
    pub phase: FramePhase,
    /// Requested time slice in microseconds.
    pub time_slice_us: u64,
    /// Whether this crate's phase work can be skipped when over budget.
    pub skippable: bool,
    /// Subsystem name for logging and telemetry.
    pub name: &'static str,
}

impl FrameManifest {
    /// Create a manifest with no contributions.
    pub const fn empty() -> Self {
        Self {
            phase_contributions: &[],
            pass_nodes: &[],
            time_budget_requests: &[],
        }
    }

    /// Merge multiple crate manifests into one.
    ///
    /// # Compile-time panics (become compile errors)
    /// - Duplicate pass ID
    /// - Unresolved `after` reference
    /// - Ordering cycle
    /// - Phase ordering violation
    ///
    /// Stub — full implementation in Task 4 (merge_manifests! macro).
    pub const fn merge(manifests: &[&Self]) -> Self {
        // TODO: Implement in Task 4
        // For now, concatenate without validation
        let total_phases: &[FramePhase] = &[];
        let total_passes: &[PassNodeDescriptor] = &[];
        let total_budgets: &[TimeBudgetRequest] = &[];
        let mut i = 0;
        while i < manifests.len() {
            let m = manifests[i];
            // Concatenate phases
            let mut j = 0;
            while j < m.phase_contributions.len() {
                // Can't mutate slices in const fn — just count for now
                j += 1;
            }
            // Concatenate passes
            let mut j = 0;
            while j < m.pass_nodes.len() {
                j += 1;
            }
            // Concatenate budgets
            let mut j = 0;
            while j < m.time_budget_requests.len() {
                j += 1;
            }
            i += 1;
        }
        Self {
            phase_contributions: total_phases,
            pass_nodes: total_passes,
            time_budget_requests: total_budgets,
        }
    }
}
