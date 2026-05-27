use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Text alignment options for RichText.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Text styling flags for individual segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Rich text segment type.
pub enum RichTextSegment {
    /// Plain text with optional styling and color.
    Text {
        content: String,
        style: TextStyle,
        color: Option<[f32; 4]>,
        align: Option<TextAlign>,
    },
    /// Inline code snippet (monospace, colored).
    Code(String),
    /// Inline image with configurable display size.
    Image {
        path: String,
        width: f32,
        height: f32,
    },
    /// Inline image sized relative to text height.
    /// The image height is `text_height * scale` and width is derived
    /// from `aspect_ratio` (width/height). If `aspect_ratio` is None,
    /// the image is square (1:1).
    InlineImage {
        path: String,
        /// Multiplier on the current text_size to determine image height.
        /// Default is 1.0 (image height = text_size).
        scale: f32,
        /// Optional aspect ratio (width / height). None means 1:1.
        aspect_ratio: Option<f32>,
    },
}

impl RichTextSegment {
    /// Returns the text content if this is a Text variant.
    pub fn text_content(&self) -> Option<&str> {
        match self {
            RichTextSegment::Text { content, .. } => Some(content),
            _ => None,
        }
    }

    /// Returns true if this segment has bold styling.
    pub fn is_bold(&self) -> bool {
        match self {
            RichTextSegment::Text { style, .. } => style.bold,
            RichTextSegment::Code(_) => false,
            _ => false,
        }
    }

    /// Returns true if this segment has italic styling.
    pub fn is_italic(&self) -> bool {
        match self {
            RichTextSegment::Text { style, .. } => style.italic,
            _ => false,
        }
    }

    /// Returns true if this segment has underline styling.
    pub fn is_underline(&self) -> bool {
        match self {
            RichTextSegment::Text { style, .. } => style.underline,
            _ => false,
        }
    }

    /// Returns the per-segment color override, if any.
    pub fn color(&self) -> Option<[f32; 4]> {
        match self {
            RichTextSegment::Text { color, .. } => *color,
            _ => None,
        }
    }

    /// Returns the per-segment alignment override, if any.
    pub fn align(&self) -> Option<TextAlign> {
        match self {
            RichTextSegment::Text { align, .. } => *align,
            _ => None,
        }
    }
}

/// A component for displaying mixed content (text, code, images) with alignment,
/// word wrapping, per-segment styling, and inline images.
pub struct RichText {
    segments: Vec<RichTextSegment>,
    align: TextAlign,
    line_height: f32,
    text_size: f32,
    text_color: [f32; 4],
    /// Whether to wrap text at the rect width. Default: true.
    wrap: bool,
    /// Extra spacing between wrapped lines within the same segment, in pixels.
    /// Default: 2.0.
    wrap_line_gap: f32,
}

impl Default for RichText {
    fn default() -> Self {
        Self::new()
    }
}

