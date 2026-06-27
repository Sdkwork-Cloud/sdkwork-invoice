# Invoice PRD

Status: active
Owner: SDKWork maintainers
Application: invoice
Updated: 2026-06-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8

## 1. Background And Problem

Invoice issuance, retrieval, and compliance metadata must be auditable and separated from order/payment write paths.

This repository is a **T1 commerce capability building block**. The `sdkwork-commerce` monolith has been dissolved; this repository is self-contained with its own domain logic, persistence, HTTP route builders, API server, and IAM middleware for the **invoice** capability.

## 2. Target Users

Finance operators, buyers downloading invoices, and compliance reviewers.

## 3. Goals And Non-Goals

### Goals

- Own invoice SQL and app invoice HTTP routers with T1 `*-standalone-gateway` IAM wrappers.

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
- Commerce invoice integration tests pass via T1 standalone-gateway IAM wrappers.

## 7. Phases

- Phase 1 (complete): SQL + app invoice router owned in sdkwork-invoice.
- Phase 2 (complete): mutating invoice routes require Idempotency-Key and Sdkwork-Request-Hash via command header validation.

## 8. Linked Requirements

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8
- Component contract: `specs/component.spec.json` (when present)
- Machine contracts: local `specs/`, future `apis/`, and generated `sdks/`

## 9. Open Questions

- Tax identifier and regional compliance fields before production launch.
