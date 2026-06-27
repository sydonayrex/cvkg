/// Insert a value into the environment
pub fn insert<K: super::EnvKey>(value: K::Value) {
    let store =
        super::ENVIRONMENT.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut env_map = store.lock().unwrap_or_else(|p| p.into_inner());
    env_map.insert(std::any::TypeId::of::<K>(), Box::new(value));
}
/// Remove a value from the environment.
pub fn remove<K: super::EnvKey>() {
    if let Some(store) = super::ENVIRONMENT.get() {
        let mut env_map = store.lock().unwrap_or_else(|p| p.into_inner());
        env_map.remove(&std::any::TypeId::of::<K>());
    }
}
