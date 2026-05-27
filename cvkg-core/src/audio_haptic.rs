//! Audio and Haptic Feedback — Item 14
//!
//! OS-agnostic audio and haptic feedback abstractions.
//! Platform implementations are behind feature flags.
//!
//! # OS-agnostic design
//! The traits here use no platform-specific types. Platform backends
//! are selected via cfg flags in the renderer, not here.

use std::sync::Arc;

/// Audio engine trait for playing sounds and spatial audio.
///
/// Implementations are platform-specific:
/// - Desktop: rodio or cpal backend
/// - Web: Web Audio API via wasm-bindgen
pub trait AudioEngine: Send + Sync {
    /// Play a named sound at the given volume (0.0 to 1.0).
    fn play_sound(&self, name: &str, volume: f32);

    /// Play a spatial sound at a 3D position relative to the listener.
    fn play_spatial(&self, name: &str, position: [f32; 3], volume: f32);

    /// Set the listener's position in 3D space for spatial audio.
    fn set_listener_position(&self, _position: [f32; 3]) {}

    /// Stop all currently playing sounds.
    fn stop_all(&self) {}

    /// Play an embedded audio buffer (e.g., from include_bytes!) at the given volume.
    ///
    /// The data slice must contain a valid WAV, OGG, or other supported audio format.
    /// Default implementation is a no-op; backends that support buffer playback
    /// should override this method.
    fn play_buffer(&self, _data: &[u8], _volume: f32) {}
}

/// No-op audio engine used when no audio backend is available.
pub struct NullAudioEngine;

impl AudioEngine for NullAudioEngine {
    fn play_sound(&self, _name: &str, _volume: f32) {}
    fn play_spatial(&self, _name: &str, _position: [f32; 3], _volume: f32) {}
    fn set_listener_position(&self, _position: [f32; 3]) {}
    fn stop_all(&self) {}
}

/// Haptic feedback engine trait.
///
/// Implementations are platform-specific:
/// - macOS: Core Haptics via objc2
/// - iOS: UIImpactFeedbackGenerator via objc2
/// - Web: Vibration API via wasm-bindgen
/// - Other: no-op
pub trait HapticEngine: Send + Sync {
    /// Trigger an impact haptic with the given intensity.
    fn impact(&self, _intensity: HapticIntensity) {}

    /// Light tap for selection changes (e.g., picker wheel, toggle).
    fn selection(&self) {}

    /// Success notification haptic.
    fn success(&self) {}

    /// Warning notification haptic.
    fn warning(&self) {}

    /// Error notification haptic.
    fn error(&self) {}

    /// Visual micro-feedback tick — subtle visual pulse for UI interactions.
    ///
    /// Unlike haptic methods which trigger physical feedback, this triggers
    /// a brief visual animation (e.g., a glow or scale pulse) synchronized
    /// with the interaction. Intensity ranges from 0.0 (barely visible) to
    /// 1.0 (strong). Default implementation is a no-op.
    fn visual_tick(&self, _intensity: f32) {}
}

/// Haptic impact intensity levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticIntensity {
    Light,
    Medium,
    Heavy,
}

/// No-op haptic engine used when no haptic backend is available.
pub struct NullHapticEngine;

impl HapticEngine for NullHapticEngine {
    fn impact(&self, _intensity: HapticIntensity) {}
    fn selection(&self) {}
    fn success(&self) {}
    fn warning(&self) {}
    fn error(&self) {}
    fn visual_tick(&self, _intensity: f32) {}
}

/// Named sound constants for the design system.
///
/// These are string identifiers for platform-loaded sounds.
pub mod sounds {
    pub const CLICK: &str = "click";
    pub const TOGGLE_ON: &str = "toggle_on";
    pub const TOGGLE_OFF: &str = "toggle_off";
    pub const SUCCESS: &str = "success";
    pub const ERROR: &str = "error";
    pub const WARNING: &str = "warning";
    pub const SCRUB: &str = "scrub";
    pub const SELECTION: &str = "selection";

    /// Embedded WAV data for the navigation tick sound.
    ///
    /// Play via `AudioEngine::play_buffer(sounds::NAVIGATION_TICK, 1.0)`.
    pub const NAVIGATION_TICK: &[u8] = include_bytes!("../assets/sounds/nav_tick.wav");

    /// Embedded WAV data for the success chime sound.
    pub const SUCCESS_CHIME: &[u8] = include_bytes!("../assets/sounds/success_chime.wav");

    /// Embedded WAV data for the warning tone sound.
    pub const WARNING_TONE: &[u8] = include_bytes!("../assets/sounds/warning_tone.wav");
}

/// Global audio engine instance.
static AUDIO_ENGINE: once_cell::sync::Lazy<Arc<dyn AudioEngine>> =
    once_cell::sync::Lazy::new(|| Arc::new(NullAudioEngine));

/// Global haptic engine instance.
static HAPTIC_ENGINE: once_cell::sync::Lazy<Arc<dyn HapticEngine>> =
    once_cell::sync::Lazy::new(|| Arc::new(NullHapticEngine));

/// Set the global audio engine.
pub fn set_audio_engine(engine: Arc<dyn AudioEngine>) {
    // Note: once_cell can't be overwritten. In production, use a Mutex<Arc<dyn AudioEngine>>.
    // For now, this is a placeholder for the API design.
    let _ = engine;
}

/// Set the global haptic engine.
pub fn set_haptic_engine(engine: Arc<dyn HapticEngine>) {
    let _ = engine;
}

/// Play a sound using the global audio engine.
pub fn play_sound(name: &str, volume: f32) {
    AUDIO_ENGINE.play_sound(name, volume);
}

/// Trigger a haptic using the global haptic engine.
pub fn haptic_impact(intensity: HapticIntensity) {
    HAPTIC_ENGINE.impact(intensity);
}

/// Trigger selection haptic.
pub fn haptic_selection() {
    HAPTIC_ENGINE.selection();
}

/// Trigger success haptic.
pub fn haptic_success() {
    HAPTIC_ENGINE.success();
}

/// Trigger error haptic.
pub fn haptic_error() {
    HAPTIC_ENGINE.error();
}
