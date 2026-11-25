//! Shared platform contract implemented by each CVKG render target.

/// Capabilities a render host must expose.
pub trait Platform: Send + Sync {
    /// Stable target name used in diagnostics and tests.
    fn name(&self) -> &'static str;

    /// Whether the target runs without a native window server.
    fn is_headless(&self) -> bool;
}
