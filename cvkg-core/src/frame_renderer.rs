use crate::Renderer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Type for frame callback lists.
pub(crate) type SubscriberList<T> = Arc<std::sync::Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>>;

pub trait FrameRenderer<E = ()>: Renderer {
    fn begin_frame(&mut self) -> E;
    fn render_frame(&mut self) {
        // Default implementation does nothing - override for custom frame rendering
    }
    fn end_frame(&mut self, encoder: E);
}

/// Phase 3b/3c: thread-local translation set by `NativeRenderer::push_vnode`
/// (and friends). GPU-side primitives that already draw screen-pixel
/// rects/lines/text can read this via `current_renderer_translation()` and
/// add it to emitted vertices. Default is `(0.0, 0.0)`, so components and
/// primitives that don't opt in are unaffected.
///
/// Set by `push_vnode`/`push_translation` on the parent Renderer
/// (typically `NativeRenderer`); cleared by `pop_vnode`/`pop_translation`.
///
/// Packed as a `u64` so the static is naturally `Sync` (Cell is not).
/// Bit 0..=32 = `f32 x.reinterpret_as_u32()`, bit 33..=64 = `f32 y`.
/// Atomic with Relaxed ordering — racing writers produce benign overlap;
/// we don't need ordering against other memory.
#[doc(hidden)]
pub static RENDERER_TRANSLATION: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
fn pack_translation(t: glam::Vec2) -> u64 {
    let x_bits = t.x.to_bits() as u64;
    let y_bits = t.y.to_bits() as u64;
    (y_bits << 32) | x_bits
}

#[inline(always)]
fn unpack_translation(bits: u64) -> glam::Vec2 {
    let x = f32::from_bits(bits as u32);
    let y = f32::from_bits((bits >> 32) as u32);
    glam::Vec2::new(x, y)
}

/// Read the current cumulative translation pushed by the parent
/// Renderer via `push_translation` / `push_vnode`. Returns
/// `glam::Vec2::ZERO` if no translation is active.
#[inline(always)]
pub fn current_renderer_translation() -> glam::Vec2 {
    unpack_translation(RENDERER_TRANSLATION.load(Ordering::Relaxed))
}

/// Set the current cumulative translation. Called by the parent
/// Renderer's `push_vnode` / `push_translation`. Components and
/// primitives should not call this directly.
#[inline(always)]
#[doc(hidden)]
pub fn set_renderer_translation(translation: glam::Vec2) {
    RENDERER_TRANSLATION.store(pack_translation(translation), Ordering::Relaxed);
}

/// Safely invoke all subscribers with a value, returning the count of successful invocations.
pub(crate) fn invoke_subscribers_safely<T>(subs: &SubscriberList<T>, val: &T) -> usize
where
    T: 'static,
{
    let guard = match subs.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            tracing::warn!("[invoke_subscribers_safely] subscriber lock poisoned, recovering");
            poisoned.into_inner()
        }
    };
    for cb in guard.iter() {
        cb(val);
    }
    guard.len()
}

/// Invoke only subscribers with the given component IDs.
/// The subscriber list is indexed by component ID (position in vec).
pub(crate) fn invoke_subscribers_by_id<T>(subs: &SubscriberList<T>, val: &T, ids: &[u64]) -> usize
where
    T: 'static,
{
    let guard = match subs.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            tracing::warn!("[invoke_subscribers_by_id] subscriber lock poisoned, recovering");
            poisoned.into_inner()
        }
    };
    let mut count = 0;
    for id in ids {
        if let Some(cb) = guard.get(*id as usize) {
            cb(val);
            count += 1;
        }
    }
    count
}
