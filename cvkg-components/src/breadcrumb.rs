use crate::theme;
use crate::{draw_focus_ring, FONT_SM, SPACE_XS};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Breadcrumb item representing a single navigation segment.
#[derive(Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub is_current: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            is_current: false,
        }
    }

    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn current(mut self, is_current: bool) -> Self {
        self.is_current = is_current;
        self
    }
}

/// Breadcrumb navigation component.
/// Displays a path of navigation links separated by a delimiter.
#[derive(Clone)]
pub struct Breadcrumb {
    pub(crate) items: Vec<BreadcrumbItem>,
    pub(crate) separator: String,
    pub(crate) color: [f32; 4],
    pub(crate) current_color: [f32; 4],
    pub(crate) font_size: f32,
    pub(crate) on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Breadcrumb {
    pub fn new(items: Vec<BreadcrumbItem>) -> Self {
        Self {
            items,
            separator: "/".to_string(),
            color: theme::text_muted(),
            current_color: theme::text(),
            font_size: FONT_SM,
            on_select: None,
        }
    }

    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn current_color(mut self, color: [f32; 4]) -> Self {
        self.current_color = color;
        self
    }

    pub fn theme_color(mut self, key: &str) -> Self {
        self.color = theme::color(key);
        self
    }

    pub fn theme_current_color(mut self, key: &str) -> Self {
        self.current_color = theme::color(key);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the callback invoked when a breadcrumb item is activated
    /// (via Enter/Space key or pointer click).
    /// The callback receives the index of the activated item.
    pub fn on_select(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }
}

impl View for Breadcrumb {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Hashes for internal state: focused flag and focused item index.
        let focus_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "breadcrumb".hash(&mut s);
            self.items.len().hash(&mut s);
            "focus".hash(&mut s);
            s.finish()
        };
        let focused_index_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "breadcrumb".hash(&mut s);
            self.items.len().hash(&mut s);
            "focused_index".hash(&mut s);
            s.finish()
        };

        let (is_focused, set_focused) = cvkg_vdom::use_state(focus_hash, false);
        let (focused_index, set_focused_index) =
            cvkg_vdom::use_state(focused_index_hash, 0usize);

        renderer.push_vnode(rect, "Breadcrumb");
        renderer.set_aria_role("navigation");
        renderer.set_aria_label("Breadcrumb");

        let mut x = rect.x;
        let y = rect.y;
        let total = self.items.len();

        // Track each item's x-position for click targeting.
        let mut item_rects: Vec<(f32, f32)> = Vec::with_capacity(total);

        for (i, item) in self.items.iter().enumerate() {
            let is_last = i == total - 1;
            let is_item_focused = is_focused && i == focused_index;
            let color = if item.is_current || is_last {
                self.current_color
            } else {
                self.color
            };

            let item_x = x;
            renderer.draw_text(&item.label, x, y, self.font_size, color);

            // Measure text width for positioning
            let (w, _) = renderer.measure_text(&item.label, self.font_size);
            item_rects.push((item_x, w));
            x += w;

            // Draw separator between items
            if !is_last {
                renderer.draw_text(&self.separator, x, y, self.font_size, self.color);
                let (sep_w, _) = renderer.measure_text(&self.separator, self.font_size);
                x += sep_w + SPACE_XS;
            }

            // Focus ring on the focused item.
            if is_item_focused {
                let focus_rect = Rect {
                    x: item_x,
                    y,
                    width: w,
                    height: self.font_size,
                };
                draw_focus_ring(renderer, focus_rect);
            }
        }

        // ── Keyboard navigation ──
        // ArrowLeft/ArrowRight move between items, Enter/Space activate,
        // Home/End jump to first/last, Tab exits (default behavior).
        let item_count = self.items.len();
        let on_select_kb = self.on_select.clone();
        let set_idx = set_focused_index.clone();
        let current_focused = focused_index;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowLeft" => {
                            if item_count > 0 {
                                let next = if current_focused == 0 {
                                    item_count - 1
                                } else {
                                    current_focused - 1
                                };
                                set_idx(next);
                            }
                        }
                        "ArrowRight" => {
                            if item_count > 0 {
                                let next = (current_focused + 1) % item_count;
                                set_idx(next);
                            }
                        }
                        "Home" => {
                            if item_count > 0 {
                                set_idx(0);
                            }
                        }
                        "End" => {
                            if item_count > 0 {
                                set_idx(item_count - 1);
                            }
                        }
                        "Enter" | " " => {
                            if let Some(ref cb) = on_select_kb {
                                (cb)(current_focused);
                            }
                        }
                        "Tab" => {
                            // Allow default Tab behavior to exit the breadcrumb.
                        }
                        _ => {}
                    }
                }
            }),
        );

        // ── Pointer click handlers per item ──
        for (i, &(item_x, item_w)) in item_rects.iter().enumerate() {
            let on_select_click = self.on_select.clone();
            let target_idx = i;
            let set_idx_click = set_focused_index.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |event| {
                    if let Event::PointerClick { x, .. } = event
                        && x >= item_x
                        && x <= item_x + item_w
                    {
                        set_idx_click(target_idx);
                        if let Some(ref cb) = on_select_click {
                            (cb)(target_idx);
                        }
                    }
                }),
            );
        }

        // ── Focus handlers ──
        let set_focused_in = set_focused.clone();
        renderer.register_handler(
            "focus",
            Arc::new(move |_| {
                (set_focused_in)(true);
            }),
        );

        let set_focused_out = set_focused.clone();
        renderer.register_handler(
            "blur",
            Arc::new(move |_| {
                (set_focused_out)(false);
            }),
        );

        renderer.pop_vnode();
    }
}
