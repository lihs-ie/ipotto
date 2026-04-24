use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use gcloud_googleapis::pubsub::v1::PubsubMessage;
use gcloud_pubsub::client::{Client, ClientConfig};
use serde_json::Value;
use uuid::Uuid;

use crate::{acl::messaging::EventPublisherPort, errors::DomainError};

use super::{PubSubEventEnvelope, PubSubEventMetadata};

/// Default mapping between application-level `event_type` strings and
/// their Pub/Sub topic names. Adding a new event requires an explicit
/// entry here so publishers do not silently drop messages when the
/// mapping is missing.
pub fn default_topic_mapping() -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    // Stock catalog updates — consumed by ipo-api notification handler.
    mapping.insert(
        "ipo-info-updated".to_string(),
        "ipo-info-updated".to_string(),
    );
    // Lottery result confirmations — consumed by ipo-api notification handler.
    mapping.insert(
        "ipo-result-updated".to_string(),
        "ipo-result-updated".to_string(),
    );
    // Generic notification fan-out bus (LINE / Email / Slack).
    mapping.insert(
        "ipo-notification".to_string(),
        "ipo-notification".to_string(),
    );
    mapping.insert(
        "ApplicationCompleted".to_string(),
        "ipo-notification".to_string(),
    );
    mapping.insert(
        "ApplicationFailed".to_string(),
        "ipo-notification".to_string(),
    );
    mapping.insert(
        "ImageAuthenticationFailed".to_string(),
        "ipo-notification".to_string(),
    );
    mapping.insert(
        "OperationErrorOccurred".to_string(),
        "ipo-notification".to_string(),
    );
    mapping
}

/// Production Pub/Sub publisher that emits `PubSubEventEnvelope` JSON
/// payloads through `google-cloud-pubsub`'s async gRPC client. Honours
/// `PUBSUB_EMULATOR_HOST` via `ClientConfig::default().with_auth()` so
/// local docker-compose and CI runs hit the emulator without any code
/// changes.
#[derive(Clone)]
pub struct PubSubEventPublisher {
    service_name: String,
    client: Arc<Client>,
    topic_mapping: Arc<HashMap<String, String>>,
}

impl core::fmt::Debug for PubSubEventPublisher {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PubSubEventPublisher")
            .field("service_name", &self.service_name)
            .field("topic_mapping_size", &self.topic_mapping.len())
            .finish()
    }
}

impl PubSubEventPublisher {
    /// Builds a Pub/Sub publisher for the provided service, using
    /// `default_topic_mapping` and the ambient auth (ADC or emulator).
    /// `project_id` must match the GCP project whose topics the service
    /// publishes to; when `PUBSUB_EMULATOR_HOST` is set it must also
    /// match the emulator project (`GCP_PROJECT`, defaults to
    /// `ipotto-local` in this repo's scripts).
    pub async fn new(
        service_name: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Result<Self, DomainError> {
        Self::with_topic_mapping(service_name, project_id, default_topic_mapping()).await
    }

    /// Builds a Pub/Sub publisher with a caller-supplied topic mapping.
    /// Useful in tests or when a service needs to publish to alternate
    /// topics (e.g. DLQ smoke tests).
    pub async fn with_topic_mapping(
        service_name: impl Into<String>,
        project_id: impl Into<String>,
        topic_mapping: HashMap<String, String>,
    ) -> Result<Self, DomainError> {
        let mut config = ClientConfig::default().with_auth().await.map_err(|error| {
            DomainError::PubSubPublishError {
                reason: format!("failed to build Pub/Sub client config: {error}"),
            }
        })?;
        config.project_id = Some(project_id.into());
        let client =
            Client::new(config)
                .await
                .map_err(|error| DomainError::PubSubPublishError {
                    reason: format!("failed to construct Pub/Sub client: {error}"),
                })?;
        Ok(Self {
            service_name: service_name.into(),
            client: Arc::new(client),
            topic_mapping: Arc::new(topic_mapping),
        })
    }

    fn resolve_topic(&self, event_type: &str) -> Result<&str, DomainError> {
        resolve_topic_from_mapping(&self.topic_mapping, event_type)
    }
}

fn resolve_topic_from_mapping<'a>(
    topic_mapping: &'a HashMap<String, String>,
    event_type: &str,
) -> Result<&'a str, DomainError> {
    topic_mapping
        .get(event_type)
        .map(String::as_str)
        .ok_or_else(|| DomainError::PubSubPublishError {
            reason: format!("no Pub/Sub topic mapping configured for event_type: {event_type}"),
        })
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
        let topic_name = self.resolve_topic(event_type)?;
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

        let topic = self.client.topic(topic_name);
        let publisher = topic.new_publisher(None);
        let awaiter = publisher
            .publish(PubsubMessage {
                data: body.into_bytes(),
                ..Default::default()
            })
            .await;
        awaiter
            .get()
            .await
            .map_err(|error| DomainError::PubSubPublishError {
                reason: format!("Pub/Sub publish failed: {error}"),
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{default_topic_mapping, resolve_topic_from_mapping};

    #[test]
    fn default_mapping_covers_known_event_types() {
        let mapping = default_topic_mapping();
        assert_eq!(
            mapping.get("ipo-info-updated").map(String::as_str),
            Some("ipo-info-updated")
        );
        assert_eq!(
            mapping.get("ApplicationCompleted").map(String::as_str),
            Some("ipo-notification")
        );
        assert_eq!(
            mapping.get("ImageAuthenticationFailed").map(String::as_str),
            Some("ipo-notification")
        );
    }

    #[test]
    fn resolve_topic_maps_known_event_to_topic() {
        let mapping = default_topic_mapping();
        let topic =
            resolve_topic_from_mapping(&mapping, "ipo-info-updated").expect("mapping exists");
        assert_eq!(topic, "ipo-info-updated");
    }

    #[test]
    fn resolve_topic_surfaces_missing_mapping_as_pubsub_error() {
        let mut mapping = default_topic_mapping();
        mapping.remove("ipo-info-updated");
        let result = resolve_topic_from_mapping(&mapping, "ipo-info-updated");
        assert!(result.is_err());
    }
}
