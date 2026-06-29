use crate::{CreateInvoiceApplicationCommand, InvoiceApplicationDraft};
use sdkwork_contract_service::CommerceServiceError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceProviderCommand {
    IssueInvoice,
    DownloadInvoice,
    VoidInvoice,
}

pub struct InvoiceProviderPortRequirement;

pub trait InvoiceProviderPort {
    fn issue_invoice(
        &self,
        command: &CreateInvoiceApplicationCommand,
    ) -> Result<InvoiceApplicationDraft, CommerceServiceError>;
}

pub const INVOICE_REPOSITORY_PORT: &str = "invoice.repository";
pub const INVOICE_PROVIDER_PORT: &str = "invoice.provider";
pub const IDEMPOTENCY_REPOSITORY_PORT: &str = "idempotency.repository";

impl InvoiceProviderPortRequirement {
    pub fn standard_commands() -> Vec<InvoiceProviderCommand> {
        vec![
            InvoiceProviderCommand::IssueInvoice,
            InvoiceProviderCommand::DownloadInvoice,
            InvoiceProviderCommand::VoidInvoice,
        ]
    }
}
