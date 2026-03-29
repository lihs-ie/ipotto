use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Login ID for a securities account.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginId(String);

impl LoginId {
    /// Creates a login ID.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(DomainError::IncompleteCredential);
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for LoginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginId")
            .field("value", &"********")
            .finish()
    }
}
