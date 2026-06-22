//! Visual components for CVKG.
//!
//! Progress bars, spinners, status bars, analytics displays, and decorative elements.

pub mod progress;
pub mod status_bar;
pub mod analytics;
pub mod telemetry;
pub mod well;
pub mod spinner;
pub mod empty_state;
pub mod carousel;
pub mod decorators;
pub mod effects;

pub use carousel::HatiCarousel;
pub use empty_state::EmptyState;
pub use progress::{ProgressVariant, SkollProgress};
pub use spinner::{HatiSpinner, SpinnerVariant};
pub use status_bar::StatusBar;
pub use telemetry::TelemetryView;
pub use well::MimirsWell;
pub use analytics::{ChartType, ValkyrieAnalytics};
pub use decorators::*;
pub use effects::*;
