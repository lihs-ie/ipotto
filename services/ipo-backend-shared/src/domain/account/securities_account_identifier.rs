use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::DomainError;

/// Identifier for a securities account aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecuritiesAccountIdentifier(String);

impl SecuritiesAccountIdentifier {
    /// Creates an identifier from an existing ULID string.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        Ulid::from_string(&value).map_err(|error| DomainError::InvalidIdentifier {
            kind: "securities_account_identifier".to_string(),
            reason: error.to_string(),
        })?;
        Ok(Self(value))
    }

    /// Generates a new identifier.
    pub fn generate() -> Self {
        Self(Ulid::new().to_string())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}
