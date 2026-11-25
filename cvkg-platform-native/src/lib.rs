//! Native desktop platform marker.

use cvkg_platform_core::Platform;

/// Host backed by a native window and GPU device.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativePlatform;

impl Platform for NativePlatform {
    fn name(&self) -> &'static str {
        "native"
    }

    fn is_headless(&self) -> bool {
        false
    }
}
