// Standalone reproducer for bugs found in cvkg-flow audit
// Run: rustc audit_repro.rs -o audit_repro && ./audit_repro

use std::f32;

fn main() {
    bug1_spline_easing_elastic_broken();
    bug2_negative_hue_wrapping();
    bug3_ribbon_tangent_panic();
    println!("\n=== All audit reproducers completed ===");
}

// BUG 1: SplineEasing::elastic() is defeated by constructor clamping
fn bug1_spline_easing_elastic_broken() {
    println!("=== BUG 1: SplineEasing elastic easing broken ===");

    // Simulating SplineEasing::new(0.68, -0.55, 0.27, 1.55)
    let x1 = 0.68f32.clamp(0.0, 1.0);  // 0.68
    let y1 = (-0.55f32).clamp(0.0, 1.0); // CLAMPED to 0.0! Was -0.55
    let x2 = 0.27f32.clamp(0.0, 1.0);  // 0.27
    let y2 = 1.55f32.clamp(0.0, 1.0);  // CLAMPED to 1.0! Was 1.55

    println!("  Expected elastic control points: (0.68, -0.55, 0.27, 1.55)");
    println!("  Actual after clamp:               ({:.3}, {:.3}, {:.3}, {:.3})", x1, y1, x2, y2);
    assert_eq!(y1, 0.0, "y1=-0.55 was clamped to 0.0 -- elastic overshoot LOST!");
    assert_eq!(y2, 1.0, "y2=1.55 was clamped to 1.0 -- elastic overshoot LOST!");
    println!("  => CONFIRMED: elastic easing behaves identically to a non-elastic curve\n");
}

// BUG 2: OklchColor::with_hue() doesn't handle negative hue
fn bug2_negative_hue_wrapping() {
    println!("=== BUG 2: OklchColor negative hue handling ===");

    // Simulating OklchColor::with_hue() -> "h: h % 360.0"
    fn wrap_hue(h: f32) -> f32 {
        h % 360.0
    }

    let tests = [
        (0.0f32, 0.0),      // fine
        (400.0, 40.0),       // fine: 400 % 360 = 40
        (-90.0, 270.0),      // EXPECT 270, but Rust % gives -90.0
        (-360.0, 0.0),       // fine: -360 % 360 = 0
        (-450.0, 270.0),     // EXPECT 270, but Rust % gives -90.0
    ];

    for (input, expected) in tests {
        let actual = wrap_hue(input);
        let correct = (actual - expected).abs() < 0.001;
        println!("  wrap_hue({:.1}) => {:.1} (expected {:.1}) {}", input, actual, expected,
                 if correct { "OK" } else { "FAIL" });
        if !correct {
            println!("  => CONFIRMED: negative hue wraps to negative value instead of [0, 360)");
        }
    }
    println!();
}

// BUG 3: Ribbon tangent code panics on 0/1-segment curves
fn bug3_ribbon_tangent_panic() {
    println!("=== BUG 3: Ribbon tangent panics on small curves ===");

    // Simulating the tangent logic in build_ribbon_batch
    // Uses tessellate_bezier_uniform or tessellate_bezier with 0/1 segments
    fn compute_tangents(points: &[f32]) {
        for i in 0..points.len() {
            // This is the exact logic from ribbon.rs build_ribbon_batch lines 294-298
            if i + 1 < points.len() {
                // Forward difference - safe
            } else {
                // BACKWARD DIFFERENCE: points[i - 1] with i=0 => PANIC!
                let _val = points[i - 1]; // i-1 = usize::MAX when i=0
            }
        }
    }

    // Test with 1 point (segments=0 case)
    let single_point = vec![0.0f32];
    println!("  Testing tangent code with 1 point (segments=0)...");
    let panicked = std::panic::catch_unwind(|| {
        compute_tangents(&single_point);
    });
    if panicked.is_err() {
        println!("  => CONFIRMED: PANIC on 1-point curve (segments=0)\n");
    } else {
        println!("  No panic (unexpected)");
    }

    // Test with 2 points (segments=1 case)
    let two_points = vec![0.0f32, 1.0f32];
    println!("  Testing tangent code with 2 points (segments=1)...");
    let panicked = std::panic::catch_unwind(|| {
        compute_tangents(&two_points);
    });
    if panicked.is_err() {
        println!("  => CONFIRMED: PANIC on 2-point curve (segments=1)\n");
    } else {
        println!("  No panic");
    }
}
