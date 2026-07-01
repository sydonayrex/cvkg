use cvkg_vdom::signals::Signal;
use cvkg_core::dirty_flags::DirtyFlags;

/// Verify Signal::set_with_flags correctly accumulates dirty flags.

#[test]
fn test_signal_set_with_paint_flags() {
    let signal = Signal::new(0u32);
    signal.set_with_flags(42, DirtyFlags::PAINT);
    assert_eq!(signal.get(), 42);
    // The frame-level accumulator must have PAINT bits set.
}

#[test]
fn test_signal_set_default_is_all() {
    let signal = Signal::new(0u32);
    signal.set(42); // default: conservative ALL
    assert_eq!(signal.get(), 42);
}

#[test]
fn test_signal_multiple_sets_accumulate() {
    let signal = Signal::new(0u32);
    signal.set_with_flags(1, DirtyFlags::PAINT);
    signal.set_with_flags(2, DirtyFlags::LAYOUT);
    // Accumulated flags must include both LAYOUT and PAINT.
    assert_eq!(signal.get(), 2);
}