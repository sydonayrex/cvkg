use thiserror::Error;

// ── Error Type ──────────────────────────────────────────────────────────────

/// Errors that can occur during filter evaluation.
#[derive(Error, Debug)]
pub enum FilterError {
    /// The filter graph has a cycle.
    #[error("filter graph contains a cycle")]
    CyclicGraph,
    /// A referenced input could not be resolved.
    #[error("unresolved filter input: {0}")]
    UnresolvedInput(String),
    /// WGPU operation failed.
    #[error("WGPU error: {0}")]
    Wgpu(String),
    /// Filter region is invalid (zero or negative size).
    #[error("invalid filter region: {0}x{1}")]
    InvalidRegion(f32, f32),
    /// Texture allocation failed.
    #[error("texture allocation failed: {0}")]
    TextureError(String),
}

impl From<wgpu::Error> for FilterError {
    fn from(e: wgpu::Error) -> Self {
        FilterError::Wgpu(e.to_string())
    }
}

// ── Core Types ───────────────────────────────────────────────────────────────

/// WGPU device/queue pair, stored as Arcs for cheap cloning.
#[derive(Clone)]
pub struct GpuContext {
    pub device: std::sync::Arc<wgpu::Device>,
    pub queue: std::sync::Arc<wgpu::Queue>,
}

/// Context for a single filter evaluation pass.
pub struct FilterContext<'a> {
    /// The source texture to filter.
    pub source_view: &'a wgpu::TextureView,
    /// The filter region in pixel coordinates.
    pub region: (u32, u32, u32, u32), // x, y, width, height
    /// The element's bounding box in user space (for objectBoundingBox resolution).
    pub element_bbox: usvg::NonZeroRect,
    /// Color interpolation mode.
    pub color_interpolation: usvg::filter::ColorInterpolation,
    /// Backdrop texture for glassmorphism
    pub backdrop_view: Option<&'a wgpu::TextureView>,
    /// Time parameter for animated filters
    pub time: f32,
    /// The full screen size (width, height)
    pub screen_size: (u32, u32),
}

/// Result of evaluating a single filter primitive.
pub struct FilterResult {
    /// The output texture view.
    pub output_view: std::sync::Arc<wgpu::TextureView>,
    /// The actual pixel region covered by this result.
    pub region: (u32, u32, u32, u32),
}

/// A resolved input reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedInput {
    SourceGraphic,
    SourceAlpha,
    BackdropImage,
    BackdropAlpha,
    NodeIndex(usize),
}

// ── Filter Region Resolution ─────────────────────────────────────────────────

/// Compute the pixel extent of a filter primitive's region.
///
/// Handles `objectBoundingBox` (percentages relative to the element's bbox)
/// and `userSpaceOnUse` (absolute units).
pub fn resolve_filter_region(
    primitive_rect: usvg::NonZeroRect,
    element_bbox: usvg::NonZeroRect,
    filter_units: FilterUnits,
    padding: f32,
) -> (u32, u32, u32, u32) {
    let (x, y, w, h) = match filter_units {
        FilterUnits::ObjectBoundingBox => {
            let x = element_bbox.x() + primitive_rect.x() / 100.0 * element_bbox.width();
            let y = element_bbox.y() + primitive_rect.y() / 100.0 * element_bbox.height();
            let w = primitive_rect.width() / 100.0 * element_bbox.width();
            let h = primitive_rect.height() / 100.0 * element_bbox.height();
            (x, y, w, h)
        }
        FilterUnits::UserSpaceOnUse => (
            primitive_rect.x(),
            primitive_rect.y(),
            primitive_rect.width(),
            primitive_rect.height(),
        ),
    };

    // Apply padding for filters that extend beyond their nominal region.
    let x = x - padding;
    let y = y - padding;
    let w = w + padding * 2.0;
    let h = h + padding * 2.0;

    (
        x.max(0.0) as u32,
        y.max(0.0) as u32,
        w.max(1.0) as u32,
        h.max(1.0) as u32,
    )
}

/// Filter coordinate system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterUnits {
    ObjectBoundingBox,
    UserSpaceOnUse,
}

/// Compute padding for filters that extend beyond their nominal region.
pub fn filter_padding(kind: &usvg::filter::Kind) -> f32 {
    match kind {
        usvg::filter::Kind::GaussianBlur(gb) => {
            let sx = gb.std_dev_x().get();
            let sy = gb.std_dev_y().get();
            // 3-sigma rule covers 99.7% of the Gaussian.
            (sx.max(sy) * 3.0).ceil()
        }
        usvg::filter::Kind::DropShadow(ds) => {
            let blur = (ds.std_dev_x().get() + ds.std_dev_y().get()) * 1.5;
            let offset = (ds.dx().abs() + ds.dy().abs()).ceil();
            blur + offset
        }
        usvg::filter::Kind::Morphology(m) => {
            let rx = m.radius_x().get();
            let ry = m.radius_y().get();
            rx.max(ry).ceil()
        }
        usvg::filter::Kind::ConvolveMatrix(cm) => {
            let data = cm.matrix();
            let tx = data.target_x() as f32;
            let ty = data.target_y() as f32;
            let cols = data.columns() as f32;
            let rows = data.rows() as f32;
            // Padding = half kernel size minus target offset.
            let px = (cols / 2.0 - tx).max(0.0);
            let py = (rows / 2.0 - ty).max(0.0);
            let_pad_x_y(px, py)
        }
        _ => 0.0,
    }
}

