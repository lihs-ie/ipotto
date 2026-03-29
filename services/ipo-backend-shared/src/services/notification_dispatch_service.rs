use crate::domain::notification::{
    NotificationChannel, NotificationEventType, NotificationSetting,
};

/// Service for resolving notification channels for an event.
pub struct NotificationDispatchService;

impl NotificationDispatchService {
    /// Returns channels that should receive the specified event.
    pub fn resolve_channels_for_event(
        setting: &NotificationSetting,
        event_type: NotificationEventType,
    ) -> Vec<&NotificationChannel> {
        setting.find_active_channels_for_event(event_type)
    }
}
