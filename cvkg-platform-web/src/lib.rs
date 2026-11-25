//! Browser platform marker.
//!
//! The demo renders into a canvas, so this platform has no native surface.

use cvkg_platform_core::Platform;

/// Host backed by a browser canvas.
#[derive(Clone, Copy, Debug, Default)]
pub struct WebPlatform;

impl Platform for WebPlatform {
    fn name(&self) -> &'static str {
        "web"
    }

    fn is_headless(&self) -> bool {
        true
    }
}
