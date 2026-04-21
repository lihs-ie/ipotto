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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::NotificationDispatchService;
    use crate::domain::notification::{
        ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
        NotificationSetting,
    };

    fn build_channel(channel_type: ChannelType, enabled: bool) -> NotificationChannel {
        let mut destination = BTreeMap::new();
        match channel_type {
            ChannelType::Line => {
                destination.insert("token".to_string(), "token".to_string());
            }
            ChannelType::Email => {
                destination.insert("address".to_string(), "user@example.com".to_string());
            }
            ChannelType::Slack => {
                destination.insert("webhookUrl".to_string(), "https://example.com".to_string());
            }
        }

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(NotificationEventType::ApplicationCompleted, true);

        NotificationChannel::create(
            channel_type,
            ChannelDestination::new(channel_type, destination).expect("destination"),
            enabled,
            subscriptions,
        )
        .expect("channel")
    }

    #[test]
    fn returns_channels_subscribed_to_event_when_setting_enabled() {
        let channel = build_channel(ChannelType::Email, true);
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");

        let resolved = NotificationDispatchService::resolve_channels_for_event(
            &setting,
            NotificationEventType::ApplicationCompleted,
        );

        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn returns_empty_when_setting_disabled() {
        let channel = build_channel(ChannelType::Email, true);
        let setting = NotificationSetting::create(vec![channel]).expect("setting");

        let resolved = NotificationDispatchService::resolve_channels_for_event(
            &setting,
            NotificationEventType::ApplicationCompleted,
        );

        assert!(resolved.is_empty());
    }

    #[test]
    fn skips_channels_disabled_at_channel_level() {
        let disabled_channel = build_channel(ChannelType::Email, false);
        let enabled_channel = build_channel(ChannelType::Slack, true);
        let mut setting =
            NotificationSetting::create(vec![disabled_channel, enabled_channel]).expect("setting");
        setting.enable().expect("enable");

        let resolved = NotificationDispatchService::resolve_channels_for_event(
            &setting,
            NotificationEventType::ApplicationCompleted,
        );

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].channel_type(), ChannelType::Slack);
    }
}
