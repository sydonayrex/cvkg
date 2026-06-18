//! Display and typography components.
//!
//! ScrollArea -- scrollable area with custom scrollbar.
//! Typography -- text style component (H1-H4/Body/Caption/Overline).
//! Icon -- centered glyph component.
//! BackgroundPattern -- decorative background pattern (dots/grid/lines/hexagons).
//!
//! All components use cvkg theme system (theme::*) for full themability.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// ----------------------------------------------------------------------------
// ScrollArea -- scrollable area with custom scrollbar
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ScrollArea {
    /// Content height (may exceed visible area).
    pub content_height: f32,
    /// Visible height.
    pub height: f32,
    /// Scroll offset.
    pub scroll_y: f32,
    /// Width.
    pub width: f32,
    /// Whether to show scrollbar.
    pub show_scrollbar: bool,
}

impl ScrollArea {
    /// Create a new ScrollArea.
    pub fn new() -> Self {
        Self {
            content_height: 400.0,
            height: 200.0,
            scroll_y: 0.0,
            width: 300.0,
            show_scrollbar: true,
        }
    }

    /// Set content height.
    pub fn content_height(mut self, h: f32) -> Self {
        self.content_height = h;
        self
    }

    /// Set visible height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set scroll offset.
    pub fn scroll_y(mut self, y: f32) -> Self {
        self.scroll_y = y;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Show/hide scrollbar.
    pub fn scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ScrollArea {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ScrollArea");
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        if self.show_scrollbar && self.content_height > self.height {
            let sb_x = rect.x + rect.width - 8.0;
            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: rect.y + 4.0,
                    width: 4.0,
                    height: rect.height - 8.0,
                },
                2.0,
                theme::surface_elevated(),
            );
            let thumb_ratio = self.height / self.content_height;
            let thumb_h = (rect.height - 8.0) * thumb_ratio;
            let max_scroll = (self.content_height - self.height).max(0.0);
            let scroll_frac = if max_scroll > 0.0 {
                self.scroll_y / max_scroll
            } else {
                0.0
            };
            let thumb_y = rect.y + 4.0 + scroll_frac * (rect.height - 8.0 - thumb_h);
            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: thumb_y,
                    width: 4.0,
                    height: thumb_h,
                },
                2.0,
                theme::text_muted(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// Typography -- text style component
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypographyVariant {
    H1,
    H2,
    H3,
    H4,
    Body,
    Caption,
    Overline,
}

#[derive(Clone)]
pub struct Typography {
    /// Text content.
    pub text: String,
    /// Style variant.
    pub variant: TypographyVariant,
    /// Color override.
    pub color: Option<[f32; 4]>,
}

impl Typography {
    /// Create a new Typography.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            variant: TypographyVariant::Body,
            color: None,
        }
    }

    /// Set the variant.
    pub fn variant(mut self, v: TypographyVariant) -> Self {
        self.variant = v;
        self
    }

    /// Set the color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = Some(c);
        self
    }
}

impl View for Typography {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Typography");
        let size = match self.variant {
            TypographyVariant::H1 => 32.0,
            TypographyVariant::H2 => 24.0,
            TypographyVariant::H3 => 20.0,
            TypographyVariant::H4 => 16.0,
            TypographyVariant::Body => 14.0,
            TypographyVariant::Caption => 12.0,
            TypographyVariant::Overline => 10.0,
        };
        let default_color = match self.variant {
            TypographyVariant::Caption => theme::text_muted(),
            TypographyVariant::Overline => theme::text_dim(),
            _ => theme::text(),
        };
        let color = self.color.unwrap_or(default_color);
        renderer.draw_text(&self.text, rect.x, rect.y + size, size, color);
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let size = match self.variant {
            TypographyVariant::H1 => 32.0,
            TypographyVariant::H2 => 24.0,
            TypographyVariant::H3 => 20.0,
            TypographyVariant::H4 => 16.0,
            TypographyVariant::Body => 14.0,
            TypographyVariant::Caption => 12.0,
            TypographyVariant::Overline => 10.0,
        };
        let (tw, th) = renderer.measure_text(&self.text, size);
        Size {
            width: tw,
            height: th,
        }
    }
}

