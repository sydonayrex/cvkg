import re

with open('/D/rex/projects/cvkg/cvkg-render-gpu/tests/integrated_ui_scenarios.rs', 'r') as f:
    code = f.read()

# Add benchmark_and_capture function if not present
if "fn benchmark_and_capture" not in code:
    insertion_idx = code.find("#[test]")
    helper_fn = """
/// Benchmark and capture frame. Runs the provided layout function for 60 frames,
/// measures average frame time to ensure >= 60 FPS, and returns the final captured pixels.
fn benchmark_and_capture(
    renderer: &mut GpuRenderer,
    mut layout_fn: impl FnMut(&mut GpuRenderer),
) -> Vec<u8> {
    let start_time = std::time::Instant::now();
    for _ in 0..60 {
        layout_fn(renderer);
    }
    let elapsed = start_time.elapsed();
    let avg_ms = (elapsed.as_secs_f32() / 60.0) * 1000.0;
    assert!(avg_ms <= 16.7, "Performance degradation: UI rendering failed to maintain 60 FPS. Average frame took {:.2}ms", avg_ms);
    
    // One more frame and readback
    layout_fn(renderer);
    pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame")
}

"""
    code = code[:insertion_idx] + helper_fn + code[insertion_idx:]

# We want to replace the block:
#     let encoder = renderer.begin_frame_headless();
#     ... layout ...
#     renderer.render_frame();
#     renderer.end_frame(encoder);
#     let pixels = capture_frame(&mut renderer);
# With:
#     let pixels = benchmark_and_capture(&mut renderer, |renderer| {
#         let encoder = renderer.begin_frame_headless();
#         ... layout ...
#         renderer.render_frame();
#         renderer.end_frame(encoder);
#     });

def replace_test_body(match):
    full_match = match.group(0)
    
    # Extract the parts
    before_encoder = match.group(1)
    layout = match.group(2)
    after_capture = match.group(3)
    
    # We also need to fix Z-indexes for layered items automatically if we can, or just let them be and use coverage tests.
    
    new_body = f"{before_encoder}    let pixels = benchmark_and_capture(&mut renderer, |renderer| {{\n        let encoder = renderer.begin_frame_headless();{layout}        renderer.render_frame();\n        renderer.end_frame(encoder);\n    }});\n{after_capture}"
    return new_body

pattern = r"(    let mut renderer = [^\n]+\n.*?)(    let encoder = renderer\.begin_frame_headless\(\);\n(.*?)(?:    //.*\n)*        renderer\.render_frame\(\);\n        renderer\.end_frame\(encoder\);\n)(?:    let pixels = capture_frame\(&mut renderer\);\n)"

code = re.sub(pattern, replace_test_body, code, flags=re.DOTALL)

with open('/D/rex/projects/cvkg/cvkg-render-gpu/tests/integrated_ui_scenarios.rs', 'w') as f:
    f.write(code)

print("Transform applied")
