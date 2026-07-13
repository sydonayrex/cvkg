//! cvkg Gallery — Component showcase using NiflheimSidebar + MimirSpotlight + macOS Tahoe design.

pub mod sidebar;
pub mod canvas;
pub mod state;
pub mod spotlight;
pub mod props_panel;
pub mod docs_panel;
pub mod app;

pub use self::state::GalleryState;
pub use self::sidebar::GallerySidebar;
pub use self::canvas::{GalleryCanvas, CanvasToolbar};
pub use self::spotlight::GallerySpotlight;
pub use self::props_panel::PropsPanel;
pub use self::docs_panel::DocsPanel;
pub use self::app::GalleryApp;