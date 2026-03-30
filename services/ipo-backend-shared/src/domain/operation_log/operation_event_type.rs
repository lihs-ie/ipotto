use serde::{Deserialize, Serialize};

/// Event type recorded in an operation log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum OperationEventType {
    FetchStocks,
    ApplyLottery,
    CheckLotteryResult,
    NotificationDispatch,
    ConnectionTest,
    Other,
}

impl OperationEventType {
    /// Returns the stable string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FetchStocks => "fetch_stocks",
            Self::ApplyLottery => "apply_lottery",
            Self::CheckLotteryResult => "check_lottery_result",
            Self::NotificationDispatch => "notification_dispatch",
            Self::ConnectionTest => "connection_test",
            Self::Other => "other",
        }
    }
}
