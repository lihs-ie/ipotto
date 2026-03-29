use crate::errors::DomainError;

/// Secret storage port.
pub trait CredentialStorePort: Send + Sync {
    /// Saves a secret value under the provided key.
    fn save(&self, key: &str, value: &str) -> Result<(), DomainError>;

    /// Gets a secret value by key.
    fn get(&self, key: &str) -> Result<String, DomainError>;

    /// Deletes a secret value by key.
    fn delete(&self, key: &str) -> Result<(), DomainError>;

    /// Returns whether the secret exists.
    fn exists(&self, key: &str) -> Result<bool, DomainError>;
}
