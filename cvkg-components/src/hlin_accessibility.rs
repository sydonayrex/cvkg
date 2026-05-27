//! Hlin Accessibility - Accessibility infrastructure components
//!
//! Hlin the Aesir goddess provides comfort and protection - this accessibility
//! system ensures inclusive design with keyboard nav, screen readers, and semantic support.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Accessibility node for tree structure
#[derive(Debug, Clone)]
pub struct A11yNode {
    pub id: String,
    pub role: A11yRole,
    pub label: String,
    pub description: String,
    pub children: Vec<A11yNode>,
}

/// Semantic accessibility roles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum A11yRole {
    Button,
    Link,
    Text,
    Image,
    Heading,
    List,
    ListItem,
    Form,
    Input,
    Navigation,
    Main,
    Banner,
    ContentInfo,
}

/// Focus management state
#[derive(Debug, Clone)]
pub struct FocusState {
    pub focused_id: Option<String>,
    pub focus_order: Vec<String>,
    pub trap_active: bool,
}

/// Hlin Accessibility Infrastructure for inclusive design
pub struct HlinAccessibility {
    pub tree: Vec<A11yNode>,
    pub focus_state: FocusState,
    pub high_contrast: bool,
    pub reduced_motion: bool,
}

impl Default for HlinAccessibility {
    fn default() -> Self {
        Self::new()
    }
}

impl HlinAccessibility {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            focus_state: FocusState {
                focused_id: None,
                focus_order: Vec::new(),
                trap_active: false,
            },
            high_contrast: false,
            reduced_motion: Self::detect_reduced_motion(),
        }
    }

    /// Detect OS-level reduced motion preference.
    /// Checks environment variables and platform-specific settings.
    fn detect_reduced_motion() -> bool {
        // Check GTK/GNOME setting
        if let Ok(val) = std::env::var("GTK_THEME")
            && (val.to_lowercase().contains("reduced") || val.to_lowercase().contains("no-anim"))
        {
            return true;
        }
        // Check GNOME accessibility setting via gsettings (if available)
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "enable-animations"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.trim() == "false" {
                return true;
            }
        }
        // Check macOS reduced motion
        if let Ok(val) = std::env::var("MACOS_REDUCED_MOTION")
            && (val == "1" || val.to_lowercase() == "true")
        {
            return true;
        }
        // Check Windows accessibility (via environment)
        if let Ok(val) = std::env::var("ACCESSIBILITY_REDUCED_MOTION")
            && (val == "1" || val.to_lowercase() == "true")
        {
            return true;
        }
        // Check generic NO_ANIMATIONS flag
        if let Ok(val) = std::env::var("NO_ANIMATIONS")
            && (val == "1" || val.to_lowercase() == "true")
        {
            return true;
        }
        false
    }

    /// Add an accessibility node
    pub fn node(mut self, id: &str, role: A11yRole, label: &str) -> Self {
        self.tree.push(A11yNode {
            id: id.to_string(),
            role,
            label: label.to_string(),
            description: String::new(),
            children: Vec::new(),
        });
        self.focus_state.focus_order.push(id.to_string());
        self
    }

    /// Add a child node to a parent by ID
    pub fn child(mut self, parent_id: &str, id: &str, role: A11yRole, label: &str) -> Self {
        let child = A11yNode {
            id: id.to_string(),
            role,
            label: label.to_string(),
            description: String::new(),
            children: Vec::new(),
        };
        if let Some(parent) = self.tree.iter_mut().find(|n| n.id == parent_id) {
            parent.children.push(child);
        } else {
            // Parent not found; add as top-level
            self.tree.push(child);
        }
        self.focus_state.focus_order.push(id.to_string());
        self
    }

    /// Add keyboard focus trap
    pub fn trap(mut self, active: bool) -> Self {
        self.focus_state.trap_active = active;
        self
    }

    /// Enable high contrast mode
    pub fn high_contrast(mut self, enabled: bool) -> Self {
        self.high_contrast = enabled;
        self
    }

    /// Enable reduced motion (overrides OS detection)
    pub fn reduced_motion(mut self, enabled: bool) -> Self {
        self.reduced_motion = enabled;
        self
    }

    /// Check if reduced motion is active (OS-detected or manually set)
    pub fn is_reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// Get the accessibility tree root nodes
    pub fn tree(&self) -> &[A11yNode] {
        &self.tree
    }

    /// Get the current focus state
    pub fn focus_state(&self) -> &FocusState {
        &self.focus_state
    }

    /// Set the currently focused element by ID
    pub fn set_focused(&mut self, id: Option<String>) {
        self.focus_state.focused_id = id;
    }

    /// Advance focus to the next element in focus order
    pub fn focus_next(&mut self) {
        if self.focus_state.focus_order.is_empty() {
            return;
        }
        let next = match &self.focus_state.focused_id {
            Some(current) => {
                if let Some(pos) = self
                    .focus_state
                    .focus_order
                    .iter()
                    .position(|id| id == current)
                {
                    let next_pos = (pos + 1) % self.focus_state.focus_order.len();
                    Some(self.focus_state.focus_order[next_pos].clone())
                } else {
                    Some(self.focus_state.focus_order[0].clone())
                }
            }
            None => Some(self.focus_state.focus_order[0].clone()),
        };
        self.focus_state.focused_id = next;
    }

    /// Advance focus to the previous element in focus order
    pub fn focus_prev(&mut self) {
        if self.focus_state.focus_order.is_empty() {
            return;
        }
        let prev = match &self.focus_state.focused_id {
            Some(current) => {
                if let Some(pos) = self
                    .focus_state
                    .focus_order
                    .iter()
                    .position(|id| id == current)
                {
                    let prev_pos = if pos == 0 {
                        self.focus_state.focus_order.len() - 1
                    } else {
                        pos - 1
                    };
                    Some(self.focus_state.focus_order[prev_pos].clone())
                } else {
                    Some(self.focus_state.focus_order[0].clone())
                }
            }
            None => Some(self.focus_state.focus_order[0].clone()),
        };
        self.focus_state.focused_id = prev;
    }
}