impl RichText {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            align: TextAlign::Left,
            line_height: 20.0,
            text_size: 14.0,
            text_color: theme::text(),
            wrap: true,
            wrap_line_gap: 2.0,
        }
    }

    /// Set text alignment. Default is `TextAlign::Left`.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set the base text size in pixels. Default is 14.0.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// Set the line height in pixels. Default is 20.0.
    pub fn line_height(mut self, height: f32) -> Self {
        self.line_height = height;
        self
    }

    /// Set the default text color. Default is white `theme::text()`.
    pub fn text_color(mut self, color: [f32; 4]) -> Self {
        self.text_color = color;
        self
    }

    /// Enable or disable word wrapping. Default is true.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set the extra gap between wrapped lines within a segment. Default is 2.0.
    pub fn wrap_line_gap(mut self, gap: f32) -> Self {
        self.wrap_line_gap = gap;
        self
    }

    /// Append a plain text segment.
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.segments.push(RichTextSegment::Text {
            content: t.into(),
            style: TextStyle::default(),
            color: None,
            align: None,
        });
        self
    }

    /// Append a plain text segment with bold styling.
    pub fn bold(self, t: impl Into<String>) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text { style, .. }) = s.segments.last_mut() {
            style.bold = true;
        }
        s
    }

    /// Append a plain text segment with italic styling.
    pub fn italic(self, t: impl Into<String>) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text { style, .. }) = s.segments.last_mut() {
            style.italic = true;
        }
        s
    }

    /// Append a plain text segment with underline styling.
    pub fn underline(self, t: impl Into<String>) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text { style, .. }) = s.segments.last_mut() {
            style.underline = true;
        }
        s
    }

    /// Append a plain text segment with a specific color.
    pub fn color(self, t: impl Into<String>, color: [f32; 4]) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text { color: c, .. }) = s.segments.last_mut() {
            *c = Some(color);
        }
        s
    }

    /// Append a plain text segment with a specific alignment override.
    pub fn aligned(self, t: impl Into<String>, align: TextAlign) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text { align: a, .. }) = s.segments.last_mut() {
            *a = Some(align);
        }
        s
    }

    /// Append a plain text segment with bold, italic, underline, and color.
    pub fn styled(
        self,
        t: impl Into<String>,
        bold: bool,
        italic: bool,
        underline: bool,
        color: [f32; 4],
    ) -> Self {
        let mut s = self.text(t);
        if let Some(RichTextSegment::Text {
            style, color: c, ..
        }) = s.segments.last_mut()
        {
            style.bold = bold;
            style.italic = italic;
            style.underline = underline;
            *c = Some(color);
        }
        s
    }

    /// Append an inline code segment.
    pub fn code(mut self, c: impl Into<String>) -> Self {
        self.segments.push(RichTextSegment::Code(c.into()));
        self
    }

    /// Append an inline image with custom display size.
    pub fn image(mut self, path: impl Into<String>, width: f32, height: f32) -> Self {
        self.segments.push(RichTextSegment::Image {
            path: path.into(),
            width,
            height,
        });
        self
    }

    /// Append an inline image with default size (16x16).
    pub fn image_small(self, path: impl Into<String>) -> Self {
        self.image(path, 16.0, 16.0)
    }

    /// Append an inline image with medium size (24x24).
    pub fn image_medium(self, path: impl Into<String>) -> Self {
        self.image(path, 24.0, 24.0)
    }

    /// Append an inline image with large size (40x40).
    pub fn image_large(self, path: impl Into<String>) -> Self {
        self.image(path, 40.0, 40.0)
    }

    /// Append an inline image that flows with text at the baseline.
    /// The image height is `text_size * scale`. Width is derived from `aspect_ratio`.
    /// If `aspect_ratio` is None, the image is square.
    ///
    /// Example: `inline_image("icon.png", 1.2, Some(2.0))` creates an image
    /// that is 1.2x the text height and twice as wide as it is tall.
    pub fn inline_image(
        mut self,
        path: impl Into<String>,
        scale: f32,
        aspect_ratio: Option<f32>,
    ) -> Self {
        self.segments.push(RichTextSegment::InlineImage {
            path: path.into(),
            scale,
            aspect_ratio,
        });
        self
    }

    /// Append a square inline image at 1x text height.
    pub fn inline_image_small(self, path: impl Into<String>) -> Self {
        self.inline_image(path, 1.0, None)
    }

    /// Compute the X position for a segment based on alignment and measured width.
    fn aligned_x(&self, x: f32, rect_width: f32, segment_width: f32, align: TextAlign) -> f32 {
        match align {
            TextAlign::Left => x,
            TextAlign::Center => x + (rect_width - segment_width) / 2.0,
            TextAlign::Right => x + rect_width - segment_width,
            TextAlign::Justify => x,
        }
    }
}

impl View for RichText {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut y = rect.y;

        let mut text_spans = Vec::new();

        let runic_align = match self.align {
            TextAlign::Left => cvkg_runic_text::TextAlign::Start,
            TextAlign::Center => cvkg_runic_text::TextAlign::Center,
            TextAlign::Right => cvkg_runic_text::TextAlign::End,
            TextAlign::Justify => cvkg_runic_text::TextAlign::Justify,
        };

