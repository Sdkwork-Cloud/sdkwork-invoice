use std::collections::HashMap;

use sdkwork_commerce_contract_service::CommerceServiceError;
use sdkwork_commerce_invoice_service::{
    CancelOwnerInvoiceCommand, CreateOwnerInvoiceCommand, InvoiceDetailQuery, InvoiceItemRecord,
    InvoiceListPage, InvoiceListQuery, InvoiceRecord, SubmitOwnerInvoiceCommand,
    UpdateOwnerInvoiceCommand,
};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SqliteCommerceInvoiceStore {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
struct InvoiceRow {
    id: String,
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: String,
    order_id: String,
    payment_id: String,
    title_id: String,
    status: String,
    invoice_no: Option<String>,
    invoice_code: Option<String>,
    document_url: Option<String>,
    created_at: String,
    issued_at: Option<String>,
    updated_at: String,
}

impl SqliteCommerceInvoiceStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_invoices(
        &self,
        query: InvoiceListQuery,
    ) -> Result<InvoiceListPage, CommerceServiceError> {
        let total = count_invoices(&self.pool, &query).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, owner_user_id, order_id, payment_id,
                   title_id, status, invoice_no, invoice_code, document_url,
                   created_at, issued_at, updated_at
            FROM commerce_invoice
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? IS NULL))
              AND owner_user_id = CAST(? AS TEXT)
              AND (? IS NULL OR status = ?)
            ORDER BY COALESCE(issued_at, created_at) DESC, id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(query.status.as_deref())
        .bind(query.status.as_deref())
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list invoices", error))?;

        let invoice_rows = rows.iter().map(map_invoice_row).collect::<Vec<_>>();
        let items_by_invoice =
            load_items_by_invoice(&self.pool, &query.tenant_id, invoice_rows.as_slice()).await?;
        let invoices = invoice_rows
            .into_iter()
            .map(|row| invoice_from_row(row, &items_by_invoice))
            .collect::<Result<Vec<_>, _>>()?;

        InvoiceListPage::new(invoices, total, query.page_no(), query.limit())
    }

    pub async fn retrieve_invoice(
        &self,
        query: InvoiceDetailQuery,
    ) -> Result<Option<InvoiceRecord>, CommerceServiceError> {
        let row = sqlx::query(
            r#"
            SELECT id, tenant_id, organization_id, owner_user_id, order_id, payment_id,
                   title_id, status, invoice_no, invoice_code, document_url,
                   created_at, issued_at, updated_at
            FROM commerce_invoice
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? IS NULL))
              AND owner_user_id = CAST(? AS TEXT)
              AND id = CAST(? AS TEXT)
            "#,
        )
        .bind(&query.tenant_id)
        .bind(query.organization_id.as_deref())
        .bind(query.organization_id.as_deref())
        .bind(&query.owner_user_id)
        .bind(&query.invoice_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve invoice", error))?;

        let Some(row) = row else {
            return Ok(None);
        };
        let invoice_row = map_invoice_row(&row);
        let items_by_invoice = load_items_by_invoice(
            &self.pool,
            &query.tenant_id,
            std::slice::from_ref(&invoice_row),
        )
        .await?;

        invoice_from_row(invoice_row, &items_by_invoice).map(Some)
    }

    pub async fn create_owner_invoice(
        &self,
        command: CreateOwnerInvoiceCommand,
    ) -> Result<InvoiceRecord, CommerceServiceError> {
        let now = invoice_command_timestamp();
        let invoice_id = format!("invoice-{now}");
        let title_id = format!("title-{now}");
        let item_id = format!("invoice-item-{now}");
        let order_id = format!("manual-{invoice_id}");
        let payment_id = format!("manual-payment-{invoice_id}");

        let mut tx =
            self.pool.begin().await.map_err(|error| {
                store_error("failed to begin create invoice transaction", error)
            })?;

        sqlx::query(
            r#"
            INSERT INTO commerce_invoice_title
                (id, tenant_id, owner_user_id, title_type, name, tax_no, created_at, updated_at)
            VALUES (?, CAST(? AS TEXT), CAST(? AS TEXT), ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&title_id)
        .bind(&command.tenant_id)
        .bind(&command.owner_user_id)
        .bind(&command.title_type)
        .bind(&command.title)
        .bind(command.tax_no.as_deref())
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert invoice title", error))?;

        sqlx::query(
            r#"
            INSERT INTO commerce_invoice
                (id, tenant_id, organization_id, owner_user_id, order_id, payment_id, title_id,
                 status, invoice_no, invoice_code, document_url, created_at, issued_at, updated_at)
            VALUES
                (?, CAST(? AS TEXT), CAST(? AS TEXT), CAST(? AS TEXT), ?, ?, ?, 'draft', NULL, NULL, NULL, ?, NULL, ?)
            "#,
        )
        .bind(&invoice_id)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .bind(&order_id)
        .bind(&payment_id)
        .bind(&title_id)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert invoice", error))?;

        sqlx::query(
            r#"
            INSERT INTO commerce_invoice_item
                (id, tenant_id, invoice_id, order_item_id, title, amount, tax_amount, created_at)
            VALUES (?, CAST(? AS TEXT), ?, NULL, ?, ?, '0.00', ?)
            "#,
        )
        .bind(&item_id)
        .bind(&command.tenant_id)
        .bind(&invoice_id)
        .bind(&command.title)
        .bind(&command.total_amount)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert invoice item", error))?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit create invoice transaction", error))?;

        let query = InvoiceDetailQuery::new(
            &command.tenant_id,
            command.organization_id.as_deref(),
            &command.owner_user_id,
            &invoice_id,
        )?;
        self.retrieve_invoice(query)
            .await?
            .ok_or_else(|| CommerceServiceError::storage("created invoice was not readable"))
    }

    pub async fn submit_owner_invoice(
        &self,
        command: SubmitOwnerInvoiceCommand,
    ) -> Result<InvoiceRecord, CommerceServiceError> {
        let now = invoice_command_timestamp();
        let updated = sqlx::query(
            r#"
            UPDATE commerce_invoice
            SET status = 'submitted', updated_at = ?
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? IS NULL))
              AND owner_user_id = CAST(? AS TEXT)
              AND id = CAST(? AS TEXT)
              AND LOWER(COALESCE(status, '')) IN ('draft', 'failed')
            "#,
        )
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .bind(&command.invoice_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("failed to submit invoice", error))?;

        if updated.rows_affected() == 0 {
            return Err(CommerceServiceError::conflict(
                "invoice is not submittable or was not found",
            ));
        }

        let query = InvoiceDetailQuery::new(
            &command.tenant_id,
            command.organization_id.as_deref(),
            &command.owner_user_id,
            &command.invoice_id,
        )?;
        self.retrieve_invoice(query)
            .await?
            .ok_or_else(|| CommerceServiceError::not_found("invoice was not found"))
    }

    pub async fn cancel_owner_invoice(
        &self,
        command: CancelOwnerInvoiceCommand,
    ) -> Result<(), CommerceServiceError> {
        let now = invoice_command_timestamp();
        let updated = sqlx::query(
            r#"
            UPDATE commerce_invoice
            SET status = 'cancelled', updated_at = ?
            WHERE tenant_id = CAST(? AS TEXT)
              AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? IS NULL))
              AND owner_user_id = CAST(? AS TEXT)
              AND id = CAST(? AS TEXT)
              AND LOWER(COALESCE(status, '')) IN ('issued', 'completed')
            "#,
        )
        .bind(&now)
        .bind(&command.tenant_id)
        .bind(command.organization_id.as_deref())
        .bind(command.organization_id.as_deref())
        .bind(&command.owner_user_id)
        .bind(&command.invoice_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("failed to cancel invoice", error))?;

        if updated.rows_affected() == 0 {
            return Err(CommerceServiceError::conflict(
                "invoice is not cancellable or was not found",
            ));
        }

        let _ = command.cancel_reason;
        Ok(())
    }

    pub async fn update_owner_invoice(
        &self,
        command: UpdateOwnerInvoiceCommand,
    ) -> Result<InvoiceRecord, CommerceServiceError> {
        let query = InvoiceDetailQuery::new(
            &command.tenant_id,
            command.organization_id.as_deref(),
            &command.owner_user_id,
            &command.invoice_id,
        )?;
        let Some(existing) = self.retrieve_invoice(query.clone()).await? else {
            return Err(CommerceServiceError::not_found("invoice was not found"));
        };
        if !matches!(
            existing.status.to_ascii_lowercase().as_str(),
            "draft" | "failed"
        ) {
            return Err(CommerceServiceError::conflict(
                "invoice is not editable in its current status",
            ));
        }

        let now = invoice_command_timestamp();
        if command.title.is_some() || command.tax_no.is_some() {
            sqlx::query(
                r#"
                UPDATE commerce_invoice_title
                SET name = COALESCE(?, name),
                    tax_no = COALESCE(?, tax_no),
                    updated_at = ?
                WHERE id = ?
                  AND tenant_id = CAST(? AS TEXT)
                  AND owner_user_id = CAST(? AS TEXT)
                "#,
            )
            .bind(command.title.as_deref())
            .bind(command.tax_no.as_deref())
            .bind(&now)
            .bind(&existing.title_id)
            .bind(&command.tenant_id)
            .bind(&command.owner_user_id)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("failed to update invoice title", error))?;
        }

        sqlx::query(
            r#"
            UPDATE commerce_invoice
            SET updated_at = ?
            WHERE id = ?
              AND tenant_id = CAST(? AS TEXT)
              AND owner_user_id = CAST(? AS TEXT)
            "#,
        )
        .bind(&now)
        .bind(&command.invoice_id)
        .bind(&command.tenant_id)
        .bind(&command.owner_user_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("failed to update invoice", error))?;

        let _ = (
            command.bank_account,
            command.bank_name,
            command.register_address,
            command.register_phone,
        );
        self.retrieve_invoice(query)
            .await?
            .ok_or_else(|| CommerceServiceError::not_found("invoice was not found"))
    }
}

