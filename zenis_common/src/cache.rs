use std::{hash::Hash, num::NonZeroUsize, sync::Mutex};

use lru::LruCache;

pub struct Cache<K, V> {
    inner_cache: Mutex<LruCache<K, V>>,
}

impl<K: Eq + Hash, V: Clone> Cache<K, V> {
    pub fn new(size: usize) -> Self {
        Self {
            inner_cache: Mutex::new(LruCache::new(NonZeroUsize::new(size).unwrap())),
        }
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner_cache.lock().unwrap().pop(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.inner_cache.lock().unwrap().contains(key)
    }

    pub fn get_cloned(&self, key: &K) -> Option<V> {
        self.inner_cache.lock().unwrap().get(key).cloned()
    }

    /// Inserts a key into the cache. If the key already exists, replaces it and returns the old value.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut cache = self.inner_cache.lock().unwrap();
        cache.put(key, value)
    }
}
