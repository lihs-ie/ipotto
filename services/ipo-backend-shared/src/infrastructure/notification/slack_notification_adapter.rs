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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use wiremock::{
        matchers::{body_string_contains, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::SlackNotificationAdapter;
    use crate::{
        acl::notification::{NotificationEvent, NotificationPort},
        domain::{
            account::SecuritiesAccountIdentifier,
            application::ApplicationIdentifier,
            notification::{ChannelDestination, ChannelType},
            stock::{Shares, StockIdentifier, Yen},
        },
        events::ApplicationCompleted,
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
    async fn sends_slack_notification_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/slack"))
            .and(body_string_contains("\"text\""))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let mut destination = BTreeMap::new();
        destination.insert("webhookUrl".to_string(), format!("{}/slack", server.uri()));
        let adapter = SlackNotificationAdapter::new(reqwest::Client::new());

        adapter
            .send(
                &build_event(),
                &ChannelDestination::new(ChannelType::Slack, destination).expect("destination"),
            )
            .await
            .expect("send");
    }
}
