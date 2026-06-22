use crate::vnode::{NodeId, VNode};
use crate::VDom;
use serde::{Deserialize, Serialize};

/// A single node in the accessibility tree, extracted from the VDOM.
///
/// Used by `A11yInspector` to display the real accessibility tree structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNodeEntry {
    /// ARIA role (e.g., "button", "group", "slider")
    pub role: String,
    /// Accessible label for the node
    pub label: String,
    /// Current value display (e.g., "65%" for sliders)
    pub value: Option<String>,
    /// Whether the node is currently focused
    pub focused: bool,
    /// Whether the node is enabled
    pub enabled: bool,
    /// Tree depth for indentation
    pub depth: u32,
}

impl VNode {
    /// Convert this VNode to an AccessKit node for accessibility tree generation.
    ///
    /// # Contract
    /// Maps the standard ARIA attributes (label, value, description, disabled, hidden, roles)
    /// into platform-neutral AccessKit layout-bounded descriptors.
    pub fn to_accesskit_node(&self) -> accesskit::Node {
        let mut node = accesskit::Node::new(match self.aria_role.as_str() {
            // All 53 AriaRole variants mapped to AccessKit Role equivalents
            "alert" => accesskit::Role::Alert,
            "alertdialog" => accesskit::Role::AlertDialog,
            "article" => accesskit::Role::Article,
            "banner" => accesskit::Role::Banner,
            "button" => accesskit::Role::Button,
            "checkbox" => accesskit::Role::CheckBox,
            "columnheader" => accesskit::Role::ColumnHeader,
            "combobox" => accesskit::Role::ComboBox,
            "complementary" => accesskit::Role::Complementary,
            "contentinfo" => accesskit::Role::ContentInfo,
            "dialog" => accesskit::Role::Dialog,
            "form" => accesskit::Role::Form,
            "grid" => accesskit::Role::Grid,
            "gridcell" => accesskit::Role::GridCell,
            "heading" => accesskit::Role::Heading,
            "img" => accesskit::Role::Image,
            "link" => accesskit::Role::Link,
            "list" => accesskit::Role::List,
            "listbox" => accesskit::Role::ListBox,
            "listitem" => accesskit::Role::ListItem,
            "main" => accesskit::Role::Main,
            "menu" => accesskit::Role::Menu,
            "menubar" => accesskit::Role::MenuBar,
            "menuitem" => accesskit::Role::MenuItem,
            "menuitemcheckbox" => accesskit::Role::MenuItemCheckBox,
            "menuitemradio" => accesskit::Role::MenuItemRadio,
            "navigation" => accesskit::Role::Navigation,
            "none" => accesskit::Role::GenericContainer,
            "note" => accesskit::Role::Note,
            "option" => accesskit::Role::ListBoxOption,
            "presentation" => accesskit::Role::GenericContainer,
            "progressbar" => accesskit::Role::ProgressIndicator,
            "radio" => accesskit::Role::RadioButton,
            "radiogroup" => accesskit::Role::RadioGroup,
            "region" => accesskit::Role::Region,
            "row" => accesskit::Role::Row,
            "rowgroup" => accesskit::Role::RowGroup,
            "rowheader" => accesskit::Role::RowHeader,
            "search" => accesskit::Role::Search,
            "separator" => accesskit::Role::Splitter,
            "slider" => accesskit::Role::Slider,
            "spinbutton" => accesskit::Role::SpinButton,
            "status" => accesskit::Role::Status,
            "switch" => accesskit::Role::Switch,
            "tab" => accesskit::Role::Tab,
            "table" => accesskit::Role::Table,
            "tablist" => accesskit::Role::TabList,
            "tabpanel" => accesskit::Role::TabPanel,
            "textbox" => accesskit::Role::TextInput,
            "toolbar" => accesskit::Role::Toolbar,
            "tooltip" => accesskit::Role::Tooltip,
            "tree" => accesskit::Role::Tree,
            "treeitem" => accesskit::Role::TreeItem,

            // Non-ARIA utility roles used by the codebase
            "text" => accesskit::Role::Label,
            "group" => accesskit::Role::Group,
            "window" => accesskit::Role::Window,
            "password" => accesskit::Role::TextInput,
            "application" => accesskit::Role::Application,
            "colorwell" => accesskit::Role::ColorWell,

            _ => accesskit::Role::Unknown,
        });

        if let Some(label) = &self.aria_props.label {
            node.set_label(label.clone());
        }

        if let Some(desc) = &self.aria_props.description {
            node.set_description(desc.clone());
        }

        if let Some(val) = &self.aria_props.value {
            node.set_value(val.clone());
        }

        // Expose ARIA slider/progress/meter numeric values to accesskit
        if let Some(now) = self.aria_props.aria_valuenow {
            node.set_numeric_value(now as f64);
        }
        if let Some(min) = self.aria_props.aria_valuemin {
            node.set_min_numeric_value(min as f64);
        }
        if let Some(max) = self.aria_props.aria_valuemax {
            node.set_max_numeric_value(max as f64);
        }

        if self.aria_props.disabled {
            node.set_disabled();
        }

        if self.aria_props.hidden {
            node.set_hidden();
        }

        node.set_bounds(accesskit::Rect {
            x0: self.layout.x as f64,
            y0: self.layout.y as f64,
            x1: (self.layout.x + self.layout.width) as f64,
            y1: (self.layout.y + self.layout.height) as f64,
        });

        node.set_children(
            self.children
                .iter()
                .map(|id| accesskit::NodeId(id.0))
                .collect::<Vec<_>>(),
        );

        node
    }
}

