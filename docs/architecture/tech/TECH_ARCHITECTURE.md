# Invoice Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-06-24
Specs: ARCHITECTURE_DECISION_SPEC.md, RUST_CODE_SPEC.md, API_SPEC.md, WEB_FRAMEWORK_SPEC.md, DATABASE_FRAMEWORK_SPEC.md

## Document Map

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8

## 1. Architecture Overview

`sdkwork-invoice` is a **T1 capability repository** in the commerce domain. It owns domain services, SQL repositories, HTTP route builders, and a standalone gateway with IAM middleware. The `sdkwork-commerce (deleted)` monolith has been dissolved; each T1 capability repository is self-contained.

```text
T1 invoice crate  →  build_*_router()     (no IAM)
T1 standalone-gateway     →  with_request_identity / with_backend_request_identity
```

Migration status: **complete**.

## 2. Technology Choices

- **Rust** domain services and SQLx repositories (`RUST_CODE_SPEC.md`)
- **Axum** HTTP routers integrated via `sdkwork-web-framework` (`WEB_FRAMEWORK_SPEC.md`)
- **sqlx** for Postgres/SQLite repository implementations (`DATABASE_FRAMEWORK_SPEC.md`)
- **Sibling path dependencies** from this repository's `Cargo.toml` — cross-T1 references use `sdkwork_commerce_*` crate names per `sdkwork-<domain>-<capability>-service` naming

## 3. System Boundaries And Modules

| Layer | Owner | Notes |
| --- | --- | --- |
| Domain commands/queries | `sdkwork-invoice-service` | Business validation and ports |
| SQL repositories | `sdkwork-commerce (deleted)-invoice-repository-sqlx` | Tenant-scoped persistence |
| HTTP route builders | sdkwork-routes-invoice-app-api, sdkwork-routes-invoice-backend-api | `build_*_router` exports without IAM |
| IAM / gateway composition | `sdkwork-api-invoice-standalone-gateway` | IAM middleware at T1 standalone-gateway |
| OpenAPI / SDK authority | `sdkwork-invoice/sdks/` | Per-T1 SDK families |

## 4. Directory And Package Layout

Standard 7-crate capability workspace:

- `crates/sdkwork-invoice-service/`
- `crates/sdkwork-commerce (deleted)-invoice-repository-sqlx/`
- `crates/sdkwork-routes-invoice-app-api/`
- `crates/sdkwork-routes-invoice-backend-api/`
- `crates/sdkwork-invoice-database-host/`
- `crates/sdkwork-invoice-service-host/`
- `crates/sdkwork-api-invoice-standalone-gateway/`

No PC application root in this repository yet.

## 5. API, SDK, And Data Ownership

- App API prefix: `/app/v3/api/invoices`
- Backend API prefix: `/backend/v3/api/invoices`
- Table prefix: `commerce_` for capability-owned tables (`DOMAIN_SPEC` domain=commerce)
- Public SDK consumption: generated per-T1 SDK families; do not hand-craft raw HTTP (`SDK_SPEC.md`)

## 6. Security, Privacy, And Observability

- Authentication and tenant context are applied at the T1 `*-standalone-gateway` IAM middleware; handlers read `IamAppContext` from extensions.
- Write routes require idempotency and request-hash headers where applicable (`API_SPEC.md`, `SECURITY_SPEC.md`).
- Ledger, payment, and account mutations must fail closed on validation errors.
- Structured errors use `CommerceServiceError` contracts; do not leak internal SQL details to clients.

## 7. Deployment And Runtime Topology

- Local development: `cargo test --workspace` in this repository.
- Independent deployment via `sdkwork-api-invoice-standalone-gateway`; production gateway routing is owned by deployment/app topology specs.

## 8. Architecture Decision Index

- Commerce repository dissolution: `../sdkwork-specs/MIGRATION_SPEC.md` §8

## 9. Verification

```bash
cargo test --workspace
```
