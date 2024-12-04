use axum::{
    routing::get,
    Router,
};

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

async fn probe_handler() -> &'static str {
    "Hello, Domain Exporter!"
}
