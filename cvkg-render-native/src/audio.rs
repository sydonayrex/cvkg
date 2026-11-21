use std::sync::Arc;

// =============================================================================
// AUDIO / HAPTIC ENGINES -- Cross-platform micro-feedback
// =============================================================================

/// Cross-platform audio engine using rodio for spatialized sound cues.
/// Rodio 0.22 API: DeviceSinkBuilder::open_default_sink() → MixerDeviceSink.
/// Playback via rodio::play(&mixer, cursor).
pub struct RodioAudioEngine {
    sink: rodio::MixerDeviceSink,
    /// Thread that constructed the engine. Used to detect (and report) use
    /// from another thread, which would violate the `Send`/`Sync` claim below.
    created_on: std::thread::ThreadId,
}

// SAFETY (P0-2): `MixerDeviceSink` is not `Send`/`Sync` on some platforms, but
// the engine is constructed on the app's main thread, stored in `App`, and
// only driven from that thread's event loop. There is no public API that moves
// a `RodioAudioEngine` to another thread or shares a reference across threads;
// the `Send + Sync` bound exists solely to satisfy the `AudioEngine` trait used
// by `App` fields. `play_buffer` warns when it observes a foreign thread so
// misuse is visible instead of silently causing UB. If audio ever needs to be
// played from workers, wrap the sink in a thread-confined actor instead of
// relying on these impls.
unsafe impl Send for RodioAudioEngine {}
// SAFETY (P0-2): as above — `&self` is only shared within the creating thread.
// `Sync` is the stronger claim and is justified by the same confinement; the
// runtime check in `play_buffer` detects violations in debug and release.
unsafe impl Sync for RodioAudioEngine {}

impl RodioAudioEngine {
    /// Create a new audio engine. Falls back to None if audio init fails.
    pub fn new() -> Option<Self> {
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => {
                tracing::info!("[Native] Audio engine initialized (rodio)");
                Some(Self {
                    sink,
                    created_on: std::thread::current().id(),
                })
            }
            Err(e) => {
                tracing::warn!("[Native] Audio init failed (no sound): {}", e);
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
                tracing::warn!("[Native] Unknown sound: {}", name);
                return;
            }
        };
        self.play_buffer(data, volume);
    }

    fn play_buffer(&self, data: &[u8], _volume: f32) {
        // P0-2: detect violations of the thread-confinement invariant that the
        // unsafe `Send`/`Sync` impls above rely on. Warn rather than panic so a
        // late misuse degrades to missing audio, not a crash.
        if std::thread::current().id() != self.created_on {
            tracing::warn!(
                "[Native] Audio engine used from a foreign thread; this violates its Send/Sync contract"
            );
        }
        use std::io::Cursor;
        let cursor = Cursor::new(data.to_vec());
        let mixer = self.sink.mixer();
        match rodio::play(mixer, cursor) {
            Ok(_sink) => {}
            Err(e) => tracing::warn!("[Native] Audio play failed: {}", e),
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
