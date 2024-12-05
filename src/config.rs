use std::time::Duration;

pub struct Config {
    pub cache_ttl: Duration,
    pub whois_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // default cache ttl is 12 hours
            cache_ttl: Duration::from_secs(24 * 3600),

            // default whois timeout is 10 seconds
            whois_timeout: Duration::from_secs(10),
        }
    }
}