impl VDom {
    /// Query the accessibility tree from the VDOM.
    ///
    /// Traverses the VDOM tree from the root, collecting all nodes with
    /// ARIA roles and labels into a flat list suitable for display in
    /// the A11yInspector.
    pub fn query_accessibility_tree(
        &self,
        root: Option<NodeId>,
    ) -> Vec<A11yNodeEntry> {
        let mut result = Vec::new();
        if let Some(root_id) = root {
            self.collect_a11y_nodes(root_id, 0, &mut result);
        }
        result
    }

    /// Recursively collect A11y nodes from the VDOM tree.
    fn collect_a11y_nodes(
        &self,
        id: NodeId,
        depth: u32,
        result: &mut Vec<A11yNodeEntry>,
    ) {
        if let Some(node) = self.nodes.get(&id) {
            // Only include nodes that have meaningful ARIA roles
            // (skip "presentation" and "none" which are structural only)
            if node.aria_role != "presentation" && node.aria_role != "none" {
                let label = node.aria_props.label.clone().unwrap_or_default();
                let value = node.aria_props.aria_valuenow.map(|v| {
                    let min = node.aria_props.aria_valuemin.unwrap_or(0.0);
                    let max = node.aria_props.aria_valuemax.unwrap_or(100.0);
                    let pct = if max > min {
                        ((v - min) / (max - min) * 100.0).round() as u32
                    } else {
                        0
                    };
                    format!("{}%", pct)
                });

                let focused = *self
                    .focused_node
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    == Some(id);

                result.push(A11yNodeEntry {
                    role: node.aria_role.clone(),
                    label,
                    value,
                    focused,
                    enabled: !node.aria_props.disabled,
                    depth,
                });
            }

            // Recurse into children
            for child_id in &node.children {
                self.collect_a11y_nodes(*child_id, depth + 1, result);
            }
        }
    }

    /// Build a complete AccessKit tree update from the current VDOM state.
    ///
    /// Generates a `TreeUpdate` containing all nodes with proper parent-child
    /// relationships, roles, labels, descriptions, bounds, and states.
    pub fn build_accesskit_tree(&self) -> accesskit::TreeUpdate {
        let mut nodes: Vec<(accesskit::NodeId, accesskit::Node)> = Vec::new();

        if let Some(root_id) = self.root {
            self.build_accesskit_node(root_id, &mut nodes);
        }

        accesskit::TreeUpdate {
            nodes,
            tree: Some(accesskit::Tree::new(accesskit::NodeId(0))),
            focus: self
                .focused_node
                .lock()
                .ok()
                .and_then(|f| *f)
                .map(|id| accesskit::NodeId(id.0))
                .unwrap_or(accesskit::NodeId(0)),
            tree_id: accesskit::TreeId::ROOT,
        }
    }