// ----------------------------------------------------------------------------
// Icon -- centered glyph component
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Icon {
    /// Icon character/text.
    pub glyph: String,
    /// Icon size.
    pub size: f32,
    /// Color.
    pub color: [f32; 4],
}

impl Icon {
    /// Create a new Icon.
    pub fn new(glyph: &str) -> Self {
        Self {
            glyph: glyph.to_string(),
            size: 24.0,
            color: theme::text(),
        }
    }

    /// Set the size.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set the color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
}

impl View for Icon {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Icon");
        let (tw, th) = renderer.measure_text(&self.glyph, self.size);
        renderer.draw_text(
            &self.glyph,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - th) / 2.0,
            self.size,
            self.color,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.size,
            height: self.size,
        }
    }
}

// ----------------------------------------------------------------------------
// BackgroundPattern -- decorative background pattern
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgPattern {
    Dots,
    Grid,
    Lines,
    Hexagons,
}

#[derive(Clone)]
pub struct BackgroundPattern {
    /// Pattern type.
    pub pattern: BgPattern,
    /// Pattern color.
    pub color: [f32; 4],
    /// Spacing between elements.
    pub spacing: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl BackgroundPattern {
    /// Create a new BackgroundPattern.
    pub fn new() -> Self {
        Self {
            pattern: BgPattern::Dots,
            color: [
                theme::border()[0],
                theme::border()[1],
                theme::border()[2],
                0.3,
            ],
            spacing: 20.0,
            width: 400.0,
            height: 300.0,
        }
    }

    /// Set the pattern type.
    pub fn pattern(mut self, p: BgPattern) -> Self {
        self.pattern = p;
        self
    }

    /// Set the color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set the spacing.
    pub fn spacing(mut self, s: f32) -> Self {
        self.spacing = s;
        self
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for BackgroundPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl View for BackgroundPattern {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BackgroundPattern");
        renderer.fill_rect(rect, theme::bg());
        match self.pattern {
            BgPattern::Dots => {
                let mut y = rect.y;
                while y < rect.y + rect.height {
                    let mut x = rect.x;
                    while x < rect.x + rect.width {
                        renderer.fill_ellipse(
                            Rect {
                                x,
                                y,
                                width: 2.0,
                                height: 2.0,
                            },
                            self.color,
                        );
                        x += self.spacing;
                    }
                    y += self.spacing;
                }
            }
            BgPattern::Grid => {
                let mut x = rect.x;
                while x < rect.x + rect.width {
                    renderer.draw_line(x, rect.y, x, rect.y + rect.height, self.color, 0.5);
                    x += self.spacing;
                }
                let mut y = rect.y;
                while y < rect.y + rect.height {
                    renderer.draw_line(rect.x, y, rect.x + rect.width, y, self.color, 0.5);
                    y += self.spacing;
                }
            }
            BgPattern::Lines => {
                let mut y = rect.y;
                while y < rect.y + rect.height {
                    renderer.draw_line(rect.x, y, rect.x + rect.width, y, self.color, 0.5);
                    y += self.spacing;
                }
            }
            BgPattern::Hexagons => {
                let mut row = 0;
                let mut y = rect.y;
                while y < rect.y + rect.height {
                    let offset = if row % 2 == 0 {
                        0.0
                    } else {
                        self.spacing * 0.5
                    };
                    let mut x = rect.x + offset;
                    while x < rect.x + rect.width {
                        renderer.stroke_ellipse(
                            Rect {
                                x: x - 6.0,
                                y: y - 6.0,
                                width: 12.0,
                                height: 12.0,
                            },
                            self.color,
                            0.5,
                        );
                        x += self.spacing;
                    }
                    y += self.spacing * 0.866;
                    row += 1;
                }
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}
