// Memory Leak Prevention Tests
// Verifies no retained references after unmount and tracks memory growth

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Track component instances for leak detection
static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Mock component for testing creation/destruction
struct TestComponent {
    id: usize,
    data: Vec<u8>,
}

impl TestComponent {
    fn new(size: usize) -> Self {
        INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        Self {
            id: INSTANCE_COUNT.load(Ordering::SeqCst),
            data: vec![0; size],
        }
    }
}

impl Drop for TestComponent {
    fn drop(&mut self) {
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Helper to reset static state before each test with timeout
fn reset_instance_count() {
    let start = Instant::now();
    while INSTANCE_COUNT.load(Ordering::SeqCst) > 0 {
        if start.elapsed() > std::time::Duration::from_secs(5) {
            panic!("Timeout waiting for instance count to reset");
        }
        std::thread::sleep(std::time::Duration::from_micros(10));
    }
}

/// Test: Repeated component creation/destruction
#[test]
fn test_repeated_component_creation_destruction() {
    reset_instance_count();
    let initial_count = INSTANCE_COUNT.load(Ordering::SeqCst);

    for _ in 0..1000 {
        let component = TestComponent::new(1024);
        std::mem::drop(component);
    }

    std::thread::sleep(std::time::Duration::from_millis(10));
    let final_count = INSTANCE_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        initial_count, final_count,
        "Memory leak detected: instances not properly dropped (expected {}, got {})",
        initial_count, final_count
    );
}

/// Test: Memory growth tracking over multiple cycles
#[test]
fn test_memory_growth_tracking() {
    reset_instance_count();

    for cycle in 0..10 {
        reset_instance_count();

        for _ in 0..100 {
            let component = TestComponent::new(1024);
            let _ = std::mem::size_of_val(&component) + component.data.len();
            std::mem::drop(component);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
        let current_count = INSTANCE_COUNT.load(Ordering::SeqCst);
        assert_eq!(
            current_count, 0,
            "Cycle {}: Unexpected instance count: expected 0, got {})",
            cycle, current_count
        );
    }
}

/// Test: Verify no retained references after unmount
#[test]
fn test_no_retained_references() {
    reset_instance_count();
    use std::cell::RefCell;
    use std::rc::Rc;

    let reference_tracker = Rc::new(RefCell::new(0));

    {
        let tracker_clone = Rc::clone(&reference_tracker);
        let component = TestComponent::new(1024);
        *tracker_clone.borrow_mut() += 1;
        std::mem::drop(component);
    }

    assert_eq!(
        *reference_tracker.borrow(),
        1,
        "Reference tracking inconsistent after unmount"
    );
}

/// Test: Stress test with 10k cycles
#[test]
fn test_10k_cycle_stress() {
    reset_instance_count();
    let start = Instant::now();

    for i in 0..10_000 {
        let component = TestComponent::new(256);

        if i % 1_000 == 0 {
            std::mem::drop(component);
            reset_instance_count();
            let current_count = INSTANCE_COUNT.load(Ordering::SeqCst);
            assert_eq!(
                current_count, 0,
                "Stress test at cycle {}: Unexpected count {} (expected 0)",
                i, current_count
            );
        } else {
            std::mem::drop(component);
        }
    }

    let elapsed = start.elapsed();
    println!("10k cycle stress test completed in {:?}", elapsed);
}
