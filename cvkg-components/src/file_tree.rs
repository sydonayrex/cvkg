use cvkg_core::{
Event, Never, Rect, Renderer, Size, SizeProposal, View};
use crate::theme;
use std::sync::Arc;

/// Represents the type of a file in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Folder,
    Text,
    Image,
    Audio,
    Video,
    Archive,
    Unknown,
}

/// A single item (file or folder) in the FileTree.
#[derive(Clone)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub kind: FileKind,
    pub children: Option<Vec<FileItem>>,
    pub is_expanded: bool,
    pub is_selected: bool,
}

/// A high-fidelity YggdrasilTree component for hierarchical data management.
/// Named after the World Tree, whose roots and branches connect all realms.
pub struct YggdrasilTree {
    pub items: Vec<FileItem>, // Keeping FileItem for now but can be generalized later
    pub on_toggle: Arc<dyn Fn(String) + Send + Sync>,
    pub on_select: Arc<dyn Fn(String) + Send + Sync>,
    pub on_hover: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    pub on_drag_start: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_drop: Option<Arc<dyn Fn(String, String) + Send + Sync>>,
}

impl YggdrasilTree {
    /// Creates a new YggdrasilTree.
    pub fn new(
        items: Vec<FileItem>,
        on_toggle: impl Fn(String) + Send + Sync + 'static,
        on_select: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            items,
            on_toggle: Arc::new(on_toggle),
            on_select: Arc::new(on_select),
            on_hover: None,
            on_drag_start: None,
            on_drop: None,
        }
    }

    pub fn with_hover(mut self, on_hover: impl Fn(Option<String>) + Send + Sync + 'static) -> Self {
        self.on_hover = Some(Arc::new(on_hover));
        self
    }

    pub fn with_drag_drop(
        mut self,
        on_drag_start: impl Fn(String) + Send + Sync + 'static,
        on_drop: impl Fn(String, String) + Send + Sync + 'static,
    ) -> Self {
        self.on_drag_start = Some(Arc::new(on_drag_start));
        self.on_drop = Some(Arc::new(on_drop));
        self
    }
}

impl View for YggdrasilTree {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "YggdrasilTree");
        let mut current_y = rect.y;
        let row_h = 28.0;
        let indent_w = 20.0;

        for item in &self.items {
            current_y = self.render_item(
                renderer, item, rect.x, current_y, rect.width, row_h, indent_w, 0,
            );
            if current_y > rect.y + rect.height {
                break;
            }
        }
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(300.0),
            height: proposal.height.unwrap_or(400.0),
        }
    }
}

impl YggdrasilTree {
    fn render_item(
        &self,
        renderer: &mut dyn Renderer,
        item: &FileItem,
        x: f32,
        y: f32,
        width: f32,
        row_h: f32,
        indent_w: f32,
        depth: usize,
    ) -> f32 {
        let item_x = x + (depth as f32 * indent_w);
        let row_rect = Rect {
            x,
            y,
            width,
            height: row_h,
        };

        renderer.push_vnode(row_rect, "YggdrasilTreeRow");
        renderer.set_key(&item.id);

        // 1. Selection Highlight
        if item.is_selected {
            renderer.fill_rect(row_rect, [0.0, 1.0, 1.0, 0.1]);
            renderer.stroke_rect(
                Rect {
                    x: row_rect.x,
                    y: row_rect.y,
                    width: 2.0,
                    height: row_rect.height,
                },
                theme::accent(),
                1.0,
            );
        }

        // 2. Expansion Indicator (Arrow)
        if item.children.is_some() {
            let arrow_rect = Rect {
                x: item_x + 4.0,
                y: y + 8.0,
                width: 12.0,
                height: 12.0,
            };
            renderer.push_vnode(arrow_rect, "ExpansionArrow");
            let color = theme::text_muted();
            if item.is_expanded {
                // Down arrow
                renderer.draw_line(
                    arrow_rect.x,
                    arrow_rect.y + 2.0,
                    arrow_rect.x + 6.0,
                    arrow_rect.y + 8.0,
                    color,
                    1.5,
                );
                renderer.draw_line(
                    arrow_rect.x + 6.0,
                    arrow_rect.y + 8.0,
                    arrow_rect.x + 12.0,
                    arrow_rect.y + 2.0,
                    color,
                    1.5,
                );
            } else {
                // Right arrow
                renderer.draw_line(
                    arrow_rect.x + 2.0,
                    arrow_rect.y,
                    arrow_rect.x + 8.0,
                    arrow_rect.y + 6.0,
                    color,
                    1.5,
                );
                renderer.draw_line(
                    arrow_rect.x + 8.0,
                    arrow_rect.y + 6.0,
                    arrow_rect.x + 2.0,
                    arrow_rect.y + 12.0,
                    color,
                    1.5,
                );
            }

            let id = item.id.clone();
            let on_toggle = self.on_toggle.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    on_toggle(id.clone());
                }),
            );
            renderer.pop_vnode();
        }

        // 3. Icon (Kind-specific)
        let icon_x = item_x + 20.0;
        let icon_color = match item.kind {
            FileKind::Folder => [0.0, 0.8, 1.0, 0.9],
            FileKind::Image => [0.9, 0.4, 0.8, 0.9],
            FileKind::Archive => [0.9, 0.8, 0.2, 0.9],
            _ => [0.7, 0.7, 0.8, 0.9],
        };
        renderer.fill_rect(
            Rect {
                x: icon_x,
                y: y + 6.0,
                width: 16.0,
                height: 16.0,
            },
            [icon_color[0], icon_color[1], icon_color[2], 0.2],
        );
        renderer.stroke_rect(
            Rect {
                x: icon_x,
                y: y + 6.0,
                width: 16.0,
                height: 16.0,
            },
            icon_color,
            1.0,
        );

        // 4. Label
        let label_color = if item.is_selected {
            theme::text()
        } else {
            [0.9, 0.9, 0.9, 0.8]
        };
        renderer.draw_text(&item.name, icon_x + 24.0, y + 19.0, 13.0, label_color);

        // 5. Interaction (Selection, Hover, Drag/Drop)
        {
            let id = item.id.clone();
            let on_select = self.on_select.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |ev| {
                    if let Event::PointerClick { .. } = ev {
                        on_select(id.clone());
                    }
                }),
            );

            if let Some(on_hover) = &self.on_hover {
                let id_enter = item.id.clone();
                let on_hover_enter = on_hover.clone();
                renderer.register_handler(
                    "pointerenter",
                    Arc::new(move |_| {
                        on_hover_enter(Some(id_enter.clone()));
                    }),
                );

                let on_hover_exit = on_hover.clone();
                renderer.register_handler(
                    "pointerexit",
                    Arc::new(move |_| {
                        on_hover_exit(None);
                    }),
                );
            }

            if let Some(on_drag) = &self.on_drag_start {
                let id_drag = item.id.clone();
                let on_drag_c = on_drag.clone();
                renderer.register_handler(
                    "pointerdown",
                    Arc::new(move |_| {
                        on_drag_c(id_drag.clone());
                    }),
                );
            }
        }

        renderer.pop_vnode(); // FileTreeRow

        let mut next_y = y + row_h;
        if item.is_expanded
            && let Some(children) = &item.children
        {
            for child in children {
                next_y = self.render_item(
                    renderer,
                    child,
                    x,
                    next_y,
                    width,
                    row_h,
                    indent_w,
                    depth + 1,
                );
            }
        }

        next_y
    }
}
