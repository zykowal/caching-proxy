use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use hyper::{HeaderMap, StatusCode, body::Bytes};

#[derive(Clone)]
pub struct CacheEntry {
    pub response_body: Bytes,
    pub status_code: StatusCode,
    pub headers: HeaderMap,
    pub created_at: Instant,
}

impl CacheEntry {
    pub fn new(response_body: Bytes, status_code: StatusCode, headers: HeaderMap) -> Self {
        Self {
            response_body,
            status_code,
            headers,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

#[derive(Clone)]
pub struct MemoryCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl MemoryCache {
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_entries,
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired(self.default_ttl) {
                Some(entry.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set(&self, key: String, entry: CacheEntry) {
        let mut cache = self.cache.write().unwrap();

        if cache.len() >= self.max_entries {
            cache.clear();
        }

        cache.insert(key, entry);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let count = cache.len();
        cache.clear();
        println!("Cache cleared! Removed {count} entries.");
    }

    pub fn size(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }
}
