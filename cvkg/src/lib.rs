pub use cvkg_anim as anim;
pub use cvkg_components as components;
pub use cvkg_core as core;
pub use cvkg_layout as layout;
pub use cvkg_scene as scene;
pub use cvkg_themes as themes;

#[cfg(feature = "gpu")]
pub use cvkg_render_gpu as render;

#[cfg(feature = "native")]
pub use cvkg_render_native as native;

pub mod prelude {
    pub use cvkg_components::Color;
    pub use cvkg_components::*;
    pub use cvkg_core::{Binding, Rect, State, View};
    pub use cvkg_layout::{HStack, VStack, ZStack};
}
