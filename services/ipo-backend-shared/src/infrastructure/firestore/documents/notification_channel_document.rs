use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    domain::notification::{
        ChannelDestination, ChannelIdentifier, ChannelType, NotificationChannel,
        NotificationEventType,
    },
    errors::DomainError,
};

/// Firestore sub-document for `NotificationChannel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationChannelDocument {
    pub identifier: String,
    pub channel_type: ChannelType,
    pub destination: BTreeMap<String, String>,
    pub enabled: bool,
    pub subscriptions: BTreeMap<NotificationEventType, bool>,
}

impl NotificationChannelDocument {
    pub fn from_domain(channel: &NotificationChannel) -> Self {
        Self {
            identifier: channel.identifier().value().to_string(),
            channel_type: channel.channel_type(),
            destination: channel.destination().values().clone(),
            enabled: channel.enabled(),
            subscriptions: channel.subscriptions().clone(),
        }
    }

    pub fn to_domain(&self) -> Result<NotificationChannel, DomainError> {
        NotificationChannel::reconstruct(
            ChannelIdentifier::new(self.identifier.clone())?,
            self.channel_type,
            ChannelDestination::new(self.channel_type, self.destination.clone())?,
            self.enabled,
            self.subscriptions.clone(),
        )
    }
}
