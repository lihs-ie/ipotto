use std::sync::Arc;

use ipo_backend_shared::{
    acl::notification::{NotificationEvent, NotificationPort},
    domain::notification::{ChannelDestination, ChannelType},
    errors::DomainError,
};

#[derive(Clone)]
pub struct NotificationPortRegistry {
    line: Arc<dyn NotificationPort + Send + Sync>,
    email: Arc<dyn NotificationPort + Send + Sync>,
    slack: Arc<dyn NotificationPort + Send + Sync>,
}

impl NotificationPortRegistry {
    pub fn new(
        line: Arc<dyn NotificationPort + Send + Sync>,
        email: Arc<dyn NotificationPort + Send + Sync>,
        slack: Arc<dyn NotificationPort + Send + Sync>,
    ) -> Self {
        Self { line, email, slack }
    }

    pub async fn send(
        &self,
        channel_type: ChannelType,
        event: &NotificationEvent,
        destination: &ChannelDestination,
    ) -> Result<(), DomainError> {
        match channel_type {
            ChannelType::Line => self.line.send(event, destination).await,
            ChannelType::Email => self.email.send(event, destination).await,
            ChannelType::Slack => self.slack.send(event, destination).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use async_trait::async_trait;
    use ipo_backend_shared::{
        acl::notification::{NotificationEvent, NotificationPort},
        domain::notification::{ChannelDestination, ChannelType},
        errors::DomainError,
        events::OperationErrorOccurred,
    };
    use tokio::sync::Mutex;

    use super::NotificationPortRegistry;

    #[derive(Debug, Default)]
    struct RecordingPort {
        calls: Mutex<usize>,
    }

    #[async_trait]
    impl NotificationPort for RecordingPort {
        async fn send(
            &self,
            _event: &NotificationEvent,
            _destination: &ChannelDestination,
        ) -> Result<(), DomainError> {
            *self.calls.lock().await += 1;
            Ok(())
        }

        fn validate_destination(
            &self,
            _destination: &ChannelDestination,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    fn destination(channel_type: ChannelType) -> ChannelDestination {
        let mut values = BTreeMap::new();
        match channel_type {
            ChannelType::Line => {
                values.insert("token".to_string(), "line-token".to_string());
            }
            ChannelType::Email => {
                values.insert("address".to_string(), "notify@example.com".to_string());
            }
            ChannelType::Slack => {
                values.insert(
                    "webhookUrl".to_string(),
                    "https://example.com/webhook".to_string(),
                );
            }
        }
        ChannelDestination::new(channel_type, values).expect("destination")
    }

    #[tokio::test]
    async fn routes_send_calls_by_channel_type() {
        let line = Arc::new(RecordingPort::default());
        let email = Arc::new(RecordingPort::default());
        let slack = Arc::new(RecordingPort::default());
        let registry = NotificationPortRegistry::new(line.clone(), email.clone(), slack.clone());
        let event = NotificationEvent::OperationErrorOccurred(OperationErrorOccurred {
            service_name: "ipo-api".to_string(),
            operation_type: "notification_dispatch".to_string(),
            error_message: "failed".to_string(),
            occurred_at: chrono::Utc::now(),
        });

        registry
            .send(ChannelType::Email, &event, &destination(ChannelType::Email))
            .await
            .expect("send email");
        registry
            .send(ChannelType::Slack, &event, &destination(ChannelType::Slack))
            .await
            .expect("send slack");

        assert_eq!(*line.calls.lock().await, 0);
        assert_eq!(*email.calls.lock().await, 1);
        assert_eq!(*slack.calls.lock().await, 1);
    }
}
