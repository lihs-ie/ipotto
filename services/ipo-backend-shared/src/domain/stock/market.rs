use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Market segment for an IPO stock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Market {
    Prime,
    Standard,
    Growth,
}

impl Market {
    /// Creates a market from a string representation.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref() {
            "Prime" | "プライム" => Ok(Self::Prime),
            "Standard" | "スタンダード" => Ok(Self::Standard),
            "Growth" | "グロース" => Ok(Self::Growth),
            other => Err(DomainError::InvalidMarket {
                reason: format!("unsupported market: {other}"),
            }),
        }
    }

    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prime => "Prime",
            Self::Standard => "Standard",
            Self::Growth => "Growth",
        }
    }
}
