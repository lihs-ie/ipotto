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
