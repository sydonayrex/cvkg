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
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

pub use cvkg_core::layout::EdgeInsets;
use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect, Size, SizeProposal};
use std::collections::HashMap;
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static ACTIVE_LAYOUT_NODES: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
}

/// Helper function to prevent layout calculation cycles in recursive size queries.
/// If a view is already being traversed on the current thread, returns the fallback size.
fn with_layout_cycle_guard<F, R>(hash: u64, fallback: R, f: F) -> R
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
    let res = f();
    ACTIVE_LAYOUT_NODES.with(|nodes| {
        nodes.borrow_mut().remove(&hash);
    });
    res
}

/// Helper function to prevent layout calculation cycles in recursive subview placements.
fn with_layout_cycle_guard_void<F>(hash: u64, f: F)
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
    f();
    ACTIVE_LAYOUT_NODES.with(|nodes| {
        nodes.borrow_mut().remove(&hash);
    });
}

/// The central Taffy engine that computes flexbox and grid layouts.
/// Stored opaquely inside `cvkg_core::LayoutCache::engine`.
pub struct TaffyLayoutEngine {
    pub tree: taffy::TaffyTree,
    pub node_map: HashMap<u64, taffy::NodeId>,
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TaffyLayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: taffy::TaffyTree::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn get_or_insert_engine(cache: &mut LayoutCache) -> &mut Self {
        if cache.engine.is_none() {
            cache.engine = Some(Box::new(TaffyLayoutEngine::new()));
        }
        cache
            .engine
            .as_mut()
            .unwrap()
            .downcast_mut::<TaffyLayoutEngine>()
            .unwrap()
    }
}

/// Manages active physics transitions for layout bounding boxes.
pub struct AnimationEngine {
    pub active_transitions: HashMap<u64, cvkg_anim::physics::ViscousSpring>,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            active_transitions: HashMap::new(),
        }
    }

    pub fn get_or_insert_engine(cache: &mut LayoutCache) -> &mut Self {
        if cache.animators.is_none() {
            cache.animators = Some(Box::new(AnimationEngine::new()));
        }
        cache
            .animators
            .as_mut()
            .unwrap()
            .downcast_mut::<AnimationEngine>()
            .unwrap()
    }
}

use taffy::prelude::*;

fn taffy_alignment(alignment: cvkg_core::Alignment) -> Option<taffy::AlignItems> {
    match alignment {
        cvkg_core::Alignment::Leading => Some(taffy::AlignItems::Start),
        cvkg_core::Alignment::Center => Some(taffy::AlignItems::Center),
        cvkg_core::Alignment::Trailing => Some(taffy::AlignItems::End),
        cvkg_core::Alignment::Top => Some(taffy::AlignItems::Start),
        cvkg_core::Alignment::Bottom => Some(taffy::AlignItems::End),
    }
}

fn taffy_distribution(dist: cvkg_core::Distribution) -> Option<taffy::JustifyContent> {
    match dist {
        cvkg_core::Distribution::Leading => Some(taffy::JustifyContent::Start),
        cvkg_core::Distribution::Center => Some(taffy::JustifyContent::Center),
        cvkg_core::Distribution::Trailing => Some(taffy::JustifyContent::End),
        cvkg_core::Distribution::SpaceBetween => Some(taffy::JustifyContent::SpaceBetween),
        cvkg_core::Distribution::Fill => Some(taffy::JustifyContent::Stretch),
        _ => None,
    }
}

/// Taffy flex layout parameters.
struct FlexParams {
    dir: taffy::FlexDirection,
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    bounds: Rect,
    container_hash: u64,
}

