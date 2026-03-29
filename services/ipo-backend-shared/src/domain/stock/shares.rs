use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Share quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
