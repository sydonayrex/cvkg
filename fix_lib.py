import re

path = "/D/rex/projects/cvkg/cvkg-core/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

# 1. State<T>
state_old = """pub struct State<T: Clone + Send + Sync + 'static> {
    value: Arc<std::sync::RwLock<T>>,
    subscribers: Arc<std::sync::RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(std::sync::RwLock::new(value)),
            subscribers: Arc::new(std::sync::RwLock::new(Vec::new())),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.write().unwrap() = value;
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
        
        let current = self.get();
        let subs = self.subscribers.read().unwrap();
        for cb in subs.iter() {
            cb(&current);
        }
    }

    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        self.subscribers.write().unwrap().push(Box::new(callback));
    }
}"""

state_new = """#[derive(Clone)]
pub struct State<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    subscribers: Arc<std::sync::Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    pub fn new(value: T) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let tvar = Arc::new(stm::TVar::new(value.clone()));
        Self {
            swap: Arc::new(arc_swap::ArcSwap::from_pointee(value)),
            #[cfg(not(target_arch = "wasm32"))]
            tvar,
            subscribers: Arc::new(std::sync::Mutex::new(Vec::new())),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }

    pub fn set(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = Arc::clone(&self.tvar);
            let v = value.clone();
            let _ = stm::atomically(move |tx| tvar.write(tx, v.clone()));
        }
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
        let current = self.get();
        let subs = self.subscribers.lock().unwrap();
        for cb in subs.iter() {
            cb(&current);
        }
    }

    pub fn mutate<F: Fn(&T) -> T>(&self, f: F) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = Arc::clone(&self.tvar);
            let new_val = stm::atomically(move |tx| {
                let current = tvar.read(tx)?;
                let next = f(&current);
                tvar.write(tx, next.clone())?;
                Ok(next)
            });
            self.swap.store(Arc::new(new_val.clone()));
            self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
            let subs = self.subscribers.lock().unwrap();
            for cb in subs.iter() {
                cb(&new_val);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.set(f(&self.get()));
        }
    }

    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        self.subscribers.lock().unwrap().push(Box::new(callback));
    }
}"""

content = content.replace(state_old, state_new)

# 2. Binding<T>
binding_old = """pub struct Binding<T: Clone + Send + Sync + 'static> {
    value: Arc<std::sync::RwLock<T>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> Binding<T> {
    pub fn from_state(state: &State<T>) -> Self {
        Self {
            value: Arc::clone(&state.value),
            version: Arc::clone(&state.version),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, value: T) {
        *self.value.write().unwrap() = value;
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    }

    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
}"""

binding_new = """#[derive(Clone)]
pub struct Binding<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> Binding<T> {
    pub fn from_state(state: &State<T>) -> Self {
        Self {
            swap: Arc::clone(&state.swap),
            #[cfg(not(target_arch = "wasm32"))]
            tvar: Arc::clone(&state.tvar),
            version: Arc::clone(&state.version),
        }
    }

    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }

    pub fn set(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = Arc::clone(&self.tvar);
            let v = value.clone();
            let _ = stm::atomically(move |tx| tvar.write(tx, v.clone()));
        }
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    }

    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn transact_pair<A, B, F>(state_a: &State<A>, state_b: &State<B>, f: F)
where
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
    F: Fn(&A, &B) -> (A, B),
{
    let tvar_a = Arc::clone(&state_a.tvar);
    let tvar_b = Arc::clone(&state_b.tvar);
    let (new_a, new_b) = stm::atomically(move |tx| {
        let a = tvar_a.read(tx)?;
        let b = tvar_b.read(tx)?;
        let (na, nb) = f(&a, &b);
        tvar_a.write(tx, na.clone())?;
        tvar_b.write(tx, nb.clone())?;
        Ok((na, nb))
    });
    state_a.swap.store(Arc::new(new_a.clone()));
    state_b.swap.store(Arc::new(new_b.clone()));
    state_a.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    state_b.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    {
        let subs = state_a.subscribers.lock().unwrap();
        for cb in subs.iter() { cb(&new_a); }
    }
    {
        let subs = state_b.subscribers.lock().unwrap();
        for cb in subs.iter() { cb(&new_b); }
    }
}"""

