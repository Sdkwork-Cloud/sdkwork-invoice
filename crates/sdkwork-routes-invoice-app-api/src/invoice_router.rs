use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_invoice_service::{
    CancelOwnerInvoiceCommand, CreateOwnerInvoiceCommand, InvoiceDetailQuery, InvoiceItemRecord,
    InvoiceListPage, InvoiceListQuery, InvoiceRecord, SubmitOwnerInvoiceCommand,
    UpdateOwnerInvoiceCommand,
};
use sdkwork_commerce_invoice_repository_sqlx::{
    PostgresCommerceInvoiceStore, SqliteCommerceInvoiceStore,
};
use sdkwork_iam_context_service::IamAppContext;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, SqlitePool};

use crate::command_headers::{validate_app_write_payload, write_payload_with_route_param};
use crate::subject::{app_runtime_subject_from_extension, AppRuntimeSubject};

pub type CommerceInvoiceFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, CommerceServiceError>> + Send + 'a>>;

pub trait CommerceInvoiceStore: Send + Sync {
    fn list_invoices<'a>(
        &'a self,
        query: InvoiceListQuery,
    ) -> CommerceInvoiceFuture<'a, InvoiceListPage>;

    fn retrieve_invoice<'a>(
        &'a self,
        query: InvoiceDetailQuery,
    ) -> CommerceInvoiceFuture<'a, Option<InvoiceRecord>>;

    fn create_owner_invoice<'a>(
        &'a self,
        command: CreateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord>;

    fn submit_owner_invoice<'a>(
        &'a self,
        command: SubmitOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord>;

    fn cancel_owner_invoice<'a>(
        &'a self,
        command: CancelOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, ()>;

    fn update_owner_invoice<'a>(
        &'a self,
        command: UpdateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord>;
}

#[derive(Clone)]
struct AppInvoiceState {
    store: Arc<dyn CommerceInvoiceStore>,
}

#[derive(Debug, Deserialize)]
struct InvoiceListQueryParams {
    page: Option<i64>,
    #[serde(rename = "pageSize", alias = "page_size")]
    page_size: Option<i64>,
    status: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInvoiceApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoiceCollectionResponse {
    items: Vec<InvoiceResponse>,
    total: i64,
    page: i64,
    page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoiceResourceResponse {
    item: InvoiceResponse,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateInvoiceRequest {
    title: String,
    tax_no: Option<String>,
    title_type: Option<String>,
    total_amount: Option<serde_json::Value>,
    #[serde(rename = "type")]
    invoice_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelInvoiceRequest {
    cancel_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInvoiceRequest {
    bank_account: Option<String>,
    bank_name: Option<String>,
    register_address: Option<String>,
    register_phone: Option<String>,
    tax_no: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoiceMutationResponse {
    invoice_id: String,
    status: String,
    title: String,
    title_type: String,
    total_amount: String,
    #[serde(rename = "type")]
    invoice_type: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoiceResponse {
    id: String,
    tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    organization_id: Option<String>,
    owner_user_id: String,
    order_id: String,
    payment_id: String,
    title_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoice_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invoice_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_url: Option<String>,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    issued_at: Option<String>,
    updated_at: String,
    total_amount: String,
    items: Vec<InvoiceItemResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoiceItemResponse {
    id: String,
    tenant_id: String,
    invoice_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_item_id: Option<String>,
    title: String,
    amount: String,
    tax_amount: String,
    created_at: String,
}

impl CommerceInvoiceStore for SqliteCommerceInvoiceStore {
    fn list_invoices<'a>(
        &'a self,
        query: InvoiceListQuery,
    ) -> CommerceInvoiceFuture<'a, InvoiceListPage> {
        Box::pin(async move { self.list_invoices(query).await })
    }

    fn retrieve_invoice<'a>(
        &'a self,
        query: InvoiceDetailQuery,
    ) -> CommerceInvoiceFuture<'a, Option<InvoiceRecord>> {
        Box::pin(async move { self.retrieve_invoice(query).await })
    }

    fn create_owner_invoice<'a>(
        &'a self,
        command: CreateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.create_owner_invoice(command).await })
    }

    fn submit_owner_invoice<'a>(
        &'a self,
        command: SubmitOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.submit_owner_invoice(command).await })
    }

    fn cancel_owner_invoice<'a>(
        &'a self,
        command: CancelOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_invoice(command).await })
    }

    fn update_owner_invoice<'a>(
        &'a self,
        command: UpdateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.update_owner_invoice(command).await })
    }
}

