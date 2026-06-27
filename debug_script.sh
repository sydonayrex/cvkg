sed -i 's/assert!(neon_pixels > 0/println!("Neon at center: {:?}", \&pixels[(150 * 512 + 150) * 4 .. (150 * 512 + 150) * 4 + 4]); assert!(neon_pixels > 0/' /D/rex/projects/cvkg/cvkg-render-gpu/tests/advanced_engine_scenarios.rs
cargo test --package cvkg-render-gpu --test advanced_engine_scenarios test_advanced_vdom_with_glassmorphism -- --nocapture
