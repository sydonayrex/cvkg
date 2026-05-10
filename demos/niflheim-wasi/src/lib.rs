use cvkg_components::niflheim_demo;

#[unsafe(no_mangle)]
pub extern "C" fn cvkg_init() {
    // Initialize logging if available via WASI
    // log::info!("Niflheim WASI Demo Initialized");
}

#[unsafe(no_mangle)]
pub extern "C" fn cvkg_update() {
    // Update logic could go here
}

#[unsafe(no_mangle)]
pub extern "C" fn cvkg_render() {
    let demo = niflheim_demo();
    
    // In a server-side WASM context, we might not have a real GPU renderer inside the guest.
    // However, we can use a "Null" or "Command-Recording" renderer to verify the View tree works.
    
    // For this verification, we'll just ensure the demo view can be instantiated.
    let _Body = demo;
}
