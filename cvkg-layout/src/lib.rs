//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect, Size, SizeProposal};

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
        let n = subviews.len();
        if n == 0 {
            return Vec::new();
        }

        let mut rects = vec![Rect::zero(); n];
        let mut child_sizes = Vec::with_capacity(n);
        let mut total_fixed_width = 0.0;
        let mut total_flex_weight = 0.0;
        let mut flex_indices = Vec::new();

        // Pass 1: Categorize children and measure fixed ones
        for (i, child) in subviews.iter().enumerate() {
            let weight = child.flex_weight();
            if weight > 0.0 {
                total_flex_weight += weight;
                flex_indices.push(i);
                child_sizes.push(Size::ZERO); // Placeholder
            } else {
                let desired = child.size_that_fits(
                    SizeProposal::new(Some(bounds.width), Some(bounds.height)),
                    &[],
                    cache,
                );
                child_sizes.push(desired);
                total_fixed_width += desired.width;
            }
        }

        let total_spacing = spacing * (n - 1) as f32;
        let available_for_flex = (bounds.width - total_fixed_width - total_spacing).max(0.0);

        // Pass 2: Measure and size flexible children
        for &idx in &flex_indices {
            let weight = subviews[idx].flex_weight();
            let flex_width = (weight / total_flex_weight) * available_for_flex;
            let desired = subviews[idx].size_that_fits(
                SizeProposal::new(Some(flex_width), Some(bounds.height)),
                &[],
                cache,
            );
            // Flexible children take the width assigned by flex, but height can still be intrinsic or frame-constrained
            child_sizes[idx] = Size {
                width: flex_width,
                height: desired.height,
            };
        }

        let content_width = if total_flex_weight > 0.0 {
            bounds.width - total_spacing
        } else {
            total_fixed_width
        } + total_spacing;

        let (mut x, actual_spacing) = match distribution {
            Distribution::Leading | Distribution::Fill if total_flex_weight > 0.0 => {
                (bounds.x, spacing)
            }
            Distribution::Leading | Distribution::Fill => (bounds.x, spacing),
            Distribution::Trailing => (bounds.x + bounds.width - content_width, spacing),
            Distribution::Center => (bounds.x + (bounds.width - content_width) / 2.0, spacing),
            Distribution::SpaceBetween => {
                let s = if n > 1 {
                    (bounds.width - (total_fixed_width + available_for_flex)) / (n - 1) as f32
                } else {
                    0.0
                };
                (bounds.x, s)
            }
            _ => (bounds.x, spacing), // Simplification for mixed flex/distribution
        };

        for i in 0..n {
            let size = child_sizes[i];
            let y = match alignment {
                Alignment::Top => bounds.y,
                Alignment::Bottom => bounds.y + bounds.height - size.height,
                _ => bounds.y + (bounds.height - size.height) / 2.0,
            };

            rects[i] = Rect {
                x,
                y,
                width: size.width,
                height: size.height,
            };
            x += size.width + actual_spacing;
        }
        rects
    }
}

impl LayoutView for HStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in subviews.iter().enumerate() {
            let child_size = child.size_that_fits(proposal, &[], cache);
            width += child_size.width;
            height = height.max(child_size.height);

            if i < subviews.len() - 1 {
                width += self.spacing;
            }
        }

        Size {
            width: proposal.width.unwrap_or(width),
            height: proposal.height.unwrap_or(height),
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
        let rects = Self::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &views,
            cache,
        );

        for (child, rect) in subviews.iter_mut().zip(rects) {
            child.place_subviews(rect, &mut [], cache);
        }
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
        let n = subviews.len();
        if n == 0 {
            return Vec::new();
        }

        let mut rects = vec![Rect::zero(); n];
        let mut child_sizes = Vec::with_capacity(n);
        let mut total_fixed_height = 0.0;
        let mut total_flex_weight = 0.0;
        let mut flex_indices = Vec::new();

        // Pass 1: Categorize children and measure fixed ones
        for (i, child) in subviews.iter().enumerate() {
            let weight = child.flex_weight();
            if weight > 0.0 {
                total_flex_weight += weight;
                flex_indices.push(i);
                child_sizes.push(Size::ZERO); // Placeholder
            } else {
                let desired = child.size_that_fits(
                    SizeProposal::new(Some(bounds.width), Some(bounds.height)),
                    &[],
                    cache,
                );
                child_sizes.push(desired);
                total_fixed_height += desired.height;
            }
        }

        let total_spacing = spacing * (n - 1) as f32;
        let available_for_flex = (bounds.height - total_fixed_height - total_spacing).max(0.0);

        // Pass 2: Measure and size flexible children
        for &idx in &flex_indices {
            let weight = subviews[idx].flex_weight();
            let flex_height = (weight / total_flex_weight) * available_for_flex;
            let desired = subviews[idx].size_that_fits(
                SizeProposal::new(Some(bounds.width), Some(flex_height)),
                &[],
                cache,
            );
            child_sizes[idx] = Size {
                width: desired.width,
                height: flex_height,
            };
        }

        let content_height = if total_flex_weight > 0.0 {
            bounds.height - total_spacing
        } else {
            total_fixed_height
        } + total_spacing;

        let (mut y, actual_spacing) = match distribution {
            Distribution::Leading | Distribution::Fill if total_flex_weight > 0.0 => {
                (bounds.y, spacing)
            }
            Distribution::Leading | Distribution::Fill => (bounds.y, spacing),
            Distribution::Trailing => (bounds.y + bounds.height - content_height, spacing),
            Distribution::Center => (bounds.y + (bounds.height - content_height) / 2.0, spacing),
            Distribution::SpaceBetween => {
                let s = if n > 1 {
                    (bounds.height - (total_fixed_height + available_for_flex)) / (n - 1) as f32
                } else {
                    0.0
                };
                (bounds.y, s)
            }
            _ => (bounds.y, spacing),
        };

        for i in 0..n {
            let size = child_sizes[i];
            let x = match alignment {
                Alignment::Leading => bounds.x,
                Alignment::Trailing => bounds.x + bounds.width - size.width,
                _ => bounds.x + (bounds.width - size.width) / 2.0,
            };

            rects[i] = Rect {
                x,
                y,
                width: size.width,
                height: size.height,
            };
            y += size.height + actual_spacing;
        }
        rects
    }
}

