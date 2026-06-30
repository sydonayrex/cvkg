use crate::theme;
use cvkg_core::{
    Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

/// Node structure for RichTreeView hierarchical item.
#[derive(Debug, Clone)]
pub struct TreeViewNode {
    /// Label text.
    pub label: String,
    /// Icon glyph representation.
    pub icon: Option<String>,
    /// Children nested nodes.
    pub children: Vec<TreeViewNode>,
    /// Whether the node is currently expanded.
    pub is_expanded: bool,
    /// Selection highlight.
    pub is_selected: bool,
}

impl TreeViewNode {
    /// Create a new leaf or folder tree view node.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: Vec::new(),
            is_expanded: false,
            is_selected: false,
        }
    }

    /// Add an icon to this node.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Add a child node.
    pub fn child(mut self, child: TreeViewNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set default expanded state.
    pub fn expanded(mut self, exp: bool) -> Self {
        self.is_expanded = exp;
        self
    }

    /// Set default selected state.
    pub fn selected(mut self, sel: bool) -> Self {
        self.is_selected = sel;
        self
    }
}

/// RichTreeView component presenting hierarchical tree data.
///
/// # Contract
/// - Nodes can be recursively nested.
/// - Expansion toggle adjusts vertical spacing accordingly.
pub struct RichTreeView {
    pub(crate) root_nodes: Vec<TreeViewNode>,
}

impl Default for RichTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl RichTreeView {
    /// Create a new empty RichTreeView.
    pub fn new() -> Self {
        Self {
            root_nodes: Vec::new(),
        }
    }

    /// Add root-level nodes.
    pub fn roots(mut self, roots: Vec<TreeViewNode>) -> Self {
        self.root_nodes = roots;
        self
    }
}

impl View for RichTreeView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 6.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 6.0, theme::border(), 1.0);

        let mut current_y = rect.y + 10.0;
        let indent_w = 16.0;
        let row_h = 24.0;

        fn draw_node(
            renderer: &mut dyn Renderer,
            node: &TreeViewNode,
            rect: &Rect,
            y: &mut f32,
            indent: f32,
            indent_w: f32,
            row_h: f32,
        ) {
            if *y + row_h > rect.y + rect.height {
                return;
            }

            let row_rect = Rect {
                x: rect.x + 4.0,
                y: *y,
                width: rect.width - 8.0,
                height: row_h,
            };

            // Selection background
            if node.is_selected {
                renderer.fill_rounded_rect(row_rect, 4.0, theme::accent());
            }

            let text_color = if node.is_selected {
                theme::with_alpha(theme::text(), 0.95)
            } else {
                theme::text()
            };

            // Expansion arrow placeholder
            if !node.children.is_empty() {
                let arrow = if node.is_expanded { "▼" } else { "▶" };
                renderer.draw_text_raw(
                    arrow,
                    rect.x + indent + 8.0,
                    *y + row_h / 2.0 - 4.0,
                    10.0,
                    text_color,
                );
            }

            let offset_x = if !node.children.is_empty() { 16.0 } else { 8.0 };

            // Optional Icon
            let icon_w = if let Some(ref icon) = node.icon {
                renderer.draw_text_raw(
                    icon,
                    rect.x + indent + offset_x + 4.0,
                    *y + row_h / 2.0 - 4.0,
                    12.0,
                    text_color,
                );
                16.0
            } else {
                0.0
            };

            // Label text
            renderer.draw_text_raw(
                &node.label,
                rect.x + indent + offset_x + icon_w + 8.0,
                *y + row_h / 2.0 - 4.0,
                11.0,
                text_color,
            );

            *y += row_h + 2.0;

            // Render children recursively if expanded
            if node.is_expanded {
                for child in &node.children {
                    draw_node(renderer, child, rect, y, indent + indent_w, indent_w, row_h);
                }
            }
        }

        for node in &self.root_nodes {
            draw_node(renderer, node, &rect, &mut current_y, 0.0, indent_w, row_h);
        }

        // Keyboard: Arrow keys to navigate, Enter to expand/collapse
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Enter" | " " => {
                            // Tree navigation handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );
    }
}

impl LayoutView for RichTreeView {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        fn count_expanded_nodes(node: &TreeViewNode) -> usize {
            let mut count = 1;
            if node.is_expanded {
                for child in &node.children {
                    count += count_expanded_nodes(child);
                }
            }
            count
        }

        let total_nodes: usize = self.root_nodes.iter().map(count_expanded_nodes).sum();
        let height = 20.0 + (total_nodes as f32 * 26.0);
        Size {
            width: 250.0,
            height: height.max(150.0),
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
