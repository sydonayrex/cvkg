use cvkg_core::layout::Rect;

/// The current input modality that the layout engine adapts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutModality {
    /// Precise pointer (mouse, trackpad, stylus).  Use intrinsic sizes.
    #[default]
    Pointer,
    /// Touch input.  Enforce a minimum tap-target size of 44×44 logical pts.
    Touch,
    /// Accessibility zoom is active.  Touch rules apply and spacing is doubled.
    AccessibilityZoom,
}

impl LayoutModality {
    /// Minimum tap-target dimension for this modality (logical pixels).
    pub fn min_tap_target(self) -> f32 {
        match self {
            LayoutModality::Pointer => 0.0,
            LayoutModality::Touch => 44.0,
            LayoutModality::AccessibilityZoom => 44.0,
        }
    }

    /// Spacing multiplier applied on top of the view's configured spacing.
    pub fn spacing_multiplier(self) -> f32 {
        match self {
            LayoutModality::Pointer => 1.0,
            LayoutModality::Touch => 1.25,
            LayoutModality::AccessibilityZoom => 2.0,
        }
    }

    /// Apply this modality's minimum tap-target constraint to a measured size.
    pub fn adapt_size(self, size: cvkg_core::Size) -> cvkg_core::Size {
        let min = self.min_tap_target();
        cvkg_core::Size {
            width: size.width.max(min),
            height: size.height.max(min),
        }
    }
}

/// A focusable element produced by `compute_focus_order`.
#[derive(Debug, Clone, PartialEq)]
pub struct FocusCandidate {
    /// Stable identity — matches `LayoutView::view_hash()`.
    pub hash: u64,
    /// Post-layout bounding rect, in the root coordinate space.
    pub rect: Rect,
    /// Explicit tab index, if the view has one.  `None` means natural order.
    pub tab_index: Option<i32>,
}

/// Compute a deterministic keyboard-focus traversal order for a flat list of candidates.
pub fn compute_focus_order(mut candidates: Vec<FocusCandidate>) -> Vec<u64> {
    let mut explicit: Vec<FocusCandidate> = candidates
        .iter()
        .filter(|c| c.tab_index.map_or(false, |t| t > 0))
        .cloned()
        .collect();
    candidates.retain(|c| !c.tab_index.map_or(false, |t| t > 0));

    explicit.sort_by(|a, b| {
        let ta = a.tab_index.unwrap_or(i32::MAX);
        let tb = b.tab_index.unwrap_or(i32::MAX);
        ta.cmp(&tb)
            .then_with(|| a.rect.y.total_cmp(&b.rect.y))
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });

    let row_bucket = |r: &Rect| (r.y / 8.0).floor() as i32;
    candidates.sort_by(|a, b| {
        row_bucket(&a.rect)
            .cmp(&row_bucket(&b.rect))
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });

    explicit
        .into_iter()
        .chain(candidates)
        .map(|c| c.hash)
        .collect()
}

/// Validate that the focus order computed by `compute_focus_order` is consistent with visual reading order.
pub fn validate_reading_order(order: &[FocusCandidate]) -> Result<(), String> {
    let natural: Vec<&FocusCandidate> = order
        .iter()
        .filter(|c| !c.tab_index.map_or(false, |t| t > 0))
        .collect();

    let row_bucket = |r: &Rect| (r.y / 8.0).floor() as i32;
    for window in natural.windows(2) {
        let a = window[0];
        let b = window[1];
        if row_bucket(&b.rect) < row_bucket(&a.rect) {
            return Err(format!(
                "reading order violation: view 0x{:X} (y≈{:.1}) precedes view 0x{:X} (y≈{:.1}) visually",
                b.hash, b.rect.y, a.hash, a.rect.y
            ));
        }
        if row_bucket(&a.rect) == row_bucket(&b.rect) && b.rect.x < a.rect.x - 1.0 {
            return Err(format!(
                "reading order violation: view 0x{:X} (x≈{:.1}) precedes view 0x{:X} (x≈{:.1}) on same row",
                b.hash, b.rect.x, a.hash, a.rect.x
            ));
        }
    }
    Ok(())
}
