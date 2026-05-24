use cvkg_core::{
Never, Rect, Renderer, Size, SizeProposal, View};
use crate::theme;
use std::sync::Arc;

/// Breadcrumb navigation component.
pub struct Breadcrumb {
    pub(crate) items: Vec<BreadcrumbItem>,
}

pub struct BreadcrumbItem {
    pub label: String,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(
        mut self,
        label: impl Into<String>,
        on_click: Option<impl Fn() + Send + Sync + 'static>,
    ) -> Self {
        self.items.push(BreadcrumbItem {
            label: label.into(),
            on_click: on_click.map(|f| Arc::new(f) as Arc<dyn Fn() + Send + Sync>),
        });
        self
    }
}

impl View for Breadcrumb {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut current_x = rect.x;
        for (i, item) in self.items.iter().enumerate() {
            let (tw, th) = renderer.measure_text(&item.label, 14.0);

            let color = if item.on_click.is_some() {
                theme::accent()
            } else {
                theme::text_muted()
            };
            renderer.draw_text(
                &item.label,
                current_x,
                rect.y + (rect.height - th) / 2.0,
                14.0,
                color,
            );
            current_x += tw + 8.0;

            if i < self.items.len() - 1 {
                let separator = "/";
                let (sw, _) = renderer.measure_text(separator, 14.0);
                renderer.draw_text(
                    separator,
                    current_x,
                    rect.y + (rect.height - th) / 2.0,
                    14.0,
                    theme::text_dim(),
                );
                current_x += sw + 8.0;
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0;
        for (i, item) in self.items.iter().enumerate() {
            let (tw, _) = renderer.measure_text(&item.label, 14.0);
            width += tw + 8.0;
            if i < self.items.len() - 1 {
                let (sw, _) = renderer.measure_text("/", 14.0);
                width += sw + 8.0;
            }
        }
        Size {
            width: proposal.width.unwrap_or(width),
            height: 24.0,
        }
    }
}

/// Pagination component for navigating through pages.
pub struct Pagination {
    pub(crate) current_page: usize,
    pub(crate) total_pages: usize,
    pub(crate) on_page_change: Arc<dyn Fn(usize) + Send + Sync>,
}

impl Pagination {
    pub fn new(
        current_page: usize,
        total_pages: usize,
        on_page_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            current_page,
            total_pages,
            on_page_change: Arc::new(on_page_change),
        }
    }
}

impl View for Pagination {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let btn_w = 32.0;
        let spacing = 8.0;
        let mut current_x = rect.x;

        let on_page_change = self.on_page_change.clone();
        let current_page = self.current_page;

        // Previous button
        let prev_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.push_vnode(prev_rect, "PaginationPrev");
        renderer.fill_rounded_rect(prev_rect, 4.0, theme::surface_elevated());
        renderer.draw_text(
            "<",
            prev_rect.x + 10.0,
            prev_rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        let on_prev = on_page_change.clone();
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |_| {
                if current_page > 1 {
                    on_prev(current_page - 1);
                }
            }),
        );
        renderer.pop_vnode();
        current_x += btn_w + spacing;

        // Page numbers (simplified)
        for p in 1..=self.total_pages.min(5) {
            let page_rect = Rect {
                x: current_x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            };
            renderer.push_vnode(page_rect, "PaginationPage");
            let is_current = p == self.current_page;
            let bg = if is_current {
                theme::accent()
            } else {
                theme::surface()
            };
            renderer.fill_rounded_rect(page_rect, 4.0, bg);
            let p_str = p.to_string();
            renderer.draw_text(
                &p_str,
                page_rect.x + 10.0,
                page_rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );

            let on_p = on_page_change.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    on_p(p);
                }),
            );
            renderer.pop_vnode();
            current_x += btn_w + spacing;
        }

        // Next button
        let next_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.push_vnode(next_rect, "PaginationNext");
        renderer.fill_rounded_rect(next_rect, 4.0, theme::surface_elevated());
        renderer.draw_text(
            ">",
            next_rect.x + 10.0,
            next_rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        let total = self.total_pages;
        let on_next = on_page_change.clone();
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |_| {
                if current_page < total {
                    on_next(current_page + 1);
                }
            }),
        );
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let btn_w = 32.0;
        let spacing = 8.0;
        let count = 2 + self.total_pages.min(5);
        Size {
            width: count as f32 * (btn_w + spacing),
            height: 32.0,
        }
    }
}
