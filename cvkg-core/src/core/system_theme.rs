// =============================================================================
// SYSTEM THEME DETECTION -- Dark/Light mode detection
// =============================================================================
//
// OS-agnostic theme detection. Checks the CVKG_THEME environment variable first,
// then falls back to dark mode (safe default).
//
// Platform backends may override this with native OS queries (e.g.,
// dark-light crate on desktop, prefers-color-scheme on web).

/// The detected system theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SystemTheme {
    /// Dark mode (default).
    #[default]
    Dark,
    /// Light mode.
    Light,
}

/// Detect the current system theme.
///
/// Checks `CVKG_THEME` environment variable first:
/// - `"dark"` → `SystemTheme::Dark`
/// - `"light"` → `SystemTheme::Light`
/// - unset or any other value → `SystemTheme::Dark` (default)
///
/// Platform backends can call this and override with native detection
/// (e.g., `dark-light` crate on desktop, `prefers-color-scheme` on web).
pub fn detect_system_theme() -> SystemTheme {
    std::env::var("CVKG_THEME")
        .ok()
        .and_then(|v| match v.as_str() {
            "light" => Some(SystemTheme::Light),
            "dark" => Some(SystemTheme::Dark),
            _ => None,
        })
        .unwrap_or(SystemTheme::Dark)
}

