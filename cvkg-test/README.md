# cvkg-test

**cvkg-test** provides visual testing, benchmarking, and snapshot comparison utilities for CVKG applications.

## What This Crate Does

- Provides visual regression testing with snapshot comparison
- Provides headless rendering for automated tests
- Provides benchmarking utilities for performance measurement
- Implements property-based state testing

## What This Crate Does NOT Do

- Does not provide assertion macros for general use
- Does not provide mocking frameworks
- Does not handle CI configuration

## Public API Overview

### Test Harness

```rust
/// Set up a headless renderer for visual testing
pub fn setup_headless_renderer(width: u32, height: u32) -> SurtrRenderer;

/// Capture a frame from the renderer for comparison
pub fn capture_frame(renderer: &mut SurtrRenderer) -> Vec<u8>;

/// Compare two images for visual differences
pub fn compare_images(expected: &[u8], actual: &[u8]) -> Result<(), VisualDiff>;
```

### Visual Regression

```rust
/// Snapshot testing for visual components
pub struct SnapshotTester {
    snapshots_dir: PathBuf,
}
impl SnapshotTester {
    pub fn new(snapshots_dir: impl Into<PathBuf>) -> Self;
    pub fn assert_snapshot(&self, name: &str, image: &[u8]);
}
```

### Benchmarks

```rust
/// Performance benchmarks for rendering
#[bench] fn bench_large_tree_render(b: &mut Bencher);
#[bench] fn bench_vdom_diff(b: &mut Bencher);
```

## Usage Example

```rust
use cvkg_test::{setup_headless_renderer, compare_images};

#[test]
fn test_button_rendering() {
    let mut renderer = setup_headless_renderer(800, 600);
    // Render and compare
}
```

## Known Limitations

- Snapshot tests require GPU for headless rendering
- Baselines must be updated manually when intentional changes are made
- Performance tests are not deterministic; run multiple times for reliable results