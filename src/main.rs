use axum::{
    extract::Query,
    routing::get,
    Router,
    response::IntoResponse,
};
use serde::Deserialize;

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
    
    // build basic metrics response
    let response = format!(
        r#"# HELP domain_expiry_days Days until domain expiry
# TYPE domain_expiry_days gauge
domain_expiry_days{{domain="{}"}} -1
# HELP domain_probe_success Displays whether or not the domain probe was successful
# TYPE domain_probe_success gauge
domain_probe_success{{domain="{}"}} 0
"#,
        target, target
    );

    // set response headers
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        response
    )
}
