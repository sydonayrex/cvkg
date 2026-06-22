use crate::theme;
use crate::{RADIUS_MD, RADIUS_SM};
use cvkg_core::{Event, Never, Rect, Renderer, Size, SizeProposal, View};
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

/// Internal UI state for the YggdrasilTree components.
#[derive(Clone, Debug, Default)]
pub struct YggdrasilTreeState {
    /// ID of the item currently being inline renamed.
    pub active_rename_id: Option<String>,
    /// Buffer text for the active inline rename.
    pub rename_text: String,
    /// ID of the item for which the context menu is open.
    pub context_menu_item_id: Option<String>,
    /// Coordinates [x, y] where the context menu is open.
    pub context_menu_pos: Option<[f32; 2]>,
    /// ID of the last clicked item (for Shift range selection).
    pub last_clicked_id: Option<String>,
    /// Timestamp of the last click (for double-click detection).
    pub last_click_time: f32,
}

/// A high-fidelity YggdrasilTree component for hierarchical data management.
/// Named after the World Tree, whose roots and branches connect all realms.
#[doc(alias = "TreeView")]
pub struct YggdrasilTree {
    pub items: Vec<FileItem>,
    pub on_toggle: Arc<dyn Fn(String) + Send + Sync>,
    pub on_select: Arc<dyn Fn(String) + Send + Sync>,
    pub on_hover: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    pub on_drag_start: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_drop: Option<Arc<dyn Fn(String, String) + Send + Sync>>,
    pub on_rename: Option<Arc<dyn Fn(String, String) + Send + Sync>>,
    pub on_delete: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_open: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub on_copy_path: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl YggdrasilTree {
    /// Creates a new YggdrasilTree with basic selection and toggle callbacks.
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
            on_rename: None,
            on_delete: None,
            on_open: None,
            on_copy_path: None,
        }
    }

    /// Builder method to supply hover callback.
    pub fn with_hover(mut self, on_hover: impl Fn(Option<String>) + Send + Sync + 'static) -> Self {
        self.on_hover = Some(Arc::new(on_hover));
        self
    }

    /// Builder method to supply drag-and-drop callbacks.
    pub fn with_drag_drop(
        mut self,
        on_drag_start: impl Fn(String) + Send + Sync + 'static,
        on_drop: impl Fn(String, String) + Send + Sync + 'static,
    ) -> Self {
        self.on_drag_start = Some(Arc::new(on_drag_start));
        self.on_drop = Some(Arc::new(on_drop));
        self
    }

    /// Builder method to supply rename callback.
    pub fn with_rename(
        mut self,
        on_rename: impl Fn(String, String) + Send + Sync + 'static,
    ) -> Self {
        self.on_rename = Some(Arc::new(on_rename));
        self
    }

    /// Builder method to supply delete callback.
    pub fn with_delete(mut self, on_delete: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_delete = Some(Arc::new(on_delete));
        self
    }

    /// Builder method to supply open callback.
    pub fn with_open(mut self, on_open: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_open = Some(Arc::new(on_open));
        self
    }

    /// Builder method to supply copy path callback.
    pub fn with_copy_path(mut self, on_copy_path: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_copy_path = Some(Arc::new(on_copy_path));
        self
    }
}

/// Get tree state from global component storage.
fn get_tree_state(tree_id: u64) -> YggdrasilTreeState {
    let s = cvkg_core::load_system_state();
    if let Some(state_arc) = s.get_component_state::<YggdrasilTreeState>(tree_id) {
        state_arc.read().ok().map(|g| g.clone()).unwrap_or_default()
    } else {
        YggdrasilTreeState::default()
    }
}

/// Update tree state inside global component storage.
fn update_tree_state<F>(tree_id: u64, f: F)
where
    F: FnOnce(&mut YggdrasilTreeState),
{
    let s = cvkg_core::load_system_state();
    if s.get_component_state::<YggdrasilTreeState>(tree_id)
        .is_none()
    {
        cvkg_core::update_system_state(|st| {
            let mut new_st = st.clone();
            new_st.set_component_state(tree_id, YggdrasilTreeState::default());
            new_st
        });
    }
    let s = cvkg_core::load_system_state();
    if let Some(state_arc) = s.get_component_state::<YggdrasilTreeState>(tree_id) {
        let mut lock = state_arc.write().unwrap_or_else(|e| {
            log::warn!("Lock poisoned, recovering...");
            e.into_inner()
        });
        f(&mut lock);
    }
}

/// Helper to flat-list all visible items for index-based selection range resolution.
fn collect_visible_items(items: &[FileItem], list: &mut Vec<String>) {
    for item in items {
        list.push(item.id.clone());
        if item.is_expanded
            && let Some(children) = &item.children
        {
            collect_visible_items(children, list);
        }
    }
}

