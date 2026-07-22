# Invoice PRD

Status: active
Owner: SDKWork maintainers
Application: invoice
Updated: 2026-07-22
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## 1. Background And Problem

Buyers and finance teams need auditable invoice requests, line items, status transitions, and
summary statistics without coupling invoice lifecycle to order or payment persistence.

## 2. Target Users

Buyers, merchant finance operators, compliance reviewers, and support teams.

## 3. Goals And Non-Goals

Goals:

- Own invoice creation, retrieval, update, submission, cancellation, items, and statistics.
- Expose a tenant-isolated app API with a generated typed SDK.
- Enforce bounded pagination, idempotent commands, standard errors, and privacy-safe responses.

Non-goals:

- Payment capture, tax-engine implementation, or administrator backend workflows without approved
  product requirements.

## 4. Scope

The active scope is the buyer/merchant app invoice workflow. No backend API or SDK is declared.

## 5. User Scenarios

- A buyer requests and later retrieves an invoice.
- A merchant reviews paginated invoice records and line items.
- An authorized user submits or cancels an invoice with idempotency protection.
- Finance users view aggregate invoice status counts without a full-record download.

## 6. Success Metrics

- Every active app route is represented exactly once in the app authority and SDK family.
- Lists are bounded at the repository layer and statistics use database aggregation.
- Public responses exclude tenant, organization, and owner implementation identifiers.
- Invalid command headers and business failures use standard problem responses.

## 7. Phases

- Active: nine app API operations and the app SDK family are implemented.
- Deferred: backend API/SDK work begins only after real administrator requirements are approved.
- Next: production release evidence, regional compliance requirements, and operational SLOs.

## 8. Linked Requirements

- Machine contracts: repository `specs/`, module `specs/component.spec.json`, `apis/`, and
  `sdks/*/sdk-manifest.json`.
- Standards: `../sdkwork-specs/API_SPEC.md`, `SDK_SPEC.md`, `PAGINATION_SPEC.md`, `PRIVACY_SPEC.md`,
  and `SECURITY_SPEC.md`.

## 9. Open Questions

- Regional tax identifiers and retention rules require product and compliance approval.
