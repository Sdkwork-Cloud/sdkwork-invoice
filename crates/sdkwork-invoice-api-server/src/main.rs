use axum::Router;
use sdkwork_router_invoice_app_api::build_invoice_app_router_with_framework;
use sdkwork_router_invoice_backend_api::build_invoice_backend_router_with_framework;
use sdkwork_invoice_api_server::invoice_health_router;
use sdkwork_invoice_service_host::InvoiceServiceHost;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = Arc::new(InvoiceServiceHost::new().await);
    let app = Router::new()
        .merge(invoice_health_router())
        .merge(build_invoice_app_router_with_framework(host.clone()).await)
        .merge(build_invoice_backend_router_with_framework(host).await)
        .layer(CorsLayer::permissive());
    let addr = std::env::var("INVOICE_API_BIND").unwrap_or_else(|_| "0.0.0.0:18098".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
