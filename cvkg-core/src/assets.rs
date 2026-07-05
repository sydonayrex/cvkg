//! Centralized asset server and hot-reloading subsystem.
//! Handles file watching, loading, and notifying runtime components when files change.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Listener trait for receiving asset mutation events.
pub trait AssetHotReloadListener: Send + Sync {
    /// Called when an asset at the specified path has been modified.
    fn on_asset_changed(&self, path: &Path);
}

/// The centralized manager for registered, watched, and loaded assets.
pub struct AssetServer {
    /// Registered listeners grouped by path for change notifications.
    listeners: Arc<Mutex<HashMap<PathBuf, Vec<Box<dyn AssetHotReloadListener>>>>>,
    /// Background file watcher context/thread representation.
    _watcher_handle: Option<std::thread::JoinHandle<()>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    /// Creates a new AssetServer instance and spawns a background thread
    /// to poll watched file paths for changes every 500ms.
    pub fn new() -> Self {
        let listeners: Arc<Mutex<HashMap<PathBuf, Vec<Box<dyn AssetHotReloadListener>>>>> = 
            Arc::new(Mutex::new(HashMap::new()));
        let listeners_clone = Arc::clone(&listeners);
        
        let handle = std::thread::spawn(move || {
            let mut last_modified = HashMap::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                // Safely extract paths currently watched
                let paths_to_check: Vec<PathBuf> = {
                    if let Ok(map) = listeners_clone.lock() {
                        map.keys().cloned().collect()
                    } else {
                        continue;
                    }
                };
                
                for path in paths_to_check {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let is_changed = match last_modified.get(&path) {
                                Some(&prev) => modified > prev,
                                None => {
                                    // Store initial time but don't trigger reload on startup
                                    last_modified.insert(path.clone(), modified);
                                    false
                                }
                            };
                            if is_changed {
                                last_modified.insert(path.clone(), modified);
                                if let Ok(map) = listeners_clone.lock() {
                                    if let Some(list) = map.get(&path) {
                                        for listener in list {
                                            let path_ref: &Path = &path;
                                            listener.on_asset_changed(path_ref);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            listeners,
            _watcher_handle: Some(handle),
        }
    }

    /// Registers a listener for modifications to a specific file or directory path.
    ///
    /// # Arguments
    /// * `path` - The system path to watch.
    /// * `listener` - The callback target implementing `AssetHotReloadListener`.
    pub fn watch<P: AsRef<Path>>(&self, path: P, listener: Box<dyn AssetHotReloadListener>) {
        let path_buf = path.as_ref().to_path_buf();
        let mut listeners = self.listeners.lock().unwrap();
        listeners.entry(path_buf).or_default().push(listener);
    }

    /// Simulates/triggers an asset modification event programmatically.
    ///
    /// # Arguments
    /// * `path` - The path representing the changed file.
    pub fn trigger_reload<P: AsRef<Path>>(&self, path: P) {
        let path_ref = path.as_ref();
        let listeners = self.listeners.lock().unwrap();
        if let Some(list) = listeners.get(path_ref) {
            for listener in list {
                listener.on_asset_changed(path_ref);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Simple listener helper that flags execution when called.
    struct TestListener {
        called: Arc<AtomicBool>,
    }

    impl AssetHotReloadListener for TestListener {
        fn on_asset_changed(&self, _path: &Path) {
            self.called.store(true, Ordering::SeqCst);
        }
    }

    /// Verifies that watching a path and triggering a reload correctly calls the listener.
    #[test]
    fn test_asset_server_trigger() {
        let server = AssetServer::new();
        let called = Arc::new(AtomicBool::new(false));
        let listener = TestListener { called: Arc::clone(&called) };
        let path = PathBuf::from("test_asset.gltf");
        
        server.watch(&path, Box::new(listener));
        server.trigger_reload(&path);
        
        assert!(called.load(Ordering::SeqCst));
    }
}
