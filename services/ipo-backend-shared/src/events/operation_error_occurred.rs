use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Event emitted when a cross-cutting operation error occurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationErrorOccurred {
    pub service_name: String,
    pub operation_type: String,
    pub error_message: String,
    pub occurred_at: DateTime<Utc>,
}
