use crate::theme;
use crate::{Color, FONT_SM, FONT_XS, FontWeight, Orientation, SPACE_SM, SPACE_XS};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_runic_text as runic;
use std::sync::Arc;

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
        let mut style = runic::TextStyle::new("SF Pro Text", self.font_size);
        if let FontWeight::Bold = self.font_weight {
            style = style.with_weight(700);
        }
        let clr = self.color.as_array();
        style.color = [
            (clr[0] * 255.0) as u8,
            (clr[1] * 255.0) as u8,
            (clr[2] * 255.0) as u8,
            (clr[3] * 255.0) as u8,
        ];
        let span = runic::TextSpan::new(&self.content, style);
        if let Some(shaped) = renderer.shape_rich_text(
            &[span],
            Some(rect.width),
            runic::TextAlign::Start,
            runic::TextOverflow::Clip,
        ) {
            renderer.draw_shaped_text(&shaped, rect.x, rect.y);
        } else {
            renderer.draw_text(&self.content, rect.x, rect.y, self.font_size, clr);
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut style = runic::TextStyle::new("SF Pro Text", self.font_size);
        if let FontWeight::Bold = self.font_weight {
            style = style.with_weight(700);
        }
        let clr = self.color.as_array();
        style.color = [
            (clr[0] * 255.0) as u8,
            (clr[1] * 255.0) as u8,
            (clr[2] * 255.0) as u8,
            (clr[3] * 255.0) as u8,
        ];
        let span = runic::TextSpan::new(&self.content, style);
        if let Some(shaped) = renderer.shape_rich_text(
            &[span],
            proposal.width,
            runic::TextAlign::Start,
            runic::TextOverflow::Clip,
        ) {
            Size {
                width: shaped.width,
                height: shaped.height,
            }
        } else {
            let (w, h) = renderer.measure_text(&self.content, self.font_size);
            Size {
                width: w,
                height: h,
            }
        }
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
    pub(crate) size: BadgeSize,
    pub(crate) on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    pub(crate) dot_indicator: bool,
    pub(crate) count_only: bool,
}

impl Badge {
    /// Create a new Badge component with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Default,
            size: BadgeSize::Md,
            on_click: None,
            dot_indicator: false,
            count_only: false,
        }
    }

    /// Set the visual variant of the badge.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the size of the badge.
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// Set an optional click callback.
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    /// Enable the dot-indicator style (small dot + text) for status indicators.
    pub fn dot_indicator(mut self, enabled: bool) -> Self {
        self.dot_indicator = enabled;
        self
    }

    /// Enable count-badge mode (number in a circle) for notification counts.
    pub fn count_only(mut self, enabled: bool) -> Self {
        self.count_only = enabled;
        self
    }
}

/// Visual variants for Badge, each mapping to distinct colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
}

/// Size variants controlling badge height and padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeSize {
    /// Small: 16px height
    Sm,
    /// Medium: 24px height
    Md,
    /// Large: 32px height
    Lg,
}

impl BadgeSize {
    /// Returns the badge height in logical pixels.
    pub fn height(self) -> f32 {
        match self {
            BadgeSize::Sm => 16.0,
            BadgeSize::Md => 24.0,
            BadgeSize::Lg => 32.0,
        }
    }

    /// Returns the horizontal padding.
    pub fn h_padding(self) -> f32 {
        match self {
            BadgeSize::Sm => SPACE_XS,
            BadgeSize::Md => SPACE_SM,
            BadgeSize::Lg => SPACE_SM * 1.5,
        }
    }

    /// Returns the font size for the badge text.
    pub fn font_size(self) -> f32 {
        match self {
            BadgeSize::Sm => FONT_XS,
            BadgeSize::Md => FONT_SM,
            BadgeSize::Lg => FONT_SM,
        }
    }
}

impl BadgeVariant {
    /// Returns the (background_color, text_color) as RGBA arrays for this variant.
    pub fn colors(self) -> ([f32; 4], [f32; 4]) {
        match self {
            BadgeVariant::Default => (theme::accent(), theme::text()),
            BadgeVariant::Secondary => (theme::border_strong(), theme::text()),
            BadgeVariant::Destructive => (theme::error_color(), theme::text()),
            BadgeVariant::Outline => (theme::button_ghost_bg(), theme::accent()),
            BadgeVariant::Success => (theme::success(), theme::text()),
            BadgeVariant::Warning => (theme::warning(), theme::text()),
        }
    }
}

