use sdkwork_contract_service::CommerceServiceContract;

pub fn invoice_service_contract() -> CommerceServiceContract {
    CommerceServiceContract::new(
        "invoice",
        "commerce.invoice",
        vec!["invoices.create"],
        vec!["invoices.list", "invoices.retrieve", "invoices.titles.list"],
        vec![
            crate::ports::INVOICE_REPOSITORY_PORT,
            crate::ports::INVOICE_PROVIDER_PORT,
            crate::ports::IDEMPOTENCY_REPOSITORY_PORT,
        ],
        true,
    )
}
