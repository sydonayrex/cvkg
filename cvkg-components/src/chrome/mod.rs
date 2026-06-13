//! Chrome components — Application shell elements (menu bar, dock, toolbar).
//! These components provide the structural UI that surrounds content.

pub mod heimdall_dock;
pub use heimdall_dock::{DockItem, DockPosition, HeimdallDock, dock_item_magnification};

pub mod niflheim_sidebar;
pub use niflheim_sidebar::{NiflheimSidebar, SidebarItem, SidebarVibrancy};

pub mod nornir_bar;
pub use nornir_bar::NornirBar;

pub mod rune_inspector;
pub use rune_inspector::{InspectorPosition, RuneInspector};

pub mod valkyrie_toolbar;
pub use valkyrie_toolbar::{ToolbarItem, ToolbarSearchField, ToolbarSegmented, ValkyrieToolbar};
