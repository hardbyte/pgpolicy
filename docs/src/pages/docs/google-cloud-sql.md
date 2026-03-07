---
title: Google Cloud SQL
description: Connect pgroles to Cloud SQL for PostgreSQL — connectivity, IAM authentication, and Cloud Run deployment.
---

Platform-specific guidance for running pgroles against Cloud SQL for PostgreSQL. {% .lead %}

For general usage, see the [quick start](/docs/quick-start). For CI pipeline patterns, see [CI/CD integration](/docs/ci-cd). For the Kubernetes operator, see the [operator docs](/docs/operator).

---

## Prerequisites

- A Cloud SQL for PostgreSQL instance (PostgreSQL 14+, 16+ recommended)
- A database user with `cloudsqlsuperuser` membership (the default `postgres` user has this)

pgroles auto-detects Cloud SQL when the connecting role is a member of `cloudsqlsuperuser` and adjusts privilege warnings accordingly — for example, it will warn if your manifest requests `SUPERUSER` or `BYPASSRLS` attributes that Cloud SQL doesn't allow.

## Connecting to Cloud SQL

### Cloud SQL Auth Proxy

The [Cloud SQL Auth Proxy](https://cloud.google.com/sql/docs/postgres/sql-proxy) is the recommended connection method. It handles TLS and IAM-based authentication automatically.

```shell
# Start the proxy
cloud-sql-proxy my-project:us-central1:my-instance --port 5432

# In another shell
export DATABASE_URL='postgres://postgres:PASSWORD@127.0.0.1:5432/mydb'
pgroles diff -f pgroles.yaml
```

In **GKE**, run the proxy as a sidecar or as a standalone Deployment in the same namespace as the operator. With [Workload Identity](https://cloud.google.com/kubernetes-engine/docs/how-to/workload-identity) configured, the proxy authenticates automatically without keys.

In **GitHub Actions**, use the [setup-cloud-sql-proxy](https://github.com/google-github-actions/setup-cloud-sql-proxy) action:

```yaml
      - uses: google-github-actions/auth@v2
        with:
          workload_identity_provider: ${{ secrets.WIF_PROVIDER }}
          service_account: ${{ secrets.SA_EMAIL }}

      - uses: google-github-actions/setup-cloud-sql-proxy@v2
        with:
          instance_connection_name: my-project:us-central1:my-instance

      - name: Check for drift
        run: |
          docker run --rm --network host \
            -e DATABASE_URL="postgres://postgres:${{ secrets.DB_PASSWORD }}@127.0.0.1:5432/mydb" \
            -v "${{ github.workspace }}:/work" \
            ghcr.io/hardbyte/pgroles:latest \
            diff -f /work/pgroles.yaml --exit-code
```

See [CI/CD integration](/docs/ci-cd) for more patterns (apply-on-merge, PR comments, output formats).

### Private IP

If your Cloud SQL instance has a private IP and your workload runs in the same VPC:

```shell
export DATABASE_URL='postgres://postgres:PASSWORD@10.x.x.x:5432/mydb'
```

### Cloud SQL built-in connector (Cloud Run)

Cloud Run and App Engine can use `--add-cloudsql-instances` instead of the proxy. The connector exposes a Unix socket:

```
postgres://postgres:PASSWORD@/mydb?host=/cloudsql/MY_PROJECT:us-central1:my-instance
```

## Cloud Run job

If you don't run Kubernetes, a [Cloud Run job](https://cloud.google.com/run/docs/create-jobs) is a lightweight way to run pgroles on a schedule using the published image directly — no custom build required.

Store your manifest in Secret Manager alongside the connection string:

```shell
gcloud secrets create pgroles-manifest \
  --data-file=pgroles.yaml

gcloud secrets create pgroles-db-url \
  --data-file=- <<< 'postgres://postgres:PASSWORD@/mydb?host=/cloudsql/MY_PROJECT:us-central1:my-instance'
```

Create the job, mounting the manifest as a file via `--set-secrets`:

```shell
gcloud run jobs create pgroles-apply \
  --image ghcr.io/hardbyte/pgroles:latest \
  --set-secrets /work/pgroles.yaml=pgroles-manifest:latest \
  --set-secrets DATABASE_URL=pgroles-db-url:latest \
  --add-cloudsql-instances MY_PROJECT:us-central1:my-instance \
  --args "apply,-f,pgroles.yaml" \
  --region us-central1
```

Pinning to `:latest` means each job execution resolves the newest secret version at startup. To update the manifest, add a new secret version:

```shell
gcloud secrets versions add pgroles-manifest --data-file=pgroles.yaml
```

The next scheduled (or manual) execution picks it up automatically — no redeploy needed.

{% callout type="note" title="VPC connector" %}
If connecting via Private IP instead of the built-in Cloud SQL connector, Cloud Run needs a [Serverless VPC Access connector](https://cloud.google.com/vpc/docs/configure-serverless-vpc-access) or [Direct VPC egress](https://cloud.google.com/run/docs/configuring/vpc-direct-vpc). Replace `--add-cloudsql-instances` with `--vpc-connector my-connector` and use the private IP in your connection string.
{% /callout %}

### Schedule it

```shell
gcloud scheduler jobs create http pgroles-daily \
  --schedule "0 3 * * *" \
  --uri "https://us-central1-run.googleapis.com/apis/run.googleapis.com/v1/namespaces/MY_PROJECT/jobs/pgroles-apply:run" \
  --oauth-service-account-email MY_SA@MY_PROJECT.iam.gserviceaccount.com \
  --location us-central1
```

### Drift detection

Create a second job that diffs instead of applying:

```shell
gcloud run jobs create pgroles-drift \
  --image ghcr.io/hardbyte/pgroles:latest \
  --set-secrets /work/pgroles.yaml=pgroles-manifest:latest \
  --set-secrets DATABASE_URL=pgroles-db-url:latest \
  --add-cloudsql-instances MY_PROJECT:us-central1:my-instance \
  --args "diff,-f,pgroles.yaml,--exit-code" \
  --region us-central1
```

Exit code 2 means drift was detected.

### Custom image alternative

If you prefer baking the manifest into the image (e.g., for versioned deploys tied to image tags):

```dockerfile
FROM ghcr.io/hardbyte/pgroles:latest
COPY pgroles.yaml .
```

```shell
gcloud builds submit --tag gcr.io/MY_PROJECT/pgroles-apply
gcloud run jobs create pgroles-apply \
  --image gcr.io/MY_PROJECT/pgroles-apply \
  --set-secrets DATABASE_URL=pgroles-db-url:latest \
  --add-cloudsql-instances MY_PROJECT:us-central1:my-instance \
  --args "apply,-f,pgroles.yaml" \
  --region us-central1
```

## Cloud Build

Use the published Docker image directly in Cloud Build steps:

```yaml
steps:
  - name: ghcr.io/hardbyte/pgroles:latest
    args: ['apply', '-f', 'pgroles.yaml']
    secretEnv: ['DATABASE_URL']

availableSecrets:
  secretManager:
    - versionName: projects/MY_PROJECT/secrets/pgroles-db-url/versions/latest
      env: DATABASE_URL
```

## IAM database authentication

Cloud SQL supports [IAM database authentication](https://cloud.google.com/sql/docs/postgres/iam-authentication) for individual users, service accounts, and groups. Declare the provider in your manifest:

```yaml
auth_providers:
  - type: cloud_sql_iam
    project: my-gcp-project
```

### Role naming conventions

Cloud SQL maps IAM principals to PostgreSQL roles with specific naming rules:

| IAM principal | PostgreSQL role name | Example |
| --- | --- | --- |
| User | Full email address | `"kai@example.com"` |
| Service account | Email without `.gserviceaccount.com` | `"my-sa@my-project.iam"` |
| Group | Full group email address | `"editors@example.com"` |

### Service accounts

```yaml
roles:
  - name: "my-sa@my-project.iam"
    login: true
    comment: "IAM-authenticated service account"
```

### IAM groups

[IAM group authentication](https://cloud.google.com/sql/docs/postgres/add-manage-iam-users) lets you grant database privileges to a Cloud Identity group. All group members inherit the grants automatically on first login — you don't need to add individual members to your manifest.

```yaml
roles:
  - name: "backend-team@example.com"
    login: false
    comment: "Cloud Identity group — members authenticate individually"

grants:
  - role: "backend-team@example.com"
    privileges: [USAGE]
    on: { type: schema, name: app }
  - role: "backend-team@example.com"
    privileges: [SELECT, INSERT, UPDATE]
    on: { type: table, schema: app, name: "*" }
```

When a group member logs in for the first time, Cloud SQL creates their individual PostgreSQL role automatically and grants them the group's privileges.

{% callout type="note" title="Group membership propagation" %}
Changes to Cloud Identity group membership take about 15 minutes to propagate. However, changes to the group's database privileges take effect immediately.
{% /callout %}
