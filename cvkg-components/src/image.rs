use cvkg_core::{Never, Rect, Renderer, View};
use crate::theme;

/// Static image view
pub struct Image {
    name: String,
}

impl Image {
    /// Create a new Image view from an asset name or path.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl View for Image {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.draw_image(&self.name, rect);
    }
}

/// Asynchronous image view that loads from a URL or slow source
pub struct AsyncImage<P: View> {
    url: String,
    placeholder: P,
}

impl<P: View> AsyncImage<P> {
    /// Create a new AsyncImage view from a URL and a placeholder.
    pub fn new(url: impl Into<String>, placeholder: P) -> Self {
        Self {
            url: url.into(),
            placeholder,
        }
    }
}

impl<P: View> View for AsyncImage<P> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let manager = cvkg_core::Environment::<cvkg_core::AssetKey>::new().get();

        match manager.load_image(&self.url) {
            cvkg_core::AssetState::Loading => {
                self.placeholder.render(renderer, rect);
            }
            cvkg_core::AssetState::Ready(data) => {
                renderer.load_image(&self.url, data.as_ref().as_slice());
                renderer.draw_image(&self.url, rect);
            }
            cvkg_core::AssetState::Error(msg) => {
                // For now, draw a red border to indicate error
                renderer.stroke_rect(rect, theme::error_color(), 2.0);
                log::error!("AsyncImage error loading {}: {}", self.url, msg);
            }
        }
    }
}

/// Avatar component for displaying user profile pictures with fallbacks.
pub struct Avatar {
    pub(crate) src: Option<String>,
    pub(crate) fallback: String,
    pub(crate) size: AvatarSize,
}

impl Avatar {
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            src: None,
            fallback: fallback.into(),
            size: AvatarSize::Md,
        }
    }

    pub fn src(mut self, src: impl Into<String>) -> Self {
        self.src = Some(src.into());
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl AvatarSize {
    pub fn dimension(&self) -> f32 {
        match self {
            AvatarSize::Sm => 24.0,
            AvatarSize::Md => 40.0,
            AvatarSize::Lg => 64.0,
            AvatarSize::Xl => 96.0,
        }
    }
}

impl View for Avatar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let dim = self.size.dimension();
        let avatar_rect = Rect {
            x: rect.x + (rect.width - dim) / 2.0,
            y: rect.y + (rect.height - dim) / 2.0,
            width: dim,
            height: dim,
        };

        // Draw circular background/border
        renderer.fill_ellipse(avatar_rect, theme::surface_elevated());
        renderer.stroke_ellipse(avatar_rect, [0.0, 0.8, 1.0, 0.5], 1.5);

        if let Some(src) = &self.src {
            renderer.draw_image(src, avatar_rect);
        } else {
            // Draw fallback text centered
            let (tw, th) = renderer.measure_text(&self.fallback, dim * 0.4);
            renderer.draw_text(
                &self.fallback,
                avatar_rect.x + (dim - tw) / 2.0,
                avatar_rect.y + (dim - th) / 2.0,
                dim * 0.4,
                theme::text(),
            );
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let dim = self.size.dimension();
        cvkg_core::Size {
            width: dim,
            height: dim,
        }
    }
}
