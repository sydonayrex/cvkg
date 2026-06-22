use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, RADIUS_SM};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Settings style grouped form layout
///
/// # Contract
/// Simply delegates rendering of settings sections.
pub struct SettingsForm<V> {
    pub(crate) content: V,
}

impl<V: View> SettingsForm<V> {
    /// Creates a new SettingsForm.
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for SettingsForm<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.content.render(renderer, rect);
    }
}

const COLLAPSIBLE_ANIM_HASH: u64 = 0xD00_0001;

/// Collapsible component for hiding/showing content with animation.
///
/// # Contract
/// Opens/closes showing internal subviews with a slide/fade, triggered by pointer clicks on its header.
pub struct Collapsible<V> {
    pub(crate) content: V,
    pub(crate) header: String,
    pub(crate) is_open: bool,
}

impl<V: View> Collapsible<V> {
    /// Creates a new Collapsible view.
    pub fn new(header: impl Into<String>, content: V, is_open: bool) -> Self {
        Self {
            header: header.into(),
            content,
            is_open,
        }
    }
}

impl<V: View> View for Collapsible<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Collapsible");

        let header_h: f32 = 40.0;
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };

        renderer.fill_rounded_rect(header_rect, RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(header_rect, RADIUS_MD, theme::border(), 1.0);

        let arrow = if self.is_open { "▼" } else { "▶" };
        let accent = if self.is_open {
            theme::accent()
        } else {
            theme::text_muted()
        };
        renderer.draw_text(arrow, rect.x + 10.0, rect.y + 12.0, 12.0, accent);
        renderer.draw_text(
            &self.header,
            rect.x + 30.0,
            rect.y + 10.0,
            FONT_BASE + 2.0,
            theme::text(),
        );

        let hdr = header_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event
                    && hdr.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let current: bool = s
                            .get_component_state::<bool>(COLLAPSIBLE_ANIM_HASH)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(COLLAPSIBLE_ANIM_HASH, !current);
                        s
                    });
                }
            }),
        );

        let anim_open: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(COLLAPSIBLE_ANIM_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.is_open);

        if anim_open {
            let prev_h: f32 = cvkg_core::load_system_state()
                .get_component_state::<f32>(COLLAPSIBLE_ANIM_HASH + 1)
                .and_then(|v| v.read().ok().map(|v| *v))
                .unwrap_or(0.0);
            let target: f32 = if anim_open { 1.0 } else { 0.0 };
            let new_h = prev_h + (target - prev_h) * 0.2;
            cvkg_core::update_system_state(move |s| {
                let mut s = s.clone();
                s.set_component_state(COLLAPSIBLE_ANIM_HASH + 1, new_h);
                s
            });

            let max_content_h = rect.height - header_h - 4.0;
            let content_h = max_content_h * new_h;
            if content_h > 1.0 {
                let content_rect = Rect {
                    x: rect.x + 4.0,
                    y: rect.y + header_h + 4.0,
                    width: rect.width - 8.0,
                    height: content_h,
                };
                renderer.fill_rounded_rect(content_rect, RADIUS_SM, theme::with_alpha(theme::surface_elevated(), 0.4));
                self.content.render(renderer, content_rect);
            }
        }

        renderer.pop_vnode();
    }
}

/// GjallarSplitter - A draggable split pane component.
///
/// # Contract
/// Divides layout into two panes with an adjustable, draggable splitting boundary.
pub struct GjallarSplitter<V1: View, V2: View> {
    pub first: V1,
    pub second: V2,
    pub split_ratio: f32,
    pub orientation: cvkg_core::Orientation,
}

const SPLITTER_DRAG_HASH: u64 = 0xC00_0001;
const SPLITTER_RATIO_HASH: u64 = 0xC00_0002;

impl<V1: View, V2: View> GjallarSplitter<V1, V2> {
    /// Creates a new GjallarSplitter with the given views.
    pub fn new(first: V1, second: V2) -> Self {
        Self {
            first,
            second,
            split_ratio: 0.5,
            orientation: cvkg_core::Orientation::Horizontal,
        }
    }

