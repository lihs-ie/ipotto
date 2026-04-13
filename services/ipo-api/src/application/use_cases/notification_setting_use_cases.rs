use std::{collections::BTreeMap, sync::Arc};

use ipo_backend_shared::{
    acl::notification::NotificationEvent,
    domain::notification::{
        ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
        NotificationSetting, NotificationSettingIdentifier, NotificationSettingRepository,
    },
    errors::DomainError,
};
use serde::{Deserialize, Serialize};

use crate::infrastructure::NotificationPortRegistry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInput {
    pub channel_type: String,
    pub destination: BTreeMap<String, String>,
    pub enabled: bool,
    pub subscriptions: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationSettingInput {
    pub enabled: bool,
    pub channels: Vec<ChannelInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotificationSettingOutput {
    pub identifier: String,
    pub enabled: bool,
    pub channel_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelOutput {
    pub identifier: String,
    pub channel_type: String,
    pub destination: BTreeMap<String, String>,
    pub enabled: bool,
    pub subscriptions: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNotificationSettingOutput {
    pub identifier: String,
    pub enabled: bool,
    pub channels: Vec<ChannelOutput>,
}

pub struct UpdateNotificationSettingUseCase {
    repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
}

impl UpdateNotificationSettingUseCase {
    pub fn new(repository: Arc<dyn NotificationSettingRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        input: UpdateNotificationSettingInput,
    ) -> Result<UpdateNotificationSettingOutput, DomainError> {
        let channels = input
            .channels
            .into_iter()
            .map(build_notification_channel)
            .collect::<Result<Vec<_>, _>>()?;

        let mut setting = NotificationSetting::reconstruct(
            NotificationSettingIdentifier::default_id(),
            false,
            channels,
        )?;
        if input.enabled {
            setting.enable()?;
        } else {
            setting.disable();
        }

        self.repository.save(&setting)?;
        Ok(UpdateNotificationSettingOutput {
            identifier: setting.identifier().value().to_string(),
            enabled: setting.enabled(),
            channel_count: setting.channels().len(),
        })
    }
}

pub struct GetNotificationSettingUseCase {
    repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
}

impl GetNotificationSettingUseCase {
    pub fn new(repository: Arc<dyn NotificationSettingRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<GetNotificationSettingOutput, DomainError> {
        let setting = self.repository.find_default()?;
        Ok(GetNotificationSettingOutput {
            identifier: setting.identifier().value().to_string(),
            enabled: setting.enabled(),
            channels: setting.channels().iter().map(channel_to_output).collect(),
        })
    }
}

pub struct DispatchNotificationUseCase {
    repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
    registry: Arc<NotificationPortRegistry>,
}

impl DispatchNotificationUseCase {
    pub fn new(
        repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
        registry: Arc<NotificationPortRegistry>,
    ) -> Self {
        Self {
            repository,
            registry,
        }
    }

    pub async fn execute(
        &self,
        event: NotificationEvent,
        event_type: NotificationEventType,
    ) -> Result<usize, DomainError> {
        let setting = self.repository.find_default()?;
        let channels = setting.find_active_channels_for_event(event_type);
        for channel in &channels {
            self.registry
                .send(channel.channel_type(), &event, channel.destination())
                .await?;
        }
        Ok(channels.len())
    }
}

fn build_notification_channel(input: ChannelInput) -> Result<NotificationChannel, DomainError> {
    let channel_type = parse_channel_type(&input.channel_type)?;
    let subscriptions = input
        .subscriptions
        .into_iter()
        .map(|(key, enabled)| Ok((parse_notification_event_type(&key)?, enabled)))
        .collect::<Result<BTreeMap<_, _>, DomainError>>()?;

    NotificationChannel::create(
        channel_type,
        ChannelDestination::new(channel_type, input.destination)?,
        input.enabled,
        subscriptions,
    )
}

fn channel_to_output(channel: &NotificationChannel) -> ChannelOutput {
    ChannelOutput {
        identifier: channel.identifier().value().to_string(),
        channel_type: channel.channel_type().as_str().to_string(),
        destination: channel.destination().values().clone(),
        enabled: channel.enabled(),
        subscriptions: channel
            .subscriptions()
            .iter()
            .map(|(event_type, enabled)| (notification_event_type_as_str(*event_type), *enabled))
            .collect(),
    }
}

fn parse_channel_type(value: &str) -> Result<ChannelType, DomainError> {
    match value {
        "LINE" => Ok(ChannelType::Line),
        "Email" => Ok(ChannelType::Email),
        "Slack" => Ok(ChannelType::Slack),
        other => Err(DomainError::InvalidChannelDestination {
            channel_type: other.to_string(),
            reason: "unsupported channel type".to_string(),
        }),
    }
}

fn parse_notification_event_type(value: &str) -> Result<NotificationEventType, DomainError> {
    match value {
        "ApplicationCompleted" => Ok(NotificationEventType::ApplicationCompleted),
        "LotteryResultWon" => Ok(NotificationEventType::LotteryResultWon),
        "LotteryResultLost" => Ok(NotificationEventType::LotteryResultLost),
        "OperationError" => Ok(NotificationEventType::OperationError),
        "StockUpdated" => Ok(NotificationEventType::StockUpdated),
        other => Err(DomainError::InvalidChannelDestination {
            channel_type: "notification".to_string(),
            reason: format!("unsupported notification event type: {other}"),
        }),
    }
}

fn notification_event_type_as_str(value: NotificationEventType) -> String {
    match value {
        NotificationEventType::ApplicationCompleted => "ApplicationCompleted".to_string(),
        NotificationEventType::LotteryResultWon => "LotteryResultWon".to_string(),
        NotificationEventType::LotteryResultLost => "LotteryResultLost".to_string(),
        NotificationEventType::OperationError => "OperationError".to_string(),
        NotificationEventType::StockUpdated => "StockUpdated".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use async_trait::async_trait;
    use chrono::Utc;
    use ipo_backend_shared::{
        acl::notification::{NotificationEvent, NotificationPort},
        domain::notification::{NotificationEventType, NotificationSettingRepository},
        errors::DomainError,
        events::ApplicationCompleted,
        infrastructure::firestore::repositories::FirestoreNotificationSettingRepository,
    };
    use tokio::sync::Mutex;

    use crate::infrastructure::NotificationPortRegistry;

    use super::{
        ChannelInput, DispatchNotificationUseCase, GetNotificationSettingUseCase,
        UpdateNotificationSettingInput, UpdateNotificationSettingUseCase,
    };

    #[derive(Debug, Default)]
    struct RecordingPort {
        sends: Mutex<usize>,
    }

    #[async_trait]
    impl NotificationPort for RecordingPort {
        async fn send(
            &self,
            _event: &NotificationEvent,
            _destination: &ipo_backend_shared::domain::notification::ChannelDestination,
        ) -> Result<(), DomainError> {
            *self.sends.lock().await += 1;
            Ok(())
        }

        fn validate_destination(
            &self,
            _destination: &ipo_backend_shared::domain::notification::ChannelDestination,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn updates_gets_and_dispatches_notification_settings() {
        let repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let line = Arc::new(RecordingPort::default());
        let email = Arc::new(RecordingPort::default());
        let slack = Arc::new(RecordingPort::default());
        let registry = Arc::new(NotificationPortRegistry::new(
            line.clone(),
            email.clone(),
            slack.clone(),
        ));

        let mut destination = BTreeMap::new();
        destination.insert("address".to_string(), "notify@example.com".to_string());
        let mut subscriptions = BTreeMap::new();
        subscriptions.insert("ApplicationCompleted".to_string(), true);
        UpdateNotificationSettingUseCase::new(repository.clone())
            .execute(UpdateNotificationSettingInput {
                enabled: true,
                channels: vec![ChannelInput {
                    channel_type: "Email".to_string(),
                    destination,
                    enabled: true,
                    subscriptions,
                }],
            })
            .expect("update");

        let output = GetNotificationSettingUseCase::new(repository.clone())
            .execute()
            .expect("get");
        assert!(output.enabled);
        assert_eq!(output.channels.len(), 1);

        let sent = DispatchNotificationUseCase::new(repository, registry)
            .execute(
                NotificationEvent::ApplicationCompleted(ApplicationCompleted {
                    identifier:
                        ipo_backend_shared::domain::application::ApplicationIdentifier::generate(),
                    stock: ipo_backend_shared::domain::stock::StockIdentifier::generate(),
                    securities_account:
                        ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
                    applied_shares: ipo_backend_shared::domain::stock::Shares::new(100)
                        .expect("shares"),
                    applied_price: ipo_backend_shared::domain::stock::Yen::new(1000)
                        .expect("price"),
                    applied_at: Utc::now(),
                }),
                NotificationEventType::ApplicationCompleted,
            )
            .await
            .expect("dispatch");

        assert_eq!(sent, 1);
        assert_eq!(*email.sends.lock().await, 1);
        assert_eq!(*line.sends.lock().await, 0);
        assert_eq!(*slack.sends.lock().await, 0);
    }

    #[test]
    fn rejects_unsupported_channel_types() {
        let repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let error = UpdateNotificationSettingUseCase::new(repository)
            .execute(UpdateNotificationSettingInput {
                enabled: true,
                channels: vec![ChannelInput {
                    channel_type: "PagerDuty".to_string(),
                    destination: BTreeMap::new(),
                    enabled: true,
                    subscriptions: BTreeMap::new(),
                }],
            })
            .expect_err("invalid channel");

        assert!(matches!(
            error,
            DomainError::InvalidChannelDestination { .. }
        ));
    }
}
