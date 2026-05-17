use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("shader_spirv.rs");

    let wgsl_src = std::fs::read_to_string("src/shader.wgsl")?;

    // Parse WGSL
    let mut parser = naga::front::wgsl::Frontend::new();
    let module = parser.parse(&wgsl_src)?;

    // Validate
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let info = validator.validate(&module)?;

    // Emit SPIR-V
    let flags = naga::back::spv::Options::default();
    let writer = naga::back::spv::write_vec(&module, &info, &flags, None)?;

    // Write Rust module
    let mut f = File::create(dest_path)?;
    writeln!(f, "pub const SPIRV: &[u32] = &[")?;
    for (i, &word) in writer.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "0x{:08x}", word)?;
    }
    writeln!(f, "];")?;

    println!("cargo:rerun-if-changed=src/shader.wgsl");
    Ok(())
}
