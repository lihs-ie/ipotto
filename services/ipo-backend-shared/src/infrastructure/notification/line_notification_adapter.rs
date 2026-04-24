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
    endpoint: String,
}

impl LineNotificationAdapter {
    /// Creates a LINE adapter.
    pub fn new(client: Client) -> Self {
        Self::new_with_endpoint(client, "https://notify-api.line.me/api/notify")
    }

    /// Creates a LINE adapter with a custom endpoint.
    pub fn new_with_endpoint(client: Client, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
        }
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
            .post(&self.endpoint)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use wiremock::{
        matchers::{body_string_contains, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::LineNotificationAdapter;
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
    async fn sends_line_notification_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/notify"))
            .and(header("authorization", "Bearer line-token"))
            .and(body_string_contains("message="))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let mut destination = BTreeMap::new();
        destination.insert("token".to_string(), "line-token".to_string());
        let adapter = LineNotificationAdapter::new_with_endpoint(
            reqwest::Client::new(),
            format!("{}/notify", server.uri()),
        );

        adapter
            .send(
                &build_event(),
                &ChannelDestination::new(ChannelType::Line, destination).expect("destination"),
            )
            .await
            .expect("send");
    }
}