fn compute_taffy_flex(
    params: &FlexParams,
    subviews: &[&dyn LayoutView],
    cache: &mut LayoutCache,
) -> Vec<Rect> {
    let mut sizes = Vec::with_capacity(subviews.len());
    let mut hashes = Vec::with_capacity(subviews.len());
    let mut flex_weights = Vec::with_capacity(subviews.len());

    for child in subviews {
        let hash = child.view_hash();
        hashes.push(hash);
        flex_weights.push(child.flex_weight());

        let proposal = SizeProposal::new(Some(params.bounds.width), Some(params.bounds.height));
        let cached_size = if hash != 0 {
            cache.get_size(hash, proposal)
        } else {
            None
        };

        let size = match cached_size {
            Some(sz) => sz,
            None => {
                let sz = with_layout_cycle_guard(hash, Size::ZERO, || {
                    child.size_that_fits(proposal, &[], cache)
                });
                if hash != 0 {
                    cache.set_size(hash, proposal, sz);
                }
                sz
            }
        };
        if params.container_hash != 0 && hash != 0 {
            cache.register_parent(hash, params.container_hash);
        }
        sizes.push(size);
    }

    let engine = TaffyLayoutEngine::get_or_insert_engine(cache);
    let mut child_nodes = Vec::with_capacity(subviews.len());

    for ((&hash, &flex_weight), &size) in hashes.iter().zip(&flex_weights).zip(&sizes) {
        let style = if flex_weight > 0.0 {
            taffy::Style {
                size: taffy::Size {
                    width: if params.dir == taffy::FlexDirection::Row {
                        taffy::Dimension::Auto
                    } else {
                        taffy::Dimension::Length(size.width)
                    },
                    height: if params.dir == taffy::FlexDirection::Column {
                        taffy::Dimension::Auto
                    } else {
                        taffy::Dimension::Length(size.height)
                    },
                },
                flex_grow: flex_weight,
                flex_basis: taffy::Dimension::Percent(0.0),
                ..Default::default()
            }
        } else {
            taffy::Style {
                size: taffy::Size {
                    width: taffy::Dimension::Length(size.width),
                    height: taffy::Dimension::Length(size.height),
                },
                ..Default::default()
            }
        };

        let node = if hash != 0 {
            if let Some(&existing) = engine.node_map.get(&hash) {
                let _ = engine.tree.set_style(existing, style);
                existing
            } else {
                let new_node = engine.tree.new_leaf(style).unwrap();
                engine.node_map.insert(hash, new_node);
                new_node
            }
        } else {
            engine.tree.new_leaf(style).unwrap()
        };
        child_nodes.push(node);
    }

    let gap_val = taffy::LengthPercentage::Length(params.spacing);
    let container_style = taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: params.dir,
        gap: taffy::Size {
            width: if params.dir == taffy::FlexDirection::Row {
                gap_val
            } else {
                taffy::LengthPercentage::Length(0.0)
            },
            height: if params.dir == taffy::FlexDirection::Column {
                gap_val
            } else {
                taffy::LengthPercentage::Length(0.0)
            },
        },
        align_items: taffy_alignment(params.alignment),
        justify_content: taffy_distribution(params.distribution),
        size: taffy::Size {
            width: taffy::Dimension::Length(params.bounds.width),
            height: taffy::Dimension::Length(params.bounds.height),
        },
        ..Default::default()
    };

    let root_node = if params.container_hash != 0 {
        if let Some(&existing) = engine.node_map.get(&params.container_hash) {
            let _ = engine.tree.set_style(existing, container_style);
            let _ = engine.tree.set_children(existing, &child_nodes);
            existing
        } else {
            let new_node = engine
                .tree
                .new_with_children(container_style, &child_nodes)
                .unwrap();
            engine.node_map.insert(params.container_hash, new_node);
            new_node
        }
    } else {
        engine
            .tree
            .new_with_children(container_style, &child_nodes)
            .unwrap()
    };

    engine
        .tree
        .compute_layout(root_node, taffy::Size::MAX_CONTENT)
        .unwrap();

    let mut rects = Vec::with_capacity(subviews.len());
    for &node in &child_nodes {
        let layout = engine.tree.layout(node).unwrap();
        rects.push(Rect {
            x: params.bounds.x + layout.location.x,
            y: params.bounds.y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        });
    }

    if params.container_hash == 0 {
        let _ = engine.tree.remove(root_node);
    }

    rects
}

