use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{acl::secrets::CredentialStorePort, errors::DomainError};

/// In-memory credential store for tests and local adapters.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCredentialStore {
    secrets: Arc<Mutex<BTreeMap<String, String>>>,
}

impl InMemoryCredentialStore {
    /// Creates a new in-memory credential store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStorePort for InMemoryCredentialStore {
    fn save(&self, key: &str, value: &str) -> Result<(), DomainError> {
        self.secrets
            .lock()
            .map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get(&self, key: &str) -> Result<String, DomainError> {
        self.secrets
            .lock()
            .map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
            .get(key)
            .cloned()
            .ok_or_else(|| DomainError::SecretPayloadError {
                reason: format!("secret not found: {key}"),
            })
    }

    fn delete(&self, key: &str) -> Result<(), DomainError> {
        self.secrets
            .lock()
            .map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
            .remove(key);
        Ok(())
    }

    fn exists(&self, key: &str) -> Result<bool, DomainError> {
        Ok(self
            .secrets
            .lock()
            .map_err(|error| DomainError::SecretPayloadError {
                reason: error.to_string(),
            })?
            .contains_key(key))
    }
}
