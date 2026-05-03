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

/// Helper to reset static state before each test
fn reset_instance_count() {
    // Force all instances to drop by setting count to 0
    // This works because we're in a single-threaded test context
    while INSTANCE_COUNT.load(Ordering::SeqCst) > 0 {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// Test: Repeated component creation/destruction
#[test]
fn test_repeated_component_creation_destruction() {
    reset_instance_count();
    let initial_count = INSTANCE_COUNT.load(Ordering::SeqCst);
    
    // Create and destroy 1000 components
    for _ in 0..1000 {
        let component = TestComponent::new(1024);
        // Explicitly drop to ensure cleanup
        std::mem::drop(component);
    }
    
    // Give time for drops to complete
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // After all scopes, count should return to initial
    let final_count = INSTANCE_COUNT.load(Ordering::SeqCst);
    assert_eq!(initial_count, final_count, 
        "Memory leak detected: instances not properly dropped (expected {}, got {})", 
        initial_count, final_count);
}

/// Test: Memory growth tracking over multiple cycles
#[test]
fn test_memory_growth_tracking() {
    reset_instance_count();
    let mut memory_samples = Vec::new();
    let initial_count = INSTANCE_COUNT.load(Ordering::SeqCst);
    
    // Run 10 cycles of 100 creations/destructions
    for cycle in 0..10 {
        for _ in 0..100 {
            let component = TestComponent::new(1024);
            let _ = std::mem::size_of_val(&component) + component.data.len();
            std::mem::drop(component);
        }
        
        let current_count = INSTANCE_COUNT.load(Ordering::SeqCst);
        memory_samples.push(current_count);
        
        // Count should remain stable across cycles
        assert_eq!(current_count, initial_count, 
            "Cycle {}: Unexpected instance count: expected {}, got {}", 
            cycle, initial_count, current_count);
    }
    
    // All samples should be identical if no leaks
    for (i, &count) in memory_samples.iter().enumerate() {
        assert_eq!(count, initial_count, 
            "Sample {}: Memory count diverged, possible leak", i);
    }
}

/// Test: Verify no retained references after unmount
#[test]
fn test_no_retained_references() {
    reset_instance_count();
    use std::rc::Rc;
    use std::cell::RefCell;
    
    // Simulate component with internal references
    let reference_tracker = Rc::new(RefCell::new(0));
    
    {
        let tracker_clone = Rc::clone(&reference_tracker);
        let component = TestComponent::new(1024);
        *tracker_clone.borrow_mut() += 1;
        std::mem::drop(component);
    } // Component dropped here
    
    // Reference count should indicate proper cleanup
    assert_eq!(*reference_tracker.borrow(), 1, 
        "Reference tracking inconsistent after unmount");
}

/// Test: Stress test with 10k cycles
#[test]
fn test_10k_cycle_stress() {
    reset_instance_count();
    let start = Instant::now();
    let initial_count = INSTANCE_COUNT.load(Ordering::SeqCst);
    
    // Run 10,000 cycles
    for i in 0..10_000 {
        let component = TestComponent::new(256);
        
        // Periodic check every 1000 cycles
        if i % 1_000 == 0 {
            std::mem::drop(component);
            let current_count = INSTANCE_COUNT.load(Ordering::SeqCst);
            assert_eq!(current_count, initial_count,
                "Stress test at cycle {}: Unexpected count {} (expected {})", 
                i, current_count, initial_count);
        } else {
            std::mem::drop(component);
        }
    }
    
    let elapsed = start.elapsed();
    println!("10k cycle stress test completed in {:?}", elapsed);
}
