use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    account::SecuritiesAccountIdentifier, application::ApplicationIdentifier,
    stock::StockIdentifier,
};

/// Event emitted when an application fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationFailed {
    pub identifier: ApplicationIdentifier,
    pub stock: StockIdentifier,
    pub securities_account: SecuritiesAccountIdentifier,
    pub error_message: String,
    pub failed_at: DateTime<Utc>,
}
