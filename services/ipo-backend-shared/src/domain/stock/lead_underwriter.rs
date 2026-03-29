use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Lead underwriter of an IPO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeadUnderwriter(String);

impl LeadUnderwriter {
    /// Creates a lead underwriter.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(DomainError::InvalidCompanyName {
                reason: "lead underwriter must not be empty".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner string.
    pub fn value(&self) -> &str {
        &self.0
    }
}
