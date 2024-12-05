mod whois;
mod error;
mod cache;
mod config;

use axum::{
    extract::Query,
    routing::get,
    Router,
    response::IntoResponse,
};
use serde::Deserialize;
use chrono::Utc;
use tracing::{error, warn};
use std::sync::Arc;
use cache::DomainCache;
use config::Config;

/// Query parameters for the probe endpoint
#[derive(Deserialize)]
struct ProbeParams {
    /// Domain name to be queried
    target: String,
}

#[tokio::main]
async fn main() {
    // Initialize the tracing logger for better debugging and monitoring
    tracing_subscriber::fmt::init();

    // Load configuration from command line arguments
    let config = Arc::new(Config::from_args());
    
    // Initialize the domain cache with configured TTL
    let cache = Arc::new(DomainCache::new(config.cache_ttl));

    // Create the router with probe endpoint
    let app = Router::new()
        .route("/probe", get({
            let cache = Arc::clone(&cache);
            let config = Arc::clone(&config);
            move |params| probe_handler(params, cache, config)
        }));

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await.unwrap();
    tracing::info!("Server running on http://{}", config.listen_addr);
    
    axum::serve(listener, app).await.unwrap();
}

/// Handler for the /probe endpoint
/// 
/// This function:
/// 1. Checks the cache for existing domain information
/// 2. If not in cache, performs a WHOIS query
/// 3. Updates cache with successful query results
/// 4. Returns domain expiry information in Prometheus metrics format
async fn probe_handler(
    Query(params): Query<ProbeParams>, 
    cache: Arc<DomainCache>,
    config: Arc<Config>,
) -> impl IntoResponse {
    let target = &params.target;

    // Try to get domain information from cache first
    if let Some(entry) = cache.get(target).await {
        let now = Utc::now();
        let days = (entry.expiry_date - now).num_days();
        return format_response(target, days, 1);
    }
    
    // If not in cache, perform WHOIS query
    let (expiry_days, probe_success) = match whois::query_domain(target, &config).await {
        Ok(domain_info) => {
            let now = Utc::now();
            let days = (domain_info.expiry_date - now).num_days();
            // Only cache successful results with valid expiry dates
            if days >= 0 {
                cache.set(target.to_string(), domain_info.expiry_date).await;
            } else {
                warn!("Invalid expiry date for domain {}: {} days", target, days);
            }
            (days, 1)
        },
        Err(e) => {
            error!("Error querying domain {}: {:?}", target, e);
            (-1, 0)
        }
    };

    format_response(target, expiry_days, probe_success)
}

/// Formats the response in Prometheus metrics format
/// 
/// Returns two metrics:
/// - domain_expiry_days: Number of days until domain expiration
/// - domain_probe_success: Whether the probe was successful (1) or failed (0)
fn format_response(domain: &str, expiry_days: i64, probe_success: i32) -> impl IntoResponse {
    let response = format!(
        r#"# HELP domain_expiry_days Days until domain expiry
# TYPE domain_expiry_days gauge
domain_expiry_days{{domain="{}"}} {}
# HELP domain_probe_success Displays whether or not the domain probe was successful
# TYPE domain_probe_success gauge
domain_probe_success{{domain="{}"}} {}
"#,
        domain, expiry_days,
        domain, probe_success
    );

    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        response
    )
}

