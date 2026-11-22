use cvkg_core::EntityArena;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn observer_fires_exactly_once_after_notify() {
    let mut arena = EntityArena::new();
    let entity = arena.insert(0u32);
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();

    arena.observe(entity, move |value: &u32| {
        if *value == 42 {
            seen.fetch_add(1, Ordering::SeqCst);
        }
    });

    arena.update(entity, |value| *value = 42);
    arena.notify(entity);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn removed_handle_is_invalid_for_reads_and_notifications() {
    let mut arena = EntityArena::new();
    let entity = arena.insert(7u32);
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();

    arena.observe(entity, move |_| {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    arena.remove(entity);
    arena.notify(entity);

    assert_eq!(arena.get(entity), None);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