/// Applies view transitions to calculated layout rects.
fn apply_layout_animations(
    rects: Vec<Rect>,
    subviews: &mut [&mut dyn LayoutView],
    cache: &mut LayoutCache,
) {
    let mut transitions_to_update = Vec::new();

    for (child, target_rect) in subviews.iter().zip(&rects) {
        let hash = child.view_hash();
        if hash != 0 {
            if let Some(prev) = cache.previous_rects.get(&hash) {
                let dx = (prev.x - target_rect.x).abs();
                let dy = (prev.y - target_rect.y).abs();
                let dw = (prev.width - target_rect.width).abs();
                let dh = (prev.height - target_rect.height).abs();
                let epsilon = 1e-3;
                if dx > epsilon || dy > epsilon || dw > epsilon || dh > epsilon {
                    transitions_to_update.push((hash, *prev, *target_rect));
                }
            }
            cache.previous_rects.insert(hash, *target_rect);
        }
    }

    let mut interpolated_rects = HashMap::new();
    let delta = cache.delta_time;
    let scale = cache.scale_factor;
    let anim_engine = AnimationEngine::get_or_insert_engine(cache);

    for (hash, prev, target_rect) in transitions_to_update {
        let mut spring = if let Some(mut existing) = anim_engine.active_transitions.remove(&hash) {
            existing.position_b =
                cvkg_anim::physics::Vec3::new(target_rect.x, target_rect.y, target_rect.width);
            existing
        } else {
            cvkg_anim::physics::ViscousSpring::new(
                cvkg_anim::physics::Vec3::new(prev.x, prev.y, prev.width),
                cvkg_anim::physics::Vec3::new(target_rect.x, target_rect.y, target_rect.width),
                0.9,
                1000.0,
            )
        };
        spring.step(delta);

        // Temporal layout snapping: snap layout coordinates to integer pixels
        // only when the spring has nearly settled to prevent jitter during motion.
        let speed = (spring.velocity_a.length_sq() + spring.velocity_b.length_sq()).sqrt();
        let snap = |v: f32| (v * scale).round() / scale;

        let (rx, ry, rw) = if speed < 0.05 {
            (
                snap(spring.position_a.x),
                snap(spring.position_a.y),
                snap(spring.position_a.z),
            )
        } else {
            (
                spring.position_a.x,
                spring.position_a.y,
                spring.position_a.z,
            )
        };

        interpolated_rects.insert(
            hash,
            Rect {
                x: rx,
                y: ry,
                width: rw,
                height: target_rect.height,
            },
        );
        anim_engine.active_transitions.insert(hash, spring);
    }

    for (child, mut target_rect) in subviews.iter_mut().zip(rects) {
        let hash = child.view_hash();
        if let Some(interp) = interpolated_rects.get(&hash) {
            target_rect = *interp;
        }
        let is_visible = if let Some(viewport) = cache.viewport {
            target_rect.intersects(&viewport)
        } else {
            true
        };
        if is_visible {
            with_layout_cycle_guard_void(hash, || {
                child.place_subviews(target_rect, &mut [], cache);
            });
        }
    }
}

/// HStack - lays out children horizontally
pub struct HStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl HStack {
    /// Create a new HStack with the given spacing, alignment, and distribution
    pub fn new(spacing: f32, alignment: Alignment, distribution: Distribution) -> Self {
        Self {
            spacing,
            alignment,
            distribution,
        }
    }

    /// Compute the layout rects for children without placing them.
    pub fn compute_layout(
        spacing: f32,
        alignment: Alignment,
        distribution: Distribution,
        bounds: Rect,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        Self::compute_layout_incremental(
            spacing,
            alignment,
            distribution,
            bounds,
            0,
            subviews,
            cache,
        )
    }

    pub fn compute_layout_incremental(
        spacing: f32,
        alignment: Alignment,
        distribution: Distribution,
        bounds: Rect,
        container_hash: u64,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        compute_taffy_flex(
            &FlexParams {
                dir: taffy::FlexDirection::Row,
                spacing,
                alignment,
                distribution,
                bounds,
                container_hash,
            },
            subviews,
            cache,
        )
    }
}

