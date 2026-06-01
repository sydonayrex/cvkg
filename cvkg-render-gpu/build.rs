use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs::read_to_string;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hash all shader source files so cargo detects changes.
    // The actual wgpu shader module is created at runtime from WGSL_SRC in lib.rs.
    let wgsl_src = [
        read_to_string("src/shaders/common.wgsl")?,
        read_to_string("src/shaders/shapes.wgsl")?,
        read_to_string("src/shaders/bifrost.wgsl")?,
        read_to_string("src/shaders/bloom.wgsl")?,
        read_to_string("src/shaders/color_blind.wgsl")?,
    ]
    .join("\n");

    let mut hasher = DefaultHasher::new();
    wgsl_src.hash(&mut hasher);
    let _current_hash = hasher.finish();

    // Write a marker so cargo knows when to rerun
    let _dest_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("shader_hash.rs");

    println!("cargo:rerun-if-changed=src/shaders");
    Ok(())
}
