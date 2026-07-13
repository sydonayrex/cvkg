//! Gallery application definition

use super::{sidebar, canvas, state, spotlight, props_panel, docs_panel};
use cvkg_core::{Rect, Renderer, Size, View};
use std::sync::{Arc, Mutex};

/// Main gallery application.
#[derive(Clone)]
pub struct GalleryApp {
    state: Arc<Mutex<state::GalleryState>>,
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
                let mut s = state.lock().unwrap();
                s.select_component(name);
            }
        };

        let sidebar = sidebar::GallerySidebar::new(state.clone(), on_select);
        let canvas = canvas::GalleryCanvas::new(state.clone());
        
        let on_theme_change = {
            let state = state.clone();
            move |theme| {
                let mut s = state.lock().unwrap();
                s.theme = theme;
            }
        };
        
        let on_scale_change = {
            let state = state.clone();
            move |scale| {
                let mut s = state.lock().unwrap();
                s.scale = scale;
            }
        };
        
        let on_bg_change = {
            let state = state.clone();
            move |bg| {
                let mut s = state.lock().unwrap();
                s.canvas_bg = bg;
            }
        };
        
        let on_viewport_change = {
            let state = state.clone();
            move |viewport| {
                let mut s = state.lock().unwrap();
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
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        eprintln!("[CVKG App Render] rect: {:?}", rect);
        // Get sidebar width from state
        let sidebar_width = {
            let state = self.state.lock().unwrap();
            state.sidebar_width.max(240.0)
        };

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

        eprintln!("[CVKG App Render] sidebar_rect: {:?}", sidebar_rect);
        eprintln!("[CVKG App Render] toolbar_rect: {:?}", toolbar_rect);
        eprintln!("[CVKG App Render] canvas_rect: {:?}", canvas_rect);
        eprintln!("[CVKG App Render] props_panel_rect: {:?}", props_panel_rect);
        eprintln!("[CVKG App Render] docs_panel_rect: {:?}", docs_panel_rect);

        // Render sidebar
        renderer.fill_rect(sidebar_rect, [0.1, 0.1, 0.12, 1.0]); // Dark slate background for sidebar
        self.sidebar.render(renderer, sidebar_rect);

        // Render toolbar
        self.toolbar.render(renderer, toolbar_rect);
        
        // Render canvas
        renderer.fill_rect(canvas_rect, [0.15, 0.15, 0.18, 1.0]); // Darker gray background for canvas
        self.canvas.render(renderer, canvas_rect);
        
        // Render props panel
        renderer.fill_rect(props_panel_rect, [0.12, 0.12, 0.15, 1.0]); // Dark slate for props
        self.props_panel.render(renderer, props_panel_rect);
        
        // Render docs panel
        renderer.fill_rect(docs_panel_rect, [0.08, 0.08, 0.1, 1.0]); // Even darker slate for docs
        self.docs_panel.render(renderer, docs_panel_rect);
        
        // Render spotlight overlay if open
        let spotlight_open = {
            let state = self.state.lock().unwrap();
            state.spotlight_open
        };
        
        if spotlight_open {
            self.spotlight.render(renderer, rect);
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::SizeProposal) -> Size {
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