impl CommerceInvoiceStore for PostgresCommerceInvoiceStore {
    fn list_invoices<'a>(
        &'a self,
        query: InvoiceListQuery,
    ) -> CommerceInvoiceFuture<'a, InvoiceListPage> {
        Box::pin(async move { self.list_invoices(query).await })
    }

    fn retrieve_invoice<'a>(
        &'a self,
        query: InvoiceDetailQuery,
    ) -> CommerceInvoiceFuture<'a, Option<InvoiceRecord>> {
        Box::pin(async move { self.retrieve_invoice(query).await })
    }

    fn create_owner_invoice<'a>(
        &'a self,
        command: CreateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.create_owner_invoice(command).await })
    }

    fn submit_owner_invoice<'a>(
        &'a self,
        command: SubmitOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.submit_owner_invoice(command).await })
    }

    fn cancel_owner_invoice<'a>(
        &'a self,
        command: CancelOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, ()> {
        Box::pin(async move { self.cancel_owner_invoice(command).await })
    }

    fn update_owner_invoice<'a>(
        &'a self,
        command: UpdateOwnerInvoiceCommand,
    ) -> CommerceInvoiceFuture<'a, InvoiceRecord> {
        Box::pin(async move { self.update_owner_invoice(command).await })
    }
}

impl<T: Serialize> AppInvoiceApiResult<T> {
    fn success(data: T) -> Self {
        Self {
            code: "2000".to_owned(),
            msg: "SUCCESS".to_owned(),
            data: Some(data),
        }
    }
}

impl AppInvoiceApiResult<()> {
    fn error(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            data: None,
        }
    }
}

pub fn app_invoice_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_invoice_router(Arc::new(SqliteCommerceInvoiceStore::new(pool)))
}

pub fn app_invoice_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_invoice_router(Arc::new(PostgresCommerceInvoiceStore::new(pool)))
}

pub fn build_app_invoice_router(store: Arc<dyn CommerceInvoiceStore>) -> Router {
    Router::new()
            .route(
                "/app/v3/api/invoices",
                get(fetch_invoices).post(create_invoice),
            )
            .route("/app/v3/api/invoices/mine", get(fetch_invoices))
            .route(
                "/app/v3/api/invoices/statistics",
                get(fetch_invoice_statistics),
            )
            .route(
                "/app/v3/api/invoices/{invoiceId}",
                get(fetch_invoice).patch(update_invoice),
            )
            .route(
                "/app/v3/api/invoices/{invoiceId}/items",
                get(fetch_invoice_items),
            )
            .route(
                "/app/v3/api/invoices/{invoiceId}/submissions",
                post(submit_invoice),
            )
            .route(
                "/app/v3/api/invoices/{invoiceId}/cancellations",
                post(cancel_invoice),
            )
            .with_state(AppInvoiceState { store })
}

async fn fetch_invoices(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Query(params): Query<InvoiceListQueryParams>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match InvoiceListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        params.status.as_deref(),
        params.page,
        params.page_size,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_invoices(query).await {
        Ok(page) => Json(AppInvoiceApiResult::success(map_invoice_page(page))).into_response(),
        Err(error) => invoice_system_response("invoice read model is unavailable", error),
    }
}

