use cvkg_core::{
    AnyView, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// A docking workspace with sidebar navigation, split panes, and floating panels.
pub struct DockingWorkspace {
    pub(crate) sidebar_width: f32,
    pub(crate) header_height: f32,
    pub(crate) panels: Vec<Panel>,
    pub(crate) active_panel: Option<usize>,
}

pub struct Panel {
    pub id: String,
    pub title: String,
    pub content: AnyView,
    pub is_floating: bool,
    pub is_docked: bool,
    pub position: PanelPosition,
}

pub enum PanelPosition {
    Left,
    Right,
    Bottom,
    Center,
    Floating {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

impl Default for DockingWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl DockingWorkspace {
    pub fn new() -> Self {
        Self {
            sidebar_width: 200.0,
            header_height: 40.0,
            panels: Vec::new(),
            active_panel: None,
        }
    }

    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width;
        self
    }

    pub fn panel(
        mut self,
        id: &str,
        title: &str,
        content: impl View + Clone + 'static,
        position: PanelPosition,
    ) -> Self {
        self.panels.push(Panel {
            id: id.to_string(),
            title: title.to_string(),
            content: content.erase(),
            is_floating: matches!(position, PanelPosition::Floating { .. }),
            is_docked: !matches!(position, PanelPosition::Floating { .. }),
            position,
        });
        self
    }

    pub fn active(mut self, panel_id: &str) -> Self {
        if let Some(idx) = self.panels.iter().position(|p| p.id == panel_id) {
            self.active_panel = Some(idx);
        }
        self
    }
}

impl View for DockingWorkspace {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.panels.is_empty() {
            return;
        }

        // Render header
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: self.header_height,
        };
        renderer.fill_rect(header_rect, [0.08, 0.08, 0.12, 1.0]);
        renderer.stroke_rect(header_rect, [0.3, 0.5, 0.8, 1.0], 1.0);
        renderer.draw_text(
            "Docking Workspace",
            header_rect.x + 12.0,
            header_rect.y + 14.0,
            16.0,
            [0.9, 0.95, 1.0, 1.0],
        );

        // Render sidebar
        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y + self.header_height,
            width: self.sidebar_width,
            height: rect.height - self.header_height,
        };
        renderer.fill_rect(sidebar_rect, [0.06, 0.06, 0.1, 1.0]);
        renderer.stroke_rect(sidebar_rect, [0.2, 0.3, 0.5, 1.0], 1.0);

        let mut current_y = sidebar_rect.y + 8.0;
        for panel in &self.panels {
            if !panel.is_docked || matches!(panel.position, PanelPosition::Left) {
                let is_active = self
                    .active_panel
                    .is_some_and(|idx| self.panels.get(idx).is_some_and(|p| p.id == panel.id));

                let text_color = if is_active {
                    [0.9, 0.95, 1.0, 1.0]
                } else {
                    [0.5, 0.5, 0.6, 1.0]
                };
                renderer.draw_text(
                    &panel.title,
                    sidebar_rect.x + 12.0,
                    current_y,
                    13.0,
                    text_color,
                );

                if is_active {
                    let indicator_rect = Rect {
                        x: sidebar_rect.x + 4.0,
                        y: current_y - 2.0,
                        width: 3.0,
                        height: 14.0,
                    };
                    renderer.fill_rounded_rect(indicator_rect, 1.5, [0.0, 0.8, 1.0, 1.0]);
                }

                current_y += 28.0;
            }
        }

        // Render center content
        let center_x = rect.x + self.sidebar_width;
        let center_rect = Rect {
            x: center_x,
            y: rect.y + self.header_height,
            width: rect.width - self.sidebar_width,
            height: rect.height - self.header_height,
        };

        if let Some(active_idx) = self.active_panel {
            if let Some(panel) = self.panels.get(active_idx) {
                panel.content.render(renderer, center_rect);
            }
        } else if let Some(panel) = self.panels.first() {
            panel.content.render(renderer, center_rect);
        }
    }
}

