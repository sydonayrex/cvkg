use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use cvkg_vdom::VDom;

/// Accessibility Inspector — live a11y tree viewer.
///
/// Displays the current accessibility tree structure, showing each
/// node's role, label, value, and state.
///
/// # OS-agnostic keyboard shortcut
/// Toggle with `cmd+shift+i` (Cmd+Shift+I on macOS, Ctrl+Shift+I on Windows/Linux).
pub struct A11yInspector {
    /// Whether the inspector panel is visible.
    pub visible: bool,
    /// Width of the inspector panel.
    pub panel_width: f32,
    selected_node: Option<usize>,
    nodes: Vec<A11yNode>,
}

/// A single node in the accessibility tree.
#[derive(Clone, Debug)]
pub struct A11yNode {
    /// AccessKit role name.
    pub role: String,
    /// Accessible label.
    pub label: String,
    /// Current value (e.g., "on" for toggles, "50%" for sliders).
    pub value: Option<String>,
    /// Whether the node is focused.
    pub focused: bool,
    /// Whether the node is enabled.
    pub enabled: bool,
    /// Tree depth (for indentation).
    pub depth: u32,
}

impl A11yNode {
    /// Create a new a11y tree node.
    pub fn new(role: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            label: label.into(),
            value: None,
            focused: false,
            enabled: true,
            depth: 0,
        }
    }

    /// Set the node value.
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = Some(val.into());
        self
    }

    /// Set whether the node is focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set whether the node is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the tree depth.
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

impl Default for A11yInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl A11yInspector {
    /// Create a new accessibility inspector with demo tree data.
    pub fn new() -> Self {
        let nodes = vec![
            A11yNode::new("Window", "CVKG Showcase").depth(0),
            A11yNode::new("MenuBar", "Main Menu").depth(1),
            A11yNode::new("Menu", "File").depth(2),
            A11yNode::new("MenuItem", "New").depth(3).value("Cmd+N"),
            A11yNode::new("MenuItem", "Open\u{2026}")
                .depth(3)
                .value("Cmd+O"),
            A11yNode::new("MenuItem", "Save").depth(3).value("Cmd+S"),
            A11yNode::new("MenuItem", "Close").depth(3).value("Cmd+W"),
            A11yNode::new("Menu", "Edit").depth(2),
            A11yNode::new("MenuItem", "Undo").depth(3).value("Cmd+Z"),
            A11yNode::new("MenuItem", "Redo")
                .depth(3)
                .value("Cmd+Shift+Z"),
            A11yNode::new("MenuItem", "Copy").depth(3).value("Cmd+C"),
            A11yNode::new("MenuItem", "Paste").depth(3).value("Cmd+V"),
            A11yNode::new("SplitView", "Main Content").depth(1),
            A11yNode::new("Sidebar", "Component List").depth(2),
            A11yNode::new("Button", "Buttons").depth(3).focused(true),
            A11yNode::new("Button", "Inputs").depth(3),
            A11yNode::new("Button", "Layout").depth(3),
            A11yNode::new("Button", "Overlays").depth(3),
            A11yNode::new("Button", "Visual").depth(3),
            A11yNode::new("Group", "Content Area").depth(2),
            A11yNode::new("Heading", "Buttons").depth(3),
            A11yNode::new("Button", "Default Button").depth(3),
            A11yNode::new("Button", "Destructive Button").depth(3),
            A11yNode::new("Button", "Secondary Button").depth(3),
            A11yNode::new("Slider", "Volume").depth(3).value("65%"),
            A11yNode::new("CheckBox", "Dark Mode").depth(3).value("on"),
            A11yNode::new("ProgressIndicator", "Loading")
                .depth(3)
                .value("65%"),
        ];

        Self {
            visible: false,
            panel_width: 320.0,
            selected_node: None,
            nodes,
        }
    }

    /// Refresh the inspector's tree from the real VDOM accessibility state.
    ///
    /// Walks the VDOM tree from the root, collecting all nodes with
    /// meaningful ARIA roles and converting them to `A11yNode` entries.
    /// Nodes with `"presentation"` or `"none"` roles are skipped.
    pub fn refresh_from_vdom(&mut self, vdom: &VDom) {
        let root = vdom.root;
        let entries = vdom.query_accessibility_tree(root);

        self.nodes = entries
            .into_iter()
            .map(|e| A11yNode {
                role: e.role,
                label: e.label,
                value: e.value,
                focused: e.focused,
                enabled: e.enabled,
                depth: e.depth,
            })
            .collect();
    }

    /// Show the inspector panel.
    pub fn show(mut self) -> Self {
        self.visible = true;
        self
    }

    /// Hide the inspector panel.
    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Toggle visibility.
    pub fn toggle(mut self) -> Self {
        self.visible = !self.visible;
        self
    }

