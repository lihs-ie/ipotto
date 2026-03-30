use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::notification::{
        NotificationSetting, NotificationSettingIdentifier, NotificationSettingRepository,
    },
    errors::DomainError,
    infrastructure::firestore::documents::{
        NotificationChannelDocument, NotificationSettingDocument,
    },
};

/// Concrete notification setting repository with Firestore-like subcollection handling.
#[derive(Debug, Clone, Default)]
pub struct FirestoreNotificationSettingRepository {
    parent: Arc<Mutex<Option<NotificationSettingDocument>>>,
    channels: Arc<Mutex<BTreeMap<String, NotificationChannelDocument>>>,
}

impl FirestoreNotificationSettingRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationSettingRepository for FirestoreNotificationSettingRepository {
    fn find_by_id(
        &self,
        identifier: &NotificationSettingIdentifier,
    ) -> Result<Option<NotificationSetting>, DomainError> {
        let parent = self
            .parent
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .clone();
        let Some(parent) = parent else {
            return Ok(None);
        };
        if parent.identifier != identifier.value() {
            return Ok(None);
        }
        let channels = self
            .channels
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .map(|document| document.to_domain())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(parent.to_domain(channels)?))
    }

    fn save(&self, setting: &NotificationSetting) -> Result<(), DomainError> {
        *self
            .parent
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })? = Some(NotificationSettingDocument::from_domain(setting));

        let mut channels =
            self.channels
                .lock()
                .map_err(|error| DomainError::FirestoreMappingError {
                    reason: error.to_string(),
                })?;
        channels.clear();
        for channel in setting.channels() {
            let document = NotificationChannelDocument::from_domain(channel);
            channels.insert(document.identifier.clone(), document);
        }
        Ok(())
    }

    fn find_default(&self) -> Result<NotificationSetting, DomainError> {
        self.find_by_id(&NotificationSettingIdentifier::default_id())?
            .ok_or_else(|| DomainError::FirestoreMappingError {
                reason: "default notification setting not found".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::FirestoreNotificationSettingRepository;
    use crate::domain::notification::{
        ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
        NotificationSetting, NotificationSettingRepository,
    };

    fn build_channel(channel_type: ChannelType) -> NotificationChannel {
        let mut destination = BTreeMap::new();
        match channel_type {
            ChannelType::Line => {
                destination.insert("token".to_string(), "line-token".to_string());
            }
            ChannelType::Email => {
                destination.insert("address".to_string(), "notify@example.com".to_string());
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
    fn saves_default_setting_and_replaces_channels() {
        let repository = FirestoreNotificationSettingRepository::new();
        let mut initial =
            NotificationSetting::create(vec![build_channel(ChannelType::Email)]).expect("setting");
        initial.enable().expect("enable");
        repository.save(&initial).expect("save initial");

        let found = repository.find_default().expect("find default");
        assert_eq!(found.channels().len(), 1);
        assert_eq!(found.channels()[0].channel_type(), ChannelType::Email);

        let mut replaced =
            NotificationSetting::create(vec![build_channel(ChannelType::Slack)]).expect("setting");
        replaced.enable().expect("enable");
        repository.save(&replaced).expect("save replaced");

        let found = repository.find_default().expect("find replaced");
        assert_eq!(found.channels().len(), 1);
        assert_eq!(found.channels()[0].channel_type(), ChannelType::Slack);
    }
}