async fn fetch_invoice(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(invoice_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match InvoiceDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &invoice_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_invoice(query).await {
        Ok(Some(item)) => Json(AppInvoiceApiResult::success(InvoiceResourceResponse {
            item: map_invoice(item),
        }))
        .into_response(),
        Ok(None) => not_found_response("invoice was not found"),
        Err(error) => invoice_system_response("invoice read model is unavailable", error),
    }
}

async fn fetch_invoice_statistics(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match InvoiceListQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        None,
        Some(1),
        Some(500),
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.list_invoices(query).await {
        Ok(page) => {
            let mut pending = 0_i64;
            let mut issued = 0_i64;
            let mut cancelled = 0_i64;
            for invoice in page.items {
                match invoice.status.to_ascii_lowercase().as_str() {
                    "issued" | "completed" => issued += 1,
                    "cancelled" | "canceled" => cancelled += 1,
                    _ => pending += 1,
                }
            }
            Json(AppInvoiceApiResult::success(serde_json::json!({
                "totalInvoices": page.total,
                "pendingInvoices": pending,
                "issuedInvoices": issued,
                "cancelledInvoices": cancelled,
            })))
            .into_response()
        }
        Err(error) => {
            invoice_system_response("invoice statistics read model is unavailable", error)
        }
    }
}

async fn fetch_invoice_items(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(invoice_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let query = match InvoiceDetailQuery::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &invoice_id,
    ) {
        Ok(query) => query,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.retrieve_invoice(query).await {
        Ok(Some(item)) => Json(AppInvoiceApiResult::success(
            item.items
                .into_iter()
                .map(map_invoice_item)
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Ok(None) => not_found_response("invoice was not found"),
        Err(error) => invoice_system_response("invoice items read model is unavailable", error),
    }
}

async fn create_invoice(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Json(body): Json<CreateInvoiceRequest>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let _write_headers = match validate_app_write_payload(
        &headers,
        "invoices.create",
        &body,
        |idempotency_key| fallback_request_no(&subject, "create", idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let total_amount = body
        .total_amount
        .as_ref()
        .map(|value| value.to_string().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "0.00".to_owned());
    let command = match CreateOwnerInvoiceCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &body.title,
        body.title_type.as_deref().unwrap_or("personal"),
        body.tax_no.as_deref(),
        &total_amount,
        body.invoice_type.as_deref().unwrap_or("normal"),
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.create_owner_invoice(command).await {
        Ok(record) => Json(AppInvoiceApiResult::success(map_invoice_mutation(
            record,
            &body.title,
            body.title_type.as_deref().unwrap_or("personal"),
            body.invoice_type.as_deref().unwrap_or("normal"),
        )))
        .into_response(),
        Err(error) => invoice_system_response("invoice create command failed", error),
    }
}

async fn submit_invoice(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(invoice_id): Path<String>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let payload = write_payload_with_route_param("invoiceId", &invoice_id, &serde_json::json!({}));
    let _write_headers = match validate_app_write_payload(
        &headers,
        "invoices.submit",
        &payload,
        |idempotency_key| fallback_request_no(&subject, &invoice_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match SubmitOwnerInvoiceCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &invoice_id,
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.submit_owner_invoice(command).await {
        Ok(record) => {
            let title = record
                .items
                .first()
                .map(|item| item.title.clone())
                .unwrap_or_default();
            Json(AppInvoiceApiResult::success(map_invoice_mutation(
                record, &title, "personal", "normal",
            )))
            .into_response()
        }
        Err(error) => invoice_system_response("invoice submit command failed", error),
    }
}

async fn cancel_invoice(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(invoice_id): Path<String>,
    body: Option<Json<CancelInvoiceRequest>>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let cancel_body = body.map(|Json(value)| value);
    let payload = if let Some(ref cancel_body) = cancel_body {
        write_payload_with_route_param("invoiceId", &invoice_id, cancel_body)
    } else {
        write_payload_with_route_param("invoiceId", &invoice_id, &serde_json::json!({}))
    };
    let _write_headers = match validate_app_write_payload(
        &headers,
        "invoices.cancel",
        &payload,
        |idempotency_key| fallback_request_no(&subject, &invoice_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let cancel_reason = cancel_body.and_then(|body| body.cancel_reason);
    let command = match CancelOwnerInvoiceCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &invoice_id,
        cancel_reason.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.cancel_owner_invoice(command).await {
        Ok(()) => (
            StatusCode::OK,
            Json(AppInvoiceApiResult::<()> {
                code: "2000".to_owned(),
                msg: "SUCCESS".to_owned(),
                data: None,
            }),
        )
            .into_response(),
        Err(error) => invoice_system_response("invoice cancel command failed", error),
    }
}

async fn update_invoice(
    State(state): State<AppInvoiceState>,
    runtime_context: Option<Extension<IamAppContext>>,
    headers: HeaderMap,
    Path(invoice_id): Path<String>,
    Json(body): Json<UpdateInvoiceRequest>,
) -> Response {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return unauthorized_response(message),
    };
    let payload = write_payload_with_route_param("invoiceId", &invoice_id, &body);
    let _write_headers = match validate_app_write_payload(
        &headers,
        "invoices.update",
        &payload,
        |idempotency_key| fallback_request_no(&subject, &invoice_id, idempotency_key),
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let command = match UpdateOwnerInvoiceCommand::new(
        &subject.tenant_id,
        subject.organization_id.as_deref(),
        &subject.user_id,
        &invoice_id,
        body.title.as_deref(),
        body.tax_no.as_deref(),
        body.bank_name.as_deref(),
        body.bank_account.as_deref(),
        body.register_address.as_deref(),
        body.register_phone.as_deref(),
    ) {
        Ok(command) => command,
        Err(error) => return validation_response(error.message()),
    };

    match state.store.update_owner_invoice(command).await {
        Ok(record) => Json(AppInvoiceApiResult::success(map_invoice_mutation(
            record,
            body.title.as_deref().unwrap_or(""),
            "personal",
            "normal",
        )))
        .into_response(),
        Err(error) => invoice_system_response("invoice update command failed", error),
    }
}

fn map_invoice_mutation(
    value: InvoiceRecord,
    title: &str,
    title_type: &str,
    invoice_type: &str,
) -> InvoiceMutationResponse {
    InvoiceMutationResponse {
        invoice_id: value.id,
        status: value.status,
        title: if title.is_empty() {
            value
                .items
                .first()
                .map(|item| item.title.clone())
                .unwrap_or_default()
        } else {
            title.to_owned()
        },
        title_type: title_type.to_owned(),
        total_amount: value.total_amount,
        invoice_type: invoice_type.to_owned(),
        created_at: value.created_at,
        updated_at: value.updated_at,
    }
}

fn map_invoice_page(page: InvoiceListPage) -> InvoiceCollectionResponse {
    InvoiceCollectionResponse {
        items: page.items.into_iter().map(map_invoice).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn map_invoice(value: InvoiceRecord) -> InvoiceResponse {
    InvoiceResponse {
        id: value.id,
        tenant_id: value.tenant_id,
        organization_id: value.organization_id,
        owner_user_id: value.owner_user_id,
        order_id: value.order_id,
        payment_id: value.payment_id,
        title_id: value.title_id,
        status: value.status,
        invoice_no: value.invoice_no,
        invoice_code: value.invoice_code,
        document_url: value.document_url,
        created_at: value.created_at,
        issued_at: value.issued_at,
        updated_at: value.updated_at,
        total_amount: value.total_amount,
        items: value.items.into_iter().map(map_invoice_item).collect(),
    }
}

fn map_invoice_item(value: InvoiceItemRecord) -> InvoiceItemResponse {
    InvoiceItemResponse {
        id: value.id,
        tenant_id: value.tenant_id,
        invoice_id: value.invoice_id,
        order_item_id: value.order_item_id,
        title: value.title,
        amount: value.amount,
        tax_amount: value.tax_amount,
        created_at: value.created_at,
    }
}

fn unauthorized_response(message: String) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(AppInvoiceApiResult::error("4010", message)),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(AppInvoiceApiResult::error("4001", message)),
    )
        .into_response()
}

fn not_found_response(message: impl Into<String>) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(AppInvoiceApiResult::error("4040", message)),
    )
        .into_response()
}

fn fallback_request_no(subject: &AppRuntimeSubject, invoice_id: &str, idempotency_key: &str) -> String {
    format!(
        "invoice-{}-{}-{}",
        subject.user_id, invoice_id, idempotency_key
    )
}

fn invoice_system_response(context: &str, error: CommerceServiceError) -> Response {
    match error.code() {
        "validation" => validation_response(error.message()),
        "unauthenticated" | "unauthorized" => unauthorized_response(error.message().to_owned()),
        "not-found" => not_found_response(error.message()),
        "conflict" => (
            StatusCode::CONFLICT,
            Json(AppInvoiceApiResult::error("4090", error.message())),
        )
            .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AppInvoiceApiResult::error(
                "5000",
                format!("{context}: {}", error.message()),
            )),
        )
            .into_response(),
    }
}
