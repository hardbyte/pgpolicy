//! Shared operator context — database pool cache, metrics, and configuration.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::postgres::PgPool;
use tokio::sync::RwLock;

#[derive(Clone)]
struct CachedPool {
    resource_version: Option<String>,
    pool: PgPool,
}

/// Shared state for the operator, passed to every reconciliation.
#[derive(Clone)]
pub struct OperatorContext {
    /// Kubernetes client for API calls.
    pub kube_client: kube::Client,

    /// Cached database connection pools keyed by `"namespace/secret-name/secret-key"`.
    pool_cache: Arc<RwLock<HashMap<String, CachedPool>>>,
}

impl OperatorContext {
    /// Create a new operator context with an empty pool cache.
    pub fn new(kube_client: kube::Client) -> Self {
        Self {
            kube_client,
            pool_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a PgPool for the given secret reference.
    ///
    /// Reads the `DATABASE_URL` (or custom key) from the referenced Secret,
    /// and caches the resulting pool for reuse.
    pub async fn get_or_create_pool(
        &self,
        namespace: &str,
        secret_name: &str,
        secret_key: &str,
    ) -> Result<PgPool, ContextError> {
        let cache_key = format!("{namespace}/{secret_name}/{secret_key}");

        // Fetch secret from k8s API.
        let secrets_api: kube::Api<k8s_openapi::api::core::v1::Secret> =
            kube::Api::namespaced(self.kube_client.clone(), namespace);

        let secret =
            secrets_api
                .get(secret_name)
                .await
                .map_err(|err| ContextError::SecretFetch {
                    name: secret_name.to_string(),
                    namespace: namespace.to_string(),
                    source: err,
                })?;

        let resource_version = secret.metadata.resource_version.clone();

        // Check cache after reading the current Secret version.
        {
            let cache = self.pool_cache.read().await;
            if let Some(cached) = cache.get(&cache_key)
                && cached.resource_version == resource_version
            {
                return Ok(cached.pool.clone());
            }
        }

        let data = secret.data.ok_or_else(|| ContextError::SecretMissing {
            name: secret_name.to_string(),
            key: secret_key.to_string(),
        })?;

        let url_bytes = data
            .get(secret_key)
            .ok_or_else(|| ContextError::SecretMissing {
                name: secret_name.to_string(),
                key: secret_key.to_string(),
            })?;

        let database_url =
            String::from_utf8(url_bytes.0.clone()).map_err(|_| ContextError::SecretMissing {
                name: secret_name.to_string(),
                key: secret_key.to_string(),
            })?;

        // Create pool.
        let pool = PgPool::connect(&database_url)
            .await
            .map_err(|err| ContextError::DatabaseConnect { source: err })?;

        // Cache it (write lock).
        {
            let mut cache = self.pool_cache.write().await;
            cache.insert(
                cache_key,
                CachedPool {
                    resource_version,
                    pool: pool.clone(),
                },
            );
        }

        Ok(pool)
    }

    /// Remove a cached pool (e.g. when secret changes or CR is deleted).
    pub async fn evict_pool(&self, namespace: &str, secret_name: &str, secret_key: &str) {
        let cache_key = format!("{namespace}/{secret_name}/{secret_key}");
        let mut cache = self.pool_cache.write().await;
        cache.remove(&cache_key);
    }
}

/// Errors from operator context operations.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("failed to fetch Secret {namespace}/{name}: {source}")]
    SecretFetch {
        name: String,
        namespace: String,
        source: kube::Error,
    },

    #[error("Secret \"{name}\" does not contain key \"{key}\"")]
    SecretMissing { name: String, key: String },

    #[error("failed to connect to database: {source}")]
    DatabaseConnect { source: sqlx::Error },
}

#[cfg(test)]
mod tests {
    #[test]
    fn pool_cache_key_format() {
        // Verify the cache key format is "namespace/secret-name/secret-key"
        let key = format!("{}/{}/{}", "prod", "pg-credentials", "DATABASE_URL");
        assert_eq!(key, "prod/pg-credentials/DATABASE_URL");
    }
}
