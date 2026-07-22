# sdkwork-invoice

repository-kind: application

SDKWork commerce invoice capability. It owns invoice domain rules, tenant-scoped SQLx repositories,
the app route surface, a reusable API assembly, and a standalone gateway.

## Active Surfaces

| Surface | API authority | SDK family | Operations |
| --- | --- | --- | ---: |
| app-api | `sdkwork-invoice-app-api` | `sdkwork-invoice-app-sdk` | 9 |
| backend-api | none | none | 0 |

No Invoice backend authority, route crate, or SDK family is declared until approved operator
requirements introduce real operations.

## Repository Layout

- `apis/`: owner-only app OpenAPI contracts.
- `crates/`: domain, repository, route, host, assembly, and standalone runtime crates.
- `database/`: invoice-owned database contracts and migrations.
- `sdks/`: the Invoice app SDK family workspace.
- `specs/`: application-wide component and IAM contracts.
- `tools/`: deterministic API/SDK materialization.
- `apps/`: application-root index; no Invoice UI is implemented here.

Directory rules come from `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`. API and SDK authority comes
from route manifests, OpenAPI, `sdk-manifest.json`, and component specs rather than this README.

## Verification

```powershell
pnpm check
cargo test --workspace
```

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)
- [apps directory index](apps/README.md)
