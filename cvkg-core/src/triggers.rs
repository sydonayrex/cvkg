//! Typed Event Triggers (Bevy-inspired).
//!
//! Provides type-safe event handlers dispatched by `(NodeId, TypeId)` instead
//! of stringly-typed key lookup. Backward-compatible with the legacy
//! `register_handler("string", arc_fn)` API.

use crate::KvasirId;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for typed event handlers to improve readability.
pub type EventHandler<E> = Arc<dyn Fn(&E, &mut EventCtx) + Send + Sync>;

/// Marker trait for events that can be dispatched through the VDOM trigger registry.
///
/// # Contract
/// Implementors must be `Send + 'static` so handlers can downcast the boxed payload.
pub trait TriggerEvent: Send + 'static {}

/// Context passed to typed event handlers.
///
/// Provides access to the signal registry and re-render request flag.
pub struct EventCtx<'a> {
    /// Triggers already collected during the current dispatch.
    pub request_rerender: bool,
    /// Signal registry scoped to the current render.
    pub signals: Option<&'a mut crate::dirty_flags::DirtyFlags>,
}

impl<'a> EventCtx<'a> {
    /// Create a default event context.
    pub fn new() -> Self {
        Self {
            request_rerender: false,
            signals: None,
        }
    }
}

impl<'a> Default for EventCtx<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Default implementations of common CVKG trigger events.
impl TriggerEvent for crate::event::Event {}

/// Typed registry mapping `(node_id, event_type_id)` → handlers.
pub struct TriggerRegistry {
    handlers: HashMap<(KvasirId, TypeId), Vec<Box<dyn Any + Send + Sync>>>,
}

impl Default for TriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a typed handler for `E` on the given node.
    pub fn on<E: TriggerEvent>(
        &mut self,
        node_id: KvasirId,
        handler: impl Fn(&E, &mut EventCtx) + Send + Sync + 'static,
    ) {
        let arc: EventHandler<E> = Arc::new(handler);
        let key = (node_id, TypeId::of::<E>());
        self.handlers
            .entry(key)
            .or_default()
            .push(Box::new(arc) as Box<dyn Any + Send + Sync>);
    }

    /// Dispatch a typed event to all handlers registered on the node + event type.
    /// Returns whether any handler requested a re-render.
    pub fn dispatch<E: TriggerEvent>(
        &self,
        node_id: KvasirId,
        event: &E,
        ctx: &mut EventCtx,
    ) -> bool {
        let key = (node_id, TypeId::of::<E>());
        let mut rerender = false;
        if let Some(handlers) = self.handlers.get(&key) {
            for h in handlers {
                if let Some(f) = h.downcast_ref::<EventHandler<E>>() {
                    f(event, ctx);
                    if ctx.request_rerender {
                        rerender = true;
                    }
                }
            }
        }
        rerender
    }

    /// Removes all handlers for a particular node (used when a VNode is removed).
    pub fn clear_node(&mut self, node_id: KvasirId) {
        self.handlers.retain(|(id, _), _| *id != node_id);
    }

    /// Total number of registered handlers across all nodes/events.
    pub fn handler_count(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug)]
    struct MyEvent {
        value: u32,
    }
    impl TriggerEvent for MyEvent {}

    #[derive(Debug)]
    struct OtherEvent;
    impl TriggerEvent for OtherEvent {}

    fn node() -> KvasirId {
        KvasirId(42)
    }

    #[test]
    fn test_register_and_dispatch() {
        let counter = Arc::new(AtomicU32::new(0));
        let c1 = counter.clone();
        let mut reg = TriggerRegistry::new();
        reg.on::<MyEvent>(node(), move |event, _ctx| {
            assert_eq!(event.value, 7);
            c1.fetch_add(1, Ordering::SeqCst);
        });
        let result = reg.dispatch::<MyEvent>(node(), &MyEvent { value: 7 }, &mut EventCtx::new());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert!(!result);
    }

    #[test]
    fn test_no_handlers_returns_false() {
        let reg = TriggerRegistry::new();
        let result = reg.dispatch::<MyEvent>(node(), &MyEvent { value: 0 }, &mut EventCtx::new());
        assert!(!result);
    }

    #[test]
    fn test_multiple_handlers() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut reg = TriggerRegistry::new();
        for _ in 0..3 {
            let c = counter.clone();
            reg.on::<MyEvent>(node(), move |_, _| {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }
        reg.dispatch::<MyEvent>(node(), &MyEvent { value: 0 }, &mut EventCtx::new());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_rerender_flag_propagates() {
        let mut reg = TriggerRegistry::new();
        reg.on::<MyEvent>(node(), |_, ctx| {
            ctx.request_rerender = true;
        });
        let result = reg.dispatch::<MyEvent>(node(), &MyEvent { value: 0 }, &mut EventCtx::new());
        assert!(result);
    }

    #[test]
    fn test_clear_node() {
        let mut reg = TriggerRegistry::new();
        reg.on::<MyEvent>(node(), |_, _| {});
        reg.on::<OtherEvent>(node(), |_, _| {});
        assert_eq!(reg.handler_count(), 2);
        reg.clear_node(node());
        assert_eq!(reg.handler_count(), 0);
    }

    #[test]
    fn test_isolated_by_event_type() {
        let counter = Arc::new(AtomicU32::new(0));
        let c1 = counter.clone();
        let mut reg = TriggerRegistry::new();
        reg.on::<MyEvent>(node(), move |_, _| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        // Dispatch a different event type — should not fire handler.
        reg.dispatch::<OtherEvent>(node(), &OtherEvent, &mut EventCtx::new());
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_isolated_by_node_id() {
        let counter = Arc::new(AtomicU32::new(0));
        let c1 = counter.clone();
        let mut reg = TriggerRegistry::new();
        reg.on::<MyEvent>(node(), move |_, _| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        // Different node should not fire.
        reg.dispatch::<MyEvent>(KvasirId(99), &MyEvent { value: 0 }, &mut EventCtx::new());
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
