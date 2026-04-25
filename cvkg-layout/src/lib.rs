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

use cvkg_core::{LayoutCache, LayoutView, Rect, Size, SizeProposal};

/// HStack - lays out children horizontally
pub struct HStack {
    spacing: f32,
    children: Vec<Box<dyn LayoutView>>,
}

impl HStack {
    /// Create a new HStack with the given spacing
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    /// Add a view to the HStack
    pub fn add_view<V: LayoutView + 'static>(&mut self, view: V) {
        self.children.push(Box::new(view));
    }

    /// Compute the layout rects for children without placing them.
    pub fn compute_layout(
        spacing: f32,
        bounds: Rect,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(subviews.len());
        let mut x = bounds.x;
        for child in subviews {
            let desired = child.size_that_fits(SizeProposal::unspecified(), &[], cache);
            rects.push(Rect {
                x,
                y: bounds.y,
                width: desired.width,
                height: bounds.height,
            });
            x += desired.width + spacing;
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
        // For HStack, we want to know how much space we need
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in subviews.iter().enumerate() {
            let child_size = child.size_that_fits(proposal, &[], cache);
            width += child_size.width;
            height = height.max(child_size.height);

            // Add spacing between children (not after the last one)
            if i < subviews.len() - 1 {
                width += self.spacing;
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let mut x = bounds.x;
        let y = bounds.y;

        for (_i, child) in subviews.iter_mut().enumerate() {
            // Get child's desired size
            let desired_size = child.size_that_fits(SizeProposal::unspecified(), &[], cache);

            // In HStack, we give the child as much height as available, but only the width it needs
            let child_rect = Rect {
                x,
                y,
                width: desired_size.width,
                height: bounds.height,
            };

            child.place_subviews(child_rect, &mut [], cache);

            // Move x position for next child
            x += desired_size.width + self.spacing;
        }
    }
}

/// VStack - lays out children vertically
pub struct VStack {
    spacing: f32,
    children: Vec<Box<dyn LayoutView>>,
}

impl VStack {
    /// Create a new VStack with the given spacing
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    /// Add a view to the VStack
    pub fn add_view<V: LayoutView + 'static>(&mut self, view: V) {
        self.children.push(Box::new(view));
    }

    /// Compute the layout rects for children without placing them.
    pub fn compute_layout(
        spacing: f32,
        bounds: Rect,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(subviews.len());
        let mut y = bounds.y;
        for child in subviews {
            let desired = child.size_that_fits(SizeProposal::unspecified(), &[], cache);
            rects.push(Rect {
                x: bounds.x,
                y,
                width: bounds.width,
                height: desired.height,
            });
            y += desired.height + spacing;
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
        // For VStack, we want to know how much space we need
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in subviews.iter().enumerate() {
            let child_size = child.size_that_fits(proposal, &[], cache);
            width = width.max(child_size.width);
            height += child_size.height;

            // Add spacing between children (not after the last one)
            if i < subviews.len() - 1 {
                height += self.spacing;
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let x = bounds.x;
        let mut y = bounds.y;

        for (_i, child) in subviews.iter_mut().enumerate() {
            // Get child's desired size
            let desired_size = child.size_that_fits(SizeProposal::unspecified(), &[], cache);

            // In VStack, we give the child as much width as available, but only the height it needs
            let child_rect = Rect {
                x,
                y,
                width: bounds.width,
                height: desired_size.height,
            };

            child.place_subviews(child_rect, &mut [], cache);

            // Move y position for next child
            y += desired_size.height + self.spacing;
        }
    }
}

/// ZStack - lays out children on top of each other
pub struct ZStack {
    children: Vec<Box<dyn LayoutView>>,
}

impl ZStack {
    /// Create a new ZStack
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a view to the ZStack
    pub fn add_view<V: LayoutView + 'static>(&mut self, view: V) {
        self.children.push(Box::new(view));
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
