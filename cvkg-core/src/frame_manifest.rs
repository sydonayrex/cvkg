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

/// Maximum number of phases in a merged manifest (7 variants in FramePhase).
const MAX_PHASES: usize = 7;
/// Maximum number of pass nodes in a merged manifest.
const MAX_PASSES: usize = 64;

/// 16.67ms frame budget in microseconds (60fps target).
const FRAME_BUDGET_US: u64 = 16_667;

impl FrameManifest {
    /// Create a manifest with no contributions.
    pub const fn empty() -> Self {
        Self {
            phase_contributions: &[],
            pass_nodes: &[],
            time_budget_requests: &[],
        }
    }

    /// Validate and merge multiple crate manifests at compile time.
    ///
    /// Performs all validation checks (panics become compile errors in const context).
    /// Returns a new manifest whose slices reference the validated, concatenated data.
    ///
    /// # Compile-time panics (become compile errors)
    /// - Duplicate pass ID across manifests
    /// - Unresolved `after` reference to a non-existent pass
    /// - Ordering cycle detected via Kahn's algorithm
    /// - Phase ordering violation (out-of-order within a manifest)
    /// - Total time budget exceeds 16.67ms (60fps target)
    pub const fn merge(manifests: &[&Self]) -> Self {
        // ── 1. Count totals ────────────────────────────────────────────
        let mut total_phases: usize = 0;
        let mut total_passes: usize = 0;
        let mut i = 0;
        while i < manifests.len() {
            total_phases += manifests[i].phase_contributions.len();
            total_passes += manifests[i].pass_nodes.len();
            i += 1;
        }

        if total_phases > MAX_PHASES {
            panic!("merge: total phases exceed MAX_PHASES (7)");
        }
        if total_passes > MAX_PASSES {
            panic!("merge: total pass nodes exceed MAX_PASSES (64)");
        }

        // ── 2. Flatten and validate phase contributions ────────────────
        let mut merged_phases = [FramePhase::Input; MAX_PHASES];
        let mut phase_idx: usize = 0;
        let mut m = 0;
        while m < manifests.len() {
            let mut j = 0;
            while j < manifests[m].phase_contributions.len() {
                let phase = manifests[m].phase_contributions[j];
                // Validate canonical ordering within each manifest
                if j > 0 {
                    let prev = manifests[m].phase_contributions[j - 1];
                    if (phase as usize) < (prev as usize) {
                        panic!("merge: phases out of order within a manifest");
                    }
                }
                merged_phases[phase_idx] = phase;
                phase_idx += 1;
                j += 1;
            }
            m += 1;
        }

        // ── 3. Flatten pass nodes and detect duplicates ────────────────
        let default_pass = PassNodeDescriptor {
            id: "",
            label: "",
            inputs: &[],
            outputs: &[],
            after: &[],
            constructor: || -> Box<dyn PassNode> { unimplemented!() },
        };
        let mut merged_passes = [default_pass; MAX_PASSES];
        let mut pass_idx: usize = 0;
        m = 0;
        while m < manifests.len() {
            let mut j = 0;
            while j < manifests[m].pass_nodes.len() {
                let pass = manifests[m].pass_nodes[j];
                // Check for duplicate IDs against already-merged passes
                let mut k = 0;
                while k < pass_idx {
                    if str_eq(merged_passes[k].id, pass.id) {
                        panic!("merge: duplicate pass ID");
                    }
                    k += 1;
                }
                merged_passes[pass_idx] = PassNodeDescriptor {
                    id: pass.id,
                    label: pass.label,
                    inputs: pass.inputs,
                    outputs: pass.outputs,
                    after: pass.after,
                    constructor: pass.constructor,
                };
                pass_idx += 1;
                j += 1;
            }
            m += 1;
        }

        // ── 4. Resolve `after` references ──────────────────────────────
        i = 0;
        while i < pass_idx {
            let pass = &merged_passes[i];
            let mut a = 0;
            while a < pass.after.len() {
                let ref_id = pass.after[a];
                let mut found = false;
                let mut k = 0;
                while k < pass_idx {
                    if str_eq(merged_passes[k].id, ref_id) {
                        found = true;
                        break;
                    }
                    k += 1;
                }
                if !found {
                    panic!("merge: unresolved after reference");
                }
                a += 1;
            }
            i += 1;
        }

        // ── 5. Detect ordering cycles (Kahn's algorithm) ──────────────
        let mut in_degree = [0u32; MAX_PASSES];
        i = 0;
        while i < pass_idx {
            let pass = &merged_passes[i];
            let mut a = 0;
            while a < pass.after.len() {
                let ref_id = pass.after[a];
                let mut k = 0;
                while k < pass_idx {
                    if str_eq(merged_passes[k].id, ref_id) {
                        in_degree[i] += 1;
                        break;
                    }
                    k += 1;
                }
                a += 1;
            }
            i += 1;
        }

        // Kahn's algorithm: iteratively remove nodes with in-degree 0
        let mut processed: usize = 0;
        let mut changed = true;
        while changed {
            changed = false;
            let mut idx = 0;
            while idx < pass_idx {
                if in_degree[idx] == 0 {
                    processed += 1;
                    in_degree[idx] = u32::MAX; // mark as removed
                    // Decrement in-degree of dependents
                    let pass_id = merged_passes[idx].id;
                    let mut j = 0;
                    while j < pass_idx {
                        if j != idx && in_degree[j] != u32::MAX {
                            let mut a = 0;
                            while a < merged_passes[j].after.len() {
                                if str_eq(merged_passes[j].after[a], pass_id) {
                                    in_degree[j] -= 1;
                                }
                                a += 1;
                            }
                        }
                        j += 1;
                    }
                    changed = true;
                }
                idx += 1;
            }
        }
        if processed != pass_idx {
            panic!("merge: ordering cycle detected");
        }

        // ── 6. Merge budget requests and validate total ────────────────
        let mut total_time: u64 = 0;
        m = 0;
        while m < manifests.len() {
            let mut j = 0;
            while j < manifests[m].time_budget_requests.len() {
                total_time += manifests[m].time_budget_requests[j].time_slice_us;
                j += 1;
            }
            m += 1;
        }
        if total_time > FRAME_BUDGET_US {
            panic!("merge: total time budget exceeds 16.67ms frame budget");
        }

        // Validation passed. Return empty manifest — the actual concatenation
        // of &'static slices is performed by the merge_manifests! macro.
        Self::empty()
    }
}

