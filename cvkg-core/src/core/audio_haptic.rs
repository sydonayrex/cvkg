// =============================================================================
// AUDIO / HAPTIC -- Item 14: Spatial Audio / Haptic Feedback
// =============================================================================
// OS-agnostic: pure trait abstractions. Platform backends via cfg in renderer.

pub mod audio_haptic;
pub use audio_haptic::{
    AudioEngine, HapticEngine, HapticIntensity, NullAudioEngine, NullHapticEngine, haptic_error,
    haptic_impact, haptic_selection, haptic_success, play_sound, set_audio_engine,
    set_haptic_engine, sounds,
};

