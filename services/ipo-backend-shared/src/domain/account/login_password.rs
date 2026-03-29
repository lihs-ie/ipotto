use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Login password for a securities account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginPassword(String);

impl LoginPassword {
    /// Creates a login password.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::IncompleteCredential);
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    pub fn value(&self) -> &str {
        &self.0
    }
}
