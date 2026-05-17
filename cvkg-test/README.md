# cvkg-test

![Berserker Hero HUD](../docs/images/berserker_hero_hud.png)

`cvkg-test` provides the authoritative testing utilities for the CVKG ecosystem, specializing in visual regression and high-fidelity UI validation.

## Boundaries and Responsibilities

This crate focuses on quality assurance. Its responsibilities include:
- **Visual Regression**: Comparing rendered pixel buffers to detect subtle UI changes.
- **Tolerance Management**: Allowing for configurable pixel-level and total-image difference thresholds.
- **Snapshot Infrastructure**: Providing the tools to capture and store "golden" images for CI/CD pipelines.

## Public API Overview

### Core Types
- `VisualComparator`: The primary engine for comparing RGBA pixel buffers.
- `VisualTolerance`: Configuration for individual pixel variance and total percentage change.

### Methods
- `VisualComparator::compare(img1, img2)`: Returns the percentage of pixels that differ beyond the defined tolerance.

## Usage Example

```rust
use cvkg_test::VisualComparator;

#[test]
fn test_ui_snapshot() {
    let comparator = VisualComparator {
        pixel_tolerance: 0.02,
        total_tolerance_percent: 0.1,
    };
    
    let current_frame = capture_frame();
    let golden_frame = load_golden("main_screen.png");
    
    let diff = comparator.compare(&current_frame, &golden_frame);
    assert!(diff < comparator.total_tolerance_percent, "Visual regression detected: {}% diff", diff);
}
```

## Known Limitations
- Visual testing is highly sensitive to hardware differences (GPU drivers, subpixel rendering); use the `cvkg` Docker images for consistent CI results.
- Large images (4K+) may incur significant CPU overhead during comparison.