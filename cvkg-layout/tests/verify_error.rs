#[cfg(test)]
mod verify_error {
    use cvkg_layout::LayoutError;

    #[test]
    fn test_constraint_conflict_message() {
        let e = LayoutError::ConstraintConflict {
            node_id: 42,
            reason: "circular flex basis".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("node 42"), "Should contain node id");
        assert!(msg.contains("circular flex basis"), "Should contain reason");
        assert!(msg.contains("Check flex properties"), "Should have guidance text");
    }

    #[test]
    fn test_capacity_exceeded_message() {
        let e = LayoutError::CapacityExceeded("node limit 10000 reached".into());
        let msg = e.to_string();
        assert!(msg.contains("node limit 10000 reached"));
        assert!(msg.contains("Reduce UI complexity"));
    }

    #[test]
    fn test_invalid_float_message() {
        let e = LayoutError::InvalidFloat { node_id: 7 };
        let msg = e.to_string();
        assert!(msg.contains("node 7"));
        assert!(msg.contains("NaN"));
        assert!(msg.contains("intrinsic_size"));
    }

    #[test]
    fn test_internal_message() {
        let e = LayoutError::Internal("taffy returned Err".into());
        let msg = e.to_string();
        assert!(msg.contains("taffy returned Err"));
    }

    #[test]
    fn test_debug_trait() {
        let e = LayoutError::Internal("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("Internal"));
    }

    #[test]
    fn test_std_error_impl() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&LayoutError::Internal("x".into()));
        assert_error(&LayoutError::ConstraintConflict { node_id: 0, reason: "x".into() });
    }
}
