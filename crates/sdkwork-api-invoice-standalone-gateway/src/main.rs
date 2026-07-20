use sdkwork_api_invoice_assembly::assemble_api_router;
use sdkwork_invoice_service_host::InvoiceServiceHost;
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = Arc::new(InvoiceServiceHost::new().await);
    let business = assemble_api_router(host).await.router
        .layer(sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_INVOICE_ENVIRONMENT"],
            &["SDKWORK_INVOICE_CORS_ALLOWED_ORIGINS", "SDKWORK_CORS_ALLOWED_ORIGINS"],
        ));
    let app = service_router(business, ServiceRouterConfig::default().with_always_ready());
    let addr = std::env::var("INVOICE_API_BIND").unwrap_or_else(|_| "0.0.0.0:18098".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
