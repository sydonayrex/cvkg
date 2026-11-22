use cvkg_core::{AssetHotReloadListener, AssetServer};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FlagListener(Arc<AtomicBool>);

impl AssetHotReloadListener for FlagListener {
    fn on_asset_changed(&self, _path: &Path) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn watch_gltf_invokes_the_caller_listener() {
    let server = AssetServer::default();
    let called = Arc::new(AtomicBool::new(false));
    server.watch_gltf("model.glb", Box::new(FlagListener(called.clone())));

    server.trigger_reload("model.glb");

    assert!(called.load(Ordering::SeqCst));
}
