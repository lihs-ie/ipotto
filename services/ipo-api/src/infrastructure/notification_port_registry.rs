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