impl LayoutView for HStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: proposal.width.unwrap_or(10000.0),
            height: proposal.height.unwrap_or(10000.0),
        };
        let rects = Self::compute_layout_incremental(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            self.view_hash(),
            subviews,
            cache,
        );

        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for r in rects {
            max_w = max_w.max(r.x + r.width);
            max_h = max_h.max(r.y + r.height);
        }
        Size {
            width: max_w,
            height: max_h,
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let views: Vec<&dyn LayoutView> =
            subviews.iter().map(|v| &**v as &dyn LayoutView).collect();
        let rects = Self::compute_layout_incremental(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            self.view_hash(),
            &views,
            cache,
        );
        apply_layout_animations(rects, subviews, cache);
    }
}

/// VStack - lays out children vertically
pub struct VStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl VStack {
    /// Create a new VStack with the given spacing, alignment, and distribution
    pub fn new(spacing: f32, alignment: Alignment, distribution: Distribution) -> Self {
        Self {
            spacing,
            alignment,
            distribution,
        }
    }

    /// Compute the layout rects for children without placing them.
    pub fn compute_layout(
        spacing: f32,
        alignment: Alignment,
        distribution: Distribution,
        bounds: Rect,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        Self::compute_layout_incremental(
            spacing,
            alignment,
            distribution,
            bounds,
            0,
            subviews,
            cache,
        )
    }

    pub fn compute_layout_incremental(
        spacing: f32,
        alignment: Alignment,
        distribution: Distribution,
        bounds: Rect,
        container_hash: u64,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        compute_taffy_flex(
            &FlexParams {
                dir: taffy::FlexDirection::Column,
                spacing,
                alignment,
                distribution,
                bounds,
                container_hash,
            },
            subviews,
            cache,
        )
    }
}

impl LayoutView for VStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: proposal.width.unwrap_or(10000.0),
            height: proposal.height.unwrap_or(10000.0),
        };
        let rects = Self::compute_layout_incremental(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            self.view_hash(),
            subviews,
            cache,
        );

        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for r in rects {
            max_w = max_w.max(r.x + r.width);
            max_h = max_h.max(r.y + r.height);
        }
        Size {
            width: max_w,
            height: max_h,
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let views: Vec<&dyn LayoutView> =
            subviews.iter().map(|v| &**v as &dyn LayoutView).collect();
        let rects = Self::compute_layout_incremental(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            self.view_hash(),
            &views,
            cache,
        );
        apply_layout_animations(rects, subviews, cache);
    }
}

/// ZStack - lays out children on top of each other
pub struct ZStack {}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ZStack {
    /// Create a new ZStack
    pub fn new() -> Self {
        Self {}
    }
}

impl LayoutView for ZStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let self_hash = self.view_hash();

        for child in subviews.iter() {
            let child_hash = child.view_hash();
            if self_hash != 0 && child_hash != 0 {
                cache.register_parent(child_hash, self_hash);
            }
            let child_size = with_layout_cycle_guard(child_hash, Size::ZERO, || {
                child.size_that_fits(proposal, &[], cache)
            });
            width = width.max(child_size.width);
            height = height.max(child_size.height);
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let self_hash = self.view_hash();
        for child in subviews.iter_mut() {
            let child_hash = child.view_hash();
            if self_hash != 0 && child_hash != 0 {
                cache.register_parent(child_hash, self_hash);
            }
            let is_visible = if let Some(viewport) = cache.viewport {
                bounds.intersects(&viewport)
            } else {
                true
            };
            if is_visible {
                with_layout_cycle_guard_void(child_hash, || {
                    child.place_subviews(bounds, &mut [], cache);
                });
            }
        }
    }
}

/// Spacer - a layout view that expands to fill available space
pub struct Spacer;

