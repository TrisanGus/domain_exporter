mod whois;
mod error;

use axum::{
    extract::Query,
    routing::get,
    Router,
    response::IntoResponse,
};
use serde::Deserialize;
use chrono::Utc;
use tracing::error;


// query params struct
#[derive(Deserialize)]
struct ProbeParams {
    target: String,
}

#[tokio::main]
async fn main() {
    // init tracing logger
    tracing_subscriber::fmt::init();

    // create router
    let app = Router::new()
        .route("/probe", get(probe_handler));

    // start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9222").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:9222");
    
    axum::serve(listener, app).await.unwrap();
}

// update handler
async fn probe_handler(Query(params): Query<ProbeParams>) -> impl IntoResponse {
    let target = &params.target;
    
    // execute WHOIS query
    let (expiry_days, probe_success) = match whois::query_domain(target).await {
        Ok(domain_info) => {
            let now = Utc::now();
            let days = (domain_info.expiry_date - now).num_days();
            (days, 1)
        },
        Err(e) => {
            error!("Error querying domain: {:?}", e);
            (-1, 0)
        }
    };

    // build metrics response
    let response = format!(
        r#"# HELP domain_expiry_days Days until domain expiry
# TYPE domain_expiry_days gauge
domain_expiry_days{{domain="{}"}} {}
# HELP domain_probe_success Displays whether or not the domain probe was successful
# TYPE domain_probe_success gauge
domain_probe_success{{domain="{}"}} {}
"#,
        target, expiry_days,
        target, probe_success
    );

    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        response
    )
}
