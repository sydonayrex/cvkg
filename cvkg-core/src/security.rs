use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// SecurityError defines possible security-related failures.
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Capability denied: {0:?}")]
    CapabilityDenied(Capability),
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
}

/// EnvironmentShield provides low-level hardware timing probes to detect analysis environments.
pub struct EnvironmentShield;

impl EnvironmentShield {
    /// Detects if the current environment exhibits timing anomalies characteristic of a VM or debugger.
    /// Inspired by tailslayer's DRAM refresh and rdtsc timing probes.
    pub fn probe_analysis_risk() -> f32 {
        // Use instruction jitter measurement for baseline analysis
        let jitter = Self::measure_instruction_jitter();
        
        // Detect analysis environment anomalies
        let analysis_detected = Self::detect_analysis_environment();
        
        // Combine signals: jitter variance + analysis detection
        if analysis_detected {
            jitter * 2.0
        } else {
            jitter
        }
    }

    /// Actively enforces mitigations based on the detected analysis risk.
    pub fn enforce_mitigation(risk: f32) {
        if risk > 0.8 {
            log::warn!("CRITICAL ANALYSIS RISK DETECTED ({:.2}): Terminating CVKG Runtime.", risk);
            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(0xDEADC0DEu32 as i32);
            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            panic!("CVKG_SECURITY_TERMINATION_SIGNAL");
        } else if risk > 0.4 {
            log::warn!("MODERATE ANALYSIS RISK DETECTED ({:.2}): Activating Deceptive Shields.", risk);
            Self::inject_timing_noise();
        }
    }

    /// Injects random micro-delays to sabotage side-channel analysis and precise profiling.
    pub fn inject_timing_noise() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::Duration;
            let mut rng = 42u64; // Simple LCG for noise
            for _ in 0..10 {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let nanos = rng % 500 ; // 0-500ns noise
                std::thread::sleep(Duration::from_nanos(nanos));
            }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            // WASM spin-wait since thread::sleep is unavailable
            let mut _x: u64 = 0;
            for i in 0..100000 { _x = _x.wrapping_add(i as u64); }
        }
    }

    fn measure_instruction_jitter() -> f32 {
        let mut samples = Vec::with_capacity(100);
        for _ in 0..100 {
            #[cfg(not(target_arch = "wasm32"))]
            let start = std::time::Instant::now();
            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            let start = js_sys::Date::now();
            #[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
            let start = std::time::SystemTime::now();
            
            let mut _x: u64 = 0;
            for i in 0..1000 { _x = _x.wrapping_add(i as u64); }
            
            #[cfg(not(target_arch = "wasm32"))]
            samples.push(start.elapsed().as_nanos() as f32);
            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            samples.push((js_sys::Date::now() - start) as f32);
            #[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
            samples.push(start.elapsed().map(|d| d.as_nanos() as f32).unwrap_or(0.0));
        }
        
        let avg = samples.iter().sum::<f32>() / samples.len() as f32;
        let variance = samples.iter().map(|s| (s - avg).powi(2)).sum::<f32>() / samples.len() as f32;
        variance.sqrt()
    }

    fn detect_analysis_environment() -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            // SAFETY: _rdtsc is safe to call on x86/x86_64 architectures.
            // It reads the time-stamp counter register which returns a monotonically
            // increasing value. The difference t2-t1 measures cycle count for the
            // instruction execution, which is used to detect timing anomalies
            // characteristic of VM or debugger analysis environments.
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::_rdtsc;
            #[cfg(target_arch = "x86")]
            use std::arch::x86::_rdtsc;
            
            let t1 = _rdtsc();
            let _ = _rdtsc();
            let t2 = _rdtsc();
            (t2 - t1) > 1000
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            // In WASM, check for time clamping (Spectre mitigation)
            let t1 = js_sys::Date::now();
            let mut _x: u64 = 0;
            for i in 0..10000 { _x = _x.wrapping_add(i as u64); }
            let t2 = js_sys::Date::now();
            t1 == t2
        }
        #[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
        { false }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "wasm32")))]
        { false }
    }
}
