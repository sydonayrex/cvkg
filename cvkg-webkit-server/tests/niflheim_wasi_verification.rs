use cvkg_webkit_server::wasm_server::NativeWasmServer;
use std::path::PathBuf;

#[tokio::test]
async fn test_niflheim_wasi_activation() -> anyhow::Result<()> {
    // Path to the built WASM demo
    let wasm_path = PathBuf::from("../target/wasm32-wasip1/debug/niflheim_wasi_demo.wasm");

    if !wasm_path.exists() {
        return Err(anyhow::anyhow!("WASM demo not found at {:?}", wasm_path));
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
