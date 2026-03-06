# pgroles-operator

Kubernetes operator crate for `pgroles`.

This crate contains the controller, CRD types, and reconciliation logic for
running `pgroles` continuously in Kubernetes against `PostgresPolicy`
resources.

## Status

This crate is currently part of the workspace but is not published to
crates.io (`publish = false`).

## What It Includes

- `PostgresPolicy` CRD types
- Reconciler and controller wiring
- Status condition updates
- Secret-backed database connectivity
- CRD generation binary (`crdgen`)

## Intended Audience

- Contributors working on the operator implementation
- Platform teams evaluating the Kubernetes reconciliation model

Operator docs: <https://github.com/hardbyte/pgroles/tree/main/docs/src/pages/docs/operator.md>
