//! # CVKG Telemetry
//!
//! Opt-in, compile-time feature-gated telemetry for accessibility and performance
//! metrics. All data is logged locally (stderr/debug log) by default. No data
//! is transmitted externally.
//!
//! ## Usage
//!
//! Enable the `telemetry` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! cvkg-telemetry = { path = "../cvkg-telemetry", features = ["telemetry"] }
//! ```
//!
//! Then use the `telemetry!` macro to record events:
//!
//! ```
//! use cvkg_telemetry::{Telemetry, TelemetryEvent};
//!
//! let mut tel = Telemetry::new();
//! tel.record(TelemetryEvent::ContrastFailure {
//!     element: "sidebar_title".into(),
//!     apca_lc: 42.0,
//!     foreground: [1.0, 1.0, 1.0, 1.0],
//!     background: [0.1, 0.1, 0.12, 1.0],
//! });
//! ```

use std::time::Instant;

// --- Events ---

/// A telemetry event representing an accessibility or performance observation.
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    /// Text contrast failed APCA threshold (Lc < 60).
    ContrastFailure {
        element: String,
        apca_lc: f32,
        foreground: [f32; 4],
        background: [f32; 4],
    },
    /// User enabled `prefers-reduced-transparency` (glass was too heavy).
    ReducedTransparencyEnabled,
    /// User enabled `prefers-reduced-motion` (animation was too much).
    ReducedMotionEnabled,
    /// Frame time exceeded 16ms budget (60 FPS target).
    FrameBudgetExceeded { frame_time_ms: f32, budget_ms: f32 },
    /// Glass element rendered (count for density tracking).
    GlassElementRendered { blur_radius: f32, rect_area: f32 },
    /// Touch target smaller than 44x44.
    SmallTouchTarget {
        element: String,
        width: f32,
        height: f32,
    },
    /// Focus ring not visible (APCA Lc < 30 against adjacent).
    FocusRingInvisible { element: String, contrast: f32 },
}

impl TelemetryEvent {
    /// Returns a human-readable description of this event.
    pub fn description(&self) -> String {
        match self {
            TelemetryEvent::ContrastFailure {
                element, apca_lc, ..
            } => {
                format!(
                    "Contrast failure: {} (APCA Lc={:.1}, threshold=60)",
                    element, apca_lc
                )
            }
            TelemetryEvent::ReducedTransparencyEnabled => {
                "User enabled prefers-reduced-transparency".into()
            }
            TelemetryEvent::ReducedMotionEnabled => "User enabled prefers-reduced-motion".into(),
            TelemetryEvent::FrameBudgetExceeded {
                frame_time_ms,
                budget_ms,
            } => {
                format!(
                    "Frame budget exceeded: {:.1}ms > {:.1}ms",
                    frame_time_ms, budget_ms
                )
            }
            TelemetryEvent::GlassElementRendered {
                blur_radius,
                rect_area,
            } => {
                format!(
                    "Glass element: blur={:.1}px area={:.0}px^2",
                    blur_radius, rect_area
                )
            }
            TelemetryEvent::SmallTouchTarget {
                element,
                width,
                height,
            } => {
                format!(
                    "Small touch target: {} ({:.0}x{:.0}, min 44x44)",
                    element, width, height
                )
            }
            TelemetryEvent::FocusRingInvisible { element, contrast } => {
                format!(
                    "Focus ring invisible: {} (APCA Lc={:.1}, threshold=30)",
                    element, contrast
                )
            }
        }
    }

    /// Returns true if this event indicates an accessibility problem.
    pub fn is_accessibility_issue(&self) -> bool {
        matches!(
            self,
            TelemetryEvent::ContrastFailure { .. }
                | TelemetryEvent::ReducedTransparencyEnabled
                | TelemetryEvent::ReducedMotionEnabled
                | TelemetryEvent::SmallTouchTarget { .. }
                | TelemetryEvent::FocusRingInvisible { .. }
        )
    }

    /// Returns true if this event indicates a performance problem.
    pub fn is_performance_issue(&self) -> bool {
        matches!(self, TelemetryEvent::FrameBudgetExceeded { .. })
    }
}

// --- Telemetry Collector ---

