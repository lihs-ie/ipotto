use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{PriceRange, Yen};

/// Pricing information for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoPricing {
    price_range: PriceRange,
    offer_price: Option<Yen>,
}

impl IpoPricing {
    /// Creates pricing information.
    pub fn new(price_range: PriceRange, offer_price: Option<Yen>) -> Result<Self, DomainError> {
        if let Some(offer_price) = offer_price {
            let min = price_range.minimum_price().value();
            let max = price_range.maximum_price().value();
            let current = offer_price.value();
            if current < min || current > max {
                return Err(DomainError::InvalidPriceRange {
                    reason: "offer price must be within the price range".to_string(),
                });
            }
        }
        Ok(Self {
            price_range,
            offer_price,
        })
    }

    /// Returns the price range.
    pub fn price_range(&self) -> &PriceRange {
        &self.price_range
    }

    /// Returns the offer price.
    pub fn offer_price(&self) -> Option<Yen> {
        self.offer_price
    }
}
