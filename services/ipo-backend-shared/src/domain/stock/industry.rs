use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Industry classification for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Industry(String);

impl Industry {
    /// Creates an industry.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(DomainError::InvalidIndustry {
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
