//! Demonstrates the docking workspace and virtual list together.
//!
//! Run with: `cargo run -p cvkg-gallery --example docking_virtual_list`

use cvkg_components::prelude::DockingWorkspace;
use cvkg_components::{PanelPosition, Text, VirtualList};

fn main() {
    let workspace = DockingWorkspace::new()
        .panel("main", "Main", Text::new("Dock content"), PanelPosition::Center)
        .active("main");
    let list = VirtualList::new((0..1000).collect::<Vec<i32>>(), 24.0, |item: &i32, _| {
        Text::new(format!("Item {item}"))
    });

    let _ = workspace;
    let _ = list;
    println!("DockingWorkspace and VirtualList constructed");
}
