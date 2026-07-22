use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceTitle {
    pub name: String,
    pub tax_no: Option<String>,
    pub title_type: InvoiceTitleType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceTitleType {
    Company,
    Personal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Submitted,
    Reviewing,
    Issued,
    Cancelled,
    Rejected,
    Voided,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceTransition {
    from: InvoiceStatus,
    to: InvoiceStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceApplicationDraft {
    pub order_id: String,
    pub payment_id: String,
    pub tenant_id: String,
    pub title_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceItemRecord {
    pub id: String,
    pub tenant_id: String,
    pub invoice_id: String,
    pub order_item_id: Option<String>,
    pub title: String,
    pub amount: String,
    pub tax_amount: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceRecord {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub order_id: String,
    pub payment_id: String,
    pub title_id: String,
    pub status: String,
    pub invoice_no: Option<String>,
    pub invoice_code: Option<String>,
    pub document_url: Option<String>,
    pub created_at: String,
    pub issued_at: Option<String>,
    pub updated_at: String,
    pub total_amount: String,
    pub items: Vec<InvoiceItemRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceListPage {
    pub items: Vec<InvoiceRecord>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceItemListPage {
    pub items: Vec<InvoiceItemRecord>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceStatistics {
    pub total: i64,
    pub pending: i64,
    pub issued: i64,
    pub cancelled: i64,
}

impl InvoiceTitle {
    pub fn company(name: &str, tax_no: &str) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("invoice title name", name)?;
        crate::validation::require_non_empty("company tax_no", tax_no)?;

        Ok(Self {
            name: name.to_string(),
            tax_no: Some(tax_no.to_string()),
            title_type: InvoiceTitleType::Company,
        })
    }
}

impl InvoiceTransition {
    pub fn new(from: InvoiceStatus, to: InvoiceStatus) -> Self {
        Self { from, to }
    }

    pub fn validate(&self) -> Result<(), CommerceServiceError> {
        match (&self.from, &self.to) {
            (InvoiceStatus::Draft, InvoiceStatus::Submitted)
            | (InvoiceStatus::Submitted, InvoiceStatus::Reviewing)
            | (InvoiceStatus::Reviewing, InvoiceStatus::Issued)
            | (InvoiceStatus::Submitted, InvoiceStatus::Cancelled)
            | (InvoiceStatus::Reviewing, InvoiceStatus::Rejected)
            | (InvoiceStatus::Issued, InvoiceStatus::Voided) => Ok(()),
            _ => Err(CommerceServiceError::invalid_state(
                "invalid invoice status transition",
            )),
        }
    }
}

impl InvoiceApplicationDraft {
    pub fn new(
        tenant_id: &str,
        order_id: &str,
        payment_id: &str,
        title_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        crate::validation::require_non_empty("tenant_id", tenant_id)?;
        crate::validation::require_non_empty("order_id", order_id)?;
        crate::validation::require_non_empty("payment_id", payment_id)?;
        crate::validation::require_non_empty("title_id", title_id)?;

        Ok(Self {
            order_id: order_id.to_string(),
            payment_id: payment_id.to_string(),
            tenant_id: tenant_id.to_string(),
            title_id: title_id.to_string(),
        })
    }
}

impl InvoiceItemRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        tenant_id: &str,
        invoice_id: &str,
        order_item_id: Option<&str>,
        title: &str,
        amount: &str,
        tax_amount: &str,
        created_at: &str,
    ) -> Result<Self, CommerceServiceError> {
        require_non_empty_service("id", id)?;
        require_non_empty_service("tenant_id", tenant_id)?;
        require_non_empty_service("invoice_id", invoice_id)?;
        require_non_empty_service("title", title)?;
        require_decimal_amount("amount", amount)?;
        require_decimal_amount("tax_amount", tax_amount)?;
        require_non_empty_service("created_at", created_at)?;

        Ok(Self {
            id: id.to_string(),
            tenant_id: tenant_id.to_string(),
            invoice_id: invoice_id.to_string(),
            order_item_id: normalize_optional_text(order_item_id),
            title: title.to_string(),
            amount: normalize_decimal(amount),
            tax_amount: normalize_decimal(tax_amount),
            created_at: created_at.to_string(),
        })
    }
}

impl InvoiceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        order_id: &str,
        payment_id: &str,
        title_id: &str,
        status: &str,
        invoice_no: Option<&str>,
        invoice_code: Option<&str>,
        document_url: Option<&str>,
        created_at: &str,
        issued_at: Option<&str>,
        updated_at: &str,
        items: Vec<InvoiceItemRecord>,
    ) -> Result<Self, CommerceServiceError> {
        require_non_empty_service("id", id)?;
        require_non_empty_service("tenant_id", tenant_id)?;
        require_non_empty_service("owner_user_id", owner_user_id)?;
        require_non_empty_service("order_id", order_id)?;
        require_non_empty_service("payment_id", payment_id)?;
        require_non_empty_service("title_id", title_id)?;
        require_non_empty_service("status", status)?;
        require_non_empty_service("created_at", created_at)?;
        require_non_empty_service("updated_at", updated_at)?;
        let total_amount = total_invoice_amount(&items)?;

        Ok(Self {
            id: id.to_string(),
            tenant_id: tenant_id.to_string(),
            organization_id: normalize_optional_text(organization_id),
            owner_user_id: owner_user_id.to_string(),
            order_id: order_id.to_string(),
            payment_id: payment_id.to_string(),
            title_id: title_id.to_string(),
            status: status.to_string(),
            invoice_no: normalize_optional_text(invoice_no),
            invoice_code: normalize_optional_text(invoice_code),
            document_url: normalize_optional_text(document_url),
            created_at: created_at.to_string(),
            issued_at: normalize_optional_text(issued_at),
            updated_at: updated_at.to_string(),
            total_amount,
            items,
        })
    }
}

impl InvoiceListPage {
    pub fn new(
        items: Vec<InvoiceRecord>,
        total: i64,
        page: i64,
        page_size: i64,
    ) -> Result<Self, CommerceServiceError> {
        if total < 0 {
            return Err(CommerceServiceError::validation(
                "invoice total must not be negative",
            ));
        }
        if page < 1 {
            return Err(CommerceServiceError::validation(
                "page must be greater than or equal to 1",
            ));
        }
        if !(1..=200).contains(&page_size) {
            return Err(CommerceServiceError::validation(
                "page_size must be between 1 and 200",
            ));
        }
        Ok(Self {
            items,
            total,
            page,
            page_size,
        })
    }
}

impl InvoiceItemListPage {
    pub fn new(
        items: Vec<InvoiceItemRecord>,
        total: i64,
        page: i64,
        page_size: i64,
    ) -> Result<Self, CommerceServiceError> {
        if total < 0 || page < 1 || !(1..=200).contains(&page_size) {
            return Err(CommerceServiceError::validation(
                "invoice item page metadata is invalid",
            ));
        }
        Ok(Self {
            items,
            total,
            page,
            page_size,
        })
    }
}

fn total_invoice_amount(items: &[InvoiceItemRecord]) -> Result<String, CommerceServiceError> {
    let mut total = DecimalAmount::zero();
    for item in items {
        total = total.checked_add(DecimalAmount::parse(&item.amount).map_err(|message| {
            CommerceServiceError::validation(format!("invoice item amount is invalid: {message}"))
        })?)?;
    }
    Ok(total.to_display_string())
}

fn require_non_empty_service(field: &str, value: &str) -> Result<(), CommerceServiceError> {
    crate::validation::require_non_empty(field, value)
}

fn require_decimal_amount(field: &str, value: &str) -> Result<(), CommerceServiceError> {
    DecimalAmount::parse(value).map(|_| ()).map_err(|message| {
        CommerceServiceError::validation(format!("{field} is invalid: {message}"))
    })
}

fn normalize_decimal(value: &str) -> String {
    DecimalAmount::parse(value)
        .map(|amount| amount.to_display_string())
        .unwrap_or_else(|_| value.trim().to_string())
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DecimalAmount {
    units: i128,
}

impl DecimalAmount {
    const SCALE: usize = 6;
    const FACTOR: i128 = 1_000_000;

    fn zero() -> Self {
        Self { units: 0 }
    }

    fn parse(value: &str) -> Result<Self, &'static str> {
        let value = value.trim();
        if value.is_empty() || value.starts_with('-') || value.starts_with('+') {
            return Err("amount must be a non-negative decimal");
        }
        let mut parts = value.split('.');
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next();
        if parts.next().is_some()
            || integer.is_empty()
            || !integer.chars().all(|ch| ch.is_ascii_digit())
        {
            return Err("amount must be a non-negative decimal");
        }
        let integer_units = integer
            .parse::<i128>()
            .map_err(|_| "amount is too large")?
            .checked_mul(Self::FACTOR)
            .ok_or("amount is too large")?;
        let fraction_units = match fraction {
            Some(fraction) => {
                if fraction.is_empty()
                    || fraction.len() > Self::SCALE
                    || !fraction.chars().all(|ch| ch.is_ascii_digit())
                {
                    return Err("amount scale must be between 0 and 6");
                }
                let mut padded = fraction.to_string();
                while padded.len() < Self::SCALE {
                    padded.push('0');
                }
                padded.parse::<i128>().map_err(|_| "amount is too large")?
            }
            None => 0,
        };
        Ok(Self {
            units: integer_units
                .checked_add(fraction_units)
                .ok_or("amount is too large")?,
        })
    }

    fn checked_add(self, other: Self) -> Result<Self, CommerceServiceError> {
        self.units
            .checked_add(other.units)
            .map(|units| Self { units })
            .ok_or_else(|| CommerceServiceError::validation("invoice total amount is too large"))
    }

    fn to_display_string(self) -> String {
        let integer = self.units / Self::FACTOR;
        let fraction = self.units % Self::FACTOR;
        if fraction == 0 {
            return integer.to_string();
        }
        let mut fraction = format!("{fraction:0width$}", width = Self::SCALE);
        while fraction.ends_with('0') {
            fraction.pop();
        }
        format!("{integer}.{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_record_sums_item_amounts_without_float_rounding() {
        let first = InvoiceItemRecord::new(
            "item-1",
            "100001",
            "invoice-1",
            None,
            "LLM usage",
            "80.00",
            "8.25",
            "2026-05-21T00:00:00Z",
        )
        .unwrap();
        let second = InvoiceItemRecord::new(
            "item-2",
            "100001",
            "invoice-1",
            None,
            "Image usage",
            "8.25",
            "0",
            "2026-05-21T00:00:00Z",
        )
        .unwrap();

        let invoice = InvoiceRecord::new(
            "invoice-1",
            "100001",
            Some("0"),
            "user-1",
            "order-1",
            "payment-1",
            "title-1",
            "issued",
            Some("INV-2026-05"),
            None,
            None,
            "2026-05-21T00:00:00Z",
            None,
            "2026-05-21T00:00:00Z",
            vec![first, second],
        )
        .unwrap();

        assert_eq!("88.25", invoice.total_amount);
    }
}
