use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Item in the asset browser.
pub struct AssetItem {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub item_type: AssetType,
}

pub enum AssetType {
    Image,
    Video,
    Audio,
    Document,
    Folder,
}

/// Asset browser component
pub struct AssetBrowser {
    pub(crate) items: Vec<AssetItem>,
    pub(crate) selected: Option<String>,
    pub(crate) view_mode: ViewMode,
}

pub enum ViewMode {
    Grid,
    List,
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            view_mode: ViewMode::Grid,
        }
    }

    pub fn items(mut self, items: Vec<AssetItem>) -> Self {
        self.items = items;
        self
    }

    pub fn select(mut self, id: &str) -> Self {
        self.selected = Some(id.to_string());
        self
    }
}

impl View for AssetBrowser {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let item_size = 80.0;
        let padding = 8.0;
        let cols = (rect.width / (item_size + padding)).floor() as usize;

        if matches!(self.view_mode, ViewMode::Grid) {
            for (i, item) in self.items.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let x = rect.x + padding + col as f32 * (item_size + padding);
                let y = rect.y + padding + row as f32 * (item_size + padding);

                let is_selected = self.selected.as_deref() == Some(&item.id);
                let bg = if is_selected {
                    theme::list_item_selected()
                } else {
                    theme::input_bg()
                };
                renderer.fill_rounded_rect(
                    Rect {
                        x,
                        y,
                        width: item_size,
                        height: item_size,
                    },
                    6.0,
                    bg,
                );
                if is_selected {
                    renderer.stroke_rounded_rect(
                        Rect {
                            x,
                            y,
                            width: item_size,
                            height: item_size,
                        },
                        6.0,
                        theme::accent(),
                        2.0,
                    );
                }
                renderer.draw_text(&item.icon, x + 20.0, y + 10.0, 32.0, theme::info());
                renderer.draw_text(
                    &item.name,
                    x + 4.0,
                    y + item_size - 20.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        } else {
            let row_h = 32.0;
            for (i, item) in self.items.iter().enumerate() {
                let y = rect.y + i as f32 * row_h;
                let item_rect = Rect {
                    x: rect.x,
                    y,
                    width: rect.width,
                    height: row_h,
                };
                renderer.fill_rect(item_rect, theme::button_ghost_bg());
                renderer.draw_text(
                    &format!("{} {}", item.icon, item.name),
                    item_rect.x + 4.0,
                    item_rect.y + 10.0,
                    12.0,
                    theme::text(),
                );
            }
        }
    }
}

impl LayoutView for AssetBrowser {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let cols = 4usize;
        let rows = (self.items.len() as f32 / cols as f32).ceil();
        Size {
            width: 400.0,
            height: rows * 100.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
