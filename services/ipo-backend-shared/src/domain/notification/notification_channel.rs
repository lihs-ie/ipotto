use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{ChannelDestination, ChannelIdentifier, ChannelType, NotificationEventType};

/// Notification channel entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationChannel {
    identifier: ChannelIdentifier,
    channel_type: ChannelType,
    destination: ChannelDestination,
    enabled: bool,
    subscriptions: BTreeMap<NotificationEventType, bool>,
}

impl NotificationChannel {
    /// Creates a new notification channel with a generated identifier.
    pub fn create(
        channel_type: ChannelType,
        destination: ChannelDestination,
        enabled: bool,
        subscriptions: BTreeMap<NotificationEventType, bool>,
    ) -> Result<Self, DomainError> {
        Self::reconstruct(
            ChannelIdentifier::generate(),
            channel_type,
            destination,
            enabled,
            subscriptions,
        )
    }

    /// Reconstructs a notification channel from persisted state.
    pub fn reconstruct(
        identifier: ChannelIdentifier,
        channel_type: ChannelType,
        destination: ChannelDestination,
        enabled: bool,
        subscriptions: BTreeMap<NotificationEventType, bool>,
    ) -> Result<Self, DomainError> {
        destination.validate_for(channel_type)?;
        Ok(Self {
            identifier,
            channel_type,
            destination,
            enabled,
            subscriptions,
        })
    }

    /// Returns whether the channel subscribes to the event type.
    pub fn is_subscribed_to(&self, event_type: NotificationEventType) -> bool {
        self.subscriptions
            .get(&event_type)
            .copied()
            .unwrap_or(false)
    }

    /// Updates the destination.
    pub fn update_destination(
        &mut self,
        destination: ChannelDestination,
    ) -> Result<(), DomainError> {
        destination.validate_for(self.channel_type)?;
        self.destination = destination;
        Ok(())
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &ChannelIdentifier {
        &self.identifier
    }

    /// Returns the channel type.
    pub fn channel_type(&self) -> ChannelType {
        self.channel_type
    }

    /// Returns the destination.
    pub fn destination(&self) -> &ChannelDestination {
        &self.destination
    }

    /// Returns whether the channel is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
