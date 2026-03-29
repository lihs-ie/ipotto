use serde::{Deserialize, Serialize};

/// Status of an IPO stock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StockStatus {
    Fetched,
    Eligible,
    Applied,
    Won,
    Lost,
    Alternate,
    Purchased,
    Declined,
    Sold,
    Excluded,
    Failed,
}

impl StockStatus {
    /// Returns whether the status can transition to the given next status.
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (
                Self::Fetched,
                Self::Eligible | Self::Excluded | Self::Failed
            ) | (
                Self::Eligible,
                Self::Applied | Self::Excluded | Self::Failed
            ) | (
                Self::Applied,
                Self::Won | Self::Lost | Self::Alternate | Self::Failed
            ) | (Self::Won, Self::Purchased | Self::Declined | Self::Sold)
                | (
                    Self::Alternate,
                    Self::Purchased | Self::Declined | Self::Lost
                )
                | (Self::Purchased, Self::Sold)
        )
    }

    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fetched => "Fetched",
            Self::Eligible => "Eligible",
            Self::Applied => "Applied",
            Self::Won => "Won",
            Self::Lost => "Lost",
            Self::Alternate => "Alternate",
            Self::Purchased => "Purchased",
            Self::Declined => "Declined",
            Self::Sold => "Sold",
            Self::Excluded => "Excluded",
            Self::Failed => "Failed",
        }
    }
}
