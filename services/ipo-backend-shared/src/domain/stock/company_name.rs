use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Company name for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyName(String);

impl CompanyName {
    /// Creates a company name.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        let length = value.chars().count();
        if !(1..=200).contains(&length) {
            return Err(DomainError::InvalidCompanyName {
                reason: "must be between 1 and 200 characters".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner string.
    pub fn value(&self) -> &str {
        &self.0
    }
}
