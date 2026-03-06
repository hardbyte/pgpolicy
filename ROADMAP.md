# pgpolicy Roadmap

## Goals

- Make the current CLI/operator behavior safe and actually convergent.
- Tighten the declarative contract so the manifest expresses intent, not just SQL-shaped inputs.
- Harden the operator only after the core reconciliation model is reliable.

## Phase 1: Correctness

- Normalize wildcard grants during inspection so `name: "*"` converges against live databases.
- Validate manifest semantics early:
  - top-level default privileges must declare `grant.role`
  - object target combinations should be checked for required/forbidden fields
  - unsupported default privilege object types should be rejected
- Execute `apply` in a single transaction.
- Fix membership option updates so `inherit`/`admin` changes do not drop membership.
- Render function grants/revokes using PostgreSQL function-signature syntax.

## Phase 2: Test Coverage

- Add live PostgreSQL tests for:
  - wildcard table/sequence/function grants
  - function grants with arguments
  - membership option changes
  - default privilege validation and reconciliation
- Add operator tests for:
  - Secret rotation
  - degraded status on failure
  - reconcile recovery after failure

## Phase 3: Declarative Boundary

- Introduce an explicit managed scope:
  - managed roles
  - managed schemas
  - whether revokes/drops are authoritative inside that scope
- Add reconcile modes:
  - `authoritative`
  - `additive`
  - `adopt`
- Treat selectors like "all tables in schema X" as first-class intent, not a string convention.
- Make owner context for default privileges explicit instead of relying on fallbacks.

## Phase 4: CLI and UX

- Add `--format json` to `validate`, `diff`, and `inspect`.
- Add a drift exit code for CI.
- Make `inspect` emit a detailed normalized graph, not just counts.
- Add a manifest/schema export command for editor and pipeline integration.

## Phase 5: Operator Hardening

- Cache pools by Secret resource version and watch for Secret updates.
- Surface `Ready`, `Reconciling`, and `Degraded` conditions consistently.
- Add rate-limited retries and clearer failure summaries.
- Add policy around deletion behavior instead of relying on implicit defaults.

## Declarative Direction

Today, pgpolicy is declarative at the manifest surface but still partly operational internally. The next step is to make the core model intent-based:

- desired selectors and edges in
- normalized current state over the same managed boundary
- diff between those two graphs
- SQL only as an execution backend

That makes the tool more predictable for both the CLI and the operator.
