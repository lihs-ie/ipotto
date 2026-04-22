use std::sync::Arc;

use crate::errors::DomainError;

/// Secret storage port.
#[async_trait::async_trait]
pub trait CredentialStorePort: Send + Sync {
    /// Saves a secret value under the provided key.
    async fn save(&self, key: &str, value: &str) -> Result<(), DomainError>;

    /// Gets a secret value by key.
    async fn get(&self, key: &str) -> Result<String, DomainError>;

    /// Deletes a secret value by key.
    async fn delete(&self, key: &str) -> Result<(), DomainError>;

    /// Returns whether the secret exists.
    async fn exists(&self, key: &str) -> Result<bool, DomainError>;
}

/// Blanket impl so that an `Arc<dyn CredentialStorePort>` trait object
/// can be passed anywhere a concrete `CredentialStorePort` value is
/// expected. Allows dependency containers to hand a single shared store
/// to both repositories (which want a trait object) and generic
/// notification adapters (which are parameterised by `S:
/// CredentialStorePort`).
#[async_trait::async_trait]
impl CredentialStorePort for Arc<dyn CredentialStorePort> {
    async fn save(&self, key: &str, value: &str) -> Result<(), DomainError> {
        (**self).save(key, value).await
    }

    async fn get(&self, key: &str) -> Result<String, DomainError> {
        (**self).get(key).await
    }

    async fn delete(&self, key: &str) -> Result<(), DomainError> {
        (**self).delete(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, DomainError> {
        (**self).exists(key).await
    }
}