impl LayoutView for VStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in subviews.iter().enumerate() {
            let child_size = child.size_that_fits(proposal, &[], cache);
            width = width.max(child_size.width);
            height += child_size.height;

            if i < subviews.len() - 1 {
                height += self.spacing;
            }
        }

        Size {
            width: proposal.width.unwrap_or(width),
            height: proposal.height.unwrap_or(height),
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
        let rects = Self::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &views,
            cache,
        );

        for (child, rect) in subviews.iter_mut().zip(rects) {
            child.place_subviews(rect, &mut [], cache);
        }
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
        // For ZStack, we want the maximum width and height of all children
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for child in subviews.iter() {
            let child_size = child.size_that_fits(proposal, &[], cache);
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
        // In ZStack, all children get the same bounds (they stack on top of each other)
        for child in subviews.iter_mut() {
            child.place_subviews(bounds, &mut [], cache);
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
                    child.place_subviews(child_rect, &mut [], cache);
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
                    child.place_subviews(child_rect, &mut [], cache);
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
        let col_count = self.columns.len();
        let row_count = self.rows.len();
        if col_count == 0 || row_count == 0 || subviews.is_empty() {
            return vec![Rect::zero(); subviews.len()];
        }

        let mut rects = vec![Rect::zero(); subviews.len()];

        // 1. Resolve placements for all children.
        // If a child has no placement, auto-position it in the next available cell in row-major order.
        let mut resolved_placements = Vec::with_capacity(subviews.len());
        let mut occupied = vec![vec![false; col_count]; row_count];

        for &opt_placement in placements.iter() {
            let placement = if let Some(p) = opt_placement {
                let resolved_col = if p.column < 0 {
                    (col_count as i32 + p.column).max(0) as usize
                } else {
                    p.column as usize
                };
                let resolved_row = if p.row < 0 {
                    (row_count as i32 + p.row).max(0) as usize
                } else {
                    p.row as usize
                };
                cvkg_core::GridPlacement {
                    column: resolved_col as i32,
                    column_span: p.column_span.max(1),
                    row: resolved_row as i32,
                    row_span: p.row_span.max(1),
                }
            } else {
                // Find next unoccupied cell
                let mut found_col = 0;
                let mut found_row = 0;
                let mut found = false;
                for (r, row) in occupied.iter().enumerate() {
                    for (c, cell) in row.iter().enumerate() {
                        if !*cell {
                            found_col = c;
                            found_row = r;
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }
                cvkg_core::GridPlacement {
                    column: found_col as i32,
                    column_span: 1,
                    row: found_row as i32,
                    row_span: 1,
                }
            };

            // Mark occupied cells
            let start_r = (placement.row as usize).min(row_count - 1);
            let end_r = (start_r + placement.row_span as usize).min(row_count);
            let start_c = (placement.column as usize).min(col_count - 1);
            let end_c = (start_c + placement.column_span as usize).min(col_count);
            for row in occupied.iter_mut().take(end_r).skip(start_r) {
                for cell in row.iter_mut().take(end_c).skip(start_c) {
                    *cell = true;
                }
            }

            resolved_placements.push(placement);
        }

        // 2. Measure content sizes of children (for Auto and MinMax sizing).
        // For simplicity, we calculate the sizes of Auto and MinMax tracks based on children that span a single track.
        let mut col_content_widths = vec![0.0f32; col_count];
        let mut row_content_heights = vec![0.0f32; row_count];

        for (i, &placement) in resolved_placements.iter().enumerate() {
            let child = subviews[i];
            let size = child.size_that_fits(
                SizeProposal::new(Some(bounds.width), Some(bounds.height)),
                &[],
                cache,
            );

            if placement.column_span == 1 {
                let c = placement.column as usize;
                if c < col_count {
                    col_content_widths[c] = col_content_widths[c].max(size.width);
                }
            }
            if placement.row_span == 1 {
                let r = placement.row as usize;
                if r < row_count {
                    row_content_heights[r] = row_content_heights[r].max(size.height);
                }
            }
        }

        // 3. Resolve Column widths
        let total_col_gaps = self.column_gap * (col_count - 1) as f32;
        let mut remaining_width = (bounds.width - total_col_gaps).max(0.0);
        let mut total_flex_weight_col = 0.0f32;
        let mut col_widths = vec![0.0f32; col_count];

        for (c, track) in self.columns.iter().enumerate() {
            match track {
                GridTrack::Fixed(w) => {
                    col_widths[c] = *w;
                    remaining_width = (remaining_width - *w).max(0.0);
                }
                GridTrack::Auto => {
                    let w = col_content_widths[c];
                    col_widths[c] = w;
                    remaining_width = (remaining_width - w).max(0.0);
                }
                GridTrack::MinMax(min, max) => {
                    let w = col_content_widths[c].clamp(*min, *max);
                    col_widths[c] = w;
                    remaining_width = (remaining_width - w).max(0.0);
                }
                GridTrack::Flex(weight) => {
                    total_flex_weight_col += *weight;
                }
            }
        }

        if total_flex_weight_col > 0.0 {
            for (c, track) in self.columns.iter().enumerate() {
                if let GridTrack::Flex(weight) = track {
                    col_widths[c] = (*weight / total_flex_weight_col) * remaining_width;
                }
            }
        }

        // 4. Resolve Row heights
        let total_row_gaps = self.row_gap * (row_count - 1) as f32;
        let mut remaining_height = (bounds.height - total_row_gaps).max(0.0);
        let mut total_flex_weight_row = 0.0f32;
        let mut row_heights = vec![0.0f32; row_count];

        for (r, track) in self.rows.iter().enumerate() {
            match track {
                GridTrack::Fixed(h) => {
                    row_heights[r] = *h;
                    remaining_height = (remaining_height - *h).max(0.0);
                }
                GridTrack::Auto => {
                    let h = row_content_heights[r];
                    row_heights[r] = h;
                    remaining_height = (remaining_height - h).max(0.0);
                }
                GridTrack::MinMax(min, max) => {
                    let h = row_content_heights[r].clamp(*min, *max);
                    row_heights[r] = h;
                    remaining_height = (remaining_height - h).max(0.0);
                }
                GridTrack::Flex(weight) => {
                    total_flex_weight_row += *weight;
                }
            }
        }

        if total_flex_weight_row > 0.0 {
            for (r, track) in self.rows.iter().enumerate() {
                if let GridTrack::Flex(weight) = track {
                    row_heights[r] = (*weight / total_flex_weight_row) * remaining_height;
                }
            }
        }

        // 5. Compute grid line coordinates
        let mut col_positions = vec![0.0f32; col_count + 1];
        let mut x_acc = bounds.x;
        for c in 0..col_count {
            col_positions[c] = x_acc;
            x_acc += col_widths[c] + self.column_gap;
        }
        col_positions[col_count] = x_acc - self.column_gap;

        let mut row_positions = vec![0.0f32; row_count + 1];
        let mut y_acc = bounds.y;
        for r in 0..row_count {
            row_positions[r] = y_acc;
            y_acc += row_heights[r] + self.row_gap;
        }
        row_positions[row_count] = y_acc - self.row_gap;

        // 6. Compute child rects based on placement and spans
        for i in 0..subviews.len() {
            let placement = resolved_placements[i];
            let c_start = (placement.column as usize).min(col_count - 1);
            let c_end = (c_start + placement.column_span as usize).min(col_count);
            let r_start = (placement.row as usize).min(row_count - 1);
            let r_end = (r_start + placement.row_span as usize).min(row_count);

            let child_x = col_positions[c_start];
            let child_y = row_positions[r_start];

            // Width spans tracks and gaps:
            let child_w = if c_end > c_start {
                col_widths[c_start..c_end].iter().sum::<f32>()
                    + self.column_gap * (c_end - c_start - 1) as f32
            } else {
                0.0
            };

            let child_h = if r_end > r_start {
                row_heights[r_start..r_end].iter().sum::<f32>()
                    + self.row_gap * (r_end - r_start - 1) as f32
            } else {
                0.0
            };

            rects[i] = Rect {
                x: child_x,
                y: child_y,
                width: child_w,
                height: child_h,
            };
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
        let rects = self.compute_layout_rects(bounds, &views, &placements, cache);

        for (child, rect) in subviews.iter_mut().zip(rects) {
            child.place_subviews(rect, &mut [], cache);
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
}
