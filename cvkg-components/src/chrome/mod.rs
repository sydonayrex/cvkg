//! Chrome components — Application shell elements (menu bar, dock, toolbar).
//! These components provide the structural UI that surrounds content.

pub mod heimdall_dock;
pub use heimdall_dock::{HeimdallDock, DockItem, DockPosition, dock_item_magnification};

pub mod niflheim_sidebar;
pub use niflheim_sidebar::{NiflheimSidebar, SidebarItem, SidebarVibrancy};

pub mod rune_inspector;
pub use rune_inspector::{RuneInspector, InspectorPosition};

pub mod valkyrie_toolbar;
pub use valkyrie_toolbar::{
    ValkyrieToolbar, ToolbarItem, ToolbarSegmented, ToolbarSearchField,
};