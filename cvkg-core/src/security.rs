use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Capability defines the granular permissions available to plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Permission to make outbound network requests.
    NetworkOutbound,
    /// Permission to listen for inbound network connections.
    NetworkInbound,
    /// Permission to read files from the host system.
    FileRead,
    /// Permission to write files to the host system.
    FileWrite,
    /// Permission to access agentic reasoning capabilities.
    AgentAccess,
    /// Permission to interact with developer tools.
    DevToolsAccess,
}

/// SandboxLimits defines the resource constraints for a plugin.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SandboxLimits {
    pub max_memory_mb: u64,
    pub max_cpu_ms_per_frame: u64,
    pub max_events_per_sec: u32,
    pub max_network_calls_per_sec: u32,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            max_cpu_ms_per_frame: 5,
            max_events_per_sec: 100,
            max_network_calls_per_sec: 10,
        }
    }
}

/// PluginManifest describes a plugin and its required capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub limits: SandboxLimits,
}

/// SecurityPolicy enforces capability-based access control.
pub struct SecurityPolicy {
    allowed_capabilities: Vec<Capability>,
}

impl SecurityPolicy {
    pub fn new(allowed_capabilities: Vec<Capability>) -> Self {
        Self { allowed_capabilities }
    }

    pub fn check_capability(&self, cap: Capability) -> bool {
        self.allowed_capabilities.contains(&cap)
    }

    /// Enforce a capability check, panicking or returning an error if denied.
    pub fn enforce(&self, cap: Capability) -> Result<(), SecurityError> {
        if self.check_capability(cap) {
            Ok(())
        } else {
            log::error!("SECURITY VIOLATION: Unauthorized access to capability {:?}", cap);
            Err(SecurityError::CapabilityDenied(cap))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Capability denied: {0:?}")]
    CapabilityDenied(Capability),
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
}
