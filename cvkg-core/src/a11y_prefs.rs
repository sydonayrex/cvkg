// =============================================================================
// ACCESSIBILITY PREFERENCES -- System accessibility settings
// =============================================================================
//
// Components and the renderer query these to adapt behavior:
// - Reduce Motion: disable non-essential animations
// - Reduce Transparency: replace glass materials with opaque surfaces
// - Increase Contrast: make borders visible, minimum alpha 0.5

thread_local! {
    /// Thread-local accessibility preferences.
    /// Defaults to no restrictions (all false).
    static ACCESSIBILITY_PREFS: std::cell::RefCell<AccessibilityPreferences> =
        std::cell::RefCell::new(AccessibilityPreferences::default());
}

/// System accessibility preferences that components and the renderer must honor.
///
/// These map to macOS System Settings > Accessibility:
/// - `reduce_motion`: Disables non-essential animations (spring, bounce, etc.)
/// - `reduce_transparency`: Replaces glass/transparent materials with opaque surfaces
/// - `increase_contrast`: Makes all borders visible, minimum alpha 0.5 for all elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccessibilityPreferences {
    /// User prefers reduced motion. Animations should be instant or very short.
    pub reduce_motion: bool,
    /// User prefers reduced transparency. Glass materials should be opaque.
    pub reduce_transparency: bool,
    /// User prefers increased contrast. Borders must be visible, min alpha 0.5.
    pub increase_contrast: bool,
}

impl AccessibilityPreferences {
    /// Detect system accessibility preferences (macOS).
    ///
    /// On non-macOS platforms, returns defaults (all false).
    /// In a production implementation, this would query the OS APIs.
    pub fn detect_from_system() -> Self {
        #[cfg(target_os = "macos")]
        {
            // Try to read macOS accessibility preferences via defaults command
            let reduce_motion = std::process::Command::new("defaults")
                .args(["read", "-g", "com.apple.universalaccess", "reduceMotion"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

            let reduce_transparency = std::process::Command::new("defaults")
                .args([
                    "read",
                    "-g",
                    "com.apple.universalaccess",
                    "reduceTransparency",
                ])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

            let increase_contrast = std::process::Command::new("defaults")
                .args([
                    "read",
                    "-g",
                    "com.apple.universalaccess",
                    "increaseContrast",
                ])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim() == "1")
                .unwrap_or(false);

            Self {
                reduce_motion,
                reduce_transparency,
                increase_contrast,
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Reduced motion: check GTK_A11Y env var or GNOME gsettings
            let reduce_motion = std::env::var("GTK_A11Y")
                .map(|v| v.to_lowercase().contains("reduce-motion"))
                .unwrap_or(false)
                || {
                    // Try gsettings for GNOME desktop animation preference
                    std::process::Command::new("gsettings")
                        .args(["get", "org.gnome.desktop.interface", "enable-animations"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim() == "'false'" || s.trim() == "false")
                        .unwrap_or(false)
                };

            // Reduced transparency is not widely supported on Linux desktops
            let reduce_transparency = false;

            // Increased contrast: check GTK_THEME for high-contrast variants
            let increase_contrast = std::env::var("GTK_THEME")
                .map(|v| v.to_lowercase().contains("highcontrast"))
                .unwrap_or(false);

            Self {
                reduce_motion,
                reduce_transparency,
                increase_contrast,
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            // Helper: run `reg query` and return the value string if found
            fn reg_query(key: &str, value_name: &str) -> Option<String> {
                Command::new("reg")
                    .args(["query", key, "/v", value_name])
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            String::from_utf8(o.stdout).ok()
                        } else {
                            None
                        }
                    })
                    .and_then(|s| {
                        // Output format: "    ValueName    REG_SZ    <value>"
                        // or REG_DWORD lines; parse the last token on the last non-empty line
                        s.lines()
                            .last()?
                            .split_whitespace()
                            .last()
                            .map(String::from)
                    })
            }

            // Reduced motion: EffectsAnimationEfficiency = 1 means reduced
            let reduce_motion = reg_query(
                "HKCU\\Control Panel\\Accessibility\\EffectsAnimationEfficiency",
                "EffectsAnimationEfficiency",
            )
            .map(|v| v == "1")
            .unwrap_or(false);

            // Reduced transparency: EnableTransparency = 0 means reduced
            let reduce_transparency = reg_query(
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "EnableTransparency",
            )
            .map(|v| v == "0")
            .unwrap_or(false);

            // Increased contrast: HighContrast = 1 means enabled
            let increase_contrast = reg_query(
                "HKCU\\Control Panel\\Accessibility\\HighContrast",
                "HighContrast",
            )
            .map(|v| v == "1")
            .unwrap_or(false);

            Self {
                reduce_motion,
                reduce_transparency,
                increase_contrast,
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Self::default()
        }
    }

    /// Apply a minimum alpha constraint for increase-contrast mode.
    pub fn min_alpha(&self, requested: f32) -> f32 {
        if self.increase_contrast {
            requested.max(0.5)
        } else {
            requested
        }
    }

    /// Returns true if glass effects should be replaced with opaque surfaces.
    pub fn should_disable_glass(&self) -> bool {
        self.reduce_transparency
    }

    /// Returns true if animations should be instant.
    pub fn should_reduce_motion(&self) -> bool {
        self.reduce_motion
    }

    /// Returns true if borders should be made visible.
    pub fn should_increase_contrast(&self) -> bool {
        self.increase_contrast
    }
}

/// Get the current accessibility preferences for this thread.
pub fn accessibility_preferences() -> AccessibilityPreferences {
    ACCESSIBILITY_PREFS.with(|p| *p.borrow())
}

/// Set the accessibility preferences for this thread.
///
/// The native renderer should call this on startup and when system
/// preferences change (via `detect_from_system()`).
pub fn set_accessibility_preferences(prefs: AccessibilityPreferences) {
    ACCESSIBILITY_PREFS.with(|p| {
        *p.borrow_mut() = prefs;
    });
}
