use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::{CacheKey, GlyphInstance};

const MAX_CACHE_SIZE: usize = 1024;

struct GlobalCacheInner {
    cache: HashMap<CacheKey, Vec<GlyphInstance>>,
    cache_order: Vec<CacheKey>,
}

static GLOBAL_SHAPE_CACHE: OnceLock<Mutex<GlobalCacheInner>> = OnceLock::new();

fn get_global_cache() -> std::sync::MutexGuard<'static, GlobalCacheInner> {
    GLOBAL_SHAPE_CACHE.get_or_init(|| {
        Mutex::new(GlobalCacheInner {
            cache: HashMap::new(),
            cache_order: Vec::new(),
        })
    }).lock().unwrap()
}

pub fn global_cache_get(key: &CacheKey) -> Option<Vec<GlyphInstance>> {
    let cache = get_global_cache();
    cache.cache.get(key).cloned()
}

pub fn global_cache_insert(key: CacheKey, value: Vec<GlyphInstance>) {
    let mut cache = get_global_cache();
    if cache.cache.len() >= MAX_CACHE_SIZE {
        if let Some(oldest) = cache.cache_order.first().cloned() {
            cache.cache.remove(&oldest);
            cache.cache_order.remove(0);
        }
    }
    cache.cache.insert(key, value);
    cache.cache_order.push(key);
}

pub fn global_cache_clear() {
    let mut cache = get_global_cache();
    cache.cache.clear();
    cache.cache_order.clear();
}

pub fn global_cache_stats() -> (usize, usize) {
    let cache = get_global_cache();
    (cache.cache.len(), MAX_CACHE_SIZE)
}

pub fn global_cache_find_glyph(glyph_cache_key: u64) -> Option<(CacheKey, GlyphInstance)> {
    let cache = get_global_cache();
    for (ck, glyphs) in &cache.cache {
        if let Some(g) = glyphs.iter().find(|g| g.cache_key == glyph_cache_key) {
            return Some((*ck, *g));
        }
    }
    None
}
