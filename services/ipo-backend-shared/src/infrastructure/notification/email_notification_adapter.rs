use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::{
    acl::{
        notification::{NotificationEvent, NotificationPort},
        secrets::CredentialStorePort,
    },
    domain::notification::{ChannelDestination, ChannelType},
    errors::DomainError,
    infrastructure::{
        notification::NotificationMessageFormatter, secrets::sendgrid_api_key_secret_name,
    },
};

/// SendGrid adapter.
#[derive(Clone)]
pub struct EmailNotificationAdapter<S>
where
    S: CredentialStorePort,
{
    client: Client,
    credential_store: S,
    from_address: String,
}

impl<S> core::fmt::Debug for EmailNotificationAdapter<S>
where
    S: CredentialStorePort,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EmailNotificationAdapter")
            .field("from_address", &self.from_address)
            .finish()
    }
}

impl<S> EmailNotificationAdapter<S>
where
    S: CredentialStorePort,
{
    /// Creates an email adapter.
    pub fn new(client: Client, credential_store: S, from_address: impl Into<String>) -> Self {
        Self {
            client,
            credential_store,
            from_address: from_address.into(),
        }
    }
}

#[async_trait]
impl<S> NotificationPort for EmailNotificationAdapter<S>
where
    S: CredentialStorePort,
{
    async fn send(
        &self,
        event: &NotificationEvent,
        destination: &ChannelDestination,
    ) -> Result<(), DomainError> {
        self.validate_destination(destination)?;
        let api_key = self
            .credential_store
            .get(sendgrid_api_key_secret_name())
            .await?;
        let to = destination.values().get("address").ok_or_else(|| {
            DomainError::InvalidChannelDestination {
                channel_type: ChannelType::Email.as_str().to_string(),
                reason: "missing address".to_string(),
            }
        })?;
        let message = NotificationMessageFormatter::format_for_email(event);
        self.client
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(api_key)
            .json(&json!({
                "from": {"email": self.from_address},
                "personalizations": [{"to": [{"email": to}]}],
                "subject": message.subject,
                "content": [{"type": "text/plain", "value": message.body}],
            }))
            .send()
            .await
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Email.as_str().to_string(),
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::NotificationSendError {
                channel_type: ChannelType::Email.as_str().to_string(),
                reason: error.to_string(),
            })?;
        Ok(())
    }

    fn validate_destination(&self, destination: &ChannelDestination) -> Result<(), DomainError> {
        destination.validate_for(ChannelType::Email)
    }
}
