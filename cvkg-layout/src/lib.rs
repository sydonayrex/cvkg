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
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

pub mod taffy_engine;
pub mod animation;
pub mod spatial;
pub mod focus;
pub mod progressive;
pub mod primitives;

pub use cvkg_core::layout::EdgeInsets;
use cvkg_core::{LayoutCache, LayoutView};
use std::cell::RefCell;
use std::collections::HashSet;

pub use taffy_engine::{
    taffy_alignment, taffy_distribution, taffy_track, Flex, Grid, GridTrack, HStack, Spacer,
    TaffyLayoutEngine, VStack, ZStack,
};
pub use animation::AnimationEngine;
pub use spatial::{LayoutSpatialEntry, LayoutSpatialIndex};
pub use focus::{compute_focus_order, validate_reading_order, LayoutModality, FocusCandidate};
pub use progressive::{ProgressiveChild, ProgressiveLayoutContext};
pub use primitives::{AspectRatio, Padding, SafeArea, SafeAreaEdges};

// P2-45: Layout capability flags for runtime feature detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutCapabilities {
    pub flexbox: bool,
    pub grid: bool,
    pub absolute: bool,
    pub container_queries: bool,
}

/// Returns the layout capabilities supported by this engine.
pub fn layout_capabilities() -> LayoutCapabilities {
    LayoutCapabilities {
        flexbox: true,
        grid: true,
        absolute: true,
        container_queries: true,
    }
}

thread_local! {
    static ACTIVE_LAYOUT_NODES: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
}

/// RAII guard that removes the hash from ACTIVE_LAYOUT_NODES on drop.
pub struct LayoutCycleGuard {
    hash: u64,
}

impl Drop for LayoutCycleGuard {
    fn drop(&mut self) {
        if self.hash != 0 {
            ACTIVE_LAYOUT_NODES.with(|nodes| {
                nodes.borrow_mut().remove(&self.hash);
            });
        }
    }
}

/// Helper function to prevent layout calculation cycles in recursive size queries.
pub fn with_layout_cycle_guard<F, R>(hash: u64, fallback: R, f: F) -> R
where
    F: FnOnce() -> R,
{
    if hash == 0 {
        return f();
    }
    let already_active = ACTIVE_LAYOUT_NODES.with(|nodes| !nodes.borrow_mut().insert(hash));
    if already_active {
        log::warn!("[Layout] Cycle detected for view hash 0x{:X}! Breaking cycle with fallback size.", hash);
        return fallback;
    }
    let _guard = LayoutCycleGuard { hash };
    f()
}

/// Helper function to prevent layout calculation cycles in recursive subview placements.
pub fn with_layout_cycle_guard_void<F>(hash: u64, f: F)
where
    F: FnOnce(),
{
    if hash == 0 {
        f();
        return;
    }
    let already_active = ACTIVE_LAYOUT_NODES.with(|nodes| !nodes.borrow_mut().insert(hash));
    if already_active {
        log::warn!("[Layout] Cycle detected for view hash 0x{:X}! Breaking cycle placement.", hash);
        return;
    }
    let _guard = LayoutCycleGuard { hash };
    f();
}

/// Compute size-that-fits for a batch of independent subviews in parallel when the parallel cargo feature is active.
pub fn size_views_parallel(
    views: &[&dyn LayoutView],
    proposal: cvkg_core::SizeProposal,
    cache: &mut LayoutCache,
) -> Vec<cvkg_core::Size> {
    if views.len() <= 1 {
        return views
            .iter()
            .map(|v| v.size_that_fits(proposal, &[], cache))
            .collect();
    }

    views
        .iter()
        .map(|v| v.size_that_fits(proposal, &[], cache))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::{Alignment, Distribution, Rect, Size, SizeProposal};

    struct MockView {
        size: Size,
        flex: f32,
    }

    impl LayoutView for MockView {
        fn size_that_fits(
            &self,
            _p: SizeProposal,
            _s: &[&dyn LayoutView],
            _c: &mut LayoutCache,
        ) -> Size {
            self.size
        }
        fn place_subviews(&self, _b: Rect, _s: &mut [&mut dyn LayoutView], _c: &mut LayoutCache) {}
        fn flex_weight(&self) -> f32 {
            self.flex
        }
    }

    #[test]
    fn test_hstack_basic() {
        let v1 = MockView {
            size: Size {
                width: 50.0,
                height: 50.0,
            },
            flex: 0.0,
        };
        let v2 = MockView {
            size: Size {
                width: 100.0,
                height: 100.0,
            },
            flex: 0.0,
        };
        let views: Vec<&dyn LayoutView> = vec![&v1, &v2];
        let mut cache = LayoutCache::new();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 200.0,
        };

        let rects = HStack::compute_layout(
            10.0,
            Alignment::Center,
            Distribution::Leading,
            bounds,
            &views,
            &mut cache,
        );

        assert_eq!(rects.len(), 2);
        assert_eq!(
            rects[0],
            Rect {
                x: 0.0,
                y: 75.0,
                width: 50.0,
                height: 50.0
            }
        );
        assert_eq!(
            rects[1],
            Rect {
                x: 60.0,
                y: 50.0,
                width: 100.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn test_vstack_flex() {
        let v1 = MockView {
            size: Size {
                width: 100.0,
                height: 50.0,
            },
            flex: 0.0,
        };
        let v2 = MockView {
            size: Size {
                width: 100.0,
                height: 0.0,
            },
            flex: 1.0,
        };
        let views: Vec<&dyn LayoutView> = vec![&v1, &v2];
        let mut cache = LayoutCache::new();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 160.0,
        };

        let rects = VStack::compute_layout(
            10.0,
            Alignment::Leading,
            Distribution::Fill,
            bounds,
            &views,
            &mut cache,
        );

        assert_eq!(rects.len(), 2);
        assert_eq!(
            rects[0],
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0
            }
        );
        assert_eq!(
            rects[1],
            Rect {
                x: 0.0,
                y: 60.0,
                width: 100.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn test_grid_layout() {
        let v1 = MockView {
            size: Size::ZERO,
            flex: 0.0,
        };
        let v2 = MockView {
            size: Size::ZERO,
            flex: 0.0,
        };
        let v3 = MockView {
            size: Size::ZERO,
            flex: 0.0,
        };
        let views: Vec<&dyn LayoutView> = vec![&v1, &v2, &v3];
        let mut cache = LayoutCache::new();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 210.0,
            height: 210.0,
        };

        let grid = Grid::new(
            vec![GridTrack::Fixed(100.0), GridTrack::Fixed(100.0)],
            vec![GridTrack::Fixed(100.0), GridTrack::Fixed(100.0)],
            10.0,
            10.0,
        );
        let placements = vec![
            Some(cvkg_core::GridPlacement {
                column: 0,
                column_span: 1,
                row: 0,
                row_span: 1,
            }),
            Some(cvkg_core::GridPlacement {
                column: 1,
                column_span: 1,
                row: 0,
                row_span: 1,
            }),
            Some(cvkg_core::GridPlacement {
                column: 0,
                column_span: 1,
                row: 1,
                row_span: 1,
            }),
        ];

        let rects = grid.compute_layout_rects(bounds, &views, &placements, &mut cache);

        assert_eq!(rects.len(), 3);
        assert_eq!(
            rects[0],
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0
            }
        );
        assert_eq!(
            rects[1],
            Rect {
                x: 110.0,
                y: 0.0,
                width: 100.0,
                height: 100.0
            }
        );
        assert_eq!(
            rects[2],
            Rect {
                x: 0.0,
                y: 110.0,
                width: 100.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn test_layout_cycle_detection() {
        struct CyclingView {
            child_hash: u64,
        }
        impl LayoutView for CyclingView {
            fn size_that_fits(
                &self,
                proposal: SizeProposal,
                _subviews: &[&dyn LayoutView],
                cache: &mut LayoutCache,
            ) -> Size {
                with_layout_cycle_guard(self.view_hash(), Size { width: 42.0, height: 42.0 }, || {
                    let recursive_self = CyclingView { child_hash: self.view_hash() };
                    let subviews: Vec<&dyn LayoutView> = vec![&recursive_self];
                    recursive_self.size_that_fits(proposal, &subviews, cache)
                })
            }
            fn place_subviews(&self, _b: Rect, _s: &mut [&mut dyn LayoutView], _c: &mut LayoutCache) {}
            fn view_hash(&self) -> u64 {
                12345
            }
        }

        let view = CyclingView { child_hash: 12345 };
        let mut cache = LayoutCache::new();
        let size = view.size_that_fits(SizeProposal::unspecified(), &[], &mut cache);
        assert_eq!(size.width, 42.0);
        assert_eq!(size.height, 42.0);
    }

    #[test]
    fn test_bottom_up_layout_invalidation() {
        let mut cache = LayoutCache::new();
        let child_hash = 100u64;
        let parent_hash = 200u64;

        cache.register_parent(child_hash, parent_hash);
        cache.set_size(child_hash, SizeProposal::unspecified(), Size { width: 10.0, height: 10.0 });
        cache.set_size(parent_hash, SizeProposal::unspecified(), Size { width: 20.0, height: 20.0 });

        assert!(cache.get_size(child_hash, SizeProposal::unspecified()).is_some());
        assert!(cache.get_size(parent_hash, SizeProposal::unspecified()).is_some());

        cache.invalidate_view(child_hash);

        assert!(cache.get_size(child_hash, SizeProposal::unspecified()).is_none());
        assert!(cache.get_size(parent_hash, SizeProposal::unspecified()).is_none());
    }

    #[test]
    fn test_viewport_aware_layout_culling() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct SpyView {
            calls: Arc<AtomicUsize>,
            hash: u64,
            rect: Rect,
        }

        impl LayoutView for SpyView {
            fn size_that_fits(&self, _p: SizeProposal, _s: &[&dyn LayoutView], _c: &mut LayoutCache) -> Size {
                Size { width: self.rect.width, height: self.rect.height }
            }
            fn place_subviews(&self, _b: Rect, _s: &mut [&mut dyn LayoutView], _c: &mut LayoutCache) {
                self.calls.fetch_add(1, Ordering::SeqCst);
            }
            fn view_hash(&self) -> u64 {
                self.hash
            }
        }

        let calls = Arc::new(AtomicUsize::new(0));
        let view1 = SpyView {
            calls: calls.clone(),
            hash: 1001,
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        };
        let view2 = SpyView {
            calls: calls.clone(),
            hash: 1002,
            rect: Rect::new(500.0, 0.0, 50.0, 50.0),
        };

        let mut cache = LayoutCache::new();
        cache.viewport = Some(Rect::new(0.0, 0.0, 55.0, 100.0));

        let mut v1 = view1;
        let mut v2 = view2;
        let mut mut_subviews: Vec<&mut dyn LayoutView> = vec![&mut v1, &mut v2];

        HStack::new(10.0, Alignment::Center, Distribution::Leading)
            .place_subviews(Rect::new(0.0, 0.0, 600.0, 100.0), &mut mut_subviews, &mut cache);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_layout_budget_thrashing_prevention() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct SpyView {
            calls: Arc<AtomicUsize>,
            hash: u64,
            rect: Rect,
        }

        impl LayoutView for SpyView {
            fn size_that_fits(&self, _p: SizeProposal, _s: &[&dyn LayoutView], _c: &mut LayoutCache) -> Size {
                Size { width: self.rect.width, height: self.rect.height }
            }
            fn place_subviews(&self, _b: Rect, _s: &mut [&mut dyn LayoutView], _c: &mut LayoutCache) {
                self.calls.fetch_add(1, Ordering::SeqCst);
            }
            fn view_hash(&self) -> u64 {
                self.hash
            }
        }

        let calls = Arc::new(AtomicUsize::new(0));
        let view = SpyView {
            calls: calls.clone(),
            hash: 2001,
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        };

        let mut cache = LayoutCache::new();
        cvkg_core::LayoutCache::set_layout_budget_deadline(Some(
            std::time::Instant::now() - std::time::Duration::from_millis(50),
        ));
        
        cache.previous_rects.insert(2001, Rect::new(10.0, 10.0, 100.0, 100.0));

        let mut v = view;
        let mut subviews: Vec<&mut dyn LayoutView> = vec![&mut v];

        HStack::new(0.0, Alignment::Center, Distribution::Leading)
            .place_subviews(Rect::new(0.0, 0.0, 500.0, 500.0), &mut subviews, &mut cache);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        
        let engine = TaffyLayoutEngine::get_or_insert_engine(&mut cache);
        assert!(!engine.node_map.contains_key(&2001));

        cvkg_core::LayoutCache::clear_layout_budget_deadline();
    }

    #[test]
    fn test_spatial_index_hit_test() {
        let mut index = LayoutSpatialIndex::new();
        let root = Rect { x: 0.0, y: 0.0, width: 1000.0, height: 1000.0 };
        let entries = vec![
            LayoutSpatialEntry { hash: 1, rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 } },
            LayoutSpatialEntry { hash: 2, rect: Rect { x: 200.0, y: 200.0, width: 50.0, height: 50.0 } },
            LayoutSpatialEntry { hash: 3, rect: Rect { x: 500.0, y: 500.0, width: 200.0, height: 200.0 } },
        ];
        index.rebuild(root, entries);

        let hits = index.hit_test(50.0, 50.0);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].hash, 1);

        let hits = index.hit_test(600.0, 600.0);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].hash, 3);

        let hits = index.hit_test(999.0, 1.0);
        assert!(hits.is_empty(), "Expected no hits, got {:?}", hits.iter().map(|e| e.hash).collect::<Vec<_>>());
    }

    #[test]
    fn test_spatial_index_query_region() {
        let mut index = LayoutSpatialIndex::new();
        let root = Rect { x: 0.0, y: 0.0, width: 500.0, height: 500.0 };
        let entries = vec![
            LayoutSpatialEntry { hash: 10, rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 } },
            LayoutSpatialEntry { hash: 20, rect: Rect { x: 400.0, y: 400.0, width: 50.0, height: 50.0 } },
        ];
        index.rebuild(root, entries);

        let region = Rect { x: 0.0, y: 0.0, width: 150.0, height: 150.0 };
        let results = index.query_region(&region);
        assert!(results.iter().any(|e| e.hash == 10));
        assert!(!results.iter().any(|e| e.hash == 20));
    }

    #[test]
    fn test_adaptive_modality_touch_enlarges_small_views() {
        let small = cvkg_core::Size { width: 20.0, height: 12.0 };
        let adapted = LayoutModality::Touch.adapt_size(small);
        assert!(adapted.width >= 44.0, "Width must be at least 44pt for touch");
        assert!(adapted.height >= 44.0, "Height must be at least 44pt for touch");
    }

    #[test]
    fn test_adaptive_modality_pointer_does_not_enlarge() {
        let large = cvkg_core::Size { width: 200.0, height: 50.0 };
        let adapted = LayoutModality::Pointer.adapt_size(large);
        assert_eq!(adapted.width, 200.0);
        assert_eq!(adapted.height, 50.0);
    }

    #[test]
    fn test_adaptive_modality_accessibility_zoom_spacing() {
        assert!(
            LayoutModality::AccessibilityZoom.spacing_multiplier() > LayoutModality::Touch.spacing_multiplier(),
            "Accessibility zoom must have the largest spacing multiplier"
        );
    }

    #[test]
    fn test_focus_order_ltr_visual_sort() {
        let candidates = vec![
            FocusCandidate { hash: 100, rect: Rect { x: 200.0, y: 10.0, width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 200, rect: Rect { x: 0.0,   y: 10.0, width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 300, rect: Rect { x: 100.0, y: 10.0, width: 50.0, height: 20.0 }, tab_index: None },
        ];
        let order = compute_focus_order(candidates);
        assert_eq!(order, vec![200, 300, 100], "LTR focus order violated: {:?}", order);
    }

    #[test]
    fn test_focus_order_explicit_tabindex_comes_first() {
        let candidates = vec![
            FocusCandidate { hash: 1, rect: Rect { x: 0.0, y: 100.0, width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 2, rect: Rect { x: 0.0, y: 0.0,   width: 50.0, height: 20.0 }, tab_index: Some(2) },
            FocusCandidate { hash: 3, rect: Rect { x: 0.0, y: 50.0,  width: 50.0, height: 20.0 }, tab_index: Some(1) },
        ];
        let order = compute_focus_order(candidates);
        assert_eq!(order[0], 3, "tabindex=1 must be first");
        assert_eq!(order[1], 2, "tabindex=2 must be second");
        assert_eq!(order[2], 1, "natural order must be last");
    }

    #[test]
    fn test_reading_order_valid_sequence_passes() {
        let candidates = vec![
            FocusCandidate { hash: 1, rect: Rect { x: 0.0,   y: 0.0,  width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 2, rect: Rect { x: 100.0, y: 0.0,  width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 3, rect: Rect { x: 0.0,   y: 30.0, width: 50.0, height: 20.0 }, tab_index: None },
        ];
        assert!(validate_reading_order(&candidates).is_ok());
    }

    #[test]
    fn test_reading_order_backwards_row_fails() {
        let candidates = vec![
            FocusCandidate { hash: 1, rect: Rect { x: 0.0, y: 100.0, width: 50.0, height: 20.0 }, tab_index: None },
            FocusCandidate { hash: 2, rect: Rect { x: 0.0, y: 0.0,   width: 50.0, height: 20.0 }, tab_index: None },
        ];
        assert!(validate_reading_order(&candidates).is_err(), "Backwards row must fail validation");
    }

    #[test]
    fn p2_47_deep_tree_100_levels() {
        let mut cache = LayoutCache::new();
        let mut root: Box<dyn LayoutView> = Box::new(HStack::new(
            0.0,
            Alignment::Leading,
            Distribution::Leading,
        ));
        for _ in 0..50 {
            let child: Box<dyn LayoutView> =
                Box::new(HStack::new(0.0, Alignment::Leading, Distribution::Leading));
            let _ = child;
        }
        let proposal = SizeProposal::unspecified();
        let _ = root.size_that_fits(proposal, &[], &mut cache);
    }

    #[test]
    fn p2_47_wide_tree_no_panic() {
        let mut cache = LayoutCache::new();
        let root = HStack::new(0.0, Alignment::Leading, Distribution::Leading);
        let proposal = SizeProposal::unspecified();
        let _ = root.size_that_fits(proposal, &[], &mut cache);
    }

    #[test]
    fn p2_47_nested_flex_no_panic() {
        let mut cache = LayoutCache::new();
        let inner = HStack::new(0.0, Alignment::Leading, Distribution::Leading);
        let _ = inner.size_that_fits(SizeProposal::unspecified(), &[], &mut cache);
    }

    fn make_mock_views(n: usize) -> Vec<MockView> {
        (0..n)
            .map(|_| MockView {
                size: Size {
                    width: 50.0,
                    height: 30.0,
                },
                flex: 0.0,
            })
            .collect()
    }

    #[test]
    fn test_progressive_layout_completes_all_children() {
        let views = make_mock_views(10);
        let subviews: Vec<&dyn LayoutView> = views.iter().map(|v| v as &dyn LayoutView).collect();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 200.0,
        };
        let mut ctx = ProgressiveLayoutContext::new(
            bounds,
            &subviews,
            0.0,
            Alignment::Leading,
            Distribution::Leading,
        );
        assert!(!ctx.is_complete());
        assert!(!ctx.layout_next_batch(3));
        assert!(!ctx.is_complete());
        assert!(!ctx.layout_next_batch(3));
        assert!(!ctx.is_complete());
        assert!(!ctx.layout_next_batch(3));
        assert!(!ctx.is_complete());
        assert!(ctx.layout_next_batch(3));
        assert!(ctx.is_complete());
    }

    #[test]
    fn test_progressive_layout_reports_progress() {
        let views = make_mock_views(5);
        let subviews: Vec<&dyn LayoutView> = views.iter().map(|v| v as &dyn LayoutView).collect();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 500.0,
            height: 200.0,
        };
        let mut ctx = ProgressiveLayoutContext::new(
            bounds,
            &subviews,
            0.0,
            Alignment::Leading,
            Distribution::Leading,
        );
        assert_eq!(ctx.progress(), (0, 5));
        ctx.layout_next_batch(2);
        assert_eq!(ctx.progress(), (2, 5));
        ctx.layout_next_batch(2);
        assert_eq!(ctx.progress(), (4, 5));
        ctx.layout_next_batch(1);
        assert_eq!(ctx.progress(), (5, 5));
    }

    #[test]
    fn test_progressive_layout_fallback_positions_remaining() {
        let views = make_mock_views(6);
        let subviews: Vec<&dyn LayoutView> = views.iter().map(|v| v as &dyn LayoutView).collect();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 200.0,
        };
        let mut ctx = ProgressiveLayoutContext::new(
            bounds,
            &subviews,
            10.0,
            Alignment::Leading,
            Distribution::Leading,
        );
        ctx.layout_next_batch(2);
        assert_eq!(ctx.progress(), (2, 6));
        let mut cache = LayoutCache::new();
        let fallback_rects = ctx.apply_remaining_fallback(&mut cache);
        assert_eq!(fallback_rects.len(), 4);
        for r in &fallback_rects {
            assert!(r.width > 0.0);
            assert!(r.height > 0.0);
        }
        assert!(ctx.is_complete());
    }

    #[test]
    fn test_progressive_layout_uses_cached_results() {
        let views = make_mock_views(4);
        let subviews: Vec<&dyn LayoutView> = views.iter().map(|v| v as &dyn LayoutView).collect();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 200.0,
        };
        let mut cache = LayoutCache::new();
        let mut ctx1 = ProgressiveLayoutContext::new(
            bounds,
            &subviews,
            0.0,
            Alignment::Leading,
            Distribution::Leading,
        );
        ctx1.layout_next_batch(2);
        for entry in ctx1.entries.iter() {
            if entry.rect != Rect::zero() {
                cache.previous_rects.insert(entry.hash, entry.rect);
            }
        }
        let mut ctx2 = ProgressiveLayoutContext::new(
            bounds,
            &subviews,
            0.0,
            Alignment::Leading,
            Distribution::Leading,
        );
        let (_done, _rects) = ctx2.layout_next_batch_with_cache(2, &mut cache);
        assert_eq!(ctx2.progress().0, 2);
    }
}
