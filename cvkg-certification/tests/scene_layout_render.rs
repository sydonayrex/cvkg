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

//! Scene → Layout → Render certification.
//!
//! Verifies that the scene graph batch/layer system produces correctly ordered
//! output for the downstream renderer. This is the cross-crate path that
//! Finding #9 identified as untested: the handoff from scene layout (batching)
//! to render order depends on both `layer_id` separation and `z_index` sorting
//! within a layer.

use cvkg_certification::*;
use cvkg_core::Rect;
use cvkg_scene::{SceneGraph, VNode};

/// Certify that the Scene layer batching system separates layers and sorts z-order.
///
/// Batch separation is the contract between `SceneGraph::batch` and the GPU
/// renderer: each unique `layer_id` maps to a separate draw call. z_index
/// ordering within a layer is what prevents draw-order artefacts.
#[test]
fn certify_scene_layer_batching() {
    let mut suite = CertificationSuite::new("Scene Layer Batching");

    // ── Check 1: distinct layer_ids produce separate batch buckets ────────
    suite.run(
        "batch_separates_layers",
        "Nodes on different layers are batched separately",
        |check| {
            let mut scene = SceneGraph::new();

            let id1 = scene.next_id();
            let mut n1 = VNode::new(
                id1,
                "Rect",
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
            );
            n1.layer_id = 0;
            scene.add_node(n1, None);

            let id2 = scene.next_id();
            let mut n2 = VNode::new(
                id2,
                "Rect",
                Rect {
                    x: 50.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
            );
            n2.layer_id = 1;
            scene.add_node(n2, Some(id1));
            scene.update_transforms();

            let visible = scene.cull(Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            });
            let batches = scene.batch(&visible);

            if batches.len() == 2 {
                check.pass();
            } else {
                check.fail(format!(
                    "Expected 2 layers in batch, got {}",
                    batches.len()
                ));
            }
        },
    );

    // ── Check 2: z_index is sorted ascending within a layer ───────────────
    suite.run(
        "batch_z_order",
        "Nodes within a layer are sorted by z_index",
        |check| {
            let mut scene = SceneGraph::new();

            // Higher z_index (draws on top) — added first
            let id1 = scene.next_id();
            let mut n1 = VNode::new(
                id1,
                "Rect",
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
            );
            n1.z_index = 5.0;
            scene.add_node(n1, None);

            // Lower z_index (draws below) — added second as child
            let id2 = scene.next_id();
            let mut n2 = VNode::new(
                id2,
                "Rect",
                Rect {
                    x: 50.0,
                    y: 0.0,
                    width: 50.0,
                    height: 50.0,
                },
            );
            n2.z_index = 2.0;
            scene.add_node(n2, Some(id1));
            scene.update_transforms();

            let visible = scene.cull(Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            });
            let batches = scene.batch(&visible);

            if let Some(layer) = batches.get(&0) {
                if layer.len() < 2 {
                    check.fail(format!(
                        "Expected at least 2 nodes in layer 0, got {}",
                        layer.len()
                    ));
                    return;
                }
                // Lower z_index should come first (ascending sort = back-to-front)
                let z_first = scene.nodes[&layer[0]].z_index;
                let z_second = scene.nodes[&layer[1]].z_index;
                if z_first <= z_second {
                    check.pass();
                } else {
                    check.fail(format!(
                        "Z-order wrong: first={} > second={}",
                        z_first, z_second
                    ));
                }
            } else {
                check.fail("No layer 0 in batch");
            }
        },
    );

    suite.report();
    assert!(
        suite.all_pass(),
        "Scene layer certification failed: {} check(s) failed",
        suite.fail_count()
    );
}
