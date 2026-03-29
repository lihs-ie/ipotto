use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::Yen;

/// Price range for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceRange {
    minimum_price: Yen,
    maximum_price: Yen,
}

impl PriceRange {
    /// Creates a price range.
    pub fn new(minimum_price: Yen, maximum_price: Yen) -> Result<Self, DomainError> {
        if minimum_price.value() == 0 {
            return Err(DomainError::InvalidPriceRange {
                reason: "minimum price must be positive".to_string(),
            });
        }
        if minimum_price.value() > maximum_price.value() {
            return Err(DomainError::InvalidPriceRange {
                reason: "minimum price must be on or below maximum price".to_string(),
            });
        }
        Ok(Self {
            minimum_price,
            maximum_price,
        })
    }

    /// Returns the minimum price.
    pub fn minimum_price(&self) -> Yen {
        self.minimum_price
    }

    /// Returns the maximum price.
    pub fn maximum_price(&self) -> Yen {
        self.maximum_price
    }
}