impl LayoutView for Spacer {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(0.0),
            height: proposal.height.unwrap_or(0.0),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Flex - a container that distributes space among its children flexibly
pub struct Flex {
    pub orientation: cvkg_core::Orientation,
    pub spacing: f32,
}

impl Flex {
    pub fn new(orientation: cvkg_core::Orientation, spacing: f32) -> Self {
        Self {
            orientation,
            spacing,
        }
    }
}

impl LayoutView for Flex {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(100.0),
            height: proposal.height.unwrap_or(100.0),
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        if subviews.is_empty() {
            return;
        }

        let self_hash = self.view_hash();
        let n = subviews.len() as f32;
        match self.orientation {
            cvkg_core::Orientation::Horizontal => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_width = (bounds.width - total_spacing) / n;
                for (i, child) in subviews.iter_mut().enumerate() {
                    let child_rect = Rect {
                        x: bounds.x + i as f32 * (item_width + self.spacing),
                        y: bounds.y,
                        width: item_width,
                        height: bounds.height,
                    };
                    let child_hash = child.view_hash();
                    if self_hash != 0 && child_hash != 0 {
                        cache.register_parent(child_hash, self_hash);
                    }
                    let is_visible = if let Some(viewport) = cache.viewport {
                        child_rect.intersects(&viewport)
                    } else {
                        true
                    };
                    if is_visible {
                        with_layout_cycle_guard_void(child_hash, || {
                            child.place_subviews(child_rect, &mut [], cache);
                        });
                    }
                }
            }
            cvkg_core::Orientation::Vertical => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_height = (bounds.height - total_spacing) / n;
                for (i, child) in subviews.iter_mut().enumerate() {
                    let child_rect = Rect {
                        x: bounds.x,
                        y: bounds.y + i as f32 * (item_height + self.spacing),
                        width: bounds.width,
                        height: item_height,
                    };
                    let child_hash = child.view_hash();
                    if self_hash != 0 && child_hash != 0 {
                        cache.register_parent(child_hash, self_hash);
                    }
                    let is_visible = if let Some(viewport) = cache.viewport {
                        child_rect.intersects(&viewport)
                    } else {
                        true
                    };
                    if is_visible {
                        with_layout_cycle_guard_void(child_hash, || {
                            child.place_subviews(child_rect, &mut [], cache);
                        });
                    }
                }
            }
        }
    }
}

/// Track sizing strategy for a single grid track (row or column).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridTrack {
    /// Exact pixel size.
    Fixed(f32),
    /// Proportional weight compared to other flex tracks.
    Flex(f32),
    /// Size based on the intrinsic size of the grid item.
    Auto,
    /// Size clamped between minimum and maximum bounds.
    MinMax(f32, f32),
}

fn taffy_track(track: GridTrack) -> taffy::TrackSizingFunction {
    match track {
        GridTrack::Fixed(v) => taffy::prelude::length(v),
        GridTrack::Flex(v) => taffy::prelude::fr(v),
        GridTrack::Auto => taffy::prelude::auto(),
        GridTrack::MinMax(min, max) => {
            taffy::prelude::minmax(taffy::prelude::length(min), taffy::prelude::length(max))
        }
    }
}

/// A layout engine that computes coordinates for children positioned in a 2D grid.
pub struct Grid {
    /// Column track sizing rules.
    pub columns: Vec<GridTrack>,
    /// Row track sizing rules.
    pub rows: Vec<GridTrack>,
    /// Empty space between columns.
    pub column_gap: f32,
    /// Empty space between rows.
    pub row_gap: f32,
}

impl Grid {
    /// Creates a new Grid layout engine.
    pub fn new(
        columns: Vec<GridTrack>,
        rows: Vec<GridTrack>,
        column_gap: f32,
        row_gap: f32,
    ) -> Self {
        Self {
            columns,
            rows,
            column_gap,
            row_gap,
        }
    }

