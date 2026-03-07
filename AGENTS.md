# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

pgroles is a declarative PostgreSQL role/privilege manager. Define roles, grants, default privileges, and memberships in YAML; diff against a live database; apply convergently (anything not in the manifest gets revoked/dropped). Think Terraform for PostgreSQL access control.

**PostgreSQL 16+** is the primary target. PG 14-15 supported with automatic SQL syntax fallback. CI tests against PG 16, 17, and 18.

## Build & Test Commands

```bash
# Build the full workspace
SQLX_OFFLINE=true cargo build --workspace

# Format + lint (always run before committing)
cargo fmt --all
SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings

# Unit tests (no database needed)
cargo test --workspace

# Integration tests (requires live PostgreSQL)
export DATABASE_URL=postgres://postgres:testpassword@localhost:5432/pgroles_test
cargo test --workspace -- --include-ignored

# Run a single test
cargo test -p pgroles-core --lib diff::tests::test_name
cargo test -p pgroles-cli --test cli live_db::diff_against_live_db -- --ignored --exact

# CRD drift check (CI enforces this)
cargo run --bin crdgen > /tmp/crd-generated.yaml
diff k8s/crd.yaml /tmp/crd-generated.yaml

# Regenerate CRD after modifying crd.rs
cargo run --bin crdgen > k8s/crd.yaml
```

**SQLX_OFFLINE=true** is required for clippy/build when no live database is available — sqlx compile-time checking is not used (no `.sqlx` directory), but the env var suppresses connection attempts.

### Local PostgreSQL for integration tests

```bash
docker run --rm --name pgroles-pg16 \
  -e POSTGRES_PASSWORD=testpassword \
  -e POSTGRES_DB=pgroles_test \
  -p 5432:5432 \
  postgres:16
```

### Docs site (Next.js + Markdoc)

```bash
cd docs && npm install && npm run dev
```

## Architecture

### Data Pipeline

```
YAML → parse_manifest() → PolicyManifest
     → expand_manifest() → ExpandedManifest (profiles × schemas resolved)
     → RoleGraph::from_expanded() → RoleGraph (desired)

DB   → inspect() → RoleGraph (current)
     → detect_pg_version() → SqlContext

diff(current, desired) → Vec<Change> → sql::render_all_with_context() → SQL
```

### Workspace Crates

- **pgroles-core** (`crates/pgroles-core/`) — Pure library, no IO. Manifest parsing, profile expansion, diff engine, SQL rendering, manifest export. All collections use `BTreeMap`/`BTreeSet` for deterministic output.
- **pgroles-inspect** (`crates/pgroles-inspect/`) — Async database introspection via `sqlx`/`pg_catalog`. Version detection, cloud provider detection (RDS, Cloud SQL, AlloyDB, Azure), drop-role safety preflight.
- **pgroles-cli** (`crates/pgroles-cli/`) — Binary crate. Thin orchestration layer over core + inspect. Subcommands: `validate`, `diff`/`plan`, `apply`, `inspect`, `generate`.
- **pgroles-operator** (`crates/pgroles-operator/`) — Kubernetes operator (WIP). Reconciles `PostgresPolicy` CRDs (`pgroles.io/v1alpha1`). Has a `crdgen` binary for generating CRD YAML.

### Key Source Files

- `pgroles-core/src/manifest.rs` — YAML schema types, `parse_manifest()`, `expand_manifest()`, profile × schema expansion
- `pgroles-core/src/model.rs` — `RoleGraph`, `RoleState`, `GrantKey`, `DefaultPrivKey`, `MembershipEdge`
- `pgroles-core/src/diff.rs` — `diff()` produces ordered `Vec<Change>`, `apply_role_retirements()` for retirement workflows
- `pgroles-core/src/sql.rs` — `render_all_with_context()`, `SqlContext` for PG version-dependent SQL
- `pgroles-core/src/export.rs` — `RoleGraph` → `PolicyManifest` for the `generate` command
- `pgroles-inspect/src/safety.rs` — Drop-role preflight checks (owned objects, privilege deps, active sessions)
- `pgroles-operator/src/crd.rs` — CRD definition, mirrors manifest schema
- `pgroles-operator/src/reconciler.rs` — Kubernetes reconciliation loop

### Diff Change Ordering

Changes are produced in dependency order: CreateRole → AlterRole → SetComment → Grant → SetDefaultPrivilege → RemoveMember → AddMember → RevokeDefaultPrivilege → Revoke → retirement steps → DropRole. `apply` executes within a single transaction.

### Convergent Model

The manifest is the **entire desired state**. Roles/grants/memberships in the database but not in the manifest are revoked/dropped. Role retirements handle explicit cleanup (reassign owned, drop owned, terminate sessions) before drops.

## CI

Four CI jobs (`.github/workflows/ci.yml`):
1. **Lint** — `cargo fmt --check`, `clippy -D warnings`, CRD drift check
2. **Unit Tests** — `cargo test --workspace`
3. **Integration Tests** — matrix of PG 16/17/18, `cargo test --workspace -- --include-ignored`
4. **E2E** — kind cluster, deploys operator, applies sample policy, verifies roles

## Project Layout

- `examples/` — Sample YAML manifests (minimal, multi-schema, custom-pattern)
- `docker/` — Dockerfile (multi-stage, has `operator` target)
- `k8s/crd.yaml` — Generated CRD (must match `crdgen` output)
- `k8s/deploy/` — Operator deployment manifests
- `k8s/samples/` — Sample PostgresPolicy CRs
- `charts/pgroles-operator/` — Helm chart
- `docs/` — Next.js documentation site (Markdoc)
