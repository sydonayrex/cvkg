use cvkg_core::dirty_flags::DirtyFlags;

/// Verify DirtyFlags constants have correct bitmasks and downstream propagation.

#[test]
fn test_dirty_flags_values() {
    assert_eq!(DirtyFlags::STATE.0, 0b1111);
    assert_eq!(DirtyFlags::LAYOUT.0, 0b0111);
    assert_eq!(DirtyFlags::PAINT.0, 0b0011);
    assert_eq!(DirtyFlags::COMPOSITE.0, 0b0001);
}

#[test]
fn test_downstream_propagation() {
    // STATE must imply all downstream layers.
    assert!(DirtyFlags::STATE.implies(DirtyFlags::LAYOUT));
    assert!(DirtyFlags::STATE.implies(DirtyFlags::PAINT));
    assert!(DirtyFlags::STATE.implies(DirtyFlags::COMPOSITE));

    // LAYOUT must imply PAINT and COMPOSITE.
    assert!(DirtyFlags::LAYOUT.implies(DirtyFlags::PAINT));
    assert!(DirtyFlags::LAYOUT.implies(DirtyFlags::COMPOSITE));

    // PAINT must imply COMPOSITE.
    assert!(DirtyFlags::PAINT.implies(DirtyFlags::COMPOSITE));

    // COMPOSITE does NOT imply anything upstream.
    assert!(!DirtyFlags::COMPOSITE.implies(DirtyFlags::PAINT));
    assert!(!DirtyFlags::COMPOSITE.implies(DirtyFlags::LAYOUT));
}

#[test]
fn test_needs_layout() {
    assert!(DirtyFlags::STATE.needs_layout());
    assert!(DirtyFlags::LAYOUT.needs_layout());
    assert!(!DirtyFlags::PAINT.needs_layout());
    assert!(!DirtyFlags::COMPOSITE.needs_layout());
}

#[test]
fn test_needs_state() {
    assert!(DirtyFlags::STATE.needs_state());
    assert!(!DirtyFlags::LAYOUT.needs_state());
    assert!(!DirtyFlags::PAINT.needs_state());
}
