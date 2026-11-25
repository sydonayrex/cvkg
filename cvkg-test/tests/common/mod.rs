//! Shared GPU availability guard for integration tests.

use fs2::FileExt;
use std::sync::Mutex;
use std::time::Duration;

static GPU_LOCK: Mutex<()> = Mutex::new(());

/// Serialize GPU-bound tests in one binary: concurrent adapter/device creation
/// can crash or hang on driver stacks that are not concurrency-safe.
pub fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Serialize GPU-bound tests across test binaries with an advisory file lock.
pub fn gpu_process_lock() -> std::fs::File {
    let path = std::env::temp_dir().join("cvkg-gpu-tests.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("open GPU test lock");
    file.lock_exclusive().expect("lock GPU test file");
    file
}

/// True when a wgpu adapter responds within five seconds.
pub fn gpu_available() -> bool {
    cvkg_render_gpu::gpu_adapter_available(Duration::from_secs(5))
}
