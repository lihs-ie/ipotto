use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
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
    pub fn published_messages(&self) -> Result<Vec<String>, DomainError> {
        self.published_messages
            .lock()
            .map_err(|error| DomainError::PubSubPublishError {
                reason: error.to_string(),
            })
            .map(|messages| messages.clone())
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
        self.published_messages
            .lock()
            .map_err(|error| DomainError::PubSubPublishError {
                reason: error.to_string(),
            })?
            .push(body);
        Ok(())
    }
}
