use serde::{Deserialize, Serialize};

use crate::events::{
    ApplicationCompleted, ApplicationFailed, ImageAuthenticationFailed, IpoInfoUpdated,
    LotteryResultConfirmed, OperationErrorOccurred,
};

/// Notification-specific event union used by notification adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationEvent {
    ApplicationCompleted(ApplicationCompleted),
    ApplicationFailed(ApplicationFailed),
    LotteryResultConfirmed(LotteryResultConfirmed),
    IpoInfoUpdated(IpoInfoUpdated),
    OperationErrorOccurred(OperationErrorOccurred),
    ImageAuthenticationFailed(ImageAuthenticationFailed),
}

impl NotificationEvent {
    /// Returns a stable event name.
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::ApplicationCompleted(_) => "application_completed",
            Self::ApplicationFailed(_) => "application_failed",
            Self::LotteryResultConfirmed(_) => "lottery_result_confirmed",
            Self::IpoInfoUpdated(_) => "ipo_info_updated",
            Self::OperationErrorOccurred(_) => "operation_error_occurred",
            Self::ImageAuthenticationFailed(_) => "image_authentication_failed",
        }
    }
}
