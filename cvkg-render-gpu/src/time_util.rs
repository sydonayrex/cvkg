//! Target-appropriate monotonic clock for the renderer (P1-4).
//!
//! `std::time::Instant::now()` panics on `wasm32-unknown-unknown`. Web code
//! uses `js_sys::Date` instead; native keeps `Instant` with full precision.

#[cfg(not(target_arch = "wasm32"))]
pub type TimeStamp = std::time::Instant;
#[cfg(target_arch = "wasm32")]
pub type TimeStamp = f64;

/// Current timestamp.
#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> TimeStamp {
    std::time::Instant::now()
}
#[cfg(target_arch = "wasm32")]
pub fn now() -> TimeStamp {
    js_sys::Date::now() / 1000.0
}

/// Seconds elapsed since `since`, always finite.
#[cfg(not(target_arch = "wasm32"))]
pub fn elapsed_since(since: &TimeStamp) -> f32 {
    since.elapsed().as_secs_f32()
}
#[cfg(target_arch = "wasm32")]
pub fn elapsed_since(since: &TimeStamp) -> f32 {
    (now() - *since) as f32
}

/// Nanoseconds elapsed since `since` (used for deterministic RNG seeding).
#[cfg(not(target_arch = "wasm32"))]
pub fn elapsed_nanos(since: &TimeStamp) -> u64 {
    since.elapsed().as_nanos() as u64
}
#[cfg(target_arch = "wasm32")]
pub fn elapsed_nanos(since: &TimeStamp) -> u64 {
    ((now() - *since) * 1_000_000_000.0) as u64
}