    /// Computes the rects for children based on track sizing and grid placements.
    pub fn compute_layout_rects(
        &self,
        bounds: Rect,
        subviews: &[&dyn LayoutView],
        placements: &[Option<cvkg_core::GridPlacement>],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        self.compute_layout_rects_incremental(bounds, 0, subviews, placements, cache)
    }

    pub fn compute_layout_rects_incremental(
        &self,
        bounds: Rect,
        container_hash: u64,
        subviews: &[&dyn LayoutView],
        placements: &[Option<cvkg_core::GridPlacement>],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        let mut hashes = Vec::with_capacity(subviews.len());
        for child in subviews {
            let hash = child.view_hash();
            hashes.push(hash);
            if container_hash != 0 && hash != 0 {
                cache.register_parent(hash, container_hash);
            }
        }

        let engine = TaffyLayoutEngine::get_or_insert_engine(cache);
        let mut child_nodes = Vec::with_capacity(subviews.len());

        for (hash, placement) in hashes.iter().zip(placements.iter()) {
            let style = if let Some(p) = placement.as_ref() {
                taffy::Style {
                    size: taffy::Size {
                        width: taffy::Dimension::Auto,
                        height: taffy::Dimension::Auto,
                    },
                    grid_column: taffy::Line {
                        start: taffy::prelude::line((p.column + 1) as i16),
                        end: taffy::prelude::span(p.column_span as u16),
                    },
                    grid_row: taffy::Line {
                        start: taffy::prelude::line((p.row + 1) as i16),
                        end: taffy::prelude::span(p.row_span as u16),
                    },
                    ..Default::default()
                }
            } else {
                taffy::Style {
                    size: taffy::Size {
                        width: taffy::Dimension::Auto,
                        height: taffy::Dimension::Auto,
                    },
                    ..Default::default()
                }
            };

            let node = if *hash != 0 {
                if let Some(&existing) = engine.node_map.get(hash) {
                    let _ = engine.tree.set_style(existing, style);
                    existing
                } else {
                    let new_node = engine.tree.new_leaf(style).unwrap();
                    engine.node_map.insert(*hash, new_node);
                    new_node
                }
            } else {
                engine.tree.new_leaf(style).unwrap()
            };
            child_nodes.push(node);
        }

        let container_style = taffy::Style {
            display: taffy::Display::Grid,
            grid_template_columns: self.columns.iter().copied().map(taffy_track).collect(),
            grid_template_rows: self.rows.iter().copied().map(taffy_track).collect(),
            gap: taffy::Size {
                width: taffy::LengthPercentage::Length(self.column_gap),
                height: taffy::LengthPercentage::Length(self.row_gap),
            },
            size: taffy::Size {
                width: taffy::Dimension::Length(bounds.width),
                height: taffy::Dimension::Length(bounds.height),
            },
            ..Default::default()
        };

        let root_node = if container_hash != 0 {
            if let Some(&existing) = engine.node_map.get(&container_hash) {
                let _ = engine.tree.set_style(existing, container_style);
                let _ = engine.tree.set_children(existing, &child_nodes);
                existing
            } else {
                let new_node = engine
                    .tree
                    .new_with_children(container_style, &child_nodes)
                    .unwrap();
                engine.node_map.insert(container_hash, new_node);
                new_node
            }
        } else {
            engine
                .tree
                .new_with_children(container_style, &child_nodes)
                .unwrap()
        };

        engine
            .tree
            .compute_layout(root_node, taffy::Size::MAX_CONTENT)
            .unwrap();

        let mut rects = Vec::with_capacity(subviews.len());
        for &node in &child_nodes {
            let layout = engine.tree.layout(node).unwrap();
            rects.push(Rect {
                x: bounds.x + layout.location.x,
                y: bounds.y + layout.location.y,
                width: layout.size.width,
                height: layout.size.height,
            });
        }

        if container_hash == 0 {
            let _ = engine.tree.remove(root_node);
        }
        rects
    }
}

