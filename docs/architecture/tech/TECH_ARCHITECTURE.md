# Invoice Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-22
Specs: ARCHITECTURE_DECISION_SPEC.md, RUST_CODE_SPEC.md, API_SPEC.md, SDK_SPEC.md, PAGINATION_SPEC.md, PRIVACY_SPEC.md,
SECURITY_SPEC.md, APPLICATION_GATEWAY_SPEC.md

## 1. Architecture Overview

`sdkwork-invoice` is a composable Rust/Axum capability. Domain and SQLx layers are independent of
HTTP; the app route crate exposes the active API; the assembly creates shared state and merges route
boundaries; the standalone gateway adds process concerns. The platform cloud gateway embeds the same
assembly rather than rebuilding Invoice state.

## 2. Technology Choices

- Rust domain services and ports.
- SQLx repositories for PostgreSQL and SQLite with store-level pagination and SQL aggregation.
- Axum route composition through `sdkwork-web-framework`.
- `sdkwork-utils-rust` response and pagination contracts.
- `@sdkwork/sdk-generator` through `sdkgen` for generated transports.

## 3. System Boundaries And Modules

| Layer | Owner |
| --- | --- |
| Domain behavior | `sdkwork-invoice-service` |
| SQL persistence | `sdkwork-invoice-repository-sqlx` |
| Database lifecycle adapter | `sdkwork-invoice-database-host` |
| Shared runtime state | `sdkwork-invoice-service-host` |
| App routes | `sdkwork-routes-invoice-app-api` |
| Router composition | `sdkwork-api-invoice-assembly` |
| Independent process | `sdkwork-api-invoice-standalone-gateway` |

## 4. Runtime Composition

```text
database pool -> invoice service host -> invoice assembly
                                    -> app router (9 operations)
invoice assembly -> standalone gateway or platform cloud gateway
```

Health, readiness, liveness, and metrics endpoints belong to the host gateway and are not exported
by the capability assembly.

## 5. API, SDK, And Data Ownership

| Surface | Route owner | API authority | SDK family |
| --- | --- | --- | --- |
| app-api | `sdkwork-routes-invoice-app-api` | `sdkwork-invoice-app-api` | `sdkwork-invoice-app-sdk` |
| backend-api | none | none | none |

The app family contains 9 operations. No Invoice backend-api or open-api surface is declared.
Database schemas and migrations are unchanged.

## 6. Security, Privacy, And Observability

IAM middleware supplies authenticated tenant context. Route permissions are
`commerce.invoices.read` and `commerce.invoices.manage`. Mutations validate idempotency and request
hash headers. Public resources omit tenant, organization, and owner identifiers; SQL failures are
mapped to ProblemDetail without internal details.

## 7. Deployment And Runtime Topology

The standalone binary supports independent validation. Production composition uses the cloud
gateway feature/dependency/runtime contract and the same assembly export. Both use the process-shared
database pool rather than creating route-local pools.

## 8. Architecture Decision Index

Creating a backend surface requires approved product requirements and a coordinated
component/route/OpenAPI/SDK contract change.

## 9. Verification

```powershell
pnpm check
cargo test --workspace
cargo clippy --workspace --tests -- -D warnings
```
