use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use tracing::{info, debug};

/// Represents a cached domain entry with expiry date and timestamp
#[derive(Clone)]
pub struct CacheEntry {
    /// The expiration date of the domain
    pub expiry_date: DateTime<Utc>,
    /// When this cache entry was created
    pub timestamp: Instant,
}

/// Thread-safe cache for storing domain expiry information
pub struct DomainCache {
    /// Internal HashMap protected by a Mutex
    cache: Mutex<HashMap<String, CacheEntry>>,
    /// How long cache entries remain valid
    ttl: Duration,
}

impl DomainCache {
    /// Creates a new DomainCache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Retrieves a domain entry from cache if it exists and is not expired
    /// 
    /// Returns None if:
    /// - Domain is not in cache
    /// - Cache entry has expired
    pub async fn get(&self, domain: &str) -> Option<CacheEntry> {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.get(domain) {
            if entry.timestamp.elapsed() < self.ttl {
                debug!("Cache hit for domain {}", domain);
                return Some(entry.clone());
            } else {
                debug!("Cache entry expired for domain {}", domain);
            }
        }
        None
    }

    /// Stores a domain entry in the cache
    /// 
    /// # Arguments
    /// * `domain` - Domain name to cache
    /// * `expiry_date` - Domain's expiration date
    pub async fn set(&self, domain: String, expiry_date: DateTime<Utc>) {
        let mut cache = self.cache.lock().await;
        cache.insert(domain.clone(), CacheEntry {
            expiry_date,
            timestamp: Instant::now(),
        });
        info!("Updated cache entry for domain {}", domain);
    }
}
