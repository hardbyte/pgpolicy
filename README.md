# pgpolicy

Declarative PostgreSQL role graph manager. Define roles, memberships, object privileges, and default privileges in YAML — pgpolicy diffs against live databases and applies changes.

## Components

- **pgpolicy-core** — Manifest parsing, profile expansion, diff engine, SQL generation. No database dependencies.
- **pgpolicy-inspect** — Live database introspection via `pg_catalog` queries (sqlx + tokio).
- **pgpolicy-cli** — Command-line tool for validating manifests, planning changes, and applying them.
- **pgpolicy-operator** — Kubernetes operator that reconciles `PostgresPolicy` custom resources against PostgreSQL databases.

## Quick Start (CLI)

```bash
# Validate a manifest
pgpolicy validate policy.yaml

# Diff two manifests (no database needed)
pgpolicy diff old-policy.yaml new-policy.yaml

# Show what would change against a live database
pgpolicy plan policy.yaml --database-url postgres://...

# Apply changes
pgpolicy apply policy.yaml --database-url postgres://... --yes
```

## Quick Start (Operator)

```bash
# Install CRD
kubectl apply -f k8s/crd.yaml

# Deploy operator
kubectl apply -k k8s/deploy/

# Create a policy
kubectl apply -f k8s/samples/sample-policy.yaml

# Watch status
kubectl get postgrespolicy -w
```

## Manifest Example

See [examples/partly-like.yaml](examples/partly-like.yaml) for a full example using schema profiles.

## License

MIT