/// Collects and logs telemetry events. All logging is local (stderr).
/// No data is transmitted externally.
#[derive(Debug, Clone)]
pub struct Telemetry {
    events: Vec<TelemetryEvent>,
    start_time: Instant,
    frame_count: u64,
    glass_element_count: u64,
    contrast_failure_count: u64,
    budget_exceeded_count: u64,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Telemetry {
    /// Creates a new telemetry collector.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            frame_count: 0,
            glass_element_count: 0,
            contrast_failure_count: 0,
            budget_exceeded_count: 0,
        }
    }

    /// Records a telemetry event.
    pub fn record(&mut self, event: TelemetryEvent) {
        match &event {
            TelemetryEvent::GlassElementRendered { .. } => self.glass_element_count += 1,
            TelemetryEvent::ContrastFailure { .. } => self.contrast_failure_count += 1,
            TelemetryEvent::FrameBudgetExceeded { .. } => self.budget_exceeded_count += 1,
            _ => {}
        }
        self.events.push(event);
    }

    /// Records a frame completion with the given frame time in milliseconds.
    pub fn record_frame(&mut self, frame_time_ms: f32) {
        self.frame_count += 1;
        if frame_time_ms > 16.0 {
            self.record(TelemetryEvent::FrameBudgetExceeded {
                frame_time_ms,
                budget_ms: 16.0,
            });
        }
    }

    /// Returns the total number of events recorded.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the number of frames recorded.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the number of glass elements rendered.
    pub fn glass_element_count(&self) -> u64 {
        self.glass_element_count
    }

    /// Returns the number of contrast failures.
    pub fn contrast_failure_count(&self) -> u64 {
        self.contrast_failure_count
    }

    /// Returns the number of frame budget exceeded events.
    pub fn budget_exceeded_count(&self) -> u64 {
        self.budget_exceeded_count
    }

    /// Returns all recorded events.
    pub fn events(&self) -> &[TelemetryEvent] {
        &self.events
    }

    /// Returns only accessibility-related events.
    pub fn accessibility_events(&self) -> Vec<&TelemetryEvent> {
        self.events
            .iter()
            .filter(|e| e.is_accessibility_issue())
            .collect()
    }

    /// Returns only performance-related events.
    pub fn performance_events(&self) -> Vec<&TelemetryEvent> {
        self.events
            .iter()
            .filter(|e| e.is_performance_issue())
            .collect()
    }

    /// Returns the elapsed time since creation.
    pub fn elapsed_secs(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    /// Returns the average FPS (frames per second).
    pub fn average_fps(&self) -> f32 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.frame_count as f32 / elapsed
        } else {
            0.0
        }
    }

    /// Logs a summary of all recorded events to stderr.
    pub fn log_summary(&self) {
        tracing::info!("=== CVKG Telemetry Summary ===");
        tracing::info!("  Elapsed: {:.1}s", self.elapsed_secs());
        tracing::info!(
            "  Frames: {} ({:.0} FPS avg)",
            self.frame_count,
            self.average_fps()
        );
        tracing::info!("  Glass elements: {}", self.glass_element_count);
        tracing::info!("  Contrast failures: {}", self.contrast_failure_count);
        tracing::info!("  Budget exceeded: {}", self.budget_exceeded_count);
        tracing::info!("  Total events: {}", self.events.len());

        let a11y = self.accessibility_events();
        if !a11y.is_empty() {
            tracing::info!("  Accessibility issues:");
            for event in &a11y {
                tracing::info!("    - {}", event.description());
            }
        }

        let perf = self.performance_events();
        if !perf.is_empty() {
            tracing::info!("  Performance issues:");
            for event in &perf {
                tracing::info!("    - {}", event.description());
            }
        }
        tracing::info!("=== End Telemetry Summary ===");
    }

    /// Clears all recorded events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

// --- Macros ---

/// Records a telemetry event when the `telemetry` feature is enabled.
/// Compiles to a no-op when the feature is disabled.
#[macro_export]
macro_rules! telemetry {
    ($tel:expr, $event:expr) => {
        #[cfg(feature = "telemetry")]
        {
            $tel.record($event);
        }
    };
}

