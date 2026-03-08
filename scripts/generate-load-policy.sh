#!/usr/bin/env bash
set -euo pipefail

schema_count="${1:-20}"
output_path="${2:-/dev/stdout}"

{
  cat <<'YAML'
---
apiVersion: pgroles.io/v1alpha1
kind: PostgresPolicy
metadata:
  name: load-policy
  namespace: default
spec:
  connection:
    secretRef:
      name: postgres-credentials
  interval: "5m"
  default_owner: postgres

  profiles:
    editor:
      grants:
        - 'on': { type: schema }
          privileges: [USAGE]
        - 'on': { type: table, name: "*" }
          privileges: [SELECT, INSERT, UPDATE, DELETE]
        - 'on': { type: sequence, name: "*" }
          privileges: [USAGE]
      default_privileges:
        - on_type: table
          privileges: [SELECT, INSERT, UPDATE, DELETE]
        - on_type: sequence
          privileges: [USAGE]
    viewer:
      grants:
        - 'on': { type: schema }
          privileges: [USAGE]
        - 'on': { type: table, name: "*" }
          privileges: [SELECT]
      default_privileges:
        - on_type: table
          privileges: [SELECT]

  schemas:
YAML

  for i in $(seq -w 1 "${schema_count}"); do
    cat <<YAML
    - name: load_${i}
      profiles: [editor, viewer]
YAML
  done
} > "${output_path}"
