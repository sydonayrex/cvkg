//! Custom error types for CVKG UI components
//! Provides better error messages with span information and suggested fixes

use std::fmt;

/// Error types for common CVKG mistakes
#[derive(Debug, Clone)]
pub enum CvkgError {
    /// Component failed to render due to invalid geometry
    InvalidGeometry {
        rect: String,
        reason: String,
        suggestion: String,
    },
    /// Missing required feature flag
    MissingFeature {
        feature: String,
        crate_name: String,
        suggestion: String,
    },
    /// Invalid view composition
    InvalidViewComposition {
        view_type: String,
        parent_type: String,
        suggestion: String,
    },
    /// Renderer initialization failed
    RendererInitFailed {
        backend: String,
        reason: String,
        suggestion: String,
    },
}

impl fmt::Display for CvkgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CvkgError::InvalidGeometry {
                rect,
                reason,
                suggestion,
            } => {
                write!(
                    f,
                    "Invalid geometry: {} - {}. Suggestion: {}",
                    rect, reason, suggestion
                )
            }
            CvkgError::MissingFeature {
                feature,
                crate_name,
                suggestion,
            } => {
                write!(
                    f,
                    "Missing feature '{}' in crate {}. {}. Run: cargo build -p {} --features {}",
                    feature, crate_name, suggestion, crate_name, feature
                )
            }
            CvkgError::InvalidViewComposition {
                view_type,
                parent_type,
                suggestion,
            } => {
                write!(
                    f,
                    "Cannot use {} inside {}. {}. Check the parent-child compatibility rules.",
                    view_type, parent_type, suggestion
                )
            }
            CvkgError::RendererInitFailed {
                backend,
                reason,
                suggestion,
            } => {
                write!(
                    f,
                    "Failed to initialize {} renderer: {}. {}",
                    backend, reason, suggestion
                )
            }
        }
    }
}

impl std::error::Error for CvkgError {}

/// Helper for creating geometry errors with suggested fixes
pub fn invalid_geometry_error(rect: &str, reason: &str) -> CvkgError {
    let suggestion = match reason {
        r if r.contains("negative") => "Ensure width and height are positive values".to_string(),
        r if r.contains("zero") => {
            "Check that dimensions are non-zero before rendering".to_string()
        }
        _ => "Verify rectangle dimensions are valid".to_string(),
    };
    CvkgError::InvalidGeometry {
        rect: rect.to_string(),
        reason: reason.to_string(),
        suggestion,
    }
}

/// Helper for creating feature flag errors with suggested fixes
pub fn missing_feature_error(feature: &str, crate_name: &str) -> CvkgError {
    let suggestion = match feature {
        "gpu" => "GPU renderer required for this feature".to_string(),
        "native" => "Native window system required".to_string(),
        "web" => "Web assembly target required".to_string(),
        _ => "Enable the required feature flag".to_string(),
    };
    CvkgError::MissingFeature {
        feature: feature.to_string(),
        crate_name: crate_name.to_string(),
        suggestion,
    }
}

/// Error span information for better diagnostics
#[derive(Debug, Clone)]
pub struct ErrorSpan {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub file: String,
}

impl ErrorSpan {
    pub fn new(file: &str, line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col + 10,
            file: file.to_string(),
        }
    }
}

/// Enhanced error with span information
#[derive(Debug)]
pub struct SpannedError {
    pub error: CvkgError,
    pub span: ErrorSpan,
    pub help: String,
}

impl fmt::Display for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}:{}-{}:{}\n  Help: {}",
            self.error,
            self.span.file,
            self.span.start_line,
            self.span.start_col,
            self.span.end_line,
            self.span.end_col,
            self.help
        )
    }
}
