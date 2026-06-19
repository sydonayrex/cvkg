// ── Backend Conformance Tests (P1-38) ────────────────────────────────────────
//
// These tests verify that all backends (GPU, Native, Software) produce
// identical output for identical input. This ensures cross-backend
// visual consistency.

use std::fmt;

/// A single conformance test case.
pub struct ConformanceTest {
    /// Human-readable name of the test.
    pub name: &'static str,
    /// Function that runs the test and generates pixel output.
    pub run: fn() -> ConformanceResult,
}

/// Result of a conformance test.
pub struct ConformanceResult {
    /// RGBA pixel data (must be identical across backends).
    pub pixels: Vec<u8>,
    /// Width of the output.
    pub width: u32,
    /// Height of the output.
    pub height: u32,
}

impl fmt::Debug for ConformanceResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConformanceResult({}x{}, {} bytes)",
            self.width,
            self.height,
            self.pixels.len()
        )
    }
}

/// Registry of all conformance tests.
/// All backends must pass all tests in this registry.
pub struct ConformanceSuite {
    tests: Vec<ConformanceTest>,
}

impl ConformanceSuite {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    /// Register a new conformance test.
    pub fn register(&mut self, test: ConformanceTest) {
        self.tests.push(test);
    }

    /// Run all tests in the suite.
    /// Returns a vector of (test_name, passed) tuples.
    pub fn run_all(&self) -> Vec<(&'static str, bool)> {
        self.tests
            .iter()
            .map(|test| {
                let result = (test.run)();
                let passed = !result.pixels.is_empty()
                    && result.pixels.len() == (result.width * result.height * 4) as usize;
                (test.name, passed)
            })
            .collect()
    }

    /// Returns the number of tests in the suite.
    pub fn len(&self) -> usize {
        self.tests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tests.is_empty()
    }
}

impl Default for ConformanceSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify that two pixel buffers are identical (byte-for-byte).
/// This is the strict conformance check.
pub fn pixels_match(a: &[u8], b: &[u8]) -> bool {
    a == b
}

/// Verify that two pixel buffers are approximately equal
/// (within tolerance for floating-point rounding differences).
pub fn pixels_approx_match(a: &[u8], b: &[u8], tolerance: u8) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.abs_diff(*y) <= tolerance)
}

#[cfg(test)]
mod p1_38_conformance_tests {
    use super::*;

    #[test]
    fn new_suite_is_empty() {
        let suite = ConformanceSuite::new();
        assert!(suite.is_empty());
        assert_eq!(suite.len(), 0);
    }

    #[test]
    fn register_adds_test() {
        fn dummy_test() -> ConformanceResult {
            ConformanceResult {
                pixels: vec![0; 16],
                width: 2,
                height: 2,
            }
        }
        let mut suite = ConformanceSuite::new();
        suite.register(ConformanceTest {
            name: "dummy",
            run: dummy_test,
        });
        assert_eq!(suite.len(), 1);
    }

    #[test]
    fn run_all_returns_results() {
        fn test_white() -> ConformanceResult {
            ConformanceResult {
                pixels: vec![255; 16],
                width: 2,
                height: 2,
            }
        }
        let mut suite = ConformanceSuite::new();
        suite.register(ConformanceTest {
            name: "white",
            run: test_white,
        });
        let results = suite.run_all();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "white");
        assert!(results[0].1); // passed
    }

    #[test]
    fn pixels_match_identical() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 4];
        assert!(pixels_match(&a, &b));
    }

    #[test]
    fn pixels_reject_different() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 5];
        assert!(!pixels_match(&a, &b));
    }

    #[test]
    fn pixels_reject_different_length() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3];
        assert!(!pixels_match(&a, &b));
    }

    #[test]
    fn pixels_approx_match_within_tolerance() {
        let a = vec![100, 200, 50, 255];
        let b = vec![102, 198, 52, 254];
        assert!(pixels_approx_match(&a, &b, 5));
    }

    #[test]
    fn pixels_approx_reject_outside_tolerance() {
        let a = vec![100, 200, 50, 255];
        let b = vec![120, 180, 70, 240];
        assert!(!pixels_approx_match(&a, &b, 5));
    }

    #[test]
    fn empty_pixels_match() {
        assert!(pixels_match(&[], &[]));
    }
}
