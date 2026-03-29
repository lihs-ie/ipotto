use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Mail address for image authentication email retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailAddress(String);

impl MailAddress {
    /// Creates a mail address.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        let valid = value.contains('@')
            && value.split('@').count() == 2
            && value
                .rsplit('.')
                .next()
                .is_some_and(|segment| !segment.is_empty());
        if !valid {
            return Err(DomainError::InvalidMailAddress {
                reason: "must be a valid email address".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    pub fn value(&self) -> &str {
        &self.0
    }
}