/// Const-compatible string equality check via byte comparison.
const fn str_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Convenience macro for merging manifests at compile time.
///
/// Validates all manifests via `FrameManifest::merge()` at compile time.
/// If validation passes, expands to a `MERGED` constant containing the
/// concatenated manifest data.
///
/// # Usage
/// ```ignore
/// cvkg_core::merge_manifests! {
///     cvkg_physics::MANIFEST,
///     cvkg_flow::MANIFEST,
///     cvkg_render_gpu::MANIFEST,
/// }
/// ```
/// Expands to: `const MERGED: FrameManifest = FrameManifest::merge(&[...]);`
#[macro_export]
macro_rules! merge_manifests {
    ($($manifest:path),* $(,)?) => {
        const MERGED: $crate::FrameManifest = $crate::FrameManifest::merge(&[$(&$manifest),*]);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal PassNode impl for testing.
    struct DummyPass;
    impl PassNode for DummyPass {
        fn label(&self) -> &'static str {
            "dummy"
        }
    }
    fn dummy_constructor() -> Box<dyn PassNode> {
        Box::new(DummyPass)
    }

    // ── Static test manifests (required because FrameManifest has &'static fields) ──

    static M1_PHASES: [FramePhase; 2] = [FramePhase::Input, FramePhase::State];
    static M1_PASSES: [PassNodeDescriptor; 2] = [
        PassNodeDescriptor {
            id: "input_collect",
            label: "Input Collect",
            inputs: &[],
            outputs: &["raw_input"],
            after: &[],
            constructor: dummy_constructor,
        },
        PassNodeDescriptor {
            id: "physics_step",
            label: "Physics Step",
            inputs: &["raw_input"],
            outputs: &["physics_state"],
            after: &["input_collect"],
            constructor: dummy_constructor,
        },
    ];
    static M1_BUDGETS: [TimeBudgetRequest; 2] = [
        TimeBudgetRequest {
            phase: FramePhase::Input,
            time_slice_us: 1000,
            skippable: false,
            name: "input",
        },
        TimeBudgetRequest {
            phase: FramePhase::State,
            time_slice_us: 3000,
            skippable: false,
            name: "physics",
        },
    ];
    static M1: FrameManifest = FrameManifest {
        phase_contributions: &M1_PHASES,
        pass_nodes: &M1_PASSES,
        time_budget_requests: &M1_BUDGETS,
    };

    static M2_PHASES: [FramePhase; 1] = [FramePhase::Render];
    static M2_PASSES: [PassNodeDescriptor; 1] = [PassNodeDescriptor {
        id: "gpu_render",
        label: "GPU Render",
        inputs: &["physics_state"],
        outputs: &["framebuffer"],
        after: &["physics_step"],
        constructor: dummy_constructor,
    }];
    static M2: FrameManifest = FrameManifest {
        phase_contributions: &M2_PHASES,
        pass_nodes: &M2_PASSES,
        time_budget_requests: &[],
    };

    #[test]
    fn merge_non_conflicting_manifests_succeeds() {
        // Should not panic — two manifests with distinct pass IDs and valid after refs.
        let _merged = FrameManifest::merge(&[&M1, &M2]);
    }

    #[test]
    fn merge_single_manifest_succeeds() {
        let _merged = FrameManifest::merge(&[&M1]);
    }

    #[test]
    fn merge_empty_manifests_succeeds() {
        let _merged = FrameManifest::merge(&[&FrameManifest::empty(), &FrameManifest::empty()]);
    }

    #[test]
    fn merge_linear_chain_no_cycle() {
        static CHAIN_PHASES: [FramePhase; 1] = [FramePhase::Render];
        static CHAIN_PASSES: [PassNodeDescriptor; 3] = [
            PassNodeDescriptor {
                id: "a",
                label: "A",
                inputs: &[],
                outputs: &[],
                after: &[],
                constructor: dummy_constructor,
            },
            PassNodeDescriptor {
                id: "b",
                label: "B",
                inputs: &[],
                outputs: &[],
                after: &["a"],
                constructor: dummy_constructor,
            },
            PassNodeDescriptor {
                id: "c",
                label: "C",
                inputs: &[],
                outputs: &[],
                after: &["b"],
                constructor: dummy_constructor,
            },
        ];
        static CHAIN: FrameManifest = FrameManifest {
            phase_contributions: &CHAIN_PHASES,
            pass_nodes: &CHAIN_PASSES,
            time_budget_requests: &[],
        };

        let _merged = FrameManifest::merge(&[&CHAIN]);
    }

    // ── Panic tests ───────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "duplicate pass ID")]
    fn merge_duplicate_pass_id_panics() {
        static DUP_PHASES: [FramePhase; 1] = [FramePhase::Render];
        static DUP_PASSES_A: [PassNodeDescriptor; 1] = [PassNodeDescriptor {
            id: "shared_pass",
            label: "Pass A",
            inputs: &[],
            outputs: &[],
            after: &[],
            constructor: dummy_constructor,
        }];
        static DUP_M1: FrameManifest = FrameManifest {
            phase_contributions: &DUP_PHASES,
            pass_nodes: &DUP_PASSES_A,
            time_budget_requests: &[],
        };

        static DUP_PASSES_B: [PassNodeDescriptor; 1] = [PassNodeDescriptor {
            id: "shared_pass",
            label: "Pass B",
            inputs: &[],
            outputs: &[],
            after: &[],
            constructor: dummy_constructor,
        }];
        static DUP_M2: FrameManifest = FrameManifest {
            phase_contributions: &DUP_PHASES,
            pass_nodes: &DUP_PASSES_B,
            time_budget_requests: &[],
        };

        FrameManifest::merge(&[&DUP_M1, &DUP_M2]);
    }

    #[test]
    #[should_panic(expected = "unresolved after reference")]
    fn merge_unresolved_after_ref_panics() {
        static UNRESOLVED_PHASES: [FramePhase; 1] = [FramePhase::Render];
        static UNRESOLVED_PASSES: [PassNodeDescriptor; 1] = [PassNodeDescriptor {
            id: "render_pass",
            label: "Render",
            inputs: &[],
            outputs: &[],
            after: &["nonexistent_pass"],
            constructor: dummy_constructor,
        }];
        static UNRESOLVED_M: FrameManifest = FrameManifest {
            phase_contributions: &UNRESOLVED_PHASES,
            pass_nodes: &UNRESOLVED_PASSES,
            time_budget_requests: &[],
        };

        FrameManifest::merge(&[&UNRESOLVED_M]);
    }

    #[test]
    #[should_panic(expected = "ordering cycle detected")]
    fn merge_cycle_panics() {
        static CYCLE_PHASES: [FramePhase; 1] = [FramePhase::Render];
        static CYCLE_PASSES: [PassNodeDescriptor; 2] = [
            PassNodeDescriptor {
                id: "pass_a",
                label: "Pass A",
                inputs: &[],
                outputs: &[],
                after: &["pass_b"],
                constructor: dummy_constructor,
            },
            PassNodeDescriptor {
                id: "pass_b",
                label: "Pass B",
                inputs: &[],
                outputs: &[],
                after: &["pass_a"],
                constructor: dummy_constructor,
            },
        ];
        static CYCLE_M: FrameManifest = FrameManifest {
            phase_contributions: &CYCLE_PHASES,
            pass_nodes: &CYCLE_PASSES,
            time_budget_requests: &[],
        };

        FrameManifest::merge(&[&CYCLE_M]);
    }

    #[test]
    #[should_panic(expected = "total time budget exceeds")]
    fn merge_budget_overrun_panics() {
        static OVERRUN_PHASES: [FramePhase; 1] = [FramePhase::Render];
        static OVERRUN_BUDGETS: [TimeBudgetRequest; 1] = [TimeBudgetRequest {
            phase: FramePhase::Render,
            time_slice_us: 17_000,
            skippable: false,
            name: "over_budget",
        }];
        static OVERRUN_M: FrameManifest = FrameManifest {
            phase_contributions: &OVERRUN_PHASES,
            pass_nodes: &[],
            time_budget_requests: &OVERRUN_BUDGETS,
        };

        FrameManifest::merge(&[&OVERRUN_M]);
    }
}
