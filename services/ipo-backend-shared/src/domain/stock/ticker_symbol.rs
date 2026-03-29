use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Ticker symbol for a listed company.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickerSymbol(String);

impl TickerSymbol {
    /// Creates a ticker symbol.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into().trim().to_string();
        let valid = value.len() == 4 && value.chars().all(|char| char.is_ascii_digit());
        if !valid {
            return Err(DomainError::InvalidTickerSymbol {
                reason: "must be a 4-digit string".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the inner string.
    pub fn value(&self) -> &str {
        &self.0
    }
}