impl View for Badge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (bg, text_color) = self.variant.colors();
        let height = self.size.height();
        let radius = height / 2.0;
        let font_size = self.size.font_size();

        let mut style = runic::TextStyle::new("SF Pro Text", font_size);
        style.color = [
            (text_color[0] * 255.0) as u8,
            (text_color[1] * 255.0) as u8,
            (text_color[2] * 255.0) as u8,
            (text_color[3] * 255.0) as u8,
        ];
        let span = runic::TextSpan::new(&self.text, style);

        if self.count_only {
            let (tw, th) = if let Some(shaped) = renderer.shape_rich_text(
                std::slice::from_ref(&span),
                None,
                runic::TextAlign::Start,
                runic::TextOverflow::Clip,
            ) {
                (shaped.width, shaped.height)
            } else {
                renderer.measure_text(&self.text, font_size)
            };
            let diameter = height.max(tw + SPACE_SM);
            let cr = Rect {
                x: rect.x,
                y: rect.y,
                width: diameter,
                height: diameter,
            };
            renderer.fill_ellipse(cr, bg);

            if let Some(shaped) = renderer.shape_rich_text(
                &[span],
                None,
                runic::TextAlign::Start,
                runic::TextOverflow::Clip,
            ) {
                renderer.draw_shaped_text(
                    &shaped,
                    rect.x + (diameter - tw) / 2.0,
                    rect.y + (diameter - th) / 2.0,
                );
            } else {
                renderer.draw_text(
                    &self.text,
                    rect.x + (diameter - tw) / 2.0,
                    rect.y + (diameter - th) / 2.0,
                    font_size,
                    text_color,
                );
            }
            return;
        }

        if self.variant == BadgeVariant::Outline {
            renderer.stroke_rounded_rect(rect, radius, text_color, 1.0);
        } else {
            renderer.fill_rounded_rect(rect, radius, bg);
        }

        let shaped_opt = renderer.shape_rich_text(
            std::slice::from_ref(&span),
            None,
            runic::TextAlign::Start,
            runic::TextOverflow::Clip,
        );
        let (tw, th) = if let Some(ref shaped) = shaped_opt {
            (shaped.width, shaped.height)
        } else {
            renderer.measure_text(&self.text, font_size)
        };

        if self.dot_indicator {
            let dot_size = 8.0;
            let dot_rect = Rect {
                x: rect.x + self.size.h_padding(),
                y: rect.y + (height - dot_size) / 2.0,
                width: dot_size,
                height: dot_size,
            };
            let dot_color = if self.variant == BadgeVariant::Outline {
                text_color
            } else {
                bg
            };
            renderer.fill_ellipse(dot_rect, dot_color);

            let text_x = rect.x + self.size.h_padding() + dot_size + SPACE_XS;
            let text_y = rect.y + (height - th) / 2.0;
            if let Some(shaped) = shaped_opt {
                renderer.draw_shaped_text(&shaped, text_x, text_y);
            } else {
                renderer.draw_text(&self.text, text_x, text_y, font_size, text_color);
            }
        } else {
            if let Some(shaped) = shaped_opt {
                renderer.draw_shaped_text(
                    &shaped,
                    rect.x + (rect.width - tw) / 2.0,
                    rect.y + (rect.height - th) / 2.0,
                );
            } else {
                renderer.draw_text(
                    &self.text,
                    rect.x + (rect.width - tw) / 2.0,
                    rect.y + (rect.height - th) / 2.0,
                    font_size,
                    text_color,
                );
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let height = self.size.height();
        let font_size = self.size.font_size();
        let h_pad = self.size.h_padding();

        let style = runic::TextStyle::new("SF Pro Text", font_size);
        let span = runic::TextSpan::new(&self.text, style);
        let tw = if let Some(shaped) = renderer.shape_rich_text(
            &[span],
            None,
            runic::TextAlign::Start,
            runic::TextOverflow::Clip,
        ) {
            shaped.width
        } else {
            renderer.measure_text(&self.text, font_size).0
        };

        if self.count_only {
            let diameter = height.max(tw + SPACE_SM);
            return Size {
                width: diameter,
                height: diameter,
            };
        }

        let mut width = tw + h_pad * 2.0;
        if self.dot_indicator {
            width += 8.0 + SPACE_XS; // dot + gap
        }

        Size { width, height }
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
    fn body(self) -> Self::Body {
        unreachable!()
    }

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
