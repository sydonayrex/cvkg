use cvkg_vdom::signals::{create_signal, create_effect};
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_signal_cross_thread() {
    let (get_val, set_val) = create_signal(0);
    
    // We'll capture the latest emitted value in a thread-safe mutex
    let latest_val = Arc::new(Mutex::new(0));
    let latest_val_clone = Arc::clone(&latest_val);

    // Create an effect that runs immediately and re-runs when get_val changes
    create_effect(move || {
        let val = get_val();
        let mut l = latest_val_clone.lock().unwrap();
        *l = val;
    });

    // Initial effect run should have populated it with 0
    assert_eq!(*latest_val.lock().unwrap(), 0);

    // Spawn a background thread to mutate the signal
    let handle = thread::spawn(move || {
        set_val(42);
        set_val(100);
    });

    handle.join().unwrap();

    // Verify the effect captured the final mutation from the background thread
    assert_eq!(*latest_val.lock().unwrap(), 100);
}
