use crate::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Modifier to set the size and alignment constraints of a view.
/// This determines the proposal size passed to the child and how the child is aligned
/// within the layout rect allocated to the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameModifier {
    /// Exact width to assign to the child view.
    pub width: Option<f32>,
    /// Exact height to assign to the child view.
    pub height: Option<f32>,
    /// Minimum width constraint for the view.
    pub min_width: Option<f32>,
    /// Maximum width constraint for the view.
    pub max_width: Option<f32>,
    /// Minimum height constraint for the view.
    pub min_height: Option<f32>,
    /// Maximum height constraint for the view.
    pub max_height: Option<f32>,
    /// The alignment strategy for positioning the child view within the frame.
    pub alignment: Alignment,
}

impl Default for FrameModifier {
    /// Returns the default frame configuration which has no constraints and center alignment.
    fn default() -> Self {
        Self::new()
    }
}

impl FrameModifier {
    /// Creates a new FrameModifier with all dimensions unspecified and center alignment.
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            alignment: Alignment::Center,
        }
    }

    /// Sets the fixed width of the frame.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the fixed height of the frame.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets both the fixed width and height of the frame.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets the minimum width constraint.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Sets the maximum width constraint.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Sets the minimum height constraint.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Sets the maximum height constraint.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Sets the alignment strategy for the child within the frame's layout bounds.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl ViewModifier for FrameModifier {
    /// Wraps the child view in a ModifiedView using this frame modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Transforms the layout size proposal offered to the child to comply with frame constraints.
    fn transform_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        let w = if let Some(width) = self.width {
            Some(width)
        } else {
            proposal.width.map(|pw| {
                pw.clamp(
                    self.min_width.unwrap_or(0.0),
                    self.max_width.unwrap_or(f32::INFINITY),
                )
            })
        };
        let h = if let Some(height) = self.height {
            Some(height)
        } else {
            proposal.height.map(|ph| {
                ph.clamp(
                    self.min_height.unwrap_or(0.0),
                    self.max_height.unwrap_or(f32::INFINITY),
                )
            })
        };
        SizeProposal {
            width: w,
            height: h,
        }
    }

    /// Constraints and transforms the child's resulting size to fit the frame's bounds.
    fn transform_size(&self, child_size: Size) -> Size {
        let w = if let Some(width) = self.width {
            width
        } else {
            child_size.width.clamp(
                self.min_width.unwrap_or(0.0),
                self.max_width.unwrap_or(f32::INFINITY),
            )
        };
        let h = if let Some(height) = self.height {
            height
        } else {
            child_size.height.clamp(
                self.min_height.unwrap_or(0.0),
                self.max_height.unwrap_or(f32::INFINITY),
            )
        };
        Size {
            width: w,
            height: h,
        }
    }

    /// Renders the frame's child view aligned within the layout rect.
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        self.render(renderer, rect);

        // If no explicit or min/max constraints, pass through without re-centering.
        // Re-centering a flex-sized child undoes the parent layout engine's placement,
        // causing flex children to appear as tiny centered blocks in empty space.
        let has_constraints = self.width.is_some()
            || self.height.is_some()
            || self.min_width.is_some()
            || self.max_width.is_some()
            || self.min_height.is_some()
            || self.max_height.is_some();

        if !has_constraints {
            // Even without fixed-size constraints we still honour the default
            // `Alignment::Center` contract.  Passing the full rect through
            // positions every unconstrained child at (rect.x, rect.y) — the top-left —
            // which is wrong for gallery panels and any centered detail view.
            if self.alignment == Alignment::Center {
                let child_size = view.intrinsic_size(
                    renderer,
                    SizeProposal::new(Some(rect.width), Some(rect.height)),
                );
                let child_x = rect.x + (rect.width - child_size.width) / 2.0;
                let child_y = rect.y + (rect.height - child_size.height) / 2.0;
                let child_rect = Rect {
                    x: child_x,
                    y: child_y,
                    width: child_size.width,
                    height: child_size.height,
                };
                eprintln!(
                    "DEBUG CENTER: rect={:?}, child_size=({}, {}), child_rect={:?}",
                    rect,
                    child_size.width,
                    child_size.height,
                    child_rect
                );
                view.render(renderer, child_rect);
                self.post_render(renderer, rect);
                return;
            }
            // Non-center alignment with no constraints: pass through unchanged.
            view.render(renderer, rect);
            self.post_render(renderer, rect);
            return;
        }

        let child_proposal =
            self.transform_proposal(SizeProposal::new(Some(rect.width), Some(rect.height)));
        let child_size = view.intrinsic_size(renderer, child_proposal);

        let mut child_x = rect.x;
        let mut child_y = rect.y;

        match self.alignment {
            Alignment::Leading => {
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
            Alignment::Trailing => {
                child_x = rect.x + rect.width - child_size.width;
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
            Alignment::Top => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
            }
            Alignment::Bottom => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
                child_y = rect.y + rect.height - child_size.height;
            }
            Alignment::Center => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
        }

        let child_rect = Rect {
            x: child_x,
            y: child_y,
            width: child_size.width,
            height: child_size.height,
        };

        view.render(renderer, child_rect);
        self.post_render(renderer, rect);
    }
}

