use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Japanese yen amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Yen(i64);

impl Yen {
    /// Creates a yen amount.
    pub fn new(amount: i64) -> Result<Self, DomainError> {
        if amount < 0 {
            return Err(DomainError::InvalidYen {
                reason: "must be non-negative".to_string(),
            });
        }
        Ok(Self(amount))
    }

    /// Returns the raw amount.
    pub fn value(&self) -> i64 {
        self.0
    }
}
