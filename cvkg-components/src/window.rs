use crate::clipped_corner::ClippedCornerNode;
use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// YggdrasilWindow - A tactical, draggable, resizable window container.
/// Named after the World Tree, the central pillar of the Norse cosmos.
pub struct YggdrasilWindow<V: View> {
    pub title: String,
    pub content: V,
    pub is_minimized: bool,
    pub border_color: [f32; 4],
}

impl<V: View> YggdrasilWindow<V> {
    pub fn new(title: impl Into<String>, content: V) -> Self {
        Self {
            title: title.into(),
            content,
            is_minimized: false,
            border_color: [1.0, 0.84, 0.0, 0.9], // Viking Gold
        }
    }

    pub fn minimized(mut self, minimized: bool) -> Self {
        self.is_minimized = minimized;
        self
    }
}

impl<V: View> View for YggdrasilWindow<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let header_height = 28.0;

        // 1. Background (Bifrost Frost)
        renderer.bifrost(rect, 20.0, 1.2, 0.9);

        // 2. Main Frame (Clipped Corner)
        let frame = ClippedCornerNode::new(cvkg_core::EmptyView)
            .border_color(self.border_color)
            .clip_size(10.0);
        frame.render(renderer, rect);

        // 3. Header
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_height,
        };
        renderer.fill_rect(
            header_rect,
            [
                self.border_color[0],
                self.border_color[1],
                self.border_color[2],
                0.2,
            ],
        );
        renderer.draw_text(
            &self.title,
            rect.x + 12.0,
            rect.y + 18.0,
            12.0,
            theme::text(),
        );

        // Header Underline
        renderer.draw_line(
            rect.x,
            rect.y + header_height,
            rect.x + rect.width,
            rect.y + header_height,
            self.border_color,
            1.0,
        );

        // 4. Content
        if !self.is_minimized {
            let content_rect = Rect {
                x: rect.x + 4.0,
                y: rect.y + header_height + 4.0,
                width: rect.width - 8.0,
                height: rect.height - header_height - 8.0,
            };
            self.content.render(renderer, content_rect);
        }
    }
}

/// GinnungagapWindow - A multi-dimensional folding window.
/// Named after the primordial void, it can fold to reveal deeper content planes.
/// This is a 'God Tier' component that enables hyper-spatial dimensionality.
pub struct GinnungagapWindow<V1: View, V2: View> {
    pub title: String,
    pub primary: V1,
    pub secondary: V2,
    pub fold_progress: f32, // 0.0 (Primary) to 1.0 (Secondary)
    pub border_color: [f32; 4],
}

impl<V1: View, V2: View> GinnungagapWindow<V1, V2> {
    pub fn new(title: impl Into<String>, primary: V1, secondary: V2) -> Self {
        Self {
            title: title.into(),
            primary,
            secondary,
            fold_progress: 0.0,
            border_color: [0.0, 1.0, 0.9, 0.9], // Bifrost Cyan
        }
    }

    pub fn fold_progress(mut self, progress: f32) -> Self {
        self.fold_progress = progress.clamp(0.0, 1.0);
        self
    }
}

