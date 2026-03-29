use async_trait::async_trait;
use reqwest::Client;

use crate::{
    acl::notification::{NotificationEvent, NotificationPort},
    domain::notification::{ChannelDestination, ChannelType},
    errors::DomainError,
    infrastructure::notification::NotificationMessageFormatter,
};

/// LINE Notify adapter.
#[derive(Debug, Clone)]
pub struct LineNotificationAdapter {
    client: Client,
}

impl LineNotificationAdapter {
    /// Creates a LINE adapter.
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NotificationPort for LineNotificationAdapter {
    async fn send(
        &self,
        event: &NotificationEvent,
        destination: &ChannelDestination,
    ) -> Result<(), DomainError> {
        self.validate_destination(destination)?;
        let token = destination.values().get("token").ok_or_else(|| {
            DomainError::InvalidChannelDestination {
                channel_type: ChannelType::Line.as_str().to_string(),
                reason: "missing token".to_string(),
            }
        })?;
        self.client
            .post("https://notify-api.line.me/api/notify")
            .bearer_auth(token)
            .form(&[(
                "message",
                NotificationMessageFormatter::format_for_line(event),
            )])
            .send()
            .await
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Line.as_str().to_string(),
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Line.as_str().to_string(),
                reason: error.to_string(),
            })?;
        Ok(())
    }

    fn validate_destination(&self, destination: &ChannelDestination) -> Result<(), DomainError> {
        destination.validate_for(ChannelType::Line)
    }
}
