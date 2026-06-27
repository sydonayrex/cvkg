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
    /// Runtime renderer error from a backend
    RendererError {
        backend: String,
        message: String,
        suggestion: String,
    },
    /// Layout constraint violation or computation failure
    LayoutError {
        node_id: Option<u64>,
        message: String,
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
            CvkgError::RendererError {
                backend,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "[{}] {}. {}",
                    backend, message, suggestion
                )
            }
            CvkgError::LayoutError {
                node_id,
                message,
                suggestion,
            } => {
                let node = node_id.map_or_else(|| "unknown".to_string(), |n| format!("0x{n:X}"));
                write!(
                    f,
                    "Layout error (node {}): {}. {}",
                    node, message, suggestion
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_geometry_display_includes_reason_and_suggestion() {
        let err = CvkgError::InvalidGeometry {
            rect: "Rect { x: 0, y: 0, w: -1, h: 10 }".into(),
            reason: "negative width".into(),
            suggestion: "Ensure width and height are positive".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("negative width"), "should contain reason");
        assert!(msg.contains("positive"), "should contain suggestion");
    }

    #[test]
    fn renderer_error_display_includes_backend_and_message() {
        let err = CvkgError::RendererError {
            backend: "gpu".into(),
            message: "device lost".into(),
            suggestion: "recreate renderer".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[gpu]"), "should tag backend");
        assert!(msg.contains("device lost"), "should contain message");
        assert!(msg.contains("recreate renderer"), "should contain suggestion");
    }

    #[test]
    fn layout_error_display_includes_node_id() {
        let err = CvkgError::LayoutError {
            node_id: Some(0xABCD),
            message: "constraint conflict".into(),
            suggestion: "check flex properties".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("0xABCD"), "should format node ID as hex");
        assert!(msg.contains("constraint conflict"), "should contain message");
    }

    #[test]
    fn layout_error_display_handles_none_node_id() {
        let err = CvkgError::LayoutError {
            node_id: None,
            message: "NaN propagated".into(),
            suggestion: "check floats".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unknown"), "should handle None node ID");
    }

    #[test]
    fn spanned_error_includes_location_info() {
        let err = SpannedError {
            error: CvkgError::MissingFeature {
                feature: "gpu".into(),
                crate_name: "cvkg-render-gpu".into(),
                suggestion: "enable gpu feature".into(),
            },
            span: ErrorSpan::new("button.rs", 42, 10),
            help: "Add feature to Cargo.toml".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("button.rs"), "should contain file");
        assert!(msg.contains("42"), "should contain line");
        assert!(msg.contains("Add feature to Cargo.toml"), "should contain help");
    }

    #[test]
    fn error_trait_bound_satisfied() {
        // Verify CvkgError satisfies std::error::Error
        let _boxed: Box<dyn std::error::Error> = Box::new(CvkgError::RendererInitFailed {
            backend: "test".into(),
            reason: "test failure".into(),
            suggestion: "fix it".into(),
        });
    }
}
