use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// IMAP host for mail access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapHost(String);

impl ImapHost {
    /// Creates an IMAP host.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(DomainError::InvalidImapHost {
                reason: "must not be empty".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    pub fn value(&self) -> &str {
        &self.0
    }
}
