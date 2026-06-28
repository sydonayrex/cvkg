use std::sync::Arc;

// =============================================================================
// AUDIO / HAPTIC ENGINES -- Cross-platform micro-feedback
// =============================================================================

/// Cross-platform audio engine using rodio for spatialized sound cues.
/// Rodio 0.22 API: DeviceSinkBuilder::open_default_sink() → MixerDeviceSink.
/// Playback via rodio::play(&mixer, cursor).
pub struct RodioAudioEngine {
    sink: rodio::MixerDeviceSink,
}

// MixerDeviceSink is not Send+Sync on some platforms, but we only use it
// from the main thread. The AudioEngine trait requires Send+Sync for use in
// App struct fields, which is safe here because we never move it across threads.
unsafe impl Send for RodioAudioEngine {}
unsafe impl Sync for RodioAudioEngine {}

impl RodioAudioEngine {
    /// Create a new audio engine. Falls back to None if audio init fails.
    pub fn new() -> Option<Self> {
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => {
                log::info!("[Native] Audio engine initialized (rodio)");
                Some(Self { sink })
            }
            Err(e) => {
                log::warn!("[Native] Audio init failed (no sound): {}", e);
                None
            }
        }
    }
}

impl cvkg_core::AudioEngine for RodioAudioEngine {
    fn play_sound(&self, name: &str, volume: f32) {
        let data: &[u8] = match name {
            "nav_tick" => cvkg_core::sounds::NAVIGATION_TICK,
            "success_chime" => cvkg_core::sounds::SUCCESS_CHIME,
            "warning_tone" => cvkg_core::sounds::WARNING_TONE,
            _ => {
                log::warn!("[Native] Unknown sound: {}", name);
                return;
            }
        };
        self.play_buffer(data, volume);
    }

    fn play_buffer(&self, data: &[u8], _volume: f32) {
        use std::io::Cursor;
        let cursor = Cursor::new(data.to_vec());
        let mixer = self.sink.mixer();
        match rodio::play(mixer, cursor) {
            Ok(_sink) => {}
            Err(e) => log::warn!("[Native] Audio play failed: {}", e),
        }
    }

    fn play_spatial(&self, name: &str, _position: [f32; 3], volume: f32) {
        // Spatial audio: play sound without positional attenuation (OS-agnostic fallback)
        self.play_sound(name, volume);
    }
}

/// Visual haptic engine that translates haptic requests into visual micro-animations.
/// Used as a cross-platform fallback where native haptics are unavailable.
pub struct VisualHapticEngine {
    last_impact: std::sync::Mutex<std::time::Instant>,
}

impl Default for VisualHapticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualHapticEngine {
    pub fn new() -> Self {
        Self {
            last_impact: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }
}

impl cvkg_core::HapticEngine for VisualHapticEngine {
    fn impact(&self, intensity: cvkg_core::HapticIntensity) {
        let _ = intensity;
        *self.last_impact.lock().unwrap_or_else(|p| p.into_inner()) = std::time::Instant::now();
    }
    fn selection(&self) {
        self.impact(cvkg_core::HapticIntensity::Light);
    }
    fn success(&self) {
        self.impact(cvkg_core::HapticIntensity::Medium);
    }
    fn warning(&self) {
        self.impact(cvkg_core::HapticIntensity::Medium);
    }
    fn error(&self) {
        self.impact(cvkg_core::HapticIntensity::Heavy);
    }
    fn visual_tick(&self, _intensity: f32) {
        *self.last_impact.lock().unwrap_or_else(|p| p.into_inner()) = std::time::Instant::now();
    }
}