/// Records a frame completion when the `telemetry` feature is enabled.
#[macro_export]
macro_rules! telemetry_frame {
    ($tel:expr, $frame_time_ms:expr) => {
        #[cfg(feature = "telemetry")]
        {
            $tel.record_frame($frame_time_ms);
        }
    };
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_new() {
        let tel = Telemetry::new();
        assert_eq!(tel.event_count(), 0);
        assert_eq!(tel.frame_count(), 0);
    }

    #[test]
    fn record_contrast_failure() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::ContrastFailure {
            element: "title".into(),
            apca_lc: 42.0,
            foreground: [1.0, 1.0, 1.0, 1.0],
            background: [0.1, 0.1, 0.12, 1.0],
        });
        assert_eq!(tel.event_count(), 1);
        assert_eq!(tel.contrast_failure_count(), 1);
        assert!(tel.events()[0].is_accessibility_issue());
    }

    #[test]
    fn record_reduced_transparency() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::ReducedTransparencyEnabled);
        assert_eq!(tel.event_count(), 1);
        assert!(tel.events()[0].is_accessibility_issue());
    }

    #[test]
    fn record_reduced_motion() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::ReducedMotionEnabled);
        assert_eq!(tel.event_count(), 1);
        assert!(tel.events()[0].is_accessibility_issue());
    }

    #[test]
    fn record_frame_budget_exceeded() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::FrameBudgetExceeded {
            frame_time_ms: 20.0,
            budget_ms: 16.0,
        });
        assert_eq!(tel.event_count(), 1);
        assert_eq!(tel.budget_exceeded_count(), 1);
        assert!(tel.events()[0].is_performance_issue());
    }

    #[test]
    fn record_glass_element() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::GlassElementRendered {
            blur_radius: 16.0,
            rect_area: 4000.0,
        });
        assert_eq!(tel.event_count(), 1);
        assert_eq!(tel.glass_element_count(), 1);
    }

    #[test]
    fn record_small_touch_target() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::SmallTouchTarget {
            element: "icon_button".into(),
            width: 24.0,
            height: 24.0,
        });
        assert_eq!(tel.event_count(), 1);
        assert!(tel.events()[0].is_accessibility_issue());
    }

    #[test]
    fn record_focus_ring_invisible() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::FocusRingInvisible {
            element: "button".into(),
            contrast: 15.0,
        });
        assert_eq!(tel.event_count(), 1);
        assert!(tel.events()[0].is_accessibility_issue());
    }

    #[test]
    fn record_frame_normal() {
        let mut tel = Telemetry::new();
        tel.record_frame(8.0);
        assert_eq!(tel.frame_count(), 1);
        assert_eq!(tel.event_count(), 0); // no budget exceeded
    }

    #[test]
    fn record_frame_exceeded() {
        let mut tel = Telemetry::new();
        tel.record_frame(20.0);
        assert_eq!(tel.frame_count(), 1);
        assert_eq!(tel.event_count(), 1); // budget exceeded event
        assert_eq!(tel.budget_exceeded_count(), 1);
    }

    #[test]
    fn accessibility_events_filter() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::ContrastFailure {
            element: "a".into(),
            apca_lc: 30.0,
            foreground: [1.0; 4],
            background: [0.0; 4],
        });
        tel.record(TelemetryEvent::GlassElementRendered {
            blur_radius: 16.0,
            rect_area: 100.0,
        });
        tel.record(TelemetryEvent::ReducedMotionEnabled);

        let a11y = tel.accessibility_events();
        assert_eq!(a11y.len(), 2); // contrast + motion
    }

    #[test]
    fn performance_events_filter() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::FrameBudgetExceeded {
            frame_time_ms: 20.0,
            budget_ms: 16.0,
        });
        tel.record(TelemetryEvent::GlassElementRendered {
            blur_radius: 16.0,
            rect_area: 100.0,
        });

        let perf = tel.performance_events();
        assert_eq!(perf.len(), 1);
    }

    #[test]
    fn clear_events() {
        let mut tel = Telemetry::new();
        tel.record(TelemetryEvent::ReducedMotionEnabled);
        assert_eq!(tel.event_count(), 1);
        tel.clear();
        assert_eq!(tel.event_count(), 0);
    }

    #[test]
    fn event_descriptions() {
        let e = TelemetryEvent::ContrastFailure {
            element: "title".into(),
            apca_lc: 42.5,
            foreground: [1.0; 4],
            background: [0.0; 4],
        };
        let desc = e.description();
        assert!(desc.contains("title"));
        assert!(desc.contains("42.5"));

        let e2 = TelemetryEvent::ReducedTransparencyEnabled;
        assert!(e2.description().contains("reduced-transparency"));

        let e3 = TelemetryEvent::FrameBudgetExceeded {
            frame_time_ms: 20.0,
            budget_ms: 16.0,
        };
        assert!(e3.description().contains("20.0"));
        assert!(e3.description().contains("16.0"));
    }

    #[test]
    fn average_fps() {
        let mut tel = Telemetry::new();
        // Simulate 60 frames
        for _ in 0..60 {
            tel.record_frame(16.0);
        }
        assert_eq!(tel.frame_count(), 60);
        // FPS will be very high since elapsed time is tiny, but should be > 0
        assert!(tel.average_fps() > 0.0);
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn telemetry_default_constructs() {
        let tel = Telemetry::default();
        assert_eq!(tel.event_count(), 0);
        assert_eq!(tel.frame_count(), 0);
    }

    #[test]
    fn telemetry_event_description() {
        let e = TelemetryEvent::ReducedMotionEnabled;
        assert!(e.description().contains("reduced-motion"));
    }

    #[test]
    fn telemetry_event_is_accessibility() {
        let e = TelemetryEvent::ReducedTransparencyEnabled;
        assert!(e.is_accessibility_issue());
        assert!(!e.is_performance_issue());
    }
}
