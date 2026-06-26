use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use sdkwork_invoice_service_host::InvoiceServiceHost;

pub fn build_invoice_backend_router(_host: Arc<InvoiceServiceHost>) -> Router {
    Router::new().route(
        "/backend/v3/api/invoices/health",
        get(|| async { "ok" }),
    )
}

pub async fn build_invoice_backend_router_with_framework(host: Arc<InvoiceServiceHost>) -> Router {
    build_invoice_backend_router(host)
}
