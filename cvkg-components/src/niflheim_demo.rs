//! # Niflheim Layout Engine Showcase Demo
//!
//! A high-fidelity layout demonstration showing:
//! - **ScrollView** with scrollbar indicators and keyboard nav
//! - **Grid** container with flexible columns and gaps
//! - **Frame and Padding** modifiers with alignment support
//! - **Overlay** modifier with click-outside dismissal

use crate::container::{ScrollView, VStack};
use crate::grid::{Grid, GridTrack};
use crate::interactive::Button;
use cvkg_core::{GridPlacement, View};

/// Returns a layout showcase view demonstrating the completed layout engine features.
pub fn niflheim_demo() -> impl View + Clone {
    let scroll_id = 0x5c80_1111;
    let popover_state_hash = 0x5c80_2222;

    // Load popover state from system state to check if popover is open
    let show_popover = cvkg_core::load_system_state()
        .get_component_state::<bool>(popover_state_hash)
        .and_then(|v| v.read().ok().map(|g| *g))
        .unwrap_or(false);

    // Sidebar: Scrollable list of 15 buttons
    let mut list_vstack = VStack::new(10.0)
        .alignment(cvkg_core::Alignment::Leading)
        .distribution(cvkg_core::Distribution::Leading);
    for i in 1..=15 {
        list_vstack = list_vstack
            .child(Button::new(format!("Item {}", i), move || {}).frame(Some(160.0), Some(30.0)));
    }

    let scroll_sidebar = ScrollView::new(list_vstack)
        .scroll_id(scroll_id)
        .content_size(180.0, 600.0)
        .scrollbar_width(6.0);

    // Main section: 2x2 Grid of cards
    let card_grid = Grid::new(
        vec![GridTrack::Flex(1.0), GridTrack::Flex(1.0)],
        vec![GridTrack::Flex(1.0), GridTrack::Flex(1.0)],
    )
    .gap(15.0)
    .child(
        Button::new("Card A", || {})
            .frame(Some(120.0), Some(80.0))
            .grid_placement(GridPlacement {
                column: 0,
                column_span: 1,
                row: 0,
                row_span: 1,
            }),
    )
    .child(
        Button::new("Card B", || {})
            .frame(Some(120.0), Some(80.0))
            .grid_placement(GridPlacement {
                column: 1,
                column_span: 1,
                row: 0,
                row_span: 1,
            }),
    )
    .child(
        // Card C triggers a popover overlay when clicked
        Button::new("Info Popover", move || {
            cvkg_core::update_system_state(move |s| {
                let mut s = s.clone();
                s.set_component_state(popover_state_hash, !show_popover);
                s
            });
        })
        .frame(Some(120.0), Some(80.0))
        .grid_placement(GridPlacement {
            column: 0,
            column_span: 1,
            row: 1,
            row_span: 1,
        }),
    )
    .child(
        Button::new("Card D", || {})
            .frame(Some(120.0), Some(80.0))
            .grid_placement(GridPlacement {
                column: 1,
                column_span: 1,
                row: 1,
                row_span: 1,
            }),
    );

    // Main layout: 2-column grid splitting sidebar and main grid
    let main_layout = Grid::new(
        vec![GridTrack::Fixed(200.0), GridTrack::Flex(1.0)],
        vec![GridTrack::Flex(1.0)],
    )
    .column_gap(20.0)
    .child(scroll_sidebar.grid_placement(GridPlacement {
        column: 0,
        column_span: 1,
        row: 0,
        row_span: 1,
    }))
    .child(card_grid.grid_placement(GridPlacement {
        column: 1,
        column_span: 1,
        row: 0,
        row_span: 1,
    }));

    // Apply overlay popover if active
    let popup_view = Button::new("Close Popover", move || {
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(popover_state_hash, false);
            s
        });
    })
    .frame(Some(140.0), Some(60.0));

    let decorated_layout = main_layout.padding(15.0);

    if show_popover {
        let dismiss_fn = std::sync::Arc::new(move || {
            cvkg_core::update_system_state(move |s| {
                let mut s = s.clone();
                s.set_component_state(popover_state_hash, false);
                s
            });
        });

        // Render overlay centered relative to the whole dashboard
        decorated_layout.overlay(
            popup_view,
            cvkg_core::Alignment::Center,
            [0.0, 0.0],
            Some(dismiss_fn),
        )
    } else {
        // Return without overlay
        decorated_layout.overlay(
            cvkg_core::EmptyView,
            cvkg_core::Alignment::Center,
            [0.0, 0.0],
            None,
        )
    }
}
