    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn panicking_subscriber_does_not_poison_mutex() {
        let state = State::new(0i32);
        let fired = Arc::new(AtomicUsize::new(0));

        // First subscriber: panics.
        let _ = state.subscribe(|_| -> () {
            panic!("subscriber 1 explodes");
        });

        // Second subscriber: should still fire.
        let fired_clone = Arc::clone(&fired);
        let _ = state.subscribe(move |v| {
            fired_clone.store(*v as usize + 1, Ordering::SeqCst);
        });

        // Trigger the state change. Subscriber 1 panics; subscriber 2 runs.
        state.set(42);

        assert_eq!(
            fired.load(Ordering::SeqCst),
            43,
            "second subscriber must fire even though first panicked"
        );

        // Critical: future state updates must still work.
        let fired2 = Arc::new(AtomicUsize::new(0));
        let fired2_clone = Arc::clone(&fired2);
        let _ = state.subscribe(move |v| {
            fired2_clone.store(*v as usize, Ordering::SeqCst);
        });
        state.set(100);
        assert_eq!(
            fired2.load(Ordering::SeqCst),
            100,
            "future updates must work after subscriber panic"
        );
    }

    #[test]
    fn all_subscribers_fire_even_if_one_panics() {
        let state = State::new(0u32);
        let count = Arc::new(AtomicUsize::new(0));

        // Mix of panicking and counting subscribers.
        let _ = state.subscribe(|_| panic!("boom 1"));
        let c1 = Arc::clone(&count);
        let _ = state.subscribe(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        let _ = state.subscribe(|_| panic!("boom 2"));
        let c2 = Arc::clone(&count);
        let _ = state.subscribe(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        state.set(1);

        // Both non-panicking subscribers must have fired.
        assert_eq!(
            count.load(Ordering::SeqCst),
            2,
            "both non-panicking subscribers should fire"
        );
    }

    #[test]
    fn invoke_subscribers_safely_returns_count() {
        // Direct unit test of the helper function.
        use std::sync::Mutex;
        let subs: SubscriberList<u32> = Arc::new(Mutex::new(Vec::new()));

        let count1 = Arc::new(AtomicUsize::new(0));
        let count1_clone = Arc::clone(&count1);
        subs.lock().unwrap().push(Box::new(move |v| {
            count1_clone.store(*v as usize, Ordering::SeqCst);
        }));

        let count2 = Arc::new(AtomicUsize::new(0));
        let count2_clone = Arc::clone(&count2);
        subs.lock().unwrap().push(Box::new(move |v| {
            count2_clone.store(*v as usize + 100, Ordering::SeqCst);
        }));

        let invoked = invoke_subscribers_safely(&subs, &7);
        assert_eq!(invoked, 2, "both subscribers should be invoked");
        assert_eq!(count1.load(Ordering::SeqCst), 7);
        assert_eq!(count2.load(Ordering::SeqCst), 107);
    }