use std::time::Duration;
use clap::Parser;
use std::env;

/// Command line arguments for the application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// How long to keep domain information in cache
    #[arg(long, default_value = "86400")]
    cache_ttl: u64,

    /// Maximum time to wait for WHOIS server response
    #[arg(long, default_value = "10")]
    whois_timeout: u64,

    /// Address and port for the HTTP server
    #[arg(long, default_value = "0.0.0.0:9222")]
    listen_addr: String,
}

/// Application configuration
pub struct Config {
    /// Cache time-to-live duration
    pub cache_ttl: Duration,
    /// WHOIS query timeout duration
    pub whois_timeout: Duration,
    /// HTTP server listen address
    pub listen_addr: String,
}

impl Config {
    /// Creates configuration from command line arguments and environment variables
    pub fn from_args() -> Self {
        let args = Args::parse();
        
        // Environment variables take precedence over command line arguments
        Self {
            cache_ttl: Duration::from_secs(
                env::var("CACHE_TTL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(args.cache_ttl)
            ),
            whois_timeout: Duration::from_secs(
                env::var("WHOIS_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(args.whois_timeout)
            ),
            listen_addr: env::var("LISTEN_ADDR")
                .unwrap_or(args.listen_addr),
        }
    }
}

/// Default configuration values
impl Default for Config {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(24 * 3600),
            whois_timeout: Duration::from_secs(10),
            listen_addr: "0.0.0.0:9222".to_string(),
        }
    }
}
