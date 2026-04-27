use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("shader_spirv.rs");
    
    let wgsl_src = std::fs::read_to_string("src/shader.wgsl")?;
    
    // Parse WGSL
    let mut parser = naga::front::wgsl::Frontend::default();
    let module = parser.parse(wgsl_src)?;
    
    // Validate
    let mut validator = naga::valid::Validator::default(
        naga::valid::ValidationFlags::all(),
    );
    validator.validate(&module)?;
    
    // Emit SPIR-V
    let mut flags = naga::back::spv::Options::default();
    let mut writer = Vec::new();
    naga::back::spv::write_string(&module, &flags, &mut writer)?;
    
    // Write Rust module
    let mut f = File::create(dest_path)?;
    writeln!(f, "pub const SPIRV: &[u32] = &";)?;
    write!(f, "[")?;
    for (i, word) in writer.chunks_exact(4).enumerate() {
        let word = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "0x{:08x}", word)?;
    }
    writeln!(f, "];")?;
    
    println!("cargo:rerun-if-changed=src/shader.wgsl");
    Ok(())
}