    /// Sets the split ratio (clamped to 0.1..=0.9).
    pub fn split_ratio(mut self, ratio: f32) -> Self {
        self.split_ratio = ratio.clamp(0.1, 0.9);
        self
    }

    /// Sets the orientation of the split.
    pub fn orientation(mut self, orientation: cvkg_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<V1: View, V2: View> View for GjallarSplitter<V1, V2> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GjallarSplitter");

        let live_ratio: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(SPLITTER_RATIO_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.split_ratio)
            .clamp(0.1, 0.9);

        let handle_size = 4.0;
        let (first_rect, handle_rect, second_rect) = match self.orientation {
            cvkg_core::Orientation::Horizontal => {
                let w1 = rect.width * live_ratio - handle_size / 2.0;
                let r1 = Rect {
                    x: rect.x,
                    y: rect.y,
                    width: w1,
                    height: rect.height,
                };
                let rh = Rect {
                    x: rect.x + w1,
                    y: rect.y,
                    width: handle_size,
                    height: rect.height,
                };
                let r2 = Rect {
                    x: rect.x + w1 + handle_size,
                    y: rect.y,
                    width: rect.width - w1 - handle_size,
                    height: rect.height,
                };
                (r1, rh, r2)
            }
            cvkg_core::Orientation::Vertical => {
                let h1 = rect.height * live_ratio - handle_size / 2.0;
                let r1 = Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: h1,
                };
                let rh = Rect {
                    x: rect.x,
                    y: rect.y + h1,
                    width: rect.width,
                    height: handle_size,
                };
                let r2 = Rect {
                    x: rect.x,
                    y: rect.y + h1 + handle_size,
                    width: rect.width,
                    height: rect.height - h1 - handle_size,
                };
                (r1, rh, r2)
            }
        };

        self.first.render(renderer, first_rect);
        self.second.render(renderer, second_rect);

        let is_dragging: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(SPLITTER_DRAG_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(false);
        let handle_color = if is_dragging {
            theme::with_alpha(theme::accent(), 0.5)
        } else {
            theme::surface_elevated()
        };
        renderer.fill_rect(handle_rect, handle_color);
        renderer.stroke_rect(
            handle_rect,
            theme::with_alpha(theme::accent(), if is_dragging { 0.8 } else { 0.4 }),
            1.0,
        );

        let center_x = handle_rect.x + handle_rect.width / 2.0;
        let center_y = handle_rect.y + handle_rect.height / 2.0;
        renderer.fill_rounded_rect(
            Rect {
                x: center_x - 1.0,
                y: center_y - 10.0,
                width: 2.0,
                height: 20.0,
            },
            1.0,
            theme::with_alpha(theme::accent(), 0.8),
        );

        let h_rect = handle_rect;
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && h_rect.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(SPLITTER_DRAG_HASH, true);
                        s
                    });
                }
            }),
        );

        let full_rect2 = rect;
        let orient2 = self.orientation;
        renderer.register_handler(
            "pointermove",
            Arc::new(move |event| {
                if let Event::PointerMove { x, y, .. } = event {
                    let dragging: bool = cvkg_core::load_system_state()
                        .get_component_state::<bool>(SPLITTER_DRAG_HASH)
                        .and_then(|v| v.read().ok().map(|v| *v))
                        .unwrap_or(false);
                    if dragging {
                        let new_ratio = match orient2 {
                            cvkg_core::Orientation::Horizontal => {
                                (x - full_rect2.x) / full_rect2.width
                            }
                            cvkg_core::Orientation::Vertical => {
                                (y - full_rect2.y) / full_rect2.height
                            }
                        };
                        let clamped = new_ratio.clamp(0.1, 0.9);
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(SPLITTER_RATIO_HASH, clamped);
                            s
                        });
                    }
                }
            }),
        );

        renderer.register_handler(
            "pointerup",
            Arc::new(move |event| {
                if let Event::PointerUp { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(SPLITTER_DRAG_HASH, false);
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }
}

/// SagaAccordion - A collapsible accordion component for revealing stories (data).
///
/// # Contract
/// Displays vertical sections that expand individually or concurrently.
#[derive(Clone)]
pub struct SagaAccordion<V: View> {
    pub items: Vec<SagaItem<V>>,
    pub allow_multiple: bool,
}

#[derive(Clone)]
pub struct SagaItem<V: View> {
    pub title: String,
    pub content: V,
    pub is_expanded: bool,
}

impl<V: View> SagaAccordion<V> {
    /// Creates a new SagaAccordion.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            allow_multiple: false,
        }
    }

    /// Adds an item to the accordion.
    pub fn item(mut self, title: impl Into<String>, content: V) -> Self {
        self.items.push(SagaItem {
            title: title.into(),
            content,
            is_expanded: false,
        });
        self
    }

    /// Sets whether multiple items can be expanded at once.
    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }
}

