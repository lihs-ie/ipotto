use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::LotteryResult;

/// Confirmed outcome for an application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LotteryOutcome {
    result: LotteryResult,
    confirmed_at: DateTime<Utc>,
}

impl LotteryOutcome {
    /// Creates a lottery outcome.
    pub fn new(result: LotteryResult, confirmed_at: DateTime<Utc>) -> Self {
        Self {
            result,
            confirmed_at,
        }
    }

    /// Returns the result.
    pub fn result(&self) -> LotteryResult {
        self.result
    }

    /// Returns when the result was confirmed.
    pub fn confirmed_at(&self) -> DateTime<Utc> {
        self.confirmed_at
    }
}
