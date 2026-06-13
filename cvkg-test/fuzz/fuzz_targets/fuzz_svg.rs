#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz the usvg parser directly to ensure it doesn't panic on malformed SVGs
    let opt = usvg::Options::default();
    let _ = usvg::Tree::from_data(data, &opt);

    // 2. Fuzz our custom SVG animation parser
    let _ = cvkg_render_gpu::draw::parse_svg_animations(data);
});