    fn build_accesskit_node(
        &self,
        node_id: NodeId,
        output: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) {
        if let Some(node) = self.nodes.get(&node_id) {
            let mut ak_node = accesskit::Node::new(match node.aria_role.as_str() {
                // All 53 AriaRole variants mapped to AccessKit Role equivalents
                "alert" => accesskit::Role::Alert,
                "alertdialog" => accesskit::Role::AlertDialog,
                "article" => accesskit::Role::Article,
                "banner" => accesskit::Role::Banner,
                "button" => accesskit::Role::Button,
                "checkbox" => accesskit::Role::CheckBox,
                "columnheader" => accesskit::Role::ColumnHeader,
                "combobox" => accesskit::Role::ComboBox,
                "complementary" => accesskit::Role::Complementary,
                "contentinfo" => accesskit::Role::ContentInfo,
                "dialog" => accesskit::Role::Dialog,
                "form" => accesskit::Role::Form,
                "grid" => accesskit::Role::Grid,
                "gridcell" => accesskit::Role::GridCell,
                "heading" => accesskit::Role::Heading,
                "img" => accesskit::Role::Image,
                "link" => accesskit::Role::Link,
                "list" => accesskit::Role::List,
                "listbox" => accesskit::Role::ListBox,
                "listitem" => accesskit::Role::ListItem,
                "main" => accesskit::Role::Main,
                "menu" => accesskit::Role::Menu,
                "menubar" => accesskit::Role::MenuBar,
                "menuitem" => accesskit::Role::MenuItem,
                "menuitemcheckbox" => accesskit::Role::MenuItemCheckBox,
                "menuitemradio" => accesskit::Role::MenuItemRadio,
                "navigation" => accesskit::Role::Navigation,
                "none" => accesskit::Role::GenericContainer,
                "note" => accesskit::Role::Note,
                "option" => accesskit::Role::ListBoxOption,
                "presentation" => accesskit::Role::GenericContainer,
                "progressbar" => accesskit::Role::ProgressIndicator,
                "radio" => accesskit::Role::RadioButton,
                "radiogroup" => accesskit::Role::RadioGroup,
                "region" => accesskit::Role::Region,
                "row" => accesskit::Role::Row,
                "rowgroup" => accesskit::Role::RowGroup,
                "rowheader" => accesskit::Role::RowHeader,
                "search" => accesskit::Role::Search,
                "separator" => accesskit::Role::Splitter,
                "slider" => accesskit::Role::Slider,
                "spinbutton" => accesskit::Role::SpinButton,
                "status" => accesskit::Role::Status,
                "switch" => accesskit::Role::Switch,
                "tab" => accesskit::Role::Tab,
                "table" => accesskit::Role::Table,
                "tablist" => accesskit::Role::TabList,
                "tabpanel" => accesskit::Role::TabPanel,
                "textbox" => accesskit::Role::TextInput,
                "toolbar" => accesskit::Role::Toolbar,
                "tooltip" => accesskit::Role::Tooltip,
                "tree" => accesskit::Role::Tree,
                "treeitem" => accesskit::Role::TreeItem,

                // Non-ARIA utility roles used by the codebase
                "text" => accesskit::Role::Label,
                "group" => accesskit::Role::Group,
                "window" => accesskit::Role::Window,
                "password" => accesskit::Role::TextInput,
                "application" => accesskit::Role::Application,
                "colorwell" => accesskit::Role::ColorWell,

                _ => accesskit::Role::Unknown,
            });

            if let Some(label) = &node.aria_props.label {
                ak_node.set_label(label.clone());
            }

            if let Some(desc) = &node.aria_props.description {
                ak_node.set_description(desc.clone());
            }

            if let Some(val) = &node.aria_props.value {
                ak_node.set_value(val.clone());
            }

            // Expose ARIA slider/progress/meter numeric values to accesskit
            if let Some(now) = node.aria_props.aria_valuenow {
                ak_node.set_numeric_value(now as f64);
            }
            if let Some(min) = node.aria_props.aria_valuemin {
                ak_node.set_min_numeric_value(min as f64);
            }
            if let Some(max) = node.aria_props.aria_valuemax {
                ak_node.set_max_numeric_value(max as f64);
            }

            if node.aria_props.disabled {
                ak_node.set_disabled();
            }

            if node.aria_props.hidden {
                ak_node.set_hidden();
            }

            ak_node.set_bounds(accesskit::Rect {
                x0: node.layout.x as f64,
                y0: node.layout.y as f64,
                x1: (node.layout.x + node.layout.width) as f64,
                y1: (node.layout.y + node.layout.height) as f64,
            });

            let child_ids: Vec<accesskit::NodeId> = node
                .children
                .iter()
                .map(|id| accesskit::NodeId(id.0))
                .collect();
            ak_node.set_children(child_ids);

            output.push((accesskit::NodeId(node_id.0), ak_node));

            for child_id in &node.children {
                self.build_accesskit_node(*child_id, output);
            }
        }
    }
}
