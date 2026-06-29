use sdkwork_invoice_service::{
    InvoiceApplicationDraft, InvoiceProviderCommand, InvoiceProviderPortRequirement, InvoiceStatus,
    InvoiceTitle, InvoiceTransition,
};

#[test]
fn validates_invoice_title_and_tax_profile() {
    let title = InvoiceTitle::company("SDKWork Ltd", "91310000MA1K000000").unwrap();

    assert_eq!(title.name, "SDKWork Ltd");
    assert_eq!(title.tax_no.as_deref(), Some("91310000MA1K000000"));
    assert!(InvoiceTitle::company("", "").is_err());
}

#[test]
fn validates_invoice_status_lifecycle() {
    assert_eq!(
        InvoiceTransition::new(InvoiceStatus::Draft, InvoiceStatus::Submitted).validate(),
        Ok(())
    );
    assert!(
        InvoiceTransition::new(InvoiceStatus::Issued, InvoiceStatus::Submitted)
            .validate()
            .is_err()
    );
}

#[test]
fn invoice_application_requires_order_and_payment_reference() {
    let draft =
        InvoiceApplicationDraft::new("100001", "order-1", "payment-1", "title-1").unwrap();

    assert_eq!(draft.order_id, "order-1");
    assert_eq!(draft.payment_id, "payment-1");
    assert!(InvoiceApplicationDraft::new("100001", "", "payment-1", "title-1").is_err());
}

#[test]
fn invoice_provider_contract_exposes_required_commands() {
    assert_eq!(
        InvoiceProviderPortRequirement::standard_commands(),
        vec![
            InvoiceProviderCommand::IssueInvoice,
            InvoiceProviderCommand::DownloadInvoice,
            InvoiceProviderCommand::VoidInvoice,
        ],
    );
}
