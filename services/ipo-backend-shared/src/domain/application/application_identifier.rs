use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::DomainError;

/// Identifier for a lottery application aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ApplicationIdentifier(String);

impl ApplicationIdentifier {
    /// Creates an identifier from an existing ULID string.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        Ulid::from_string(&value).map_err(|error| DomainError::InvalidIdentifier {
            kind: "application_identifier".to_string(),
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

impl TryFrom<String> for ApplicationIdentifier {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ApplicationIdentifier> for String {
    fn from(value: ApplicationIdentifier) -> Self {
        value.0
    }
}
