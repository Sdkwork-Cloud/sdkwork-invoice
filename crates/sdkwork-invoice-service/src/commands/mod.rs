#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateInvoiceApplicationCommand {
    pub idempotency_key: String,
    pub order_id: String,
    pub payment_id: String,
    pub tenant_id: String,
    pub title_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitInvoiceCommand {
    pub idempotency_key: String,
    pub invoice_id: String,
    pub tenant_id: String,
}
