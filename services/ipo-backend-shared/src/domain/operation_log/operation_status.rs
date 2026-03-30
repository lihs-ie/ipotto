use serde::{Deserialize, Serialize};

/// Execution status for an operation log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Succeeded,
    Failed,
}

impl OperationStatus {
    /// Returns the stable string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}
