//! Shared GPU availability guard for integration tests.

use std::sync::Mutex;
use std::time::Duration;

static GPU_LOCK: Mutex<()> = Mutex::new(());

/// Serialize GPU-bound tests in one binary: concurrent adapter/device creation
/// can crash or hang on driver stacks that are not concurrency-safe.
pub fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// True when a wgpu adapter responds within five seconds.
pub fn gpu_available() -> bool {
    cvkg_render_gpu::gpu_adapter_available(Duration::from_secs(5))
}
