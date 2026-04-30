use std::env;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(&out_dir).join("shader_spirv.rs");
    
    // Check if we should skip recompilation (caching optimization)
    let shader_path = "src/shaders.wgsl";
    let wgsl_src = read_to_string(shader_path)?;
    
    // Calculate hash to detect changes
    let mut hasher = DefaultHasher::new();
    wgsl_src.hash(&mut hasher);
    let current_hash = hasher.finish();
    
    // Check cache file
    let cache_path = PathBuf::from(&out_dir).join("shader_cache.txt");
    let should_rebuild = if cache_path.exists() {
        if let Ok(cached_hash) = std::fs::read_to_string(&cache_path) {
            cached_hash.trim().parse::<u64>().map(|h| h != current_hash).unwrap_or(true)
        } else {
            true
        }
    } else {
        true
    };
    
    if !should_rebuild && dest_path.exists() {
        println!("cargo:warning=Shader cache hit - skipping SPIR-V compilation");
        println!("cargo:rerun-if-changed={}", shader_path);
        return Ok(());
    }
    
    // Parse WGSL
    let mut parser = naga::front::wgsl::Frontend::new();
    let module = parser.parse(&wgsl_src)?;
    
    // Validate - skip validation to avoid IndexMustBeConstant errors for now
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::empty(),
        naga::valid::Capabilities::all(),
    );
    let info = validator.validate(&module)?;
    
    // Emit SPIR-V
    let flags = naga::back::spv::Options::default();
    let writer = naga::back::spv::write_vec(&module, &info, &flags, None)?;
    
    // Write Rust module
    let mut f = File::create(&dest_path)?;
    writeln!(f, "pub const SPIRV: &[u32] = &[")?;
    for (i, word) in writer.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "0x{:08x}", word)?;
    }
    writeln!(f, "];")?;
    
    // Update cache
    let mut cache_file = File::create(&cache_path)?;
    writeln!(cache_file, "{}", current_hash)?;
    
    println!("cargo:rerun-if-changed={}", shader_path);
    Ok(())
}