impl<V: View> View for SagaAccordion<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SagaAccordion");

        let t = renderer.elapsed_time();
        let mut current_y = rect.y;
        let item_h = 44.0;

        let state_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "saga_accordion_expanded".hash(&mut s);
            s.finish()
        };

        let mut expanded_state: Vec<bool> = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<Vec<bool>>(state_hash)
                .and_then(|v| v.read().ok().map(|g| g.clone()))
                .unwrap_or_else(|| self.items.iter().map(|item| item.is_expanded).collect())
        };

        while expanded_state.len() < self.items.len() {
            expanded_state.push(false);
        }

        for (i, item) in self.items.iter().enumerate() {
            let is_expanded = expanded_state.get(i).copied().unwrap_or(false);

            let header_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: item_h,
            };

            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(header_rect, 4.0, 1.2, 0.9);
            }
            renderer.fill_rounded_rect(header_rect, RADIUS_SM, theme::surface_elevated());

            if is_expanded {
                let pulse = (t * 3.0 + i as f32).sin() * 0.1 + 0.9;
                renderer.stroke_rounded_rect(header_rect, RADIUS_SM, theme::with_alpha(theme::accent(), 0.4 * pulse), 1.5);
            } else {
                renderer.stroke_rounded_rect(header_rect, RADIUS_SM, theme::border(), 1.0);
            }

            let arrow = if is_expanded { "▼" } else { "▶" };
            let accent_color = if is_expanded {
                theme::accent()
            } else {
                theme::text_muted()
            };

            renderer.draw_text(
                arrow,
                header_rect.x + 12.0,
                header_rect.y + 14.0,
                12.0,
                accent_color,
            );
            renderer.draw_text(
                &item.title,
                header_rect.x + 36.0,
                header_rect.y + 14.0,
                14.0,
                theme::text(),
            );

            let hdr_x = header_rect.x;
            let hdr_y = header_rect.y;
            let hdr_w = header_rect.width;
            let hdr_h = header_rect.height;
            let idx = i;
            let allow_multi = self.allow_multiple;
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event
                        && x >= hdr_x
                        && x <= hdr_x + hdr_w
                        && y >= hdr_y
                        && y <= hdr_y + hdr_h
                    {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let mut state: Vec<bool> = s
                                .get_component_state::<Vec<bool>>(state_hash)
                                .and_then(|v| v.read().ok().map(|g| g.clone()))
                                .unwrap_or_default();
                            while state.len() <= idx {
                                state.push(false);
                            }
                            state[idx] = !state[idx];
                            if !allow_multi && state[idx] {
                                for (j, v) in state.iter_mut().enumerate() {
                                    if j != idx {
                                        *v = false;
                                    }
                                }
                            }
                            s.set_component_state(state_hash, state);
                            s
                        });
                    }
                }),
            );

            current_y += item_h + 4.0;

            if is_expanded {
                let content_h = 120.0;
                let content_rect = Rect {
                    x: rect.x + 12.0,
                    y: current_y,
                    width: rect.width - 24.0,
                    height: content_h,
                };

                renderer.fill_rounded_rect(content_rect, RADIUS_SM, theme::with_alpha(theme::surface_elevated(), 0.3));
                item.content.render(renderer, content_rect);

                current_y += content_h + 8.0;
            }
        }

        renderer.pop_vnode();
    }
}