    /// Set the panel width.
    pub fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width.clamp(200.0, 600.0);
        self
    }

    /// Get the total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count the number of distinct roles in the tree.
    pub fn role_count(&self) -> usize {
        use std::collections::HashSet;
        let mut roles = HashSet::new();
        for node in &self.nodes {
            roles.insert(&node.role);
        }
        roles.len()
    }
}

impl View for A11yInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.visible {
            return;
        }

        renderer.push_vnode(rect, "A11yInspector");

        let panel_rect = Rect {
            x: rect.x + rect.width - self.panel_width,
            y: rect.y,
            width: self.panel_width,
            height: rect.height,
        };

        // Panel background
        renderer.fill_rect(panel_rect, [0.06, 0.06, 0.1, 0.95]);
        renderer.stroke_rect(
            Rect {
                x: panel_rect.x,
                y: panel_rect.y,
                width: 1.0,
                height: panel_rect.height,
            },
            theme::border(),
            1.0,
        );

        let pad = 12.0;
        let mut y = panel_rect.y + pad;

        // Title
        renderer.draw_text(
            "Accessibility Inspector",
            panel_rect.x + pad,
            y,
            14.0,
            theme::text(),
        );
        y += 24.0;

        // Stats — use actual counts from the real tree
        renderer.draw_text(
            &format!("{} nodes | {} roles", self.node_count(), self.role_count()),
            panel_rect.x + pad,
            y,
            11.0,
            theme::text_dim(),
        );
        y += 20.0;

        // Divider
        renderer.draw_line(
            panel_rect.x + pad,
            y,
            panel_rect.x + self.panel_width - pad,
            y,
            theme::border(),
            1.0,
        );
        y += 8.0;

        // Tree nodes
        let item_h: f32 = 22.0;
        let scroll_h = panel_rect.height - y + panel_rect.y - 40.0;

        renderer.push_clip_rect(Rect {
            x: panel_rect.x,
            y,
            width: self.panel_width,
            height: scroll_h,
        });

        for (i, node) in self.nodes.iter().enumerate() {
            let item_y = y + i as f32 * item_h;
            if item_y > panel_rect.y + panel_rect.height {
                break;
            }

            let indent = node.depth as f32 * 16.0;
            let item_x = panel_rect.x + pad + indent;

            // Selection highlight
            if self.selected_node == Some(i) {
                renderer.fill_rect(
                    Rect {
                        x: panel_rect.x + 2.0,
                        y: item_y,
                        width: self.panel_width - 4.0,
                        height: item_h,
                    },
                    [
                        theme::accent()[0],
                        theme::accent()[1],
                        theme::accent()[2],
                        0.15,
                    ],
                );
            }

            // Focus indicator
            if node.focused {
                renderer.stroke_rect(
                    Rect {
                        x: panel_rect.x + 2.0,
                        y: item_y,
                        width: self.panel_width - 4.0,
                        height: item_h,
                    },
                    theme::accent(),
                    1.0,
                );
            }

            // Role color
            let role_color = match node.role.as_str() {
                "Button" | "MenuItem" => theme::accent(),
                "CheckBox" | "Radio" => theme::toast_success(),
                "Slider" | "ProgressIndicator" => theme::warning(),
                "Heading" => [1.0, 1.0, 1.0, 0.9],
                "Window" | "SplitView" => theme::text_dim(),
                _ => theme::text_dim(),
            };

            // Role label
            renderer.draw_text(&node.role, item_x, item_y + 3.0, 10.0, role_color);

            // Node label
            let label_x = item_x + 108.0;
            let _label_w = self.panel_width - pad * 2.0 - indent - 108.0;
            let label_color = if node.enabled {
                theme::text()
            } else {
                theme::text_dim()
            };
            renderer.draw_text(&node.label, label_x, item_y + 3.0, 11.0, label_color);

            // Value badge
            if let Some(ref val) = node.value {
                let val_w = val.len() as f32 * 7.0 + 8.0;
                let val_x = panel_rect.x + self.panel_width - pad - val_w;
                renderer.fill_rounded_rect(
                    Rect {
                        x: val_x,
                        y: item_y + 3.0,
                        width: val_w,
                        height: 16.0,
                    },
                    4.0,
                    theme::surface_elevated(),
                );
                renderer.draw_text(val, val_x + 4.0, item_y + 3.0, 9.0, theme::text_dim());
            }
        }

        renderer.pop_clip_rect();

        // Footer hint
        let footer_y = panel_rect.y + panel_rect.height - 28.0;
        renderer.draw_line(
            panel_rect.x + pad,
            footer_y,
            panel_rect.x + self.panel_width - pad,
            footer_y,
            theme::border(),
            1.0,
        );
        renderer.draw_text(
            "Cmd+Shift+I toggle  |  Up/Down navigate  |  Enter inspect",
            panel_rect.x + pad,
            footer_y + 6.0,
            10.0,
            theme::text_dim(),
        );

        renderer.pop_vnode();
    }
}
