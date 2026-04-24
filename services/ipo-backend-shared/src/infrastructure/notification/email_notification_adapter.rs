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
    endpoint: String,
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
        Self::new_with_endpoint(
            client,
            credential_store,
            from_address,
            "https://api.sendgrid.com/v3/mail/send",
        )
    }

    /// Creates an email adapter with a custom endpoint.
    pub fn new_with_endpoint(
        client: Client,
        credential_store: S,
        from_address: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            client,
            credential_store,
            from_address: from_address.into(),
            endpoint: endpoint.into(),
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
            .post(&self.endpoint)
            .bearer_auth(api_key)
            .json(&json!({
                "from": {"email": self.from_address.as_str()},
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use wiremock::{
        matchers::{body_partial_json, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::EmailNotificationAdapter;
    use crate::{
        acl::{
            notification::{NotificationEvent, NotificationPort},
            secrets::CredentialStorePort,
        },
        domain::{
            account::SecuritiesAccountIdentifier,
            application::ApplicationIdentifier,
            notification::{ChannelDestination, ChannelType},
            stock::{Shares, StockIdentifier, Yen},
        },
        events::ApplicationCompleted,
        infrastructure::secrets::{sendgrid_api_key_secret_name, InMemoryCredentialStore},
    };

    fn build_event() -> NotificationEvent {
        NotificationEvent::ApplicationCompleted(ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account: SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1400).expect("price"),
            applied_at: chrono::Utc::now(),
        })
    }

    #[tokio::test]
    async fn sends_email_notification_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(header("authorization", "Bearer sendgrid-token"))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&server)
            .await;

        let store = InMemoryCredentialStore::new();
        store
            .save(sendgrid_api_key_secret_name(), "sendgrid-token")
            .await
            .expect("save api key");
        let mut destination = BTreeMap::new();
        destination.insert("address".to_string(), "notify@example.com".to_string());
        let adapter = EmailNotificationAdapter::new_with_endpoint(
            reqwest::Client::new(),
            store,
            "no-reply@example.com",
            format!("{}/mail/send", server.uri()),
        );

        adapter
            .send(
                &build_event(),
                &ChannelDestination::new(ChannelType::Email, destination).expect("destination"),
            )
            .await
            .expect("send");
    }
}
