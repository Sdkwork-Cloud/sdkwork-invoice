pub mod routes;
pub mod web_bootstrap;

pub use routes::build_invoice_backend_router_with_framework;

use axum::Router;
use sdkwork_invoice_service_host::InvoiceServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<InvoiceServiceHost>) -> Router {
    build_invoice_backend_router_with_framework(host).await
}