impl<V1: View, V2: View> View for GinnungagapWindow<V1, V2> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let p = self.fold_progress.clamp(0.0, 1.0);
        let realm = cvkg_core::load_system_state().realm;
        let header_height = 28.0;

        // 1. Base Background & Border
        renderer.bifrost(rect, 30.0, 1.1, 0.95);
        renderer.stroke_rounded_rect(rect, 8.0, self.border_color, 1.5);

        // Header
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_height,
        };
        renderer.fill_rect(
            header_rect,
            [
                self.border_color[0],
                self.border_color[1],
                self.border_color[2],
                0.2,
            ],
        );
        renderer.draw_text(
            &self.title,
            rect.x + 12.0,
            rect.y + 18.0,
            12.0,
            theme::text(),
        );
        renderer.draw_line(
            rect.x,
            rect.y + header_height,
            rect.x + rect.width,
            rect.y + header_height,
            self.border_color,
            1.0,
        );

        let content_rect = Rect {
            x: rect.x + 4.0,
            y: rect.y + header_height + 4.0,
            width: rect.width - 8.0,
            height: rect.height - header_height - 8.0,
        };

        // 2. Realm-Dependent Rendering
        if realm == cvkg_core::Realm::Midgard {
            // Midgard: Simple cross-fade switch
            if p < 0.5 {
                self.primary.render(renderer, content_rect);
            } else {
                self.secondary.render(renderer, content_rect);
            }
            return;
        }

        // ASGARD MODE: Dimensional Folding
        // Calculate the slice and transform for the 'fold' effect
        // Left Half (Primary plane folding back)
        renderer.push_transform([0.0, 0.0], [1.0 - p * 0.5, 1.0], -p * 0.15);
        renderer.push_mjolnir_slice(0.0, -content_rect.width * 0.25);
        self.primary.render(renderer, content_rect);
        renderer.pop_mjolnir_slice();
        renderer.pop_transform();

        // Right Half (Secondary plane folding in)
        renderer.push_transform([0.0, 0.0], [0.5 + p * 0.5, 1.0], (1.0 - p) * 0.15);
        renderer.push_mjolnir_slice(0.0, content_rect.width * 0.25);
        self.secondary.render(renderer, content_rect);
        renderer.pop_mjolnir_slice();
        renderer.pop_transform();

        // Dimensional Rift (Center Line Glow)
        if p > 0.05 && p < 0.95 {
            let cx = content_rect.x + content_rect.width / 2.0;
            let rift_opacity = (1.0 - (p - 0.5).abs() * 2.0).powf(2.0);
            renderer.draw_line(
                cx,
                content_rect.y,
                cx,
                content_rect.y + content_rect.height,
                [0.0, 1.0, 1.0, 0.8 * rift_opacity],
                2.0,
            );
            renderer.gungnir(
                Rect {
                    x: cx - 2.0,
                    y: content_rect.y,
                    width: 4.0,
                    height: content_rect.height,
                },
                theme::accent(),
                15.0,
                0.6 * rift_opacity,
            );
        }
    }
}

/// HiminnModal - An elevated, glassmorphic modal dialog.
/// Named after the sky/heaven, it floats above Midgard (the main UI).
pub struct HiminnModal<V: View> {
    pub content: V,
    pub is_open: bool,
    pub blur_radius: f32,
    pub border_color: [f32; 4],
}

impl<V: View> HiminnModal<V> {
    /// Creates a new HiminnModal with the given content.
    pub fn new(content: V) -> Self {
        Self {
            content,
            is_open: false,
            blur_radius: 20.0,
            border_color: theme::accent(), // Bifrost Cyan
        }
    }

    /// Sets whether the modal is open.
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Sets the blur radius for the lensing effect.
    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }
}

impl<V: View> View for HiminnModal<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_open {
            return;
        }

        renderer.push_vnode(rect, "HiminnModal");

        // 1. Overlay (Darken background)
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.4));

        // 2. Modal Centering
        let modal_width = 400.0;
        let modal_height = 300.0;
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_width) / 2.0,
            y: rect.y + (rect.height - modal_height) / 2.0,
            width: modal_width,
            height: modal_height,
        };

        // 3. Liquid Glass Lensing Effect
        //)
        renderer.bifrost(modal_rect, self.blur_radius, 1.3, 0.85);
        renderer.stroke_rounded_rect(modal_rect, 12.0, self.border_color, 1.5);

        // 4. Content
        let content_rect = Rect {
            x: modal_rect.x + 16.0,
            y: modal_rect.y + 16.0,
            width: modal_rect.width - 32.0,
            height: modal_rect.height - 32.0,
        };
        self.content.render(renderer, content_rect);

        renderer.pop_vnode();
    }
}