content = content.replace(binding_old, binding_new)

# 3. SYSTEM_STATE
sys_state_old = """use std::sync::OnceLock;

/// Global application state registry.
/// Provides synchronized access to cross-component knowledge and memory.
pub static SYSTEM_STATE: OnceLock<Arc<std::sync::RwLock<KnowledgeState>>> = OnceLock::new();

pub fn get_system_state() -> Arc<std::sync::RwLock<KnowledgeState>> {
    SYSTEM_STATE
        .get_or_init(|| Arc::new(std::sync::RwLock::new(KnowledgeState::default())))
        .clone()
}"""

sys_state_new = """use arc_swap::ArcSwap;
use std::sync::OnceLock;

/// Global application state registry.
pub static SYSTEM_STATE: OnceLock<Arc<ArcSwap<KnowledgeState>>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
static KNOWLEDGE_TVAR: OnceLock<stm::TVar<KnowledgeState>> = OnceLock::new();

pub fn get_system_state() -> Arc<ArcSwap<KnowledgeState>> {
    SYSTEM_STATE
        .get_or_init(|| Arc::new(ArcSwap::from_pointee(KnowledgeState::default())))
        .clone()
}

pub fn load_system_state() -> arc_swap::Guard<Arc<KnowledgeState>> {
    get_system_state().load()
}

pub fn update_system_state<F>(f: F)
where
    F: Fn(&KnowledgeState) -> KnowledgeState,
{
    let swap = get_system_state();
    let current = swap.load();
    let new_state = Arc::new(f(&current));
    swap.store(Arc::clone(&new_state));

    #[cfg(not(target_arch = "wasm32"))]
    {
        let tvar = KNOWLEDGE_TVAR
            .get_or_init(|| stm::TVar::new((*new_state).clone()));
        let _ = stm::atomically(|tx| tvar.write(tx, (*new_state).clone()));
    }
}

pub fn transact_system_state<F>(f: F)
where
    F: Fn(&KnowledgeState) -> KnowledgeState,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let tvar = KNOWLEDGE_TVAR
            .get_or_init(|| {
                stm::TVar::new((**get_system_state().load()).clone())
            })
            .clone();
        let new_state = stm::atomically(move |tx| {
            let current = tvar.read(tx)?;
            let next = f(&current);
            tvar.write(tx, next.clone())?;
            Ok(next)
        });
        get_system_state().store(Arc::new(new_state));
    }
    #[cfg(target_arch = "wasm32")]
    {
        update_system_state(f);
    }
}"""

content = content.replace(sys_state_old, sys_state_new)

# 4. DefaultAssetManager
default_old = """pub struct DefaultAssetManager {
    cache: Arc<std::sync::RwLock<HashMap<String, AssetState<Arc<Vec<u8>>>>>>,
}

impl DefaultAssetManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl AssetManager for DefaultAssetManager {
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>> {
        let mut cache = self.cache.write().unwrap();
        if let Some(state) = cache.get(url) {
            return state.clone();
        }

        // In the default manager, we just mark it as Loading and spawn a placeholder
        // (Real backends will override this with actual I/O)
        cache.insert(url.to_string(), AssetState::Loading);
        AssetState::Loading
    }

    fn preload_image(&self, _url: &str) {
        // No-op for default manager
    }
}"""

default_new = """pub struct DefaultAssetManager {
    cache: Arc<arc_swap::ArcSwap<HashMap<String, AssetState<Arc<Vec<u8>>>>>>,
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
}"""

content = content.replace(default_old, default_new)

with open(path, "w") as f:
    f.write(content)
