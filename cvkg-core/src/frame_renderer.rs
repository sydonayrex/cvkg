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
            eprintln!("[invoke_subscribers_safely] subscriber lock poisoned, recovering");
            poisoned.into_inner()
        }
    };
    for cb in guard.iter() {
        cb(val);
    }
    guard.len()
}
