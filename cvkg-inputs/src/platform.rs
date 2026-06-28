//! Platform detection and backend instantiation.
#![allow(dead_code)]

/// Detects the current platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux.
    Linux,
    /// macOS.
    MacOS,
    /// Windows.
    Windows,
    /// Unknown/unsupported.
    Unknown,
}

impl Platform {
    /// Returns the current platform.
    pub fn current() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                Self::Linux
            } else if #[cfg(target_os = "macos")] {
                Self::MacOS
            } else if #[cfg(target_os = "windows")] {
                Self::Windows
            } else {
                Self::Unknown
            }
        }
    }

    /// Returns true if the platform supports evdev.
    pub fn supports_evdev(&self) -> bool {
        matches!(self, Self::Linux)
    }

    /// Returns true if the platform supports gilrs.
    pub fn supports_gilrs(&self) -> bool {
        matches!(self, Self::Linux | Self::MacOS | Self::Windows)
    }
}

/// Information about a HID device.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct HidDeviceInfo {
    /// Device path (e.g., `/dev/input/event0`).
    pub path: String,
    /// Human-readable device name.
    pub name: String,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Product ID.
    pub product_id: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current_is_known() {
        let p = Platform::current();
        // In CI at least one platform should be detected
        assert_ne!(p, Platform::Unknown);
    }

    #[test]
    fn test_platform_evdev_only_on_linux() {
        #[cfg(target_os = "linux")]
        assert!(Platform::current().supports_evdev());
        #[cfg(not(target_os = "linux"))]
        assert!(!Platform::current().supports_evdev());
    }
}
