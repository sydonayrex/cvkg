//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.

//! Pipeline certification: verifies that Scene → SceneGraph queries work correctly
//! end-to-end with the spatial indexing system.
//!
//! These tests exercise the full AABB culling, spatial hash, and dirty-tracking
//! pipeline as a cross-crate integration check. They are intentionally NOT unit
//! tests — they verify the system behaves correctly when the whole pipeline runs.

use cvkg_certification::*;
use cvkg_core::Rect;
use cvkg_scene::{SceneGraph, VNode};

/// Certify that Scene spatial operations (cull, query, dirty-track) work end-to-end.
///
/// Each `suite.run` block exercises one slice of the pipeline. The suite
/// asserts `all_pass()` at the end so a single failing check fails the whole
/// certification binary.
#[test]
fn certify_scene_spatial_pipeline() {
    let mut suite = CertificationSuite::new("Scene Spatial Pipeline");

    // ── Check 1: node inside viewport is visible ──────────────────────────
    suite.run(
        "scene_create_and_cull",
        "Scene nodes can be added and culled correctly",
        |check| {
            let mut scene = SceneGraph::new();
            let id = scene.next_id();
            let rect = Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            };
            scene.add_node(VNode::new(id, "Rect", rect), None);
            scene.update_transforms();

            let viewport = Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            };
            let visible = scene.cull(viewport);
            if visible.len() == 1 && visible[0] == id {
                check.pass();
            } else {
                check.fail(format!("Expected 1 visible node, got {}", visible.len()));
            }
        },
    );

    // ── Check 2: node outside viewport is culled ──────────────────────────
    suite.run(
        "scene_culls_outside",
        "Nodes outside viewport are correctly culled",
        |check| {
            let mut scene = SceneGraph::new();
            let id = scene.next_id();
            scene.add_node(
                VNode::new(
                    id,
                    "Rect",
                    Rect {
                        x: 500.0,
                        y: 500.0,
                        width: 50.0,
                        height: 50.0,
                    },
                ),
                None,
            );
            scene.update_transforms();
            let visible = scene.cull(Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            });
            if visible.is_empty() {
                check.pass();
            } else {
                check.fail("Node outside viewport was not culled");
            }
        },
    );

    // ── Check 3: spatial hash returns overlapping candidates ─────────────
    suite.run(
        "scene_spatial_query",
        "Spatial hash query returns correct candidates",
        |check| {
            let mut scene = SceneGraph::new();
            let id = scene.next_id();
            scene.add_node(
                VNode::new(
                    id,
                    "Rect",
                    Rect {
                        x: 10.0,
                        y: 10.0,
                        width: 50.0,
                        height: 50.0,
                    },
                ),
                None,
            );
            scene.update_transforms();
            let candidates = scene.query_region(Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            });
            if candidates.contains(&id) {
                check.pass();
            } else {
                check.fail("Spatial query missed overlapping node");
            }
        },
    );

    // ── Check 4: dirty tracking accumulates and clears correctly ──────────
    suite.run(
        "scene_dirty_tracking",
        "Dirty region tracking is accurate after add and clear",
        |check| {
            let mut scene = SceneGraph::new();
            let id = scene.next_id();
            scene.add_node(
                VNode::new(
                    id,
                    "Rect",
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                    },
                ),
                None,
            );
            let dirty_before = scene.dirty_regions().len();
            scene.clear_dirty();
            let dirty_after = scene.dirty_regions().len();
            if dirty_before >= 1 && dirty_after == 0 {
                check.pass();
            } else {
                check.fail(format!(
                    "dirty_before={dirty_before}, dirty_after={dirty_after}"
                ));
            }
        },
    );

    suite.report();
    assert!(
        suite.all_pass(),
        "Pipeline certification failed: {} check(s) failed",
        suite.fail_count()
    );
}
