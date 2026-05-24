use cvkg_core::{
    AnyView, Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use crate::theme;
use std::sync::Arc;

/// System-state hash keys for VirtualList interaction state.
const VL_HOVER_HASH: u64 = 0xF00_0001;
const VL_SELECTED_HASH: u64 = 0xF00_0002;
const VL_SCROLL_HASH: u64 = 0xF00_0003;

/// A virtualized list with hover, selection, and keyboard navigation.
///
/// Only items in the visible viewport are rendered. The list responds to:
/// - Pointer move: highlights the hovered item
/// - Pointer click: selects the clicked item
/// - Arrow Up/Down: moves selection up/down
/// - Enter: triggers the on_select callback for the selected item
/// - Home/End: jumps to first/last item
/// - PageUp/PageDown: jumps by page
pub struct VirtualList<D>
where
    D: Send + Sync + 'static,
{
    data: Vec<D>,
    item_height: f32,
    view_builder: Box<dyn Fn(&D, usize) -> AnyView + Send + Sync>,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl<D> VirtualList<D>
where
    D: Send + Sync + 'static,
{
    /// Creates a new VirtualList with the given data and item builder.
    pub fn new<F, V>(data: Vec<D>, item_height: f32, builder: F) -> Self
    where
        F: Fn(&D, usize) -> V + Send + Sync + 'static,
        V: View + Clone + 'static,
    {
        Self {
            data,
            item_height,
            view_builder: Box::new(move |d, i| builder(d, i).erase()),
            on_select: None,
        }
    }

    /// Sets the selection callback, invoked with the index when an item is clicked or Enter is pressed.
    pub fn on_select(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(callback));
        self
    }
}

impl<D> View for VirtualList<D>
where
    D: Send + Sync + 'static,
{
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let data_len = self.data.len();
        if data_len == 0 {
            return;
        }

        // Read system state for hover + selection
        let hover_state: Option<usize> = cvkg_core::load_system_state()
            .get_component_state::<Option<usize>>(VL_HOVER_HASH)
            .and_then(|v| v.read().ok().map(|g| g.clone()))
            .unwrap_or(None);

        let selected_state: Option<usize> = cvkg_core::load_system_state()
            .get_component_state::<Option<usize>>(VL_SELECTED_HASH)
            .and_then(|v| v.read().ok().map(|g| g.clone()))
            .unwrap_or(None);

        let scroll_offset: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(VL_SCROLL_HASH)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(0.0);

        let _content_h = data_len as f32 * self.item_height;
        let start_idx = (scroll_offset / self.item_height).floor().max(0.0) as usize;
        let visible_count = (rect.height / self.item_height).ceil() as usize + 1;
        let end_idx = (start_idx + visible_count).min(data_len);

        // ── Render visible items ──
        for idx in start_idx..end_idx {
            if let Some(item) = self.data.get(idx) {
                let item_y = rect.y + idx as f32 * self.item_height - scroll_offset;
                let item_rect = Rect {
                    x: rect.x,
                    y: item_y,
                    width: rect.width,
                    height: self.item_height,
                };

                // Background: alternate + hover + selected
                let bg = if Some(idx) == selected_state {
                    theme::list_item_selected()
                } else if Some(idx) == hover_state {
                    theme::surface_elevated()
                } else if idx % 2 == 0 {
                    theme::surface_elevated()
                } else {
                    theme::input_bg()
                };
                renderer.fill_rect(item_rect, bg);

                // Focus ring on selected
                if Some(idx) == selected_state {
                    renderer.stroke_rect(item_rect, [0.3, 0.5, 0.8, 0.6], 1.0);
                }

                // Render the item view
                let view = (self.view_builder)(item, idx);
                view.render(renderer, item_rect);

                // ── Pointer move handler (per item rect) ──
                let ir = item_rect;
                renderer.register_handler(
                    "pointermove",
                    Arc::new(move |event| {
                        if let Event::PointerMove { x, y, .. } = event {
                            if ir.contains(x, y) {
                                cvkg_core::update_system_state(move |s| {
                                    let mut s = s.clone();
                                    s.set_component_state(VL_HOVER_HASH, Some(idx));
                                    s
                                });
                            }
                        }
                    }),
                );

                // ── Click-to-select (per item) ──
                let ir2 = item_rect;
                let on_select = self.on_select.clone();
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |event| {
                        if let Event::PointerClick { x, y, .. } = event {
                            if ir2.contains(x, y) {
                                cvkg_core::update_system_state(move |s| {
                                    let mut s = s.clone();
                                    s.set_component_state(VL_SELECTED_HASH, Some(idx));
                                    s
                                });
                                if let Some(cb) = on_select.as_ref() {
                                    cb(idx);
                                }
                            }
                        }
                    }),
                );
            }
        }

        // ── Keyboard navigation (Arrow Up/Down, Enter, Home, End, PageUp/Down) ──
        let on_select_key = self.on_select.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    let current: Option<usize> = cvkg_core::load_system_state()
                        .get_component_state::<Option<usize>>(VL_SELECTED_HASH)
                        .and_then(|v| v.read().ok().map(|g| g.clone()))
                        .unwrap_or(None);

                    let new_selected = match key.as_str() {
                        "ArrowDown" => {
                            let cur = current.unwrap_or(0);
                            Some((cur + 1).min(data_len.saturating_sub(1)))
                        }
                        "ArrowUp" => {
                            let cur = current.unwrap_or(0);
                            Some(cur.saturating_sub(1))
                        }
                        "Home" => Some(0),
                        "End" => Some(data_len.saturating_sub(1)),
                        "PageDown" => {
                            let cur = current.unwrap_or(0);
                            Some((cur + 10).min(data_len.saturating_sub(1)))
                        }
                        "PageUp" => {
                            let cur = current.unwrap_or(0);
                            Some(cur.saturating_sub(10))
                        }
                        "Enter" => {
                            if let (Some(idx), Some(cb)) = (current, on_select_key.as_ref()) {
                                cb(idx);
                            }
                            None // don't change selection
                        }
                        _ => None,
                    };

                    if let Some(ns) = new_selected {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(VL_SELECTED_HASH, Some(ns));
                            s
                        });
                    }
                }
            }),
        );
    }
}

/// A simple labeled item for VirtualList.
#[derive(Clone)]
pub struct ListItem {
    pub label: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
}

impl ListItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            subtitle: None,
            icon: None,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl View for ListItem {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let x = rect.x + 12.0;
        if let Some(icon) = &self.icon {
            renderer.draw_text(icon, x, rect.y + 2.0, 14.0, theme::info());
            renderer.draw_text(&self.label, x + 20.0, rect.y + 2.0, 14.0, [1.0, 1.0, 1.0, 0.95]);
        } else {
            renderer.draw_text(&self.label, x, rect.y + 2.0, 14.0, [1.0, 1.0, 1.0, 0.95]);
        }
        if let Some(subtitle) = &self.subtitle {
            let sub_x = if self.icon.is_some() { x + 20.0 } else { x };
            renderer.draw_text(subtitle, sub_x, rect.y + 18.0, 11.0, theme::text_muted());
        }
    }
}

impl<D> LayoutView for VirtualList<D>
where
    D: Send + Sync + 'static,
{
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let height = self.data.len() as f32 * self.item_height;
        Size {
            width: 300.0,
            height,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
        // Virtualized: subviews are placed during render
    }
}
