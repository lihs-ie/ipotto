use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::ChannelType;

/// Destination settings for a notification channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelDestination {
    values: BTreeMap<String, String>,
}

impl ChannelDestination {
    /// Creates a channel destination for the specified channel type.
    pub fn new(
        channel_type: ChannelType,
        values: BTreeMap<String, String>,
    ) -> Result<Self, DomainError> {
        let destination = Self { values };
        destination.validate_for(channel_type)?;
        Ok(destination)
    }

    /// Validates the destination against the given channel type.
    pub fn validate_for(&self, channel_type: ChannelType) -> Result<(), DomainError> {
        let required_key = match channel_type {
            ChannelType::Line => "token",
            ChannelType::Email => "address",
            ChannelType::Slack => "webhookUrl",
        };

        let valid = self
            .values
            .get(required_key)
            .is_some_and(|value| !value.trim().is_empty());
        if !valid {
            return Err(DomainError::InvalidChannelDestination {
                channel_type: channel_type.as_str().to_string(),
                reason: format!("missing required key: {required_key}"),
            });
        }

        Ok(())
    }

    /// Returns the raw destination map.
    pub fn values(&self) -> &BTreeMap<String, String> {
        &self.values
    }
}
