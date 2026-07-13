//! Sidebar navigation using NiflheimSidebar.

use crate::state::{GalleryState, Registry};
use cvkg_components::chrome::{NiflheimSidebar, SidebarItem, SidebarVibrancy};
use cvkg_core::{Rect, Renderer, View, Size, SizeProposal};
use std::sync::{Arc, Mutex};

/// Sidebar panel showing component categories.
#[derive(Clone)]
pub struct GallerySidebar {
    state: Arc<Mutex<GalleryState>>,
    on_select: Arc<dyn Fn(&str) + Send + Sync>,
}

impl GallerySidebar {
    pub fn new(
        state: Arc<Mutex<GalleryState>>,
        on_select: impl Fn(&str) + Send + Sync + 'static,
    ) -> Self {
        Self {
            state,
            on_select: Arc::new(on_select),
        }
    }

    fn build_sidebar_items(&self) -> Vec<SidebarItem> {
        let state = self.state.lock().unwrap();
        let by_category = Registry::by_category();
        let selected = state.selected_component.clone();

        // Sort categories: Forms, Overlays, Layout, Data Display, Feedback, Navigation, Advanced
        let category_order = [
            "Forms",
            "Overlays",
            "Layout",
            "Data Display",
            "Feedback",
            "Navigation",
            "Advanced",
        ];

        category_order
            .iter()
            .filter_map(|cat| {
                by_category.get(*cat).map(|components| {
                    let items: Vec<SidebarItem> = components
                        .iter()
                        .map(|c| {
                            let is_selected = selected.as_ref() == Some(&c.name.to_string());
                            SidebarItem::new(c.name, c.name)
                                .icon("")
                                .children(vec![])
                        })
                        .collect();

                    let expanded = state.expanded_categories.get(*cat).copied().unwrap_or(true);
                    SidebarItem::new(*cat, *cat)
                        .children(items)
                        .expanded(expanded)
                })
            })
            .collect()
    }
}

impl View for GallerySidebar {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let items = self.build_sidebar_items();

        let on_select = self.on_select.clone();
        let sidebar = NiflheimSidebar::new(items)
            .vibrancy(SidebarVibrancy::Translucent)
            .on_select(move |id| (on_select)(id));

        sidebar.render(renderer, rect);

        // Render divider handle on trailing edge
        let divider_x = rect.x + rect.width - 3.0;
        renderer.fill_rect(
            Rect {
                x: divider_x,
                y: rect.y,
                width: 6.0,
                height: rect.height,
            },
            [0.5, 0.5, 0.5, 0.3],
        );
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: 240.0,
            height: proposal.height.unwrap_or(800.0),
        }
    }
}

impl cvkg_core::layout::LayoutView for GallerySidebar {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size {
            width: 240.0,
            height: proposal.height.unwrap_or(800.0),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        // Leaf view
    }
}