impl LayoutView for Grid {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(200.0),
            height: proposal.height.unwrap_or(200.0),
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let views: Vec<&dyn LayoutView> =
            subviews.iter().map(|v| &**v as &dyn LayoutView).collect();
        let placements = vec![None; subviews.len()];
        let rects = self.compute_layout_rects_incremental(
            bounds,
            self.view_hash(),
            &views,
            &placements,
            cache,
        );
        apply_layout_animations(rects, subviews, cache);
    }
}

// =============================================================================
// PADDING
// =============================================================================

/// A layout view that adds padding around its child.
pub struct Padding {
    pub insets: EdgeInsets,
}

impl Padding {
    pub fn new(insets: EdgeInsets) -> Self {
        Self { insets }
    }

    pub fn uniform(value: f32) -> Self {
        Self {
            insets: EdgeInsets::all(value),
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            insets: EdgeInsets {
                top: vertical,
                bottom: vertical,
                leading: horizontal,
                trailing: horizontal,
            },
        }
    }
}

impl LayoutView for Padding {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let inner_proposal = SizeProposal::new(
            proposal
                .width
                .map(|w| (w - self.insets.leading - self.insets.trailing).max(0.0)),
            proposal
                .height
                .map(|h| (h - self.insets.top - self.insets.bottom).max(0.0)),
        );
        let self_hash = self.view_hash();
        let child_size = if subviews.is_empty() {
            Size::ZERO
        } else {
            let child_hash = subviews[0].view_hash();
            if self_hash != 0 && child_hash != 0 {
                cache.register_parent(child_hash, self_hash);
            }
            with_layout_cycle_guard(child_hash, Size::ZERO, || {
                subviews[0].size_that_fits(inner_proposal, &[], cache)
            })
        };
        Size {
            width: child_size.width + self.insets.leading + self.insets.trailing,
            height: child_size.height + self.insets.top + self.insets.bottom,
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let inner = Rect {
            x: bounds.x + self.insets.leading,
            y: bounds.y + self.insets.top,
            width: (bounds.width - self.insets.leading - self.insets.trailing).max(0.0),
            height: (bounds.height - self.insets.top - self.insets.bottom).max(0.0),
        };
        let self_hash = self.view_hash();
        for child in subviews.iter_mut() {
            let child_hash = child.view_hash();
            if self_hash != 0 && child_hash != 0 {
                cache.register_parent(child_hash, self_hash);
            }
            let is_visible = if let Some(viewport) = cache.viewport {
                inner.intersects(&viewport)
            } else {
                true
            };
            if is_visible {
                with_layout_cycle_guard_void(child_hash, || {
                    child.place_subviews(inner, &mut [], cache);
                });
            }
        }
    }
}

// =============================================================================
// SAFE AREA
// =============================================================================

/// A layout view that respects safe area insets (notches, status bars).
pub struct SafeArea {
    pub edges: SafeAreaEdges,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeAreaEdges {
    pub top: bool,
    pub bottom: bool,
    pub leading: bool,
    pub trailing: bool,
}

impl Default for SafeAreaEdges {
    fn default() -> Self {
        Self {
            top: true,
            bottom: true,
            leading: false,
            trailing: false,
        }
    }
}

impl SafeArea {
    pub fn all() -> Self {
        Self {
            edges: SafeAreaEdges {
                top: true,
                bottom: true,
                leading: true,
                trailing: true,
            },
        }
    }

    pub fn vertical() -> Self {
        Self {
            edges: SafeAreaEdges::default(),
        }
    }

    fn insets(&self) -> EdgeInsets {
        EdgeInsets {
            top: if self.edges.top { 44.0 } else { 0.0 },
            bottom: if self.edges.bottom { 34.0 } else { 0.0 },
            leading: 0.0,
            trailing: 0.0,
        }
    }
}

impl LayoutView for SafeArea {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        Padding::new(self.insets()).size_that_fits(proposal, subviews, cache)
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        Padding::new(self.insets()).place_subviews(bounds, subviews, cache);
    }
}

// =============================================================================
// ASPECT RATIO
// =============================================================================

/// Constrains a child to a specific aspect ratio.
pub struct AspectRatio {
    pub ratio: f32,
}

