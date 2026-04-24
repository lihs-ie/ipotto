use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::stock::{Shares, Yen},
    errors::DomainError,
};

/// Applied order details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedOrder {
    shares: Shares,
    price: Yen,
    ordered_at: DateTime<Utc>,
}

impl AppliedOrder {
    /// Creates an applied order.
    pub fn new(shares: Shares, price: Yen, ordered_at: DateTime<Utc>) -> Result<Self, DomainError> {
        if shares.value() == 0 {
            return Err(DomainError::InvalidShares {
                reason: "must be greater than zero".to_string(),
            });
        }
        Ok(Self {
            shares,
            price,
            ordered_at,
        })
    }

    /// Returns the share quantity.
    pub fn shares(&self) -> Shares {
        self.shares
    }

    /// Returns the order price.
    pub fn price(&self) -> Yen {
        self.price
    }

    /// Returns the timestamp when the order was placed.
    pub fn ordered_at(&self) -> DateTime<Utc> {
        self.ordered_at
    }
}
