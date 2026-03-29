use serde::{Deserialize, Serialize};

/// Lifecycle status for a lottery application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ApplicationStatus {
    Pending,
    Applied,
    ResultChecked,
}

impl ApplicationStatus {
    /// Returns whether the status can transition to the given next status.
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Applied) | (Self::Applied, Self::ResultChecked)
        )
    }

    /// Returns the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Applied => "Applied",
            Self::ResultChecked => "ResultChecked",
        }
    }
}