async fn count_invoices(
    pool: &SqlitePool,
    query: &InvoiceListQuery,
) -> Result<i64, CommerceServiceError> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM commerce_invoice
        WHERE tenant_id = CAST(? AS TEXT)
          AND ((organization_id = CAST(? AS TEXT)) OR (organization_id IS NULL AND ? IS NULL))
          AND owner_user_id = CAST(? AS TEXT)
          AND (? IS NULL OR status = ?)
        "#,
    )
    .bind(&query.tenant_id)
    .bind(query.organization_id.as_deref())
    .bind(query.organization_id.as_deref())
    .bind(&query.owner_user_id)
    .bind(query.status.as_deref())
    .bind(query.status.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|error| store_error("failed to count invoices", error))
}

async fn load_items_by_invoice(
    pool: &SqlitePool,
    tenant_id: &str,
    invoices: &[InvoiceRow],
) -> Result<HashMap<String, Vec<InvoiceItemRecord>>, CommerceServiceError> {
    if invoices.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT id, tenant_id, invoice_id, order_item_id, title, amount, tax_amount, created_at \
         FROM commerce_invoice_item WHERE tenant_id = ",
    );
    builder.push_bind(tenant_id);
    builder.push(" AND invoice_id IN (");
    {
        let mut separated = builder.separated(", ");
        for invoice in invoices {
            separated.push_bind(&invoice.id);
        }
        separated.push_unseparated(")");
    }
    builder.push(" ORDER BY created_at ASC, id ASC");

    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(|error| store_error("failed to list invoice items", error))?;

    let mut items_by_invoice: HashMap<String, Vec<InvoiceItemRecord>> = HashMap::new();
    for row in rows {
        let invoice_id = string_cell(&row, "invoice_id");
        let item = InvoiceItemRecord::new(
            &string_cell(&row, "id"),
            &string_cell(&row, "tenant_id"),
            &invoice_id,
            optional_string_cell(&row, "order_item_id").as_deref(),
            &string_cell(&row, "title"),
            &string_cell(&row, "amount"),
            &string_cell(&row, "tax_amount"),
            &string_cell(&row, "created_at"),
        )?;
        items_by_invoice.entry(invoice_id).or_default().push(item);
    }
    Ok(items_by_invoice)
}

