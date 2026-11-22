use cvkg_core::Limits;

#[test]
fn limits_reject_declared_sizes_before_allocation() {
    let limits = Limits {
        max_bytes: 16,
        max_depth: 4,
        max_nodes: 10,
        max_attributes: 8,
    };

    assert!(limits.checked_len(16).is_ok());
    assert!(limits.checked_len(17).is_err());
    assert!(limits.checked_count(10, "nodes").is_ok());
    assert!(limits.checked_count(11, "nodes").is_err());
    assert!(limits.checked_slice(&[0u8; 16]).is_ok());
    assert!(limits.checked_slice(&[0u8; 17]).is_err());
}
