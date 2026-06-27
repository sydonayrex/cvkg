use crate::RADIUS_XL;
use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Navigation stack for push/pop navigation
///
/// # Contract
/// Manages a stack of views and renders the topmost active view.
pub struct NavigationStack {
    pub(crate) stack: Vec<cvkg_core::AnyView>,
}

impl NavigationStack {
    /// Creates a new navigation stack with a root view.
    pub fn new<V: View + Clone + 'static>(root: V) -> Self {
        Self {
            stack: vec![root.erase()],
        }
    }

    /// Pushes a new view onto the stack.
    pub fn push<V: View + Clone + 'static>(&mut self, view: V) {
        self.stack.push(view.erase());
    }

    /// Pops the topmost view off the stack, if it's not the root view.
    pub fn pop(&mut self) -> Option<cvkg_core::AnyView> {
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }
}

impl View for NavigationStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NavigationStack");
        if let Some(top) = self.stack.last() {
            top.render(renderer, rect);
        }
        renderer.pop_vnode();
    }
}

/// Navigation split view for sidebar layouts
///
/// # Contract
/// Manages a two-pane layout with a collapsible sidebar and a detail pane.
pub struct NavigationSplitView<S, D> {
    pub(crate) sidebar: S,
    pub(crate) detail: D,
    pub(crate) is_collapsed: bool,
    pub(crate) sidebar_width: f32,
}

impl<S: View, D: View> NavigationSplitView<S, D> {
    /// Creates a new NavigationSplitView with sidebar and detail views.
    pub fn new(sidebar: S, detail: D) -> Self {
        Self {
            sidebar,
            detail,
            is_collapsed: false,
            sidebar_width: 300.0,
        }
    }

    /// Configures the default collapse state.
    pub fn is_collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    /// Configures the sidebar width.
    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width.clamp(150.0, 500.0);
        self
    }
}

impl<S: View, D: View> View for NavigationSplitView<S, D> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let nav_collapse_hash: u64 = 0xA00_0001;
        let is_collapsed: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(nav_collapse_hash)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.is_collapsed);
        let collapsed_width: f32 = 48.0;
        let handle_width: f32 = 4.0;
        let effective_sidebar_width = if is_collapsed {
            collapsed_width
        } else {
            self.sidebar_width.clamp(150.0, 500.0)
        };
        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: effective_sidebar_width,
            height: rect.height,
        };
        let handle_rect = Rect {
            x: rect.x + effective_sidebar_width,
            y: rect.y,
            width: handle_width,
            height: rect.height,
        };
        let detail_rect = Rect {
            x: rect.x + effective_sidebar_width + handle_width,
            y: rect.y,
            width: (rect.width - effective_sidebar_width - handle_width).max(0.0),
            height: rect.height,
        };

        if is_collapsed {
            renderer.fill_rect(sidebar_rect, theme::surface());
            renderer.stroke_rect(sidebar_rect, theme::border(), 1.0);
        } else {
            renderer.fill_rect(sidebar_rect, theme::surface());
            renderer.stroke_rect(sidebar_rect, theme::border(), 1.0);
            self.sidebar.render(renderer, sidebar_rect);
        }

        let hover_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "nav_split_handle_hover".hash(&mut s);
            s.finish()
        };
        let is_hover_handle: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(hover_hash)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(false);

        renderer.fill_rect(
            handle_rect,
            if is_hover_handle && !is_collapsed {
                theme::with_alpha(theme::accent(), 0.3)
            } else {
                theme::surface_elevated()
            },
        );
        renderer.stroke_rect(
            handle_rect,
            theme::with_alpha(theme::accent(), if is_hover_handle { 0.6 } else { 0.2 }),
            1.0,
        );

        let h_rect = handle_rect;
        renderer.register_handler(
            "pointermove",
            Arc::new(move |event| {
                if let Event::PointerMove { x, y, .. } = event {
                    let hovering = h_rect.contains(x, y);
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(hover_hash, hovering);
                        s
                    });
                }
            }),
        );

        let toggle_btn_size: f32 = 44.0;
        let toggle_rect = Rect {
            x: rect.x + effective_sidebar_width - toggle_btn_size / 2.0,
            y: rect.y + 12.0,
            width: toggle_btn_size,
            height: toggle_btn_size,
        };
        renderer.fill_rounded_rect(toggle_rect, RADIUS_XL, theme::surface_elevated());
        let chevron = if is_collapsed { "▶" } else { "◀" };
        renderer.draw_text(
            chevron,
            toggle_rect.x + 12.0,
            toggle_rect.y + 13.0,
            16.0,
            theme::text_muted(),
        );

        let tglm = toggle_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event
                    && tglm.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let nav_collapse_hash: u64 = 0xA00_0001;
                        let current: bool = s
                            .get_component_state::<bool>(nav_collapse_hash)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(nav_collapse_hash, !current);
                        s
                    });
                }
            }),
        );

        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event
                    && (key == "b" || key == "B")
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let nav_collapse_hash: u64 = 0xA00_0001;
                        let current: bool = s
                            .get_component_state::<bool>(nav_collapse_hash)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(nav_collapse_hash, !current);
                        s
                    });
                }
            }),
        );

        self.detail.render(renderer, detail_rect);
    }
}
