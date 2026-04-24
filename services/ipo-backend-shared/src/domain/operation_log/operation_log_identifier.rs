use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::errors::DomainError;

/// Identifier for an operation log aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
pub struct OperationLogIdentifier(String);

impl OperationLogIdentifier {
    /// Creates an identifier from a ULID string.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        Ulid::from_string(&value).map_err(|error| DomainError::InvalidIdentifier {
            kind: "operation_log".to_string(),
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
