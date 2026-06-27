use crate::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DefaultAssetManager {
    cache: AssetCache,
}
type AssetCache = Arc<arc_swap::ArcSwap<HashMap<String, AssetState<Arc<Vec<u8>>>>>>;

impl Default for DefaultAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAssetManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        }
    }
}

impl AssetManager for DefaultAssetManager {
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>> {
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        self.cache.rcu(|map| {
            let mut m = (**map).clone();
            m.entry(url.to_string()).or_insert(AssetState::Loading);
            m
        });
        AssetState::Loading
    }

    fn preload_image(&self, _url: &str) {}
}

use std::future::Future;

/// Suspense wrapper for asynchronous state management.
/// Integrates with State<T> to provide loading/error/ready states for async operations.
pub struct Suspense<T: Clone + Send + Sync + 'static> {
    inner: State<AssetState<T>>,
}

impl<T: Clone + Send + Sync + 'static> Default for Suspense<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> Suspense<T> {
    pub fn new() -> Self {
        Self {
            inner: State::new(AssetState::Loading),
        }
    }

    pub fn new_async<F>(future: F) -> Self
    where
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let suspense = Self::new();
        let suspense_clone = suspense.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            // P1-17 fix: use the shared fallback runtime instead of
            // spawning a new OS thread + runtime per call. If an
            // ambient tokio runtime exists, prefer it (preserves
            // caller intent). Otherwise use the shared fallback
            // runtime which is bounded to a small worker count.
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                crate::fallback_runtime().spawn(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
            }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let result = future.await;
                match result {
                    Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                    Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                }
            });
        }

        suspense
    }

    pub fn ready(value: T) -> Self {
        Self {
            inner: State::new(AssetState::Ready(value)),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            inner: State::new(AssetState::Error(message.into())),
        }
    }

    pub fn get(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn get_ref(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.get(), AssetState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.get(), AssetState::Ready(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.get(), AssetState::Error(_))
    }

    pub fn ready_value(&self) -> Option<T> {
        match self.get() {
            AssetState::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn error_message(&self) -> Option<String> {
        match self.get() {
            AssetState::Error(message) => Some(message),
            _ => None,
        }
    }

    pub fn subscribe<F: Fn(&AssetState<T>) + Send + Sync + 'static>(&self, callback: F) {
        self.inner.subscribe(callback)
    }

    pub fn inner_state(&self) -> &State<AssetState<T>> {
        &self.inner
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Suspense<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<T> for Suspense<T> {
    fn from(value: T) -> Self {
        Self::ready(value)
    }
}

impl<T: Clone + Send + Sync + 'static> From<Result<T, String>> for Suspense<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(value) => Self::ready(value),
            Err(error) => Self::error(error),
        }
    }
}
