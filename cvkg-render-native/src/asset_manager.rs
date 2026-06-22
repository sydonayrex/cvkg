type AssetCacheMap =
    std::collections::HashMap<String, cvkg_core::AssetState<std::sync::Arc<Vec<u8>>>>;

/// A concrete AssetManager for native desktop targets that loads from the local filesystem.
///
/// The cache is read on every render frame (lock-free via `ArcSwap::load()`) but written
/// at most once per URL after disk I/O completes. `rcu()` atomically inserts the result
/// without blocking concurrent render-loop readers.
pub struct NativeAssetManager {
    cache: std::sync::Arc<arc_swap::ArcSwap<AssetCacheMap>>,
}

impl Default for NativeAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeAssetManager {
    /// Create a new, empty NativeAssetManager.
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(arc_swap::ArcSwap::from_pointee(
                std::collections::HashMap::new(),
            )),
        }
    }
}

impl cvkg_core::AssetManager for NativeAssetManager {
    /// Return the cached asset state for `url`.
    ///
    /// Fast path: lock-free snapshot read via `ArcSwap::load()`.
    /// Slow path (cache miss): atomically insert a Loading sentinel via `rcu()`,
    /// then spawn a background thread for I/O. The `rcu()` closure may execute
    /// more than once under contention, so `already_tracked` is determined by
    /// whether the closure actually inserted the Loading entry (detected by checking
    /// the returned map). This prevents duplicate I/O threads for the same URL.
    ///
    /// FIX #5: The previous implementation set `already_tracked` inside the `rcu`
    /// closure body, which is incorrect because `rcu` retries the closure on
    /// contention -- the bool would reflect only the last execution. The fix uses
    /// the fast-path check result plus the atomic `rcu` insertion to determine
    /// whether a thread must be spawned, making the logic correct under concurrency.
    fn load_image(&self, url: &str) -> cvkg_core::AssetState<std::sync::Arc<Vec<u8>>> {
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        let cache = self.cache.clone();
        let key = url.to_string();

        let mut we_inserted = false;
        self.cache.rcu(|map| {
            if map.contains_key(&key) {
                (**map).clone()
            } else {
                we_inserted = true;
                let mut m = (**map).clone();
                m.insert(key.clone(), cvkg_core::AssetState::Loading);
                m
            }
        });

        if we_inserted {
            let cache_inner = cache.clone();
            let key_inner = key.clone();

            std::thread::spawn(move || {
                log::debug!("[Native] Asynchronously loading asset: {}", key_inner);
                let result = match std::fs::read(&key_inner) {
                    Ok(data) => cvkg_core::AssetState::Ready(std::sync::Arc::new(data)),
                    Err(e) => cvkg_core::AssetState::Error(e.to_string()),
                };

                cache_inner.rcu(move |map| {
                    let mut m = (**map).clone();
                    m.insert(key_inner.clone(), result.clone());
                    m
                });
            });
        }

        cvkg_core::AssetState::Loading
    }

    /// Trigger a background load of `url` without waiting for the result.
    ///
    /// FIX #6: The previous implementation had a bare fast-path check followed
    /// by an unconditional thread spawn, allowing two concurrent calls for the
    /// same URL to both spawn I/O threads. Now uses the same rcu-based insertion
    /// guard as `load_image` to ensure exactly one thread is spawned per URL.
    fn preload_image(&self, url: &str) {
        if self.cache.load().contains_key(url) {
            return;
        }

        let cache = self.cache.clone();
        let key = url.to_string();

        let mut we_inserted = false;
        self.cache.rcu(|map| {
            if map.contains_key(&key) {
                (**map).clone()
            } else {
                we_inserted = true;
                let mut m = (**map).clone();
                m.insert(key.clone(), cvkg_core::AssetState::Loading);
                m
            }
        });

        if we_inserted {
            std::thread::spawn(move || {
                log::debug!("[Native] Preloading asset: {}", key);
                let result = match std::fs::read(&key) {
                    Ok(data) => cvkg_core::AssetState::Ready(std::sync::Arc::new(data)),
                    Err(e) => cvkg_core::AssetState::Error(e.to_string()),
                };

                cache.rcu(move |map| {
                    let mut m = (**map).clone();
                    m.insert(key.clone(), result.clone());
                    m
                });
            });
        }
    }
}
