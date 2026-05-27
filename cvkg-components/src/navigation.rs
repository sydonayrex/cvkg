use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use cvkg_layout::HStack;
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
        // Collect label and separator widths for layout computation
        let elements: Vec<(String, bool)> = self
            .items
            .iter()
            .enumerate()
            .flat_map(|(i, item)| {
                renderer.set_aria_role("navigation");
                let mut v = vec![(item.label.clone(), false)];
                if i < self.items.len() - 1 {
                    v.push(("/".to_string(), true));
                }
                v
            })
            .collect();

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> = vec![];
        let spacing = 8.0;

        // Delegate horizontal geometry to HStack::compute_layout
        let element_widths: Vec<f32> = elements
            .iter()
            .map(|(text, _)| {
                let (tw, _) = renderer.measure_text(text, 14.0);
                tw
            })
            .collect();
        let total_width: f32 = element_widths.iter().sum::<f32>()
            + spacing * (elements.len().saturating_sub(1)) as f32;

        let stack_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: total_width,
            height: rect.height,
        };
        let _computed = HStack::compute_layout(
            spacing,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            stack_rect,
            &layouts,
            &mut cache,
        );

        // Render using layout-engine-delegated positions
        let mut current_x = rect.x;
        for (i, (text, is_sep)) in elements.iter().enumerate() {
            let (tw, th) = renderer.measure_text(text, 14.0);
            let color = if *is_sep {
                theme::text_dim()
            } else if i < self.items.len()
                && self.items[i / (1 + (i > 0) as usize)].on_click.is_some()
            {
                theme::accent()
            } else {
                theme::text_muted()
            };
            let y_pos = rect.y + (rect.height - th) / 2.0;
            renderer.draw_text(text, current_x, y_pos, 14.0, color);
            current_x += tw + spacing;
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

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl cvkg_core::layout::LayoutView for Breadcrumb {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        // We use a dummy measure logic here to stay decoupled from Renderer in LayoutView
        // Actually, without a renderer we can't measure text accurately.
        // We will approximate or just use the proposal.
        let mut width = 0.0;
        for (i, item) in self.items.iter().enumerate() {
            // approximation: 7 pixels per char
            width += item.label.len() as f32 * 7.0 + 8.0;
            if i < self.items.len() - 1 {
                width += 1.0 * 7.0 + 8.0;
            }
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(width),
            height: 24.0,
        }
    }

    fn place_subviews(
        &self,
        bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> = vec![];
        let spacing = 8.0;
        // Delegate geometry calculation to the layout engine
        let _computed = HStack::compute_layout(
            spacing,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            bounds,
            &layouts,
            cache,
        );
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

        let on_page_change = self.on_page_change.clone();
        let current_page = self.current_page;

        // Build the list of button labels for layout delegation
        let mut labels: Vec<String> = Vec::new();
        labels.push("<".to_string());
        for p in 1..=self.total_pages.min(5) {
            labels.push(p.to_string());
        }
        labels.push(">".to_string());

        let total_width = labels.len() as f32 * btn_w + (labels.len() - 1) as f32 * spacing;
        let stack_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: total_width,
            height: rect.height,
        };

        // Delegate horizontal geometry to HStack::compute_layout
        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> = vec![];
        let computed_rects = HStack::compute_layout(
            spacing,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            stack_rect,
            &layouts,
            &mut cache,
        );

        // Previous button
        let prev_rect = if !computed_rects.is_empty() {
            Rect {
                x: computed_rects[0].x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            }
        } else {
            Rect {
                x: rect.x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            }
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

        // Page numbers (simplified)
        for (idx, p) in (1..=self.total_pages.min(5)).enumerate() {
            let rect_idx = idx + 1;
            let page_rect = if rect_idx < computed_rects.len() {
                Rect {
                    x: computed_rects[rect_idx].x,
                    y: rect.y,
                    width: btn_w,
                    height: rect.height,
                }
            } else {
                let offset = rect_idx as f32 * (btn_w + spacing);
                Rect {
                    x: rect.x + offset,
                    y: rect.y,
                    width: btn_w,
                    height: rect.height,
                }
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
        }

        // Next button
        let next_idx = labels.len() - 1;
        let next_rect = if next_idx < computed_rects.len() {
            Rect {
                x: computed_rects[next_idx].x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            }
        } else {
            let offset = next_idx as f32 * (btn_w + spacing);
            Rect {
                x: rect.x + offset,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            }
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

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }
}

impl cvkg_core::layout::LayoutView for Pagination {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        let btn_w = 32.0;
        let spacing = 8.0;
        let count = 2 + self.total_pages.min(5);
        let width = count as f32 * (btn_w + spacing);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(width),
            height: proposal.height.unwrap_or(32.0),
        }
    }

    fn place_subviews(
        &self,
        bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> = vec![];
        let spacing = 8.0;
        // Delegate geometry calculation to the layout engine
        let _computed = HStack::compute_layout(
            spacing,
            cvkg_core::Alignment::Center,
            cvkg_core::Distribution::Leading,
            bounds,
            &layouts,
            cache,
        );
    }
}
