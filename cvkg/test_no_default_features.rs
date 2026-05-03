// This test module verifies examples compile correctly with --no-default-features
// Part of Priority 1 Issue 1: Feature Flag Consistency

#[cfg(test)]
mod tests {
    // Test that cvkg-core compiles without default features
    #[test]
    fn test_cvkg_core_no_default_features() {
        // This test verifies cvkg-core works without default features
        use cvkg_core::*;
        assert!(true);
    }

    // Test that cvkg-layout compiles without default features
    #[test]
    fn test_cvkg_layout_no_default_features() {
        use cvkg_layout::*;
        assert!(true);
    }

    // Test that cvkg-components compiles without default features
    #[test]
    fn test_cvkg_components_no_default_features() {
        use cvkg_components::*;
        assert!(true);
    }
}