impl LayoutView for DockingWorkspace {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = self.sidebar_width + 400.0;
        let height = self.header_height + 300.0;
        Size { width, height }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// SplitPane divides the workspace into resizable sections.
pub struct SplitPane {
    pub(crate) orientation: Orientation,
    pub(crate) first: AnyView,
    pub(crate) second: AnyView,
    pub(crate) split_ratio: f32,
}

pub use cvkg_core::Orientation;

impl SplitPane {
    pub fn new(
        orientation: Orientation,
        first: impl View + Clone + 'static,
        second: impl View + Clone + 'static,
    ) -> Self {
        Self {
            orientation,
            first: first.erase(),
            second: second.erase(),
            split_ratio: 0.5,
        }
    }

    pub fn split_ratio(mut self, ratio: f32) -> Self {
        self.split_ratio = ratio.clamp(0.1, 0.9);
        self
    }
}

impl View for SplitPane {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.orientation == Orientation::Horizontal {
            let first_w = rect.width * self.split_ratio;
            let first_rect = Rect {
                x: rect.x,
                y: rect.y,
                width: first_w,
                height: rect.height,
            };
            let second_rect = Rect {
                x: rect.x + first_w,
                y: rect.y,
                width: rect.width - first_w,
                height: rect.height,
            };

            // Divider
            let divider_x = rect.x + first_w;
            renderer.draw_line(
                divider_x,
                rect.y,
                divider_x,
                rect.y + rect.height,
                [0.3, 0.4, 0.6, 1.0],
                2.0,
            );

            self.first.render(renderer, first_rect);
            self.second.render(renderer, second_rect);
        } else {
            let first_h = rect.height * self.split_ratio;
            let first_rect = Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: first_h,
            };
            let second_rect = Rect {
                x: rect.x,
                y: rect.y + first_h,
                width: rect.width,
                height: rect.height - first_h,
            };

            // Divider
            let divider_y = rect.y + first_h;
            renderer.draw_line(
                rect.x,
                divider_y,
                rect.x + rect.width,
                divider_y,
                [0.3, 0.4, 0.6, 1.0],
                2.0,
            );

            self.first.render(renderer, first_rect);
            self.second.render(renderer, second_rect);
        }
    }
}

impl LayoutView for SplitPane {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 400.0,
            height: 300.0,
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

/// WorkspaceTabs provides tabbed interface for workspace panels.
pub struct WorkspaceTabs {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab: Option<usize>,
}

pub struct Tab {
    pub label: String,
    pub content: AnyView,
}

impl Default for WorkspaceTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceTabs {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
        }
    }

    pub fn tab(mut self, label: impl Into<String>, content: impl View + Clone + 'static) -> Self {
        self.tabs.push(Tab {
            label: label.into(),
            content: content.erase(),
        });
        self
    }

    pub fn active(mut self, index: usize) -> Self {
        self.active_tab = Some(index.min(self.tabs.len().saturating_sub(1)));
        self
    }
}

impl View for WorkspaceTabs {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.tabs.is_empty() {
            return;
        }

        let tab_height = 32.0;
        let tab_width = rect.width / self.tabs.len() as f32;

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = self.active_tab == Some(i);
            let tab_rect = Rect {
                x: rect.x + i as f32 * tab_width,
                y: rect.y,
                width: tab_width,
                height: tab_height,
            };

            let bg = if is_active {
                [0.1, 0.2, 0.3, 1.0]
            } else {
                [0.06, 0.06, 0.1, 1.0]
            };
            renderer.fill_rounded_rect(tab_rect, 4.0, bg);

            let color = if is_active {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.5, 0.5, 0.6, 1.0]
            };
            renderer.draw_text(&tab.label, tab_rect.x + 8.0, tab_rect.y + 10.0, 13.0, color);

            if is_active {
                renderer.stroke_rect(tab_rect, [0.0, 0.8, 1.0, 1.0], 1.0);
            }
        }

        // Render active tab content
        if let Some(active_idx) = self.active_tab
            && let Some(tab) = self.tabs.get(active_idx)
        {
            let content_rect = Rect {
                x: rect.x,
                y: rect.y + tab_height,
                width: rect.width,
                height: rect.height - tab_height,
            };
            tab.content.render(renderer, content_rect);
        }
    }
}

impl LayoutView for WorkspaceTabs {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = (self.tabs.len() as f32 * 120.0).max(200.0);
        Size {
            width,
            height: 200.0,
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