impl View for HlinAccessibility {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = if self.high_contrast {
            theme::bg()
        } else {
            theme::surface_elevated()
        };
        renderer.fill_rect(rect, bg);

        renderer.draw_text(
            "Hlin Accessibility",
            rect.x + 10.0,
            rect.y + 20.0,
            14.0,
            theme::success(),
        );

        // Status indicators
        let status_y = rect.y + 45.0;
        let hc_text = if self.high_contrast {
            "High Contrast: ON"
        } else {
            "High Contrast: OFF"
        };
        let rm_text = if self.reduced_motion {
            "Reduced Motion: ON"
        } else {
            "Reduced Motion: OFF"
        };

        renderer.draw_text(hc_text, rect.x + 10.0, status_y, 10.0, theme::success());
        renderer.draw_text(rm_text, rect.x + 130.0, status_y, 10.0, theme::success());

        // Focus indicator
        if let Some(focused) = &self.focus_state.focused_id {
            renderer.draw_text(
                &format!("Focused: {}", focused),
                rect.x + 10.0,
                status_y + 20.0,
                10.0,
                theme::text(),
            );
        }

        // Tree visualization
        let mut y = rect.y + 75.0;
        for node in &self.tree {
            let role_icon = match node.role {
                A11yRole::Button => "[B]",
                A11yRole::Link => "[L]",
                A11yRole::Text => "[T]",
                A11yRole::Image => "[I]",
                A11yRole::Heading => "[H]",
                A11yRole::Navigation => "[N]",
                _ => "[ ]",
            };
            renderer.draw_text(
                &format!("{} {} - {}", role_icon, node.id, node.label),
                rect.x + 15.0,
                y,
                9.0,
                theme::text(),
            );
            y += 18.0;
        }
    }
}

impl LayoutView for HlinAccessibility {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 280.0,
            height: 80.0 + self.tree.len() as f32 * 18.0,
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
