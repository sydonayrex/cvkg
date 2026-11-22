#![cfg(not(target_arch = "wasm32"))]
//! cvkg Gallery — Component showcase using NiflheimSidebar + MimirSpotlight + macOS Tahoe design.

pub mod app;
pub mod canvas;
pub mod docs_panel;
pub mod generated_docs;
pub mod props_panel;
pub mod sidebar;
pub mod spotlight;
pub mod state;

pub use self::app::GalleryApp;
pub use self::canvas::{CanvasToolbar, GalleryCanvas};
pub use self::docs_panel::DocsPanel;
pub use self::props_panel::PropsPanel;
pub use self::sidebar::GallerySidebar;
pub use self::spotlight::GallerySpotlight;
pub use self::state::GalleryState;
