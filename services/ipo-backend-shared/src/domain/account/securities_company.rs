use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Supported securities companies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecuritiesCompany {
    Rakuten,
}

impl SecuritiesCompany {
    /// Creates a securities company from a string value.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref() {
            "Rakuten" | "楽天証券" => Ok(Self::Rakuten),
            other => Err(DomainError::InvalidSecuritiesCompany {
                reason: format!("unsupported securities company: {other}"),
            }),
        }
    }

    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rakuten => "Rakuten",
        }
    }
}