/// Detect kind based on file extension fallback.
fn resolve_kind(name: &str, current_kind: FileKind) -> FileKind {
    if current_kind == FileKind::Folder {
        return FileKind::Folder;
    }
    let ext = name.split('.').next_back().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" => FileKind::Image,
        "txt" | "rs" | "json" | "toml" | "md" | "js" | "ts" | "css" | "html" => FileKind::Text,
        "mp3" | "wav" | "ogg" | "flac" => FileKind::Audio,
        "mp4" | "mkv" | "avi" | "mov" => FileKind::Video,
        "zip" | "tar" | "gz" | "rar" | "7z" => FileKind::Archive,
        _ => current_kind,
    }
}

impl View for YggdrasilTree {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    /// Renders the tree hierarchy, context menus, and handles click-outside closing events.
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "YggdrasilTree");

        // Close context menu if clicked outside
        let tree_id = 77777;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, button, .. } = event
                    && button != 2
                {
                    let state = get_tree_state(tree_id);
                    if let Some(pos) = state.context_menu_pos {
                        let menu_rect = Rect {
                            x: pos[0],
                            y: pos[1],
                            width: 120.0,
                            height: 96.0,
                        };
                        if !menu_rect.contains(x, y) {
                            update_tree_state(tree_id, |st| {
                                st.context_menu_item_id = None;
                                st.context_menu_pos = None;
                            });
                        }
                    }
                }
            }),
        );

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

        // Draw Context Menu if open
        let state = get_tree_state(tree_id);
        if let (Some(menu_id), Some(pos)) = (&state.context_menu_item_id, &state.context_menu_pos) {
            let menu_width = 120.0;
            let menu_height = 96.0;
            let menu_rect = Rect {
                x: pos[0],
                y: pos[1],
                width: menu_width,
                height: menu_height,
            };

            renderer.push_vnode(menu_rect, "ContextMenu");
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(menu_rect, 15.0, 1.5, 0.95);
            }
            renderer.fill_rounded_rect(menu_rect, RADIUS_MD, theme::with_alpha(theme::surface_elevated(), 0.9));
            renderer.stroke_rounded_rect(menu_rect, RADIUS_MD, theme::border(), 1.0);

            let options = ["Open", "Rename", "Delete", "Copy Path"];
            for (idx, opt) in options.iter().enumerate() {
                let opt_rect = Rect {
                    x: menu_rect.x,
                    y: menu_rect.y + 4.0 + (idx as f32 * 22.0),
                    width: menu_width,
                    height: 22.0,
                };

                renderer.draw_text(
                    opt,
                    opt_rect.x + 12.0,
                    opt_rect.y + 15.0,
                    11.0,
                    theme::text(),
                );

                let opt_str = opt.to_string();
                let item_id = menu_id.clone();
                let on_open = self.on_open.clone();
                let _on_rename = self.on_rename.clone();
                let on_delete = self.on_delete.clone();
                let on_copy_path = self.on_copy_path.clone();
                let cur_item = self.items.iter().find(|i| i.id == item_id).cloned();

                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |_| {
                        match opt_str.as_str() {
                            "Open" => {
                                if let Some(cb) = &on_open {
                                    cb(item_id.clone());
                                }
                            }
                            "Rename" => {
                                if let Some(item) = &cur_item {
                                    update_tree_state(tree_id, |st| {
                                        st.active_rename_id = Some(item_id.clone());
                                        st.rename_text = item.name.clone();
                                    });
                                }
                            }
                            "Delete" => {
                                if let Some(cb) = &on_delete {
                                    cb(item_id.clone());
                                }
                            }
                            "Copy Path" => {
                                if let Some(cb) = &on_copy_path {
                                    cb(item_id.clone());
                                }
                            }
                            _ => {}
                        }
                        update_tree_state(tree_id, |st| {
                            st.context_menu_item_id = None;
                            st.context_menu_pos = None;
                        });
                    }),
                );
            }
            renderer.pop_vnode();
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

        let tree_id = 77777;
        let state = get_tree_state(tree_id);
        let is_renaming = state.active_rename_id.as_ref() == Some(&item.id);

        // 1. Selection Highlight
        if item.is_selected {
            renderer.fill_rect(row_rect, theme::with_alpha(theme::accent(), 0.1));
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

        // 3. Icon (Kind-specific resolved)
        let resolved_kind = resolve_kind(&item.name, item.kind);
        let icon_x = item_x + 20.0;
        let icon_color = match resolved_kind {
            FileKind::Folder => theme::accent(),
            FileKind::Image => theme::secondary(),
            FileKind::Text => theme::success(),
            FileKind::Audio => theme::warning(),
            FileKind::Video => theme::secondary(),
            FileKind::Archive => theme::viking_gold(),
            _ => theme::text_muted(),
        };

        let icon_rect = Rect {
            x: icon_x,
            y: y + 6.0,
            width: 16.0,
            height: 16.0,
        };
        renderer.fill_rounded_rect(
            icon_rect,
            RADIUS_SM,
            [icon_color[0], icon_color[1], icon_color[2], 0.15],
        );
        renderer.stroke_rounded_rect(icon_rect, RADIUS_SM, icon_color, 1.0);

        if resolved_kind == FileKind::Folder {
            renderer.fill_rounded_rect(
                Rect {
                    x: icon_rect.x + 2.0,
                    y: icon_rect.y - 2.0,
                    width: 6.0,
                    height: 3.0,
                },
                1.0,
                icon_color,
            );
        } else {
            renderer.draw_line(
                icon_rect.x + 4.0,
                icon_rect.y + 5.0,
                icon_rect.x + 12.0,
                icon_rect.y + 5.0,
                icon_color,
                1.0,
            );
            renderer.draw_line(
                icon_rect.x + 4.0,
                icon_rect.y + 8.0,
                icon_rect.x + 10.0,
                icon_rect.y + 8.0,
                icon_color,
                1.0,
            );
            renderer.draw_line(
                icon_rect.x + 4.0,
                icon_rect.y + 11.0,
                icon_rect.x + 8.0,
                icon_rect.y + 11.0,
                icon_color,
                1.0,
            );
        }

        // 4. Label / Inline Rename Input
        let text_x = icon_x + 24.0;
        if is_renaming {
            let input_rect = Rect {
                x: text_x,
                y: y + 4.0,
                width: width - (text_x - x) - 8.0,
                height: row_h - 8.0,
            };
            renderer.fill_rounded_rect(input_rect, RADIUS_SM, theme::with_alpha(theme::surface_elevated(), 0.85));
            renderer.stroke_rounded_rect(input_rect, RADIUS_SM, theme::accent(), 1.0);
            renderer.draw_text(
                &state.rename_text,
                input_rect.x + 6.0,
                y + 17.0,
                12.0,
                theme::text(),
            );

            // Inline key listener to append text and handle Enter/Escape
            let rename_id = item.id.clone();
            let current_rename_text = state.rename_text.clone();
            let on_rename_c = self.on_rename.clone();
            renderer.register_handler(
                "keydown",
                Arc::new(move |event| {
                    if let Event::KeyDown { key, .. } = event {
                        if key == "Enter" || key == "Return" {
                            if let Some(cb) = &on_rename_c {
                                cb(rename_id.clone(), current_rename_text.clone());
                            }
                            update_tree_state(tree_id, |st| {
                                st.active_rename_id = None;
                            });
                        } else if key == "Escape" {
                            update_tree_state(tree_id, |st| {
                                st.active_rename_id = None;
                            });
                        } else if key == "Backspace" {
                            update_tree_state(tree_id, |st| {
                                st.rename_text.pop();
                            });
                        } else if key.len() == 1 {
                            update_tree_state(tree_id, |st| {
                                st.rename_text.push_str(&key);
                            });
                        }
                    }
                }),
            );
        } else {
            let label_color = if item.is_selected {
                theme::text()
            } else {
                theme::with_alpha(theme::text(), 0.8)
            };
            renderer.draw_text(&item.name, text_x, y + 19.0, 13.0, label_color);
        }

        // 5. Interaction (Selection, Context Menu, Drag & Drop)
        {
            let id = item.id.clone();
            let on_select = self.on_select.clone();
            let on_open_clone = self.on_open.clone();
            let tree_items = self.items.clone();
            let item_name = item.name.clone();
            let is_item_selected = item.is_selected;

            let now = renderer.elapsed_time();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |ev| {
                    if let Event::PointerClick { x, y, button, .. } = ev {
                        let state = get_tree_state(tree_id);

                        if button == 2 {
                            // Right click: Trigger Context Menu
                            update_tree_state(tree_id, |st| {
                                st.context_menu_item_id = Some(id.clone());
                                st.context_menu_pos = Some([x, y]);
                            });
                        } else if button == 0 {
                            let is_double_click = state.last_clicked_id.as_ref() == Some(&id)
                                && (now - state.last_click_time) < 0.35;
                            let is_slow_double_click = state.last_clicked_id.as_ref() == Some(&id)
                                && is_item_selected
                                && (now - state.last_click_time) >= 0.35
                                && (now - state.last_click_time) < 1.5;

                            if is_double_click {
                                if let Some(on_open) = &on_open_clone {
                                    on_open(id.clone());
                                }
                            } else if is_slow_double_click {
                                update_tree_state(tree_id, |st| {
                                    st.active_rename_id = Some(id.clone());
                                    st.rename_text = item_name.clone();
                                });
                            } else {
                                let sys_state = cvkg_core::load_system_state();
                                if sys_state.modifiers_logo || sys_state.modifiers_ctrl {
                                    on_select(id.clone());
                                } else if sys_state.modifiers_shift
                                    && state.last_clicked_id.is_some()
                                {
                                    let mut flat_list = Vec::new();
                                    collect_visible_items(&tree_items, &mut flat_list);
                                    if let (Some(idx1), Some(idx2)) = (
                                        flat_list.iter().position(|x| {
                                            Some(x) == state.last_clicked_id.as_ref()
                                        }),
                                        flat_list.iter().position(|x| x == &id),
                                    ) {
                                        let start = idx1.min(idx2);
                                        let end = idx1.max(idx2);
                                        for i in start..=end {
                                            on_select(flat_list[i].clone());
                                        }
                                    }
                                } else {
                                    on_select(id.clone());
                                }
                            }

                            update_tree_state(tree_id, |st| {
                                st.last_clicked_id = Some(id.clone());
                                st.last_click_time = now;
                            });
                        }
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
