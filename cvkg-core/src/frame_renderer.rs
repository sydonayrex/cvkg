use crate::Renderer;
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
