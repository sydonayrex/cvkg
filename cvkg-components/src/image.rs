use cvkg_core::{Never, Rect, Renderer, View};

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
                renderer.stroke_rect(rect, [1.0, 0.0, 0.0, 1.0], 2.0);
                log::error!("AsyncImage error loading {}: {}", self.url, msg);
            }
        }
    }
}
