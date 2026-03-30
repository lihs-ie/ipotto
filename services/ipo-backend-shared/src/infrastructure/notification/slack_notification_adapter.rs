use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::{
    acl::notification::{NotificationEvent, NotificationPort},
    domain::notification::{ChannelDestination, ChannelType},
    errors::DomainError,
    infrastructure::notification::NotificationMessageFormatter,
};

/// Slack webhook adapter.
#[derive(Debug, Clone)]
pub struct SlackNotificationAdapter {
    client: Client,
}

impl SlackNotificationAdapter {
    /// Creates a Slack adapter.
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NotificationPort for SlackNotificationAdapter {
    async fn send(
        &self,
        event: &NotificationEvent,
        destination: &ChannelDestination,
    ) -> Result<(), DomainError> {
        self.validate_destination(destination)?;
        let webhook_url = destination.values().get("webhookUrl").ok_or_else(|| {
            DomainError::InvalidChannelDestination {
                channel_type: ChannelType::Slack.as_str().to_string(),
                reason: "missing webhookUrl".to_string(),
            }
        })?;
        let message = NotificationMessageFormatter::format_for_slack(event);
        self.client
            .post(webhook_url)
            .json(&json!({ "text": message.text }))
            .send()
            .await
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Slack.as_str().to_string(),
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Slack.as_str().to_string(),
                reason: error.to_string(),
            })?;
        Ok(())
    }

    fn validate_destination(&self, destination: &ChannelDestination) -> Result<(), DomainError> {
        destination.validate_for(ChannelType::Slack)
    }
}
