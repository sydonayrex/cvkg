pub fn is_reduced_motion() -> bool {
    AccessibilityPreferences::detect_from_system().reduce_motion
}

use crate::AccessibilityPreferences;
/// Returns effective animation duration (0.0 if reduced motion is active).
pub fn effective_duration(secs: f32) -> f32 {
    if is_reduced_motion() { 0.0 } else { secs }
}
