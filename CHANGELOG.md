# Changelog

## [0.2.0] - 2026-06-14

### Fixed
- Removed `EnvironmentShield` from security module (contained `process::exit()` in library code)
- Fixed `set_value()` → `set_description()` in VDOM AccessKit bridge
- Fixed broken Shift+Tab in FocusTrap keyboard navigation
- Fixed hardcoded FOCUS_RING_COLOR — now uses theme-aware `theme::focus_ring()`

### Added
- Complete AccessKit role mapping for all 53 AriaRole variants
- 44px minimum touch targets across all interactive components (WCAG 2.5.8)
- Button loading state with animated spinner
- i18n support for DatePicker, Dialog, and ConsentGate components
- Cross-platform AccessibilityPreferences (Linux, Windows, macOS)
- Adaptive color tokens for background, primary, secondary, accent in light/dark modes

### Changed
- Unified accessibility role enums — removed parallel `A11yRole` enum, using `cvkg_core::AriaRole` everywhere
- `is_reduced_motion()` now uses `AccessibilityPreferences::detect_from_system()`

### Deprecated
- `FOCUS_RING_COLOR` constant — use `theme::focus_ring()` instead
