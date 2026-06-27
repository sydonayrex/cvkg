use crate::{CacheKey, GlyphInstance};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const MAX_CACHE_SIZE: usize = 1024;

struct GlobalCacheInner {
    cache: HashMap<CacheKey, Vec<GlyphInstance>>,
    cache_order: Vec<CacheKey>,
    /// Secondary index: glyph_cache_key -> (CacheKey, GlyphInstance) for O(1) lookup.
    glyph_index: HashMap<u64, (CacheKey, GlyphInstance)>,
}

static GLOBAL_SHAPE_CACHE: OnceLock<Mutex<GlobalCacheInner>> = OnceLock::new();

fn get_global_cache() -> std::sync::MutexGuard<'static, GlobalCacheInner> {
    GLOBAL_SHAPE_CACHE
        .get_or_init(|| {
            Mutex::new(GlobalCacheInner {
                cache: HashMap::new(),
                cache_order: Vec::new(),
                glyph_index: HashMap::new(),
            })
        })
        .lock()
        .unwrap()
}

pub fn global_cache_get(key: &CacheKey) -> Option<Vec<GlyphInstance>> {
    let cache = get_global_cache();
    cache.cache.get(key).cloned()
}

pub fn global_cache_insert(key: CacheKey, value: Vec<GlyphInstance>) {
    let mut cache = get_global_cache();
    // Remove existing entry from ordering if updating an existing key
    if cache.cache.contains_key(&key) {
        cache.cache_order.retain(|k| k != &key);
    } else if cache.cache.len() >= MAX_CACHE_SIZE
        && let Some(oldest) = cache.cache_order.first().cloned()
    {
        let old_keys_to_remove: Vec<u64> = cache
            .cache
            .get(&oldest)
            .map(|glyphs| glyphs.iter().map(|g| g.cache_key).collect())
            .unwrap_or_default();
        for gk in &old_keys_to_remove {
            cache.glyph_index.remove(gk);
        }
        cache.cache.remove(&oldest);
        cache.cache_order.remove(0);
    }
    for g in &value {
        cache.glyph_index.insert(g.cache_key, (key, *g));
    }
    cache.cache.insert(key, value);
    cache.cache_order.push(key);
}

pub fn global_cache_clear() {
    let mut cache = get_global_cache();
    cache.cache.clear();
    cache.cache_order.clear();
    cache.glyph_index.clear();
}

pub fn global_cache_stats() -> (usize, usize) {
    let cache = get_global_cache();
    (cache.cache.len(), MAX_CACHE_SIZE)
}

pub fn global_cache_find_glyph(glyph_cache_key: u64) -> Option<(CacheKey, GlyphInstance)> {
    let cache = get_global_cache();
    cache.glyph_index.get(&glyph_cache_key).copied()
}
