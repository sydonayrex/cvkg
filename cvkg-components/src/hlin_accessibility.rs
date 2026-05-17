//! Hlin Accessibility - Accessibility infrastructure components
//!
//! Hlin the Aesir goddess provides comfort and protection - this accessibility
//! system ensures inclusive design with keyboard nav, screen readers, and semantic support.

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
    pub(crate) tree: Vec<A11yNode>,
    pub(crate) focus_state: FocusState,
    pub(crate) high_contrast: bool,
    pub(crate) reduced_motion: bool,
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
            reduced_motion: false,
        }
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

    /// Enable reduced motion
    pub fn reduced_motion(mut self, enabled: bool) -> Self {
        self.reduced_motion = enabled;
        self
    }
}

impl View for HlinAccessibility {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = if self.high_contrast {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [0.08, 0.08, 0.12, 1.0]
        };
        renderer.fill_rect(rect, bg);

        renderer.draw_text(
            "Hlin Accessibility",
            rect.x + 10.0,
            rect.y + 20.0,
            14.0,
            [0.6, 0.9, 0.6, 1.0],
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

        renderer.draw_text(hc_text, rect.x + 10.0, status_y, 10.0, [0.7, 0.9, 0.7, 1.0]);
        renderer.draw_text(
            rm_text,
            rect.x + 130.0,
            status_y,
            10.0,
            [0.7, 0.9, 0.7, 1.0],
        );

        // Focus indicator
        if let Some(focused) = &self.focus_state.focused_id {
            renderer.draw_text(
                &format!("Focused: {}", focused),
                rect.x + 10.0,
                status_y + 20.0,
                10.0,
                [0.8, 0.9, 1.0, 1.0],
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
                [0.8, 0.9, 1.0, 1.0],
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
