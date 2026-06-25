mod owner_commands;

pub use owner_commands::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceDetailQuery {
    pub invoice_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

impl InvoiceListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, sdkwork_commerce_contract_service::CommerceServiceError> {
        validate_page(page)?;
        validate_page_size(page_size)?;
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            page,
            page_size,
            status: optional_text(status),
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }

    pub fn page_no(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(50).clamp(1, 200)
    }

    pub fn offset(&self) -> i64 {
        (self.page_no() - 1) * self.limit()
    }
}

impl InvoiceDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        invoice_id: &str,
    ) -> Result<Self, sdkwork_commerce_contract_service::CommerceServiceError> {
        Ok(Self {
            invoice_id: required_text("invoice_id", invoice_id)?,
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

fn validate_page(
    page: Option<i64>,
) -> Result<(), sdkwork_commerce_contract_service::CommerceServiceError> {
    if let Some(page) = page {
        if page < 1 {
            return Err(
                sdkwork_commerce_contract_service::CommerceServiceError::validation(
                    "page must be greater than or equal to 1",
                ),
            );
        }
    }
    Ok(())
}

fn validate_page_size(
    page_size: Option<i64>,
) -> Result<(), sdkwork_commerce_contract_service::CommerceServiceError> {
    if let Some(page_size) = page_size {
        if !(1..=200).contains(&page_size) {
            return Err(
                sdkwork_commerce_contract_service::CommerceServiceError::validation(
                    "page_size must be between 1 and 200",
                ),
            );
        }
    }
    Ok(())
}

fn required_text(
    field_name: &str,
    value: &str,
) -> Result<String, sdkwork_commerce_contract_service::CommerceServiceError> {
    crate::validation::require_non_empty(field_name, value)?;
    Ok(value.trim().to_string())
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
