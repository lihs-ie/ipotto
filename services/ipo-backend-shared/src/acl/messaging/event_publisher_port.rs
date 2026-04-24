use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::errors::DomainError;

/// Outbound event publisher port.
#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    /// Publishes an event payload to the configured messaging backend.
    async fn publish(
        &self,
        event_type: &str,
        aggregate_id: &str,
        aggregate_type: &str,
        payload: Value,
        correlation_id: Option<Uuid>,
    ) -> Result<(), DomainError>;
}
