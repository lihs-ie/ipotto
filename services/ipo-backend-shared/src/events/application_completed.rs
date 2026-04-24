use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    account::SecuritiesAccountIdentifier,
    application::ApplicationIdentifier,
    stock::{Shares, StockIdentifier, Yen},
};

/// Event emitted when an application completes successfully.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationCompleted {
    pub identifier: ApplicationIdentifier,
    pub stock: StockIdentifier,
    pub securities_account: SecuritiesAccountIdentifier,
    pub applied_shares: Shares,
    pub applied_price: Yen,
    pub applied_at: DateTime<Utc>,
}
