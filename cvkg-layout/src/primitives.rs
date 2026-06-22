pub use cvkg_core::layout::EdgeInsets;
use cvkg_core::{LayoutCache, LayoutView, Rect, Size, SizeProposal};

/// A layout view that adds padding around its child.
pub struct Padding {
    pub insets: EdgeInsets,
}

impl Padding {
    /// Creates a new Padding layout view with margins on each side.
    pub fn new(insets: EdgeInsets) -> Self {
        Self { insets }
    }

    /// Creates a Padding layout with uniform margin.
    pub fn uniform(value: f32) -> Self {
        Self {
            insets: EdgeInsets::all(value),
        }
    }

    /// Creates a Padding layout with symmetric horizontal and vertical margins.
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
            crate::with_layout_cycle_guard(child_hash, Size::ZERO, || {
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
                crate::with_layout_cycle_guard_void(child_hash, || {
                    child.place_subviews(inner, &mut [], cache);
                });
            }
        }
    }
}

/// A layout view that respects safe area insets (notches, status bars).
pub struct SafeArea {
    pub edges: SafeAreaEdges,
}

/// Active safe-area edge constraints.
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
    /// Enables safe area on all four sides.
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

    /// Enables safe area vertical edges only (top and bottom).
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

/// Constrains a child to a specific aspect ratio.
pub struct AspectRatio {
    pub ratio: f32,
}

impl AspectRatio {
    /// Creates a new AspectRatio.
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.max(0.01),
        }
    }

    /// Square aspect ratio (1.0).
    pub fn square() -> Self {
        Self::new(1.0)
    }

    /// Widescreen aspect ratio (16:9).
    pub fn widescreen() -> Self {
        Self::new(16.0 / 9.0)
    }

    /// Portrait aspect ratio (9:16).
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
        let child_size = crate::with_layout_cycle_guard(child_hash, Size::ZERO, || {
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
                crate::with_layout_cycle_guard_void(child_hash, || {
                    child.place_subviews(inner, &mut [], cache);
                });
            }
        }
    }
}