impl AspectRatio {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.max(0.01),
        }
    }

    pub fn square() -> Self {
        Self::new(1.0)
    }

    pub fn widescreen() -> Self {
        Self::new(16.0 / 9.0)
    }

    pub fn portrait() -> Self {
        Self::new(9.0 / 16.0)
    }

    fn fitted_size(&self, proposal: SizeProposal) -> Size {
        let max_w = proposal.width.unwrap_or(f32::MAX);
        let max_h = proposal.height.unwrap_or(f32::MAX);
        let w = max_w;
        let h = w / self.ratio;
        if h <= max_h {
            return Size {
                width: w,
                height: h,
            };
        }
        Size {
            width: max_h * self.ratio,
            height: max_h,
        }
    }
}

impl LayoutView for AspectRatio {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        if subviews.is_empty() {
            return self.fitted_size(proposal);
        }
        let self_hash = self.view_hash();
        let child = subviews[0];
        let child_hash = child.view_hash();
        if self_hash != 0 && child_hash != 0 {
            cache.register_parent(child_hash, self_hash);
        }
        let child_size = with_layout_cycle_guard(child_hash, Size::ZERO, || {
            child.size_that_fits(
                SizeProposal::new(Some(f32::MAX), Some(f32::MAX)),
                &[],
                cache,
            )
        });
        let intrinsic_ratio = child_size.width / child_size.height.max(0.01);
        if (intrinsic_ratio - self.ratio).abs() < 0.01 {
            return self.fitted_size(proposal);
        }
        let fit = self.fitted_size(proposal);
        let child_w = fit.width.min(child_size.width);
        let child_h = child_w / intrinsic_ratio;
        let final_h = child_h.min(fit.height);
        let final_w = final_h * intrinsic_ratio;
        Size {
            width: final_w,
            height: final_h,
        }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let fit = self.fitted_size(SizeProposal::new(Some(bounds.width), Some(bounds.height)));
        let x = bounds.x + (bounds.width - fit.width) * 0.5;
        let y = bounds.y + (bounds.height - fit.height) * 0.0;
        let inner = Rect {
            x,
            y,
            width: fit.width,
            height: fit.height,
        };
        let self_hash = self.view_hash();
        for child in subviews.iter_mut() {
            let child_hash = child.view_hash();
            if self_hash != 0 && child_hash != 0 {
                cache.register_parent(child_hash, self_hash);
            }
            let is_visible = if let Some(viewport) = cache.viewport {
                inner.intersects(&viewport)
            } else {
                true
            };
            if is_visible {
                with_layout_cycle_guard_void(child_hash, || {
                    child.place_subviews(inner, &mut [], cache);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        }; // Flex
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
        ); // 160 - 50 - 10 = 100
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
        // The cycle should be broken and return the fallback size of 42
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

        // Verify both are in the cache
        assert!(cache.get_size(child_hash, SizeProposal::unspecified()).is_some());
        assert!(cache.get_size(parent_hash, SizeProposal::unspecified()).is_some());

        // Invalidate child
        cache.invalidate_view(child_hash);

        // Child invalidation must propagate bottom-up and invalidate parent too!
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
            rect: Rect::new(500.0, 0.0, 50.0, 50.0), // Offscreen
        };

        let mut cache = LayoutCache::new();
        // Viewport only covers the first view (ends at 55.0, second child is at 60.0)
        cache.viewport = Some(Rect::new(0.0, 0.0, 55.0, 100.0));

        let mut v1 = view1;
        let mut v2 = view2;
        let mut mut_subviews: Vec<&mut dyn LayoutView> = vec![&mut v1, &mut v2];

        HStack::new(10.0, Alignment::Center, Distribution::Leading)
            .place_subviews(Rect::new(0.0, 0.0, 600.0, 100.0), &mut mut_subviews, &mut cache);

        // Since viewport-aware culling is enabled and only view1 intersects it,
        // view2.place_subviews should be bypassed/culled.
        // Therefore calls count should be 1.
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
