pub mod command_headers;
pub mod invoice_router;
pub mod routes;
pub mod subject;
pub mod web_bootstrap;

pub use invoice_router::{
    app_invoice_router_with_postgres_pool, app_invoice_router_with_sqlite_pool,
    build_app_invoice_router, CommerceInvoiceFuture, CommerceInvoiceStore,
};
pub use routes::build_invoice_app_router_with_framework;
pub use web_bootstrap::wrap_router_with_web_framework_from_env;

use axum::Router;
use sdkwork_invoice_service_host::InvoiceServiceHost;
use std::sync::Arc;

pub async fn gateway_mount(host: Arc<InvoiceServiceHost>) -> Router {
    build_invoice_app_router_with_framework(host).await
}
