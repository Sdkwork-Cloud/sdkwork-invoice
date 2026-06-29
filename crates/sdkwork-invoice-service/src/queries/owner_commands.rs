#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateOwnerInvoiceCommand {
    pub invoice_type: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tax_no: Option<String>,
    pub tenant_id: String,
    pub title: String,
    pub title_type: String,
    pub total_amount: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitOwnerInvoiceCommand {
    pub invoice_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelOwnerInvoiceCommand {
    pub cancel_reason: Option<String>,
    pub invoice_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateOwnerInvoiceCommand {
    pub bank_account: Option<String>,
    pub bank_name: Option<String>,
    pub invoice_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub register_address: Option<String>,
    pub register_phone: Option<String>,
    pub tax_no: Option<String>,
    pub tenant_id: String,
    pub title: Option<String>,
}

impl CreateOwnerInvoiceCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        title: &str,
        title_type: &str,
        tax_no: Option<&str>,
        total_amount: &str,
        invoice_type: &str,
    ) -> Result<Self, sdkwork_contract_service::CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("title", title)?;
        crate::validation::require_non_empty("title_type", title_type)?;
        crate::validation::require_non_empty("total_amount", total_amount)?;
        crate::validation::require_non_empty("invoice_type", invoice_type)?;

        Ok(Self {
            invoice_type: invoice_type.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tax_no: optional_text(tax_no),
            tenant_id: tenant_id.trim().to_string(),
            title: title.trim().to_string(),
            title_type: title_type.trim().to_ascii_lowercase(),
            total_amount: total_amount.trim().to_string(),
        })
    }
}

impl SubmitOwnerInvoiceCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        invoice_id: &str,
    ) -> Result<Self, sdkwork_contract_service::CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("invoice_id", invoice_id)?;

        Ok(Self {
            invoice_id: invoice_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl CancelOwnerInvoiceCommand {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        invoice_id: &str,
        cancel_reason: Option<&str>,
    ) -> Result<Self, sdkwork_contract_service::CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("invoice_id", invoice_id)?;

        Ok(Self {
            cancel_reason: optional_text(cancel_reason),
            invoice_id: invoice_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            tenant_id: tenant_id.trim().to_string(),
        })
    }
}

impl UpdateOwnerInvoiceCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        invoice_id: &str,
        title: Option<&str>,
        tax_no: Option<&str>,
        bank_name: Option<&str>,
        bank_account: Option<&str>,
        register_address: Option<&str>,
        register_phone: Option<&str>,
    ) -> Result<Self, sdkwork_contract_service::CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("owner_user_id", owner_user_id)?;
        crate::validation::require_non_empty("invoice_id", invoice_id)?;

        Ok(Self {
            bank_account: optional_text(bank_account),
            bank_name: optional_text(bank_name),
            invoice_id: invoice_id.trim().to_string(),
            organization_id: optional_text(organization_id),
            owner_user_id: owner_user_id.trim().to_string(),
            register_address: optional_text(register_address),
            register_phone: optional_text(register_phone),
            tax_no: optional_text(tax_no),
            tenant_id: tenant_id.trim().to_string(),
            title: optional_text(title),
        })
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
