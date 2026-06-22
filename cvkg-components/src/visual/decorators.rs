use crate::theme;
use crate::RADIUS_SM;
use cvkg_core::{Never, Rect, Renderer, View};

/// MuninAvatar - A user representation component with status indicators.
/// Named after the hybrid concept of "form/image" (Eikona).
#[doc(alias = "Avatar")]
#[derive(Clone)]
pub struct MuninAvatar {
    pub src: Option<String>,
    pub fallback: String,
    pub status: Option<AvatarStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Offline,
    Busy,
    Away,
}

impl MuninAvatar {
    /// Creates a new MuninAvatar.
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            src: None,
            fallback: fallback.into(),
            status: None,
        }
    }

    /// Sets the image source for the avatar.
    pub fn src(mut self, src: impl Into<String>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Sets the online status indicator.
    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl View for MuninAvatar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MuninAvatar");

        // 1. Base Circle
        renderer.fill_ellipse(rect, theme::surface());
        renderer.stroke_ellipse(rect, theme::border(), 1.0);

        // 2. Content
        if let Some(src) = &self.src {
            renderer.draw_image(src, rect);
        } else {
            let (tw, _) = renderer.measure_text(&self.fallback, 14.0);
            renderer.draw_text(
                &self.fallback,
                rect.x + (rect.width - tw) / 2.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );
        }

        // 3. Status Indicator
        if let Some(status) = &self.status {
            let status_size = rect.width * 0.25;
            let status_rect = Rect {
                x: rect.x + rect.width - status_size,
                y: rect.y + rect.height - status_size,
                width: status_size,
                height: status_size,
            };

            let color = match status {
                AvatarStatus::Online => theme::success(),
                AvatarStatus::Offline => theme::text_muted(),
                AvatarStatus::Busy => theme::error_color(),
                AvatarStatus::Away => theme::warning(),
            };

            renderer.fill_ellipse(status_rect, color);
            renderer.stroke_ellipse(status_rect, theme::bg(), 1.5);
        }

        renderer.pop_vnode();
    }
}

/// MerkiBadge - A status or count indicator component.
/// Named after Merki, the Norse word for mark or sign.
#[derive(Clone)]
pub struct MerkiBadge {
    pub text: String,
    pub color: [f32; 4],
}

impl MerkiBadge {
    /// Creates a new MerkiBadge.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: theme::accent(),
        }
    }

    /// Sets the color of the badge.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for MerkiBadge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MerkiBadge");

        let mut bg = self.color;
        bg[3] *= 0.2;

        renderer.fill_rounded_rect(rect, RADIUS_SM, bg);
        renderer.stroke_rounded_rect(rect, RADIUS_SM, self.color, 1.0);

        let (tw, _) = renderer.measure_text(&self.text, 10.0);
        renderer.draw_text(
            &self.text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - 10.0) / 2.0,
            10.0,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}

/// UrdrTimeline - A chronological timeline of events (the past).
/// Named after Urdr, the Norn of the Past.
#[doc(alias = "Timeline")]
#[derive(Clone)]
pub struct UrdrTimeline {
    pub items: Vec<UrdrEvent>,
}

#[derive(Clone)]
pub struct UrdrEvent {
    pub title: String,
    pub timestamp: String,
    pub description: Option<String>,
}

impl UrdrTimeline {
    /// Creates a new UrdrTimeline.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds an event to the timeline.
    pub fn event(mut self, title: impl Into<String>, timestamp: impl Into<String>) -> Self {
        self.items.push(UrdrEvent {
            title: title.into(),
            timestamp: timestamp.into(),
            description: None,
        });
        self
    }
}

impl View for UrdrTimeline {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "UrdrTimeline");

        let t = renderer.elapsed_time();
        let line_x = rect.x + 20.0;
        renderer.draw_line(
            line_x,
            rect.y,
            line_x,
            rect.y + rect.height,
            theme::text_muted(),
            1.5,
        );

        let mut current_y = rect.y + 10.0;
        let item_spacing = 50.0;

        for (i, event) in self.items.iter().enumerate() {
            // 1. Bifrost Resonance (Glowing Temporal Nodes)
            let pulse = (t * 2.0 + i as f32).sin() * 0.2 + 0.8;
            renderer.fill_ellipse(
                Rect {
                    x: line_x - 5.0,
                    y: current_y - 1.0,
                    width: 10.0,
                    height: 10.0,
                },
                theme::with_alpha(theme::accent(), 0.3 * pulse),
            );
            renderer.fill_ellipse(
                Rect {
                    x: line_x - 3.0,
                    y: current_y + 1.0,
                    width: 6.0,
                    height: 6.0,
                },
                theme::progress_fill(),
            );

            // 2. Content
            renderer.draw_text(
                &event.timestamp,
                line_x + 20.0,
                current_y - 2.0,
                10.0,
                theme::text_muted(),
            );
            renderer.draw_text(
                &event.title,
                line_x + 20.0,
                current_y + 12.0,
                13.0,
                theme::text(),
            );

            current_y += item_spacing;
        }

        renderer.pop_vnode();
    }
}

/// DraumaSkeleton - A shimmering skeleton loader for async content.
/// Named after the dreams (Drauma) of content waiting to be born.
#[derive(Clone)]
pub struct DraumaSkeleton {
    pub border_radius: f32,
    pub shimmer: bool,
}

impl DraumaSkeleton {
    /// Creates a new DraumaSkeleton.
    pub fn new() -> Self {
        Self {
            border_radius: 4.0,
            shimmer: true,
        }
    }

    /// Sets the border radius of the skeleton.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Enables or disables the shimmer effect.
    pub fn shimmer(mut self, enabled: bool) -> Self {
        self.shimmer = enabled;
        self
    }
}

impl View for DraumaSkeleton {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DraumaSkeleton");

        renderer.set_aria_role("status");
        let t = renderer.elapsed_time();

        // 1. Mimir's Refraction (Skeletal Depth)
        // Drauma represents a "spectral" presence of content
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, self.border_radius, 2.0, 0.8);
        }
        renderer.fill_rounded_rect(rect, self.border_radius, theme::with_alpha(theme::surface(), 0.6));

        // 2. Kinetic Shimmer Effect
        if self.shimmer {
            let shimmer_pos = (t * 1.2).fract(); // Slower, more spectral shimmer
            let shimmer_w = rect.width * 0.5;
            let shimmer_rect = Rect {
                x: rect.x - shimmer_w + (rect.width + shimmer_w * 2.0) * shimmer_pos,
                y: rect.y,
                width: shimmer_w,
                height: rect.height,
            };

            let shimmer_alpha = 0.15 * (1.0 - (shimmer_pos - 0.5).abs() * 2.0);
            renderer.fill_rect(shimmer_rect, theme::with_alpha(theme::accent(), shimmer_alpha));
        }

        renderer.pop_vnode();
    }
}
