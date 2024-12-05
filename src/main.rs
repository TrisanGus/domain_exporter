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

// query params struct
#[derive(Deserialize)]
struct ProbeParams {
    target: String,
}

#[tokio::main]
async fn main() {
    // init tracing logger
    tracing_subscriber::fmt::init();

    // load config from command line args
    let config = Arc::new(Config::from_args());
    // init cache
    let cache = Arc::new(DomainCache::new(config.cache_ttl));

    // create router
    let app = Router::new()
        .route("/probe", get({
            let cache = Arc::clone(&cache);
            let config = Arc::clone(&config);
            move |params| probe_handler(params, cache, config)
        }));

    // start server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await.unwrap();
    tracing::info!("Server running on http://{}", config.listen_addr);
    
    axum::serve(listener, app).await.unwrap();
}

// update handler
async fn probe_handler(
    Query(params): Query<ProbeParams>, 
    cache: Arc<DomainCache>,
    config: Arc<Config>,
) -> impl IntoResponse {
    let target = &params.target;

    // check cache
    if let Some(entry) = cache.get(target).await {
        let now = Utc::now();
        let days = (entry.expiry_date - now).num_days();
        return format_response(target, days, 1);
    }
    
    // execute WHOIS query
    let (expiry_days, probe_success) = match whois::query_domain(target, &config).await {
        Ok(domain_info) => {
            let now = Utc::now();
            let days = (domain_info.expiry_date - now).num_days();
            // Only cache successful results with valid expiry dates
            if days >= 0 {
                // update cache
                cache.set(target.to_string(), domain_info.expiry_date).await;
            } else {
                warn!("Got invalid expiry date for domain {}: {} days", target, days);
            }
            (days, 1)
        },
        Err(e) => {
            error!("Error querying domain: {:?}", e);
            (-1, 0)
        }
    };

    format_response(target, expiry_days, probe_success)
}

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

