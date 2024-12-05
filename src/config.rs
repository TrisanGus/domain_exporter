use std::time::Duration;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Cache TTL in seconds
    #[arg(long, default_value = "86400")]
    cache_ttl: u64,

    /// WHOIS query timeout in seconds
    #[arg(long, default_value = "10")]
    whois_timeout: u64,

    /// Server listen address
    #[arg(long, default_value = "0.0.0.0:9222")]
    listen_addr: String,
}

pub struct Config {
    pub cache_ttl: Duration,
    pub whois_timeout: Duration,
    pub listen_addr: String,
}

impl Config {
    pub fn from_args() -> Self {
        let args = Args::parse();
        Self {
            cache_ttl: Duration::from_secs(args.cache_ttl),
            whois_timeout: Duration::from_secs(args.whois_timeout),
            listen_addr: args.listen_addr,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(24 * 3600),
            whois_timeout: Duration::from_secs(10),
            listen_addr: "0.0.0.0:9222".to_string(),
        }
    }
}
