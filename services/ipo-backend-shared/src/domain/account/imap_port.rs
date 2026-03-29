use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// IMAP port for mail access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ImapPort(u16);

impl ImapPort {
    /// Creates an IMAP port.
    pub fn new(value: u16) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidImapPort {
                reason: "must be between 1 and 65535".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the raw value.
    pub fn value(&self) -> u16 {
        self.0
    }
}
