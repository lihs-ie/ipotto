use serde::{Deserialize, Serialize};

/// Notification event type subscribed by channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationEventType {
    ApplicationCompleted,
    LotteryResultWon,
    LotteryResultLost,
    OperationError,
    StockUpdated,
}
