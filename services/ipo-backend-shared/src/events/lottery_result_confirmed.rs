use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{
    application::{ApplicationIdentifier, LotteryResult},
    stock::StockIdentifier,
};

/// Event emitted when a lottery result is confirmed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LotteryResultConfirmed {
    pub identifier: ApplicationIdentifier,
    pub stock: StockIdentifier,
    pub lottery_result: LotteryResult,
    pub confirmed_at: DateTime<Utc>,
}