/// Modifier to set the flex weight of a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexModifier {
    pub weight: f32,
}

impl ViewModifier for FlexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn child_flex_weight<V: View>(&self, _view: &V) -> f32 {
        self.weight
    }
}

/// Modifier that specifies the column and row placement of a view inside a Grid layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPlacementModifier {
    /// The grid placement settings containing column/row indexes and spans.
    pub placement: GridPlacement,
}

impl ViewModifier for GridPlacementModifier {
    /// Wraps the child view in a ModifiedView using this modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Exposes the grid placement metadata to parent layout engines.
    fn get_grid_placement(&self) -> Option<GridPlacement> {
        Some(self.placement)
    }
}

/// Modifier to render a popover, tooltip, or menu view overlaying an anchored view.
/// It supports alignment positioning and outside-click dismissal.
#[derive(Clone)]
pub struct OverlayModifier {
    /// The overlay content view.
    pub overlay: AnyView,
    /// Where the overlay is aligned relative to the anchored view.
    pub alignment: Alignment,
    /// Additional offset in logical pixels.
    pub offset: [f32; 2],
    /// Optional dismissal callback triggered by click-outside events.
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ViewModifier for OverlayModifier {
    /// Wraps the child view in a ModifiedView using this overlay modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Renders the overlay content positioned above the child view.
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Render primary anchored view
        view.render(renderer, rect);

        // 2. Measure overlay content
        let overlay_size = self
            .overlay
            .intrinsic_size(renderer, SizeProposal::unspecified());

        // 3. Align overlay rect relative to anchored rect
        let mut overlay_x;
        let mut overlay_y;

        match self.alignment {
            Alignment::Leading => {
                overlay_x = rect.x - overlay_size.width;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
            Alignment::Trailing => {
                overlay_x = rect.x + rect.width;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
            Alignment::Top => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y - overlay_size.height;
            }
            Alignment::Bottom => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y + rect.height;
            }
            Alignment::Center => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
        }

        overlay_x += self.offset[0];
        overlay_y += self.offset[1];

        let overlay_rect = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_size.width,
            height: overlay_size.height,
        };

        // 4. Handle click-outside dismissal
        if let Some(on_dismiss) = &self.on_dismiss {
            let dismiss = on_dismiss.clone();
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event {
                        let click_inside = x >= overlay_rect.x
                            && x <= overlay_rect.x + overlay_rect.width
                            && y >= overlay_rect.y
                            && y <= overlay_rect.y + overlay_rect.height;
                        if !click_inside {
                            dismiss();
                        }
                    }
                }),
            );
        }

        // 5. Render overlay view
        self.overlay.render(renderer, overlay_rect);
    }
}

/// Modifier to offset a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetModifier {
    pub x: f32,
    pub y: f32,
}

impl OffsetModifier {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl ViewModifier for OffsetModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to set the z-index of a view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZIndexModifier {
    pub z_index: i32,
}

impl ZIndexModifier {
    pub fn new(z_index: i32) -> Self {
        Self { z_index }
    }
}

impl ViewModifier for ZIndexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Layout constraints for views
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

/// Modifier to set layout constraints
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutModifier {
    pub constraints: LayoutConstraints,
}

impl LayoutModifier {
    pub fn new(constraints: LayoutConstraints) -> Self {
        Self { constraints }
    }
}

impl ViewModifier for LayoutModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to handle platform safe areas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeAreaModifier {
    pub ignores: bool,
}

impl ViewModifier for SafeAreaModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to add elevation (shadow) to a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationModifier {
    pub level: f32,
}

impl ViewModifier for ElevationModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if self.level > 0.0 {
            let radius = self.level * 2.0;
            let offset_y = self.level * 0.5;
            let shadow_color = [0.0, 0.0, 0.0, 0.3];
            renderer.push_shadow(radius, shadow_color, [0.0, offset_y]);
            view.render(renderer, rect);
            renderer.pop_shadow();
        } else {
            view.render(renderer, rect);
        }
    }
}

/// Position modifier — offsets a view from its layout position.
/// Enables absolute-like positioning within a container.
#[derive(Clone)]
pub struct PositionModifier {
    pub x: f32,
    pub y: f32,
}

impl ViewModifier for PositionModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn transform_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.x,
            y: rect.y + self.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

// Layout subsystem
