use async_trait::async_trait;

use crate::{
    acl::notification::NotificationEvent, domain::notification::ChannelDestination,
    errors::DomainError,
};

/// Notification delivery port.
#[async_trait]
pub trait NotificationPort: Send + Sync {
    /// Sends a notification event to the destination.
    async fn send(
        &self,
        event: &NotificationEvent,
        destination: &ChannelDestination,
    ) -> Result<(), DomainError>;

    /// Validates the destination before delivery.
    fn validate_destination(&self, destination: &ChannelDestination) -> Result<(), DomainError>;
}
