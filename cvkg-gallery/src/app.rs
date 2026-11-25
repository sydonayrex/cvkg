//! Gallery application definition

use super::{canvas, docs_panel, props_panel, sidebar, spotlight, state};
use cvkg_core::{KeyModifiers, KeyShortcut, Rect, Renderer, Size, ThemeContext, View};

/// Main gallery application.
#[derive(Clone)]
pub struct GalleryApp {
    state: crate::state::GalleryStateHandle,
    sidebar: sidebar::GallerySidebar,
    canvas: canvas::GalleryCanvas,
    toolbar: canvas::CanvasToolbar,
    spotlight: spotlight::GallerySpotlight,
    props_panel: props_panel::PropsPanel,
    docs_panel: docs_panel::DocsPanel,
}

impl GalleryApp {
    /// Creates a new main gallery application view.
    pub fn new() -> Self {
        let state = state::GalleryState::global();

        let on_select = {
            let state = state.clone();
            move |name: &str| {
                let mut s = state.lock();
                s.select_component(name);
            }
        };

        let sidebar = sidebar::GallerySidebar::new(state.clone(), on_select);
        let canvas = canvas::GalleryCanvas::new(state.clone());

        let on_theme_change = {
            let state = state.clone();
            move |theme| {
                let mut s = state.lock();
                s.theme = theme;
            }
        };

        let on_scale_change = {
            let state = state.clone();
            move |scale| {
                let mut s = state.lock();
                s.scale = scale;
            }
        };

        let on_bg_change = {
            let state = state.clone();
            move |bg| {
                let mut s = state.lock();
                s.canvas_bg = bg;
            }
        };

        let on_viewport_change = {
            let state = state.clone();
            move |viewport| {
                let mut s = state.lock();
                s.viewport = viewport;
            }
        };

        let toolbar = canvas::CanvasToolbar::new(
            state.clone(),
            on_theme_change,
            on_scale_change,
            on_bg_change,
            on_viewport_change,
        );

        let spotlight = spotlight::GallerySpotlight::new(state.clone());
        let props_panel = props_panel::PropsPanel::new(state.clone());
        let docs_panel = docs_panel::DocsPanel::new(state.clone());

        Self {
            state,
            sidebar,
            canvas,
            toolbar,
            spotlight,
            props_panel,
            docs_panel,
        }
    }
}

impl View for GalleryApp {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn key_shortcuts(&self) -> Vec<KeyShortcut> {
        vec![KeyShortcut::new("i", "Toggle accessibility element inspector").with_ctrl()]
    }

