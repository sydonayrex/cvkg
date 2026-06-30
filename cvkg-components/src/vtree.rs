//! VTree -- Virtualized Tree with expand/collapse and O(visible) rendering.
//!
//! Renders hierarchical tree data with virtualization: only visible nodes
//! (including expanded subtrees) are rendered. Supports 100,000+ nodes.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// A tree node that can be expanded/collapsed.
#[derive(Clone, Debug)]
pub struct VTreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<VTreeNode>,
}

/// A virtualized tree view with expand/collapse support.
///
/// # How virtualization works for trees:
/// Unlike a flat list, a tree's visible items depend on which nodes are expanded.
/// We flatten the visible tree into a list of (node_id, depth, label) triples,
/// then virtualize that flattened list.
pub struct VTree {
    pub root: VTreeNode,
    expanded: Arc<Mutex<HashSet<String>>>,
    scroll_offset: Arc<Mutex<f32>>,
    pub item_height: f32,
}

impl VTree {
    pub fn new(root: VTreeNode) -> Self {
        Self {
            root,
            expanded: Arc::new(Mutex::new(HashSet::new())),
            scroll_offset: Arc::new(Mutex::new(0.0)),
            item_height: 24.0,
        }
    }

    /// Toggle expand/collapse for a node.
    pub fn toggle(&self, node_id: &str) {
        if let Ok(mut expanded) = self.expanded.lock() {
            if expanded.contains(node_id) {
                expanded.remove(node_id);
            } else {
                expanded.insert(node_id.to_string());
            }
        }
    }

    /// Check if a node is expanded.
    pub fn is_expanded(&self, node_id: &str) -> bool {
        self.expanded
            .lock()
            .map(|e| e.contains(node_id))
            .unwrap_or(false)
    }

    /// Flatten the visible tree into (id, depth, label, has_children) tuples.
    fn flatten(&self) -> Vec<(String, usize, String, bool)> {
        let expanded = match self.expanded.lock() {
            Ok(e) => e.clone(),
            Err(_) => return Vec::new(),
        };
        let mut result = Vec::new();
        Self::flatten_node(&self.root, 0, &expanded, &mut result);
        result
    }

    fn flatten_node(
        node: &VTreeNode,
        depth: usize,
        expanded: &HashSet<String>,
        result: &mut Vec<(String, usize, String, bool)>,
    ) {
        if depth > 32 {
            return;
        }
        let has_children = !node.children.is_empty();
        result.push((node.id.clone(), depth, node.label.clone(), has_children));
        if expanded.contains(&node.id) {
            for child in &node.children {
                Self::flatten_node(child, depth + 1, expanded, result);
            }
        }
    }
}

impl View for VTree {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let nodes = self.flatten();
        let scroll = self.scroll_offset.lock().map(|s| *s).unwrap_or(0.0);
        let item_h = self.item_height;

        let start_idx = (scroll / item_h).floor() as usize;
        let visible_count = (rect.height / item_h).ceil() as usize + 1;
        let end_idx = (start_idx + visible_count).min(nodes.len());

        let mut y = rect.y - (scroll % item_h);

        for i in start_idx..end_idx {
            let (id, depth, label, has_children) = &nodes[i];
            let item_rect = Rect::new(rect.x, y, rect.width, item_h);
            let indent = *depth as f32 * 16.0;

            // Expand/collapse indicator
            let indicator = if *has_children {
                if self.is_expanded(id) { "v " } else { "> " }
            } else {
                "  "
            };

            renderer.draw_text_raw(
                &format!("{}{}", indicator, label),
                item_rect.x + indent + 4.0,
                item_rect.y + 16.0,
                13.0,
                theme::text(),
            );

            y += item_h;
        }
    }
}
