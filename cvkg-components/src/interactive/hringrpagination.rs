use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

#[derive(Clone)]
pub struct HringrPagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl HringrPagination {
    /// Creates a new HringrPagination.
    pub fn new(total_pages: usize, on_change: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            current_page: 1,
            total_pages,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Sets the current page.
    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page.clamp(1, self.total_pages);
        self
    }
}

impl View for HringrPagination {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HringrPagination");

        let btn_w = 32.0;
        let spacing = 4.0;
        let mut current_x = rect.x;

        // 1. Previous Button
        let prev_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(prev_rect, 4.0, theme::surface());
        renderer.draw_text(
            "<",
            prev_rect.x + 10.0,
            prev_rect.y + 10.0,
            14.0,
            theme::text(),
        );
        current_x += btn_w + spacing;

        // 2. Page Numbers (Simplified)
        for i in 1..=self.total_pages.min(5) {
            let page_rect = Rect {
                x: current_x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            };
            let is_selected = i == self.current_page;
            let bg = if is_selected {
                [0.0, 0.8, 1.0, 0.4]
            } else {
                theme::surface()
            };

            renderer.fill_rounded_rect(page_rect, 4.0, bg);
            if is_selected {
                renderer.stroke_rect(page_rect, [0.0, 1.0, 1.0, 0.8], 1.0);
            }

            renderer.draw_text(
                &i.to_string(),
                page_rect.x + 10.0,
                page_rect.y + 10.0,
                13.0,
                [1.0, 1.0, 1.0, 0.9],
            );
            current_x += btn_w + spacing;
        }

        // 3. Next Button
        let next_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(next_rect, 4.0, theme::surface());
        renderer.draw_text(
            ">",
            next_rect.x + 10.0,
            next_rect.y + 10.0,
            14.0,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}

/// ValhallaRating - A tactical rating component for assessing quality.
/// Named after Valhalla, where the chosen are assessed for their worth.#[derive(Clone, Copy)]
#[derive(Clone)]
pub struct ValhallaRating {
    pub value: f32,
    pub max: usize,
}

impl ValhallaRating {
    /// Creates a new ValhallaRating.
    pub fn new(value: f32) -> Self {
        Self { value, max: 5 }
    }

    /// Sets the maximum rating value.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }
}

impl View for ValhallaRating {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ValhallaRating");
        renderer.set_aria_role("slider");
        renderer.set_aria_label("Rating");

        let t = renderer.elapsed_time();
        let star_w = rect.width / self.max as f32;
        let star_h = rect.height;

        for i in 0..self.max {
            let star_rect = Rect {
                x: rect.x + i as f32 * star_w,
                y: rect.y,
                width: star_w * 0.8,
                height: star_h,
            };

            let is_filled = (i as f32) < self.value;

            // 1. Bifrost Resonance (Glowing Star Spirits)
            let resonance = if is_filled {
                (t * 2.0 + i as f32).sin() * 0.2 + 0.8
            } else {
                1.0
            };
            let color = if is_filled {
                [1.0, 0.84, 0.0, 0.9 * resonance] // Viking Gold with resonance
            } else {
                [0.2, 0.2, 0.25, 0.3] // Dimmed stone
            };

            renderer.fill_ellipse(star_rect, color);
            if is_filled {
                // Einherjar Spirit Glow
                renderer.stroke_ellipse(star_rect, [1.0, 1.0, 0.5, 0.4 * resonance], 1.5);
            }
        }

        renderer.pop_vnode();
    }
}

/// BifrostColorPicker - A color selection component.
/// Named after the Bifrost, the rainbow bridge connecting the realms.#[derive(Clone, Copy)]
#[derive(Clone)]
pub struct BifrostColorPicker {
    pub color: [f32; 4],
}

impl BifrostColorPicker {
    /// Creates a new BifrostColorPicker.
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }
}

impl View for BifrostColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BifrostColorPicker");

        // 1. Rainbow Track (Bifrost Bridge)
        let track_h = rect.height * 0.2;
        let track_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - track_h) / 2.0,
            width: rect.width,
            height: track_h,
        };

        // Mocking a rainbow gradient with segments
        let segments = 6;
        let seg_w = rect.width / segments as f32;
        let colors = [
            theme::error_color(),
            theme::warning(),
            theme::warning(),
            theme::success(),
            theme::info(),
            theme::secondary(),
        ];

        for i in 0..segments {
            renderer.fill_rect(
                Rect {
                    x: rect.x + i as f32 * seg_w,
                    y: track_rect.y,
                    width: seg_w,
                    height: track_h,
                },
                colors[i],
            );
        }

        // 2. Mimir's Refraction (Refractive Color Indicator)
        // Heimdall's Watch: A magnifying glass effect over the selection
        let indicator_size = rect.height * 0.9;
        let indicator_rect = Rect {
            x: rect.x + (rect.width - indicator_size) / 2.0,
            y: rect.y + (rect.height - indicator_size) / 2.0,
            width: indicator_size,
            height: indicator_size,
        };

        // Advanced refractive lensing
        renderer.bifrost(indicator_rect, indicator_size / 2.0, 2.0, 0.98);
        renderer.fill_ellipse(indicator_rect, self.color);

        // Surtur's Reactive Materials (Glow Ring)
        let t = renderer.elapsed_time();
        let pulse = (t * 3.0).sin() * 0.1 + 0.9;
        renderer.stroke_ellipse(indicator_rect, [1.0, 1.0, 1.0, 0.7 * pulse], 2.0);

        renderer.pop_vnode();
    }
}

// --- GeriTransfer ---
use cvkg_core::Size;
use cvkg_core::layout::SizeProposal;
#[derive(Clone)]
pub struct GeriTransfer<T> {
    left_items: Vec<T>,
    right_items: Vec<T>,
}

impl<T: Clone> GeriTransfer<T> {
    pub fn new(left: &[T], right: &[T]) -> Self {
        Self {
            left_items: left.to_vec(),
            right_items: right.to_vec(),
        }
    }
}

impl<T: Clone + View> View for GeriTransfer<T> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GeriTransfer");
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);

        let half_w = rect.width / 2.0;
        let left_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: half_w,
            height: rect.height,
        };
        let right_rect = Rect {
            x: rect.x + half_w,
            y: rect.y,
            width: half_w,
            height: rect.height,
        };

        // Draw separator
        renderer.draw_line(
            rect.x + half_w,
            rect.y,
            rect.x + half_w,
            rect.y + rect.height,
            [0.3, 0.3, 0.3, 1.0],
            1.0,
        );

        let item_h = 30.0;
        let mut y_offset = 10.0;
        for item in &self.left_items {
            let item_rect = Rect {
                x: left_rect.x + 10.0,
                y: left_rect.y + y_offset,
                width: left_rect.width - 20.0,
                height: item_h,
            };
            item.render(renderer, item_rect);
            y_offset += item_h + 5.0;
        }

        let mut y_offset = 10.0;
        for item in &self.right_items {
            let item_rect = Rect {
                x: right_rect.x + 10.0,
                y: right_rect.y + y_offset,
                width: right_rect.width - 20.0,
                height: item_h,
            };
            item.render(renderer, item_rect);
            y_offset += item_h + 5.0;
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 400.0,
            height: 300.0,
        }
    }
}
