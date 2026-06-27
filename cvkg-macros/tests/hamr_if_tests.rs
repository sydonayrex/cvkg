use cvkg_macros::hamr_if;

#[test]
fn test_hamr_if_else_compiles() {
    // Verify the hamr_if! macro generates valid conditional code
    let condition = true;
    let result: i32 = hamr_if!((condition) {
        42
    } else {
        0
    });
    assert_eq!(result, 42);
}

#[test]
fn test_hamr_if_else_false_branch() {
    let condition = false;
    let result: i32 = hamr_if!((condition) {
        42
    } else {
        99
    });
    assert_eq!(result, 99);
}