    fn on_key_event(&self, key: &str, modifiers: KeyModifiers) -> bool {
        if modifiers.ctrl && key.eq_ignore_ascii_case("i") {
            let mut state = self.state.lock();
            state.inspector_visible = !state.inspector_visible;
            true
        } else {
            false
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Apply gallery theme to the thread-local theme context so that all
        // component helpers (theme::surface(), theme::text(), etc.) return
        // the correct semantic colors for the user-selected theme.
        // WHY here: the native renderer sets its own GPU-based theme context
        // before each frame. The gallery overrides it at the start of render
        // so all child components (sidebar, toolbar, canvas) see the right theme.
        // One mutex read per frame publishes an entity-backed snapshot. Child
        // views read that handle instead of taking the global lock again.
        let frame_state = {
            let state = self.state.lock();
            state.clone()
        };
        state::GalleryState::set_frame_snapshot(&frame_state);
        let gallery_theme = frame_state.theme;
        let theme_ctx = match gallery_theme {
            state::GalleryTheme::Light => ThemeContext::light(),
            state::GalleryTheme::Dark => ThemeContext::dark(),
            state::GalleryTheme::HighContrast => ThemeContext {
                colors: cvkg_core::SemanticColors {
                    background: cvkg_core::Color::new(0.0, 0.0, 0.0, 1.0),
                    surface: cvkg_core::Color::new(0.1, 0.1, 0.12, 1.0),
                    text: cvkg_core::Color::new(1.0, 1.0, 1.0, 1.0),
                    text_dim: cvkg_core::Color::new(0.8, 0.8, 0.8, 1.0),
                    primary: cvkg_core::Color::new(0.0, 0.6, 1.0, 1.0),
                    secondary: cvkg_core::Color::new(1.0, 1.0, 0.0, 1.0),
                    accent: cvkg_core::Color::new(1.0, 0.2, 0.4, 1.0),
                    error: cvkg_core::Color::new(1.0, 0.2, 0.2, 1.0),
                    warning: cvkg_core::Color::new(1.0, 0.8, 0.0, 1.0),
                    success: cvkg_core::Color::new(0.0, 1.0, 0.4, 1.0),
                },
                glassmorphism_enabled: false,
            },
        };
        cvkg_core::set_theme_context(theme_ctx);

        // Get sidebar width from the frame snapshot.
        let sidebar_width = frame_state.sidebar_width.max(240.0);

        // Layout: Storybook Style
        // ┌──────────────────┬──────────────────────────────────────────┐
        // │ Sidebar          │ Toolbar (flexible)                       │
        // │ (full height,    ├──────────────────────────────────────────┤
        // │  240px+)         │ Canvas (showcase preview area)           │
        // │                  ├──────────────────────────────────────────┤
        // │                  │ Bottom Panels (Props + Docs side-by-side)│
        // └──────────────────┴──────────────────────────────────────────┘

        // 1. Sidebar spans full height on the left
        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: sidebar_width,
            height: rect.height,
        };

        // Main content area to the right of the sidebar
        let main_x = rect.x + sidebar_width;
        let main_width = (rect.width - sidebar_width).max(400.0);

        // 2. Toolbar at the top of the main area
        let toolbar_rect = Rect {
            x: main_x,
            y: rect.y,
            width: main_width,
            height: 44.0,
        };

        let content_y = rect.y + 44.0;
        let content_height = (rect.height - 44.0).max(100.0);

        // 3. Canvas takes the upper part of main area
        let canvas_rect = Rect {
            x: main_x,
            y: content_y,
            width: main_width,
            height: content_height * 0.6,
        };

        // 4. Props & Docs panels share the bottom part of main area side-by-side
        let bottom_y = content_y + canvas_rect.height;
        let bottom_height = content_height - canvas_rect.height;
        let panel_width = main_width / 2.0;

        let props_panel_rect = Rect {
            x: main_x,
            y: bottom_y,
            width: panel_width,
            height: bottom_height,
        };

        let docs_panel_rect = Rect {
            x: main_x + panel_width,
            y: bottom_y,
            width: panel_width,
            height: bottom_height,
        };

        // Render sidebar — dark navy panel on the left.
        // WHY this color: visually distinct from the aurora red gradient to confirm the
        // fill renders at the correct position. Chosen to contrast without jarring contrast.
        renderer.fill_rect(sidebar_rect, [0.08, 0.08, 0.12, 1.0]);
        self.sidebar.render(renderer, sidebar_rect);

        // Render toolbar
        self.toolbar.render(renderer, toolbar_rect);

        // Render canvas
        self.canvas.render(renderer, canvas_rect);

        // Render props panel
        self.props_panel.render(renderer, props_panel_rect);

        // Render docs panel
        self.docs_panel.render(renderer, docs_panel_rect);

        // Render spotlight overlay if open
        let spotlight_open = {
            let state = self.state.lock();
            state.spotlight_open
        };

        if spotlight_open {
            self.spotlight.render(renderer, rect);
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(1280.0),
            height: proposal.height.unwrap_or(720.0),
        }
    }
}

impl cvkg_core::layout::LayoutView for GalleryApp {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(1280.0),
            height: proposal.height.unwrap_or(720.0),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        // Root view handles its own layout in render()
    }
}
