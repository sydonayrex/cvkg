use crate::theme;
use crate::RADIUS_XL;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{ElapsedTime, Never, Rect, Renderer, Size, View};

/// RunesCard - A container component with header, content, and footer sections.
/// Named after the Runes, the ancient inscribed containers of meaning.
#[doc(alias = "Card")]
#[derive(Clone)]
pub struct RunesCard<V> {
    /// Optional header content
    header: Option<V>,
    /// Optional main content
    content: Option<V>,
    /// Optional footer content
    footer: Option<V>,
}

impl<V: View> RunesCard<V> {
    /// Create a new empty RunesCard.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::{RunesCard, Text};
    /// let card: RunesCard<Text> = RunesCard::new();
    /// ```
    pub fn new() -> Self {
        Self {
            header: None,
            content: None,
            footer: None,
        }
    }

    /// Set the header content.
    pub fn header(mut self, header: V) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the main content.
    pub fn content(mut self, content: V) -> Self {
        self.content = Some(content);
        self
    }

    /// Set the footer content.
    pub fn footer(mut self, footer: V) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl<V: View> Default for RunesCard<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for RunesCard<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RunesCard");

        // Render frosted glass background
        renderer.bifrost(rect, 20.0, 1.2, 0.9);
        // Mostly clear center
        renderer.fill_rounded_rect(rect, RADIUS_XL, theme::with_alpha(theme::bg(), 0.4));
        // Neon glass border
        renderer.stroke_rounded_rect(rect, RADIUS_XL, theme::with_alpha(theme::border(), 0.6), 1.5);
        // Inner highlight
        renderer.draw_line(
            rect.x,
            rect.y + 20.0,
            rect.x + rect.width,
            rect.y + 20.0,
            theme::with_alpha(theme::border(), 0.3),
            1.0,
        );

        // Calculate sections layout
        let padding = 16.0;
        let header_height = self.header.as_ref().map_or(0.0, |h| {
            let mut dummy_renderer = DummyRenderer::new();
            let size = h.intrinsic_size(&mut dummy_renderer, SizeProposal::unspecified());
            size.height
        });
        let footer_height = self.footer.as_ref().map_or(0.0, |f| {
            let mut dummy_renderer = DummyRenderer::new();
            let size = f.intrinsic_size(&mut dummy_renderer, SizeProposal::unspecified());
            size.height
        });
        let _content_height = self.content.as_ref().map_or(0.0, |c| {
            let mut dummy_renderer = DummyRenderer::new();
            let size = c.intrinsic_size(&mut dummy_renderer, SizeProposal::unspecified());
            size.height
        });

        let has_header = self.header.is_some();
        let has_footer = self.footer.is_some();

        let header_rect = if has_header {
            Some(Rect {
                x: rect.x + padding,
                y: rect.y + padding,
                width: rect.width - 2.0 * padding,
                height: header_height,
            })
        } else {
            None
        };

        let footer_rect = if has_footer {
            Some(Rect {
                x: rect.x + padding,
                y: rect.y + rect.height - padding - footer_height,
                width: rect.width - 2.0 * padding,
                height: footer_height,
            })
        } else {
            None
        };

        let content_rect = Rect {
            x: rect.x + padding,
            y: rect.y
                + if has_header {
                    padding + header_height + 8.0
                } else {
                    padding
                },
            width: rect.width - 2.0 * padding,
            height: rect.height
                - if has_header {
                    padding + header_height + 8.0
                } else {
                    padding
                }
                - if has_footer {
                    padding + footer_height + 8.0
                } else {
                    padding
                },
        };

        // Render header
        if let (Some(header), Some(hrect)) = (&self.header, header_rect) {
            header.render(renderer, hrect);
        }

        // Render content
        if let Some(content) = &self.content {
            content.render(renderer, content_rect);
        }

        // Render footer
        if let (Some(footer), Some(frect)) = (&self.footer, footer_rect) {
            footer.render(renderer, frect);
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let padding = 32.0; // 16 * 2

        let header_height = self
            .header
            .as_ref()
            .map_or(0.0, |h| h.intrinsic_size(renderer, proposal).height);
        let footer_height = self
            .footer
            .as_ref()
            .map_or(0.0, |f| f.intrinsic_size(renderer, proposal).height);
        let content_height = self
            .content
            .as_ref()
            .map_or(0.0, |c| c.intrinsic_size(renderer, proposal).height);

        let spacing = if self.header.is_some() && self.content.is_some() {
            8.0
        } else {
            0.0
        } + if self.content.is_some() && self.footer.is_some() {
            8.0
        } else {
            0.0
        };

        let total_height = header_height + content_height + footer_height + spacing + padding;
        let max_width = self
            .header
            .as_ref()
            .map_or(0.0, |h| h.intrinsic_size(renderer, proposal).width)
            .max(
                self.content
                    .as_ref()
                    .map_or(0.0, |c| c.intrinsic_size(renderer, proposal).width),
            )
            .max(
                self.footer
                    .as_ref()
                    .map_or(0.0, |f| f.intrinsic_size(renderer, proposal).width),
            );

        Size {
            width: max_width + padding,
            height: total_height,
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl<V: View> LayoutView for RunesCard<V> {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let mut dummy_renderer = DummyRenderer::new();
        self.intrinsic_size(&mut dummy_renderer, proposal)
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
        // Card manages its own layout
    }
}

/// Dummy renderer for size calculations during layout.
/// Implements Renderer trait for intrinsic size computation.
pub struct DummyRenderer {
    _phantom: std::marker::PhantomData<()>,
}

impl DummyRenderer {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for DummyRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ElapsedTime for DummyRenderer {
    fn delta_time(&self) -> f32 {
        0.016
    }

    fn elapsed_time(&self) -> f32 {
        0.0
    }
}

impl Renderer for DummyRenderer {
    fn push_vnode(&mut self, _rect: Rect, _name: &str) {}
    fn pop_vnode(&mut self) {}
    fn set_key(&mut self, _key: &str) {}
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _width: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _color: [f32; 4], _width: f32) {
    }
    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        let mut engine = cvkg_runic_text::TextEngine::new();
        engine.shape_layout(spans, max_width, align, overflow).ok()
    }
    fn draw_image(&mut self, _path: &str, _rect: Rect) {}
    fn bifrost(&mut self, _rect: Rect, _radius: f32, _sigma: f32, _alpha: f32) {}
    fn register_handler(
        &mut self,
        _event: &str,
        _handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
    }
    fn push_clip_rect(&mut self, _rect: Rect) {}
    fn pop_clip_rect(&mut self) {}
    fn push_opacity(&mut self, _opacity: f32) {}
    fn pop_opacity(&mut self) {}
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    fn pop_transform(&mut self) {}
    fn push_shadow(&mut self, _radius: f32, _color: [f32; 4], _offset: [f32; 2]) {}
    fn pop_shadow(&mut self) {}
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
}