        for segment in &self.segments {
            if y >= rect.y + rect.height {
                break;
            }

            match segment {
                RichTextSegment::Text {
                    content,
                    style,
                    color,
                    ..
                } => {
                    let seg_color = color.unwrap_or(self.text_color);

                    let mut runic_style =
                        cvkg_runic_text::TextStyle::new("SF Pro Text", self.text_size);
                    if style.bold {
                        runic_style = runic_style.with_weight(700);
                    }
                    if style.italic {
                        runic_style = runic_style.italic();
                    }
                    if style.underline {
                        runic_style = runic_style.with_underline();
                    }
                    runic_style.color = [
                        (seg_color[0] * 255.0) as u8,
                        (seg_color[1] * 255.0) as u8,
                        (seg_color[2] * 255.0) as u8,
                        (seg_color[3] * 255.0) as u8,
                    ];

                    text_spans.push(cvkg_runic_text::TextSpan::new(content, runic_style));
                }
                RichTextSegment::Code(c) => {
                    // Flush pending text spans
                    if !text_spans.is_empty() {
                        if let Some(shaped) = renderer.shape_rich_text(
                            &text_spans,
                            if self.wrap { Some(rect.width) } else { None },
                            runic_align,
                            cvkg_runic_text::TextOverflow::WordWrap,
                        ) {
                            renderer.draw_shaped_text(&shaped, rect.x, y);
                            y += shaped.height;
                        }
                        text_spans.clear();
                    }

                    let code_bg = [0.1, 0.1, 0.1, 1.0];
                    let code_h = self.line_height + 5.0;
                    renderer.fill_rect(
                        Rect {
                            x: rect.x,
                            y,
                            width: rect.width,
                            height: code_h,
                        },
                        code_bg,
                    );
                    let (cw, _ch) = renderer.measure_text(c, self.text_size - 2.0);
                    let x = self.aligned_x(rect.x + 5.0, rect.width - 10.0, cw, self.align);
                    renderer.draw_text(c, x, y + 4.0, self.text_size - 2.0, theme::success());
                    y += code_h + 5.0;
                }
                RichTextSegment::Image {
                    path,
                    width,
                    height,
                } => {
                    if !text_spans.is_empty() {
                        if let Some(shaped) = renderer.shape_rich_text(
                            &text_spans,
                            if self.wrap { Some(rect.width) } else { None },
                            runic_align,
                            cvkg_runic_text::TextOverflow::WordWrap,
                        ) {
                            renderer.draw_shaped_text(&shaped, rect.x, y);
                            y += shaped.height;
                        }
                        text_spans.clear();
                    }

                    let img_w = *width;
                    let img_h = *height;
                    let x = self.aligned_x(rect.x, rect.width, img_w, self.align);
                    renderer.draw_image(
                        path,
                        Rect {
                            x,
                            y,
                            width: img_w,
                            height: img_h,
                        },
                    );
                    y += img_h + 5.0;
                }
                RichTextSegment::InlineImage {
                    path,
                    scale,
                    aspect_ratio,
                } => {
                    if !text_spans.is_empty() {
                        if let Some(shaped) = renderer.shape_rich_text(
                            &text_spans,
                            if self.wrap { Some(rect.width) } else { None },
                            runic_align,
                            cvkg_runic_text::TextOverflow::WordWrap,
                        ) {
                            renderer.draw_shaped_text(&shaped, rect.x, y);
                            y += shaped.height;
                        }
                        text_spans.clear();
                    }

                    let img_h = self.text_size * scale;
                    let img_w = match aspect_ratio {
                        Some(ratio) => img_h * ratio,
                        None => img_h,
                    };
                    let x = self.aligned_x(rect.x, rect.width, img_w, self.align);
                    let img_y = y + (self.text_size - img_h) / 2.0;
                    renderer.draw_image(
                        path,
                        Rect {
                            x,
                            y: img_y,
                            width: img_w,
                            height: img_h,
                        },
                    );
                    y += self.line_height;
                }
            }
        }

        // Flush remaining text spans
        if !text_spans.is_empty()
            && y < rect.y + rect.height
            && let Some(shaped) = renderer.shape_rich_text(
                &text_spans,
                if self.wrap { Some(rect.width) } else { None },
                runic_align,
                cvkg_runic_text::TextOverflow::WordWrap,
            )
        {
            renderer.draw_shaped_text(&shaped, rect.x, y);
        }
    }
}

// Helper trait impl for tests
#[cfg(test)]
trait RichTextExt {
    fn text_content(&self) -> Option<&str>;
    fn effective_align(&self, seg: &RichTextSegment) -> TextAlign;
}

#[cfg(test)]
impl RichTextExt for RichText {
    fn text_content(&self) -> Option<&str> {
        self.segments.first().and_then(|s| s.text_content())
    }

