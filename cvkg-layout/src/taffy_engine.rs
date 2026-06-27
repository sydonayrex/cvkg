use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect, Size, SizeProposal};
use std::collections::HashMap;
use taffy::prelude::*;

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
    /// Creates a new TaffyLayoutEngine.
    pub fn new() -> Self {
        Self {
            tree: taffy::TaffyTree::new(),
            node_map: HashMap::new(),
        }
    }

    /// Retrieves or initializes the TaffyLayoutEngine in the layout cache.
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

pub fn taffy_alignment(alignment: cvkg_core::Alignment) -> Option<taffy::AlignItems> {
    match alignment {
        cvkg_core::Alignment::Leading => Some(taffy::AlignItems::Start),
        cvkg_core::Alignment::Center => Some(taffy::AlignItems::Center),
        cvkg_core::Alignment::Trailing => Some(taffy::AlignItems::End),
        cvkg_core::Alignment::Top => Some(taffy::AlignItems::Start),
        cvkg_core::Alignment::Bottom => Some(taffy::AlignItems::End),
    }
}

pub fn taffy_distribution(dist: cvkg_core::Distribution) -> Option<taffy::JustifyContent> {
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
#[derive(Clone, Copy)]
pub struct FlexParams {
    pub dir: taffy::FlexDirection,
    pub spacing: f32,
    pub alignment: cvkg_core::Alignment,
    pub distribution: cvkg_core::Distribution,
    pub bounds: Rect,
    pub container_hash: u64,
}

/// Collect intrinsic sizes for all children without running the Taffy solver.
pub fn collect_child_sizes(
    subviews: &[&dyn LayoutView],
    bounds: Rect,
    cache: &mut LayoutCache,
) -> (Vec<u64>, Vec<f32>, Vec<Size>) {
    let mut sizes = Vec::with_capacity(subviews.len());
    let mut hashes = Vec::with_capacity(subviews.len());
    let mut flex_weights = Vec::with_capacity(subviews.len());

    for child in subviews {
        let hash = child.view_hash();
        hashes.push(hash);
        flex_weights.push(child.flex_weight());

        let proposal = SizeProposal::new(Some(bounds.width), Some(bounds.height));
        let cached_size = if hash != 0 {
            cache.get_size(hash, proposal)
        } else {
            None
        };

        let size = match cached_size {
            Some(sz) => sz,
            None => {
                let sz = crate::with_layout_cycle_guard(hash, Size::ZERO, || {
                    child.size_that_fits(proposal, &[], cache)
                });
                if hash != 0 {
                    cache.set_size(hash, proposal, sz);
                }
                sz
            }
        };
        if hash != 0 {
            cache.register_parent(hash, 0);
        }
        sizes.push(size);
    }

    (hashes, flex_weights, sizes)
}

/// Compute the natural (intrinsic) size of a flex container from child sizes.
pub fn intrinsic_flex_size(dir: taffy::FlexDirection, spacing: f32, sizes: &[Size]) -> Size {
    if sizes.is_empty() {
        return Size::ZERO;
    }
    let n = sizes.len();
    match dir {
        taffy::FlexDirection::Row | taffy::FlexDirection::RowReverse => {
            let total_width: f32 = sizes.iter().map(|s| s.width).sum();
            let max_height: f32 = sizes.iter().map(|s| s.height).fold(0.0, f32::max);
            Size {
                width: total_width + spacing * (n.saturating_sub(1) as f32),
                height: max_height,
            }
        }
        taffy::FlexDirection::Column | taffy::FlexDirection::ColumnReverse => {
            let max_width: f32 = sizes.iter().map(|s| s.width).fold(0.0, f32::max);
            let total_height: f32 = sizes.iter().map(|s| s.height).sum();
            Size {
                width: max_width,
                height: total_height + spacing * (n.saturating_sub(1) as f32),
            }
        }
    }
}

pub fn compute_taffy_flex(
    params: &FlexParams,
    subviews: &[&dyn LayoutView],
    cache: &mut LayoutCache,
) -> Vec<Rect> {


    if cache.is_over_budget() {
        let all_cached = subviews.iter().all(|child| {
            let hash = child.view_hash();
            hash != 0 && cache.previous_rects.contains_key(&hash)
        });
        if all_cached {
            let mut rects = Vec::with_capacity(subviews.len());
            for child in subviews {
                let hash = child.view_hash();
                rects.push(*cache.previous_rects.get(&hash).unwrap());
            }
            return rects;
        }
    }

    let (hashes, flex_weights, sizes) = collect_child_sizes(subviews, params.bounds, cache);

    for &hash in &hashes {
        if hash != 0 && params.container_hash != 0 {
            cache.register_parent(hash, params.container_hash);
        }
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

/// HStack - lays out children horizontally
pub struct HStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl HStack {
    /// Create a new HStack with the given spacing, alignment, and distribution.
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
        let (_, _, sizes) = collect_child_sizes(subviews, bounds, cache);
        intrinsic_flex_size(taffy::FlexDirection::Row, self.spacing, &sizes)
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
        crate::animation::apply_layout_animations(rects, subviews, cache);
    }
}

/// VStack - lays out children vertically
pub struct VStack {
    spacing: f32,
    alignment: Alignment,
    distribution: Distribution,
}

impl VStack {
    /// Create a new VStack with the given spacing, alignment, and distribution.
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
        let (_, _, sizes) = collect_child_sizes(subviews, bounds, cache);
        intrinsic_flex_size(taffy::FlexDirection::Column, self.spacing, &sizes)
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
        crate::animation::apply_layout_animations(rects, subviews, cache);
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
    /// Create a new ZStack.
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
            let child_size = crate::with_layout_cycle_guard(child_hash, Size::ZERO, || {
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
                crate::with_layout_cycle_guard_void(child_hash, || {
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
                        crate::with_layout_cycle_guard_void(child_hash, || {
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
                        crate::with_layout_cycle_guard_void(child_hash, || {
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

pub fn taffy_track(track: GridTrack) -> taffy::TrackSizingFunction {
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
    pub columns: Vec<GridTrack>,
    pub rows: Vec<GridTrack>,
    pub column_gap: f32,
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
        if cache.is_over_budget() {
            let mut rects = Vec::with_capacity(subviews.len());
            for child in subviews {
                let hash = child.view_hash();
                let r = if hash != 0 {
                    cache
                        .previous_rects
                        .get(&hash)
                        .copied()
                        .unwrap_or(Rect::zero())
                } else {
                    Rect::zero()
                };
                rects.push(r);
            }
            return rects;
        }

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
        crate::animation::apply_layout_animations(rects, subviews, cache);
    }
}
