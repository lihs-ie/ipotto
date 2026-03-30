use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{acl::messaging::EventPublisherPort, errors::DomainError};

use super::{PubSubEventEnvelope, PubSubEventMetadata};

/// Concrete Pub/Sub publisher that records serialized messages.
#[derive(Debug, Clone)]
pub struct PubSubEventPublisher {
    service_name: String,
    published_messages: Arc<Mutex<Vec<String>>>,
}

impl PubSubEventPublisher {
    /// Creates a publisher for the service.
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            published_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns the published message bodies.
    pub async fn published_messages(&self) -> Result<Vec<String>, DomainError> {
        Ok(self.published_messages.lock().await.clone())
    }
}

#[async_trait]
impl EventPublisherPort for PubSubEventPublisher {
    async fn publish(
        &self,
        event_type: &str,
        aggregate_id: &str,
        aggregate_type: &str,
        payload: Value,
        correlation_id: Option<Uuid>,
    ) -> Result<(), DomainError> {
        let envelope = PubSubEventEnvelope::new(
            event_type,
            aggregate_id,
            aggregate_type,
            payload,
            PubSubEventMetadata::new(self.service_name.clone(), correlation_id),
        );
        let body =
            serde_json::to_string(&envelope).map_err(|error| DomainError::PubSubPublishError {
                reason: error.to_string(),
            })?;
        self.published_messages.lock().await.push(body);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::PubSubEventPublisher;
    use crate::{
        acl::messaging::EventPublisherPort, infrastructure::messaging::PubSubEventEnvelope,
    };

    #[tokio::test]
    async fn publishes_serialized_envelope_with_expected_metadata() {
        let publisher = PubSubEventPublisher::new("ipo-service");
        let correlation_id = Uuid::new_v4();

        publisher
            .publish(
                "ipo-info-updated",
                "stock-1",
                "IpoStock",
                json!({ "companyName": "テスト株式会社" }),
                Some(correlation_id),
            )
            .await
            .expect("publish");

        let messages = publisher.published_messages().await.expect("messages");
        assert_eq!(messages.len(), 1);

        let envelope: PubSubEventEnvelope<serde_json::Value> =
            serde_json::from_str(&messages[0]).expect("envelope");
        assert_eq!(envelope.event_type, "ipo-info-updated");
        assert_eq!(envelope.aggregate_id, "stock-1");
        assert_eq!(envelope.aggregate_type, "IpoStock");
        assert_eq!(envelope.metadata.service_name, "ipo-service");
        assert_eq!(envelope.metadata.version, 1);
        assert_eq!(envelope.metadata.correlation_id, correlation_id);
        assert_eq!(envelope.payload["companyName"], "テスト株式会社");
    }
}
