use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use tracing::{info, debug};

#[derive(Clone)]
pub struct CacheEntry {
    pub expiry_date: DateTime<Utc>,
    pub timestamp: Instant,
}

pub struct DomainCache {
    cache: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl DomainCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get(&self, domain: &str) -> Option<CacheEntry> {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.get(domain) {
            if entry.timestamp.elapsed() < self.ttl {
                debug!("Domain {} cache hit", domain);
                return Some(entry.clone());
            } else {
                debug!("Domain {} cache expired", domain);
            }
        }
        None
    }

    pub async fn set(&self, domain: String, expiry_date: DateTime<Utc>) {
        let mut cache = self.cache.lock().await;
        cache.insert(domain.clone(), CacheEntry {
            expiry_date,
            timestamp: Instant::now(),
        });
        info!("Domain {} cache updated", domain);
    }
}
