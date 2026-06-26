# Invoice Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-24
Specs: ARCHITECTURE_DECISION_SPEC.md, RUST_CODE_SPEC.md, API_SPEC.md, WEB_FRAMEWORK_SPEC.md, DATABASE_FRAMEWORK_SPEC.md

## Document Map

- [TECH split alignment (commerce T0)](../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md)

## 1. Architecture Overview

`sdkwork-invoice` is a **T1 capability repository** in the commerce domain. It exposes domain services, SQL repositories, and HTTP route builders. `sdkwork-commerce` composes these crates at runtime:

```text
T1 invoice crate  →  build_*_router()     (no IAM)
T0 commerce         →  with_request_identity / with_backend_request_identity
```

Migration status: **complete**.

## 2. Technology Choices

- **Rust** domain services and SQLx repositories (`RUST_CODE_SPEC.md`)
- **Axum** HTTP routers integrated via `sdkwork-web-framework` (`WEB_FRAMEWORK_SPEC.md`)
- **sqlx** for Postgres/SQLite repository implementations (`DATABASE_FRAMEWORK_SPEC.md`)
- **Sibling path dependencies** from `sdkwork-commerce/Cargo.toml` — no duplicated domain crates in commerce

## 3. System Boundaries And Modules

| Layer | Owner | Notes |
| --- | --- | --- |
| Domain commands/queries | `sdkwork-commerce-invoice-service` | Business validation and ports |
| SQL repositories | `sdkwork-commerce-invoice-repository-sqlx` | Tenant-scoped persistence |
| HTTP route builders | sdkwork-routes-invoice-app-api, sdkwork-routes-invoice-backend-api | `build_*_router` exports without IAM |
| IAM / gateway composition | `sdkwork-commerce` | Thin wrappers only |
| OpenAPI / SDK authority | `sdkwork-commerce/sdks/` | Composed commerce SDK families |

## 4. Directory And Package Layout

Standard 7-crate capability workspace:

- `crates/sdkwork-commerce-invoice-service/`
- `crates/sdkwork-commerce-invoice-repository-sqlx/`
- `crates/sdkwork-routes-invoice-app-api/`
- `crates/sdkwork-routes-invoice-backend-api/`
- `crates/sdkwork-invoice-database-host/`
- `crates/sdkwork-invoice-service-host/`
- `crates/sdkwork-invoice-api-server/`

No PC application root in this repository yet.

## 5. API, SDK, And Data Ownership

- App API prefix: `/app/v3/api/invoices`
- Backend API prefix: `/backend/v3/api/invoices`
- Table prefix: `commerce_` for capability-owned tables (`DOMAIN_SPEC` domain=commerce)
- Public SDK consumption: generated **commerce** SDK families at T0; do not hand-craft raw HTTP (`SDK_SPEC.md`)

## 6. Security, Privacy, And Observability

- Authentication and tenant context are applied at **commerce T0** IAM middleware; handlers read `IamAppContext` from extensions.
- Write routes require idempotency and request-hash headers where applicable (`API_SPEC.md`, `SECURITY_SPEC.md`).
- Ledger, payment, and account mutations must fail closed on validation errors.
- Structured errors use `CommerceServiceError` contracts; do not leak internal SQL details to clients.

## 7. Deployment And Runtime Topology

- Local development: `cargo test --workspace` in this repository.
- Platform composition: `sdkwork-commerce` service host merges capability routers into the commerce HTTP surface.
- Independent deployment of this capability server is supported via `sdkwork-invoice-api-server` for building-block topology; production gateway routing is owned by commerce/app topology specs.

## 8. Architecture Decision Index

- [TECH-2026-06-24-commerce-capability-repo-split-alignment.md](../sdkwork-commerce/docs/architecture/tech/TECH-2026-06-24-commerce-capability-repo-split-alignment.md)

## 9. Verification

```bash
cargo test --workspace
```

From commerce T0 after boundary changes:

```bash
cd ../sdkwork-commerce
cargo test --workspace
node --test sdks/test/verify-commerce-migration-cleanup.test.mjs
```
