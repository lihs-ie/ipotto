use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Reason for excluding a stock from automated application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionReason(String);

impl ExclusionReason {
    /// Creates an exclusion reason.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(DomainError::InvalidExclusionReason {
                reason: "must not be empty".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner string.
    pub fn value(&self) -> &str {
        &self.0
    }
}
