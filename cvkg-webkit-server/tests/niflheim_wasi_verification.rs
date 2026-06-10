use cvkg_webkit_server::wasm_server::NativeWasmServer;
use std::path::PathBuf;

/// Verifies that the Niflheim WASI demo module can be loaded and executed by the WASM server.
/// If the target WebAssembly binary is not yet compiled, this test attempts to compile it.
#[tokio::test]
async fn test_niflheim_wasi_activation() -> anyhow::Result<()> {
    // Path to the built WASM demo
    let wasm_path = PathBuf::from("../target/wasm32-wasip1/debug/niflheim_wasi_demo.wasm");

    if !wasm_path.exists() {
        println!("[Verification] WASM demo not found. Automatically compiling...");
        let status = std::process::Command::new("cargo")
            .args(&[
                "build",
                "-p",
                "niflheim-wasi-demo",
                "--target",
                "wasm32-wasip1",
            ])
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to build niflheim-wasi-demo WASM target"
            ));
        }
    }

    if !wasm_path.exists() {
        return Err(anyhow::anyhow!(
            "WASM demo still not found at {:?}",
            wasm_path
        ));
    }

    println!("[Verification] Initializing NativeWasmServer...");
    let server = NativeWasmServer::new()?;

    println!("[Verification] Loading Niflheim module...");
    server.load_module(&wasm_path, true)?;

    println!("[Verification] Executing first tick...");
    server.tick()?;

    println!("[Verification] SUCCESS: Niflheim WASI demo activated and ticked.");
    Ok(())
}
