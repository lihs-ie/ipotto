use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Japanese yen amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
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

impl TryFrom<i64> for Yen {
    type Error = DomainError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Yen> for i64 {
    fn from(value: Yen) -> Self {
        value.value()
    }
}
