use std::collections::HashMap;
use std::time::Duration;

pub struct EdgeCache {
    cache: HashMap<String, CacheEntry>,
    max_entries: usize,
    default_ttl: Duration,
}

struct CacheEntry {
    data: Vec<u8>,
    inserted_at: chrono::DateTime<chrono::Utc>,
    ttl: Duration,
    version: u64,
}

impl EdgeCache {
    pub fn new(max_entries: usize, default_ttl_secs: u64) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let entry = self.cache.get(key)?;
        let age = chrono::Utc::now() - entry.inserted_at;
        if age.num_seconds() > entry.ttl.as_secs() as i64 {
            return None;
        }
        Some(&entry.data)
    }

    pub fn set(&mut self, key: impl Into<String>, data: Vec<u8>, version: u64) {
        if self.cache.len() >= self.max_entries {
            if let Some(oldest) = self.find_oldest() {
                self.cache.remove(&oldest);
            }
        }
        self.cache.insert(key.into(), CacheEntry {
            data,
            inserted_at: chrono::Utc::now(),
            ttl: self.default_ttl,
            version,
        });
    }

    pub fn invalidate(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn invalidate_pattern(&mut self, prefix: &str) {
        self.cache.retain(|k, _| !k.starts_with(prefix));
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }

    fn find_oldest(&self) -> Option<String> {
        self.cache.iter()
            .min_by_key(|(_, e)| e.inserted_at)
            .map(|(k, _)| k.clone())
    }
}
