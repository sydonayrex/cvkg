use crate::{Color, FontWeight, Orientation};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// Text view for displaying strings
#[allow(dead_code)]
#[derive(Clone)]
pub struct Text {
    pub(crate) content: String,
    pub(crate) font_size: f32,
    pub(crate) font_weight: FontWeight,
    pub(crate) color: Color,
}

impl Text {
    /// Create a new Text component with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 14.0,
            font_weight: FontWeight::Regular,
            color: Color::WHITE,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn bold(self) -> Self {
        self.font_weight(FontWeight::Bold)
    }
}

impl View for Text {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.draw_text(
            &self.content,
            rect.x,
            rect.y,
            self.font_size,
            self.color.as_array(),
        );
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (w, h) = renderer.measure_text(&self.content, self.font_size);
        Size { width: w, height: h }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for Text {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: self.content.len() as f32 * self.font_size * 0.6,
            height: self.font_size * 1.2,
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

/// Divider for separating content
#[allow(dead_code)]
#[derive(Clone)]
pub struct Divider {
    pub(crate) orientation: Orientation,
    pub(crate) width: f32,
    pub(crate) color: Color,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            width: 1.0,
            color: Color::BLACK,
        }
    }
}

impl View for Divider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (line_rect, color) = match self.orientation {
            Orientation::Horizontal => (
                Rect {
                    x: rect.x,
                    y: rect.y + (rect.height - self.width) / 2.0,
                    width: rect.width,
                    height: self.width,
                },
                self.color.as_array(),
            ),
            Orientation::Vertical => (
                Rect {
                    x: rect.x + (rect.width - self.width) / 2.0,
                    y: rect.y,
                    width: self.width,
                    height: rect.height,
                },
                self.color.as_array(),
            ),
        };
        renderer.fill_rect(line_rect, color);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        match self.orientation {
            Orientation::Horizontal => Size {
                width: proposal.width.unwrap_or(0.0),
                height: self.width,
            },
            Orientation::Vertical => Size {
                width: self.width,
                height: proposal.height.unwrap_or(0.0),
            },
        }
    }
}

/// Spacer for flexible layout gaps
#[allow(dead_code)]
#[derive(Clone)]
pub struct Spacer {
    pub(crate) min_length: f32,
}

impl Spacer {
    pub fn new(min_length: f32) -> Self {
        Self { min_length }
    }
}

impl View for Spacer {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(self.min_length),
            height: proposal.height.unwrap_or(self.min_length),
        }
    }

    fn flex_weight(&self) -> f32 {
        1.0
    }
}

impl LayoutView for Spacer {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(self.min_length),
            height: proposal.height.unwrap_or(self.min_length),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }

    fn flex_weight(&self) -> f32 {
        1.0
    }
}

/// Canvas for custom drawing
#[allow(dead_code)]
#[derive(Clone)]
pub struct Canvas<F>
where
    F: Fn(&mut dyn Renderer, Rect) + Send + Sync + Clone + 'static,
{
    pub(crate) draw_func: F,
}

impl<F> Canvas<F>
where
    F: Fn(&mut dyn Renderer, Rect) + Send + Sync + Clone + 'static,
{
    pub fn new(draw_func: F) -> Self {
        Self { draw_func }
    }
}

impl<F> View for Canvas<F>
where
    F: Fn(&mut dyn Renderer, Rect) + Send + Sync + Clone + 'static,
{
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        (self.draw_func)(renderer, rect);
    }
}

/// Shape types for drawing primitives
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Circle,
    RoundedRectangle { corner_radius: f32 },
}

/// A standard vector Shape
#[allow(dead_code)]
#[derive(Clone)]
pub struct Shape {
    pub(crate) shape_type: ShapeType,
    pub(crate) fill: Color,
}

impl Shape {
    pub fn rounded_rect(corner_radius: f32) -> Self {
        Self {
            shape_type: ShapeType::RoundedRectangle { corner_radius },
            fill: Color::BLACK,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }
}

impl View for Shape {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        match self.shape_type {
            ShapeType::Rectangle => renderer.fill_rect(rect, self.fill.as_array()),
            ShapeType::RoundedRectangle { corner_radius } => {
                renderer.fill_rounded_rect(rect, corner_radius, self.fill.as_array());
            }
            ShapeType::Circle => {
                renderer.fill_ellipse(rect, self.fill.as_array());
            }
        }
    }
}

/// Badge component for displaying small status or category tags.
#[derive(Clone)]
pub struct Badge {
    pub(crate) text: String,
    pub(crate) variant: BadgeVariant,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

impl View for Badge {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (bg, text_color) = match self.variant {
            BadgeVariant::Default => ([0.0, 0.8, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]),
            BadgeVariant::Secondary => ([0.2, 0.2, 0.25, 1.0], [1.0, 1.0, 1.0, 1.0]),
            BadgeVariant::Destructive => ([0.8, 0.1, 0.1, 1.0], [1.0, 1.0, 1.0, 1.0]),
            BadgeVariant::Outline => ([0.0, 0.0, 0.0, 0.0], [0.0, 0.8, 1.0, 1.0]),
        };

        renderer.fill_rounded_rect(rect, rect.height / 2.0, bg);
        if let BadgeVariant::Outline = self.variant {
            renderer.stroke_rounded_rect(rect, rect.height / 2.0, text_color, 1.0);
        }

        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        renderer.draw_text(
            &self.text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - th) / 2.0,
            12.0,
            text_color,
        );
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        Size { width: tw + 16.0, height: th + 8.0 }
    }
}

/// Skeleton component for displaying loading placeholders.
#[derive(Clone)]
pub struct Skeleton {
    pub(crate) width: Option<f32>,
    pub(crate) height: Option<f32>,
    pub(crate) rounded: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            rounded: true,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Skeleton {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Shimmer effect simulation
        let time = renderer.elapsed_time();
        let shimmer = (time * 2.0).sin() * 0.1 + 0.2;
        let color = [shimmer, shimmer, shimmer + 0.05, 1.0];

        if self.rounded {
            renderer.fill_rounded_rect(rect, 4.0, color);
        } else {
            renderer.fill_rect(rect, color);
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: self.width.or(proposal.width).unwrap_or(100.0),
            height: self.height.or(proposal.height).unwrap_or(20.0),
        }
    }
}
