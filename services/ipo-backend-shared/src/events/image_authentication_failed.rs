use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::account::SecuritiesAccountIdentifier;

/// Event emitted when image authentication fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAuthenticationFailed {
    pub securities_account: SecuritiesAccountIdentifier,
    pub failure_reason: String,
    pub attempt_count: u32,
    pub occurred_at: DateTime<Utc>,
}