fn invoice_from_row(
    row: InvoiceRow,
    items_by_invoice: &HashMap<String, Vec<InvoiceItemRecord>>,
) -> Result<InvoiceRecord, CommerceServiceError> {
    let items = items_by_invoice
        .get(&row.id)
        .cloned()
        .unwrap_or_else(Vec::new);
    InvoiceRecord::new(
        &row.id,
        &row.tenant_id,
        row.organization_id.as_deref(),
        &row.owner_user_id,
        &row.order_id,
        &row.payment_id,
        &row.title_id,
        &row.status,
        row.invoice_no.as_deref(),
        row.invoice_code.as_deref(),
        row.document_url.as_deref(),
        &row.created_at,
        row.issued_at.as_deref(),
        &row.updated_at,
        items,
    )
}

fn map_invoice_row(row: &sqlx::sqlite::SqliteRow) -> InvoiceRow {
    InvoiceRow {
        id: string_cell(row, "id"),
        tenant_id: string_cell(row, "tenant_id"),
        organization_id: optional_string_cell(row, "organization_id"),
        owner_user_id: string_cell(row, "owner_user_id"),
        order_id: string_cell(row, "order_id"),
        payment_id: string_cell(row, "payment_id"),
        title_id: string_cell(row, "title_id"),
        status: string_cell(row, "status"),
        invoice_no: optional_string_cell(row, "invoice_no"),
        invoice_code: optional_string_cell(row, "invoice_code"),
        document_url: optional_string_cell(row, "document_url"),
        created_at: string_cell(row, "created_at"),
        issued_at: optional_string_cell(row, "issued_at"),
        updated_at: string_cell(row, "updated_at"),
    }
}

fn string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::sqlite::SqliteRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn store_error(context: &str, error: sqlx::Error) -> CommerceServiceError {
    CommerceServiceError::storage(format!("{context}: {error}"))
}

fn invoice_command_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format!("{seconds}")
}
