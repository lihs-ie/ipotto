use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Share quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct Shares(u32);

impl Shares {
    /// Creates a share quantity.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidShares {
                reason: "must be greater than zero".to_string(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the raw quantity.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for Shares {
    type Error = DomainError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Shares> for u32 {
    fn from(value: Shares) -> Self {
        value.value()
    }
}
