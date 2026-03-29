use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{
    ChannelDestination, ChannelIdentifier, NotificationChannel, NotificationEventType,
    NotificationSettingIdentifier,
};

/// Notification setting aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationSetting {
    identifier: NotificationSettingIdentifier,
    enabled: bool,
    channels: Vec<NotificationChannel>,
}

impl NotificationSetting {
    /// Creates a default notification setting aggregate.
    pub fn create(channels: Vec<NotificationChannel>) -> Result<Self, DomainError> {
        Self::reconstruct(NotificationSettingIdentifier::default_id(), false, channels)
    }

    /// Reconstructs notification settings from persisted state.
    pub fn reconstruct(
        identifier: NotificationSettingIdentifier,
        enabled: bool,
        channels: Vec<NotificationChannel>,
    ) -> Result<Self, DomainError> {
        let setting = Self {
            identifier,
            enabled,
            channels,
        };
        setting.ensure_unique_channel_type()?;
        if setting.enabled && setting.active_channels().is_empty() {
            return Err(DomainError::NoActiveChannel);
        }
        Ok(setting)
    }

    /// Enables notifications.
    pub fn enable(&mut self) -> Result<(), DomainError> {
        if self.active_channels().is_empty() {
            return Err(DomainError::NoActiveChannel);
        }
        self.enabled = true;
        Ok(())
    }

    /// Disables notifications.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Adds a notification channel.
    pub fn add_channel(&mut self, channel: NotificationChannel) -> Result<(), DomainError> {
        if self
            .channels
            .iter()
            .any(|existing| existing.channel_type() == channel.channel_type())
        {
            return Err(DomainError::DuplicateChannelType {
                channel_type: channel.channel_type().as_str().to_string(),
            });
        }
        self.channels.push(channel);
        Ok(())
    }

    /// Removes a notification channel.
    pub fn remove_channel(&mut self, channel_id: &ChannelIdentifier) {
        self.channels
            .retain(|channel| channel.identifier() != channel_id);
    }

    /// Updates a notification channel destination.
    pub fn update_channel_destination(
        &mut self,
        channel_id: &ChannelIdentifier,
        destination: ChannelDestination,
    ) -> Result<(), DomainError> {
        let channel = self
            .channels
            .iter_mut()
            .find(|channel| channel.identifier() == channel_id)
            .ok_or_else(|| DomainError::NotificationChannelNotFound {
                channel_id: channel_id.value().to_string(),
            })?;
        channel.update_destination(destination)
    }

    /// Returns active channels for the specified event.
    pub fn find_active_channels_for_event(
        &self,
        event_type: NotificationEventType,
    ) -> Vec<&NotificationChannel> {
        if !self.enabled {
            return Vec::new();
        }
        self.channels
            .iter()
            .filter(|channel| channel.enabled() && channel.is_subscribed_to(event_type))
            .collect()
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &NotificationSettingIdentifier {
        &self.identifier
    }

    /// Returns whether notification dispatch is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the channels.
    pub fn channels(&self) -> &[NotificationChannel] {
        &self.channels
    }

    fn active_channels(&self) -> Vec<&NotificationChannel> {
        self.channels
            .iter()
            .filter(|channel| channel.enabled())
            .collect()
    }

    fn ensure_unique_channel_type(&self) -> Result<(), DomainError> {
        for (index, current) in self.channels.iter().enumerate() {
            if self
                .channels
                .iter()
                .skip(index + 1)
                .any(|other| other.channel_type() == current.channel_type())
            {
                return Err(DomainError::DuplicateChannelType {
                    channel_type: current.channel_type().as_str().to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Repository contract for notification settings.
pub trait NotificationSettingRepository {
    /// Finds a notification setting by identifier.
    fn find_by_id(
        &self,
        identifier: &NotificationSettingIdentifier,
    ) -> Result<Option<NotificationSetting>, DomainError>;

    /// Saves a notification setting aggregate.
    fn save(&self, setting: &NotificationSetting) -> Result<(), DomainError>;

    /// Returns the default notification setting aggregate.
    fn find_default(&self) -> Result<NotificationSetting, DomainError>;
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::NotificationSetting;
    use crate::{
        domain::notification::{
            ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
        },
        errors::DomainError,
    };

    fn build_channel(channel_type: ChannelType) -> NotificationChannel {
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
            true,
            subscriptions,
        )
        .expect("channel")
    }

    #[test]
    fn rejects_enable_without_active_channel() {
        let mut setting = NotificationSetting::create(Vec::new()).expect("setting");
        let result = setting.enable();
        assert!(result.is_err());
    }

    #[test]
    fn filters_channels_by_event() {
        let channel = build_channel(ChannelType::Email);
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        let channels =
            setting.find_active_channels_for_event(NotificationEventType::ApplicationCompleted);
        assert_eq!(channels.len(), 1);
    }

    #[test]
    fn rejects_duplicate_channel_types() {
        let channel1 = build_channel(ChannelType::Email);
        let channel2 = build_channel(ChannelType::Email);
        let result = NotificationSetting::reconstruct(
            crate::domain::notification::NotificationSettingIdentifier::default_id(),
            false,
            vec![channel1, channel2],
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_updating_missing_channel() {
        let mut setting =
            NotificationSetting::create(vec![build_channel(ChannelType::Email)]).expect("setting");
        let mut destination = BTreeMap::new();
        destination.insert("address".to_string(), "other@example.com".to_string());

        let result = setting.update_channel_destination(
            &crate::domain::notification::ChannelIdentifier::generate(),
            ChannelDestination::new(ChannelType::Email, destination).expect("destination"),
        );

        assert!(matches!(
            result,
            Err(DomainError::NotificationChannelNotFound { .. })
        ));
    }
}
