# Invoice PRD

Status: active
Owner: SDKWork maintainers
Application: invoice
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Platform split alignment (commerce T0): `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`

## 1. Background And Problem

Invoice issuance, retrieval, and compliance metadata must be auditable and separated from order/payment write paths.

This repository is a **T1 commerce capability building block**. `sdkwork-commerce` remains the T0 composition layer (gateway, IAM wrappers, composed SDK). This repository owns domain logic, persistence, and HTTP route builders for the **invoice** capability.

## 2. Target Users

Finance operators, buyers downloading invoices, and compliance reviewers.

## 3. Goals And Non-Goals

### Goals

- Own invoice SQL and app invoice HTTP routers with commerce T0 IAM wrappers.

### Non-Goals

- Payment capture execution.

## 4. Scope

- Invoice list/detail/create/update/submit/cancel flows.
- Invoice repository SQLx implementations.

Primary API prefixes:

- App: `/app/v3/api/invoices`
- Backend: `/backend/v3/api/invoices`

Migration status: **complete**.

## 5. User Scenarios

- A buyer requests an invoice for a completed order.
- An operator lists invoice records for a tenant.

## 6. Success Metrics

- Invoice SQL and routes owned exclusively in this repository.
- Commerce invoice integration tests pass via thin wrappers.

## 7. Phases

- Phase 1 (complete): SQL + app invoice router owned in sdkwork-invoice.
- Phase 2 (complete): mutating invoice routes require Idempotency-Key and Sdkwork-Request-Hash via command header validation.

## 8. Linked Requirements

- Commerce capability split alignment: `../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md`
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions

- Tax identifier and regional compliance fields before production launch.