#[inline(always)]
fn let_pad_x_y(px: f32, py: f32) -> f32 {
    px.max(py).ceil()
}

// ── Alpha Processing Standardization (P1-33) ─────────────────────────────────

/// Alpha mode for filter operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    /// Straight (non-premultiplied) alpha. RGB values are independent
    /// of the alpha channel.
    Straight,
    /// Premultiplied alpha. RGB values are multiplied by alpha.
    Premultiplied,
}

impl AlphaMode {
    /// Convert a single pixel from straight to premultiplied alpha.
    pub fn to_premultiplied(r: f32, g: f32, b: f32, a: f32) -> [f32; 4] {
        [r * a, g * a, b * a, a]
    }

    /// Convert a single pixel from premultiplied to straight alpha.
    pub fn to_straight(r: f32, g: f32, b: f32, a: f32) -> [f32; 4] {
        if a > 0.001 {
            [r / a, g / a, b / a, a]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        }
    }

    /// Returns true if this is Premultiplied.
    pub fn is_premultiplied(&self) -> bool {
        matches!(self, AlphaMode::Premultiplied)
    }
}

// =============================================================================
// P1-36: Large Document Scaling (Filters)
// =============================================================================

/// Filter LOD (Level of Detail) for large document scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterLod {
    /// Full quality filtering.
    Full,
    /// Reduced quality (fewer samples, smaller regions).
    Reduced,
    /// Skip filtering entirely for distant elements.
    Skip,
}

impl FilterLod {
    /// Determine LOD based on element distance from viewport.
    pub fn from_distance(distance_pixels: f32) -> Self {
        if distance_pixels < 500.0 {
            FilterLod::Full
        } else if distance_pixels < 2000.0 {
            FilterLod::Reduced
        } else {
            FilterLod::Skip
        }
    }

    /// Whether filtering should be applied at this LOD.
    pub fn should_filter(&self) -> bool {
        !matches!(self, FilterLod::Skip)
    }
}

// =============================================================================
// P2-32: Dynamic Material Effects - SourceBackdrop
// =============================================================================

/// A filter input that samples the current rendered content behind the filter region.
#[derive(Clone, Debug)]
pub struct SourceBackdrop {
    /// The backdrop texture view.
    pub view: Option<std::sync::Arc<wgpu::TextureView>>,
    /// Offset from the filter region origin.
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for SourceBackdrop {
    fn default() -> Self {
        Self {
            view: None,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

impl SourceBackdrop {
    /// Create a new SourceBackdrop from a texture view.
    pub fn new(view: wgpu::TextureView) -> Self {
        Self {
            view: Some(std::sync::Arc::new(view)),
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Set the offset for sampling.
    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Check if a backdrop is available.
    pub fn is_available(&self) -> bool {
        self.view.is_some()
    }
}

#[cfg(test)]
mod p1_33_alpha_mode_tests {
    use super::*;

    #[test]
    fn to_premultiplied_works() {
        let [r, g, b, a] = AlphaMode::to_premultiplied(0.5, 0.6, 0.7, 0.5);
        assert!((r - 0.25).abs() < 0.001);
        assert!((g - 0.30).abs() < 0.001);
        assert!((b - 0.35).abs() < 0.001);
        assert!((a - 0.5).abs() < 0.001);
    }

    #[test]
    fn to_straight_works() {
        let [r, g, b, a] = AlphaMode::to_straight(0.25, 0.30, 0.35, 0.5);
        assert!((r - 0.5).abs() < 0.001);
        assert!((g - 0.6).abs() < 0.001);
        assert!((b - 0.7).abs() < 0.001);
        assert!((a - 0.5).abs() < 0.001);
    }

    #[test]
    fn to_straight_handles_zero_alpha() {
        let [r, g, b, a] = AlphaMode::to_straight(0.1, 0.2, 0.3, 0.0);
        assert_eq!([r, g, b, a], [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn roundtrip_preserves_values() {
        let original = [0.5, 0.6, 0.7, 0.8];
        let prem = AlphaMode::to_premultiplied(original[0], original[1], original[2], original[3]);
        let back = AlphaMode::to_straight(prem[0], prem[1], prem[2], prem[3]);
        for i in 0..4 {
            assert!((original[i] - back[i]).abs() < 0.001, "component {} differs", i);
        }
    }

    #[test]
    fn is_premultiplied_works() {
        assert!(AlphaMode::Premultiplied.is_premultiplied());
        assert!(!AlphaMode::Straight.is_premultiplied());
    }
}