    fn effective_align(&self, seg: &RichTextSegment) -> TextAlign {
        match seg {
            RichTextSegment::Text { align: Some(a), .. } => *a,
            _ => self.align,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert!(!style.bold);
        assert!(!style.italic);
        assert!(!style.underline);
    }

    #[test]
    fn test_rich_text_segment_text() {
        let seg = RichTextSegment::Text {
            content: "hello".to_string(),
            style: TextStyle::default(),
            color: None,
            align: None,
        };
        assert_eq!(seg.text_content(), Some("hello"));
        assert!(!seg.is_bold());
        assert!(!seg.is_italic());
        assert!(!seg.is_underline());
        assert_eq!(seg.color(), None);
        assert_eq!(seg.align(), None);
    }

    #[test]
    fn test_rich_text_segment_bold() {
        let seg = RichTextSegment::Text {
            content: "bold".to_string(),
            style: TextStyle {
                bold: true,
                ..Default::default()
            },
            color: None,
            align: None,
        };
        assert!(seg.is_bold());
        assert!(!seg.is_italic());
    }

    #[test]
    fn test_rich_text_segment_color() {
        let seg = RichTextSegment::Text {
            content: "red".to_string(),
            style: TextStyle::default(),
            color: Some(theme::error_color()),
            align: None,
        };
        assert_eq!(seg.color(), Some(theme::error_color()));
    }

    #[test]
    fn test_rich_text_segment_align() {
        let seg = RichTextSegment::Text {
            content: "center".to_string(),
            style: TextStyle::default(),
            color: None,
            align: Some(TextAlign::Center),
        };
        assert_eq!(seg.align(), Some(TextAlign::Center));
    }

    #[test]
    fn test_rich_text_segment_code() {
        let seg = RichTextSegment::Code("let x = 1;".to_string());
        assert_eq!(seg.text_content(), None);
        assert!(!seg.is_bold());
    }

    #[test]
    fn test_rich_text_segment_inline_image() {
        let seg = RichTextSegment::InlineImage {
            path: "icon.png".to_string(),
            scale: 1.5,
            aspect_ratio: Some(2.0),
        };
        assert_eq!(seg.text_content(), None);
    }

    #[test]
    fn test_text_align_justify() {
        assert_eq!(TextAlign::Justify, TextAlign::Justify);
    }

    #[test]
    fn test_builder_methods() {
        let rt = RichText::new()
            .text("hello")
            .bold("bold text")
            .italic("italic text")
            .underline("underlined")
            .color("red text", theme::error_color())
            .aligned("right", TextAlign::Right)
            .styled("styled", true, true, true, theme::text_muted())
            .code("code")
            .image("img.png", 32.0, 32.0)
            .inline_image("icon.png", 1.2, Some(2.0))
            .inline_image_small("small.png")
            .image_small("s.png")
            .image_medium("m.png")
            .image_large("l.png")
            .align(TextAlign::Center)
            .text_size(16.0)
            .line_height(24.0)
            .text_color([0.9, 0.9, 0.9, 1.0])
            .wrap(true)
            .wrap_line_gap(4.0);

        assert_eq!(rt.segments.len(), 14);
        assert_eq!(rt.align, TextAlign::Center);
        assert_eq!(rt.text_size, 16.0);
        assert_eq!(rt.line_height, 24.0);
        assert!(rt.wrap);
        assert_eq!(rt.wrap_line_gap, 4.0);
    }

    #[test]
    fn test_backward_compat_text() {
        // Ensure the old .text() API still works
        let rt = RichText::new()
            .text("hello world")
            .code("fn main() {}")
            .image("photo.png", 40.0, 40.0);

        assert_eq!(rt.segments.len(), 3);
        assert_eq!(rt.text_content(), Some("hello world"));
    }

    #[test]
    fn test_effective_align() {
        let rt = RichText::new().align(TextAlign::Right);
        let seg = RichTextSegment::Text {
            content: "test".to_string(),
            style: TextStyle::default(),
            color: None,
            align: Some(TextAlign::Center),
        };
        assert_eq!(rt.effective_align(&seg), TextAlign::Center);

        let seg_no_align = RichTextSegment::Text {
            content: "test".to_string(),
            style: TextStyle::default(),
            color: None,
            align: None,
        };
        assert_eq!(rt.effective_align(&seg_no_align), TextAlign::Right);
    }
}
