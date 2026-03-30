use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PubSubEventMetadata;

/// Pub/Sub event envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubSubEventEnvelope<T> {
    pub message_id: Uuid,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub payload: T,
    pub metadata: PubSubEventMetadata,
}

impl<T> PubSubEventEnvelope<T> {
    /// Creates a new event envelope.
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        payload: T,
        metadata: PubSubEventMetadata,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            aggregate_type: aggregate_type.into(),
            payload,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Version;

    use super::PubSubEventEnvelope;
    use crate::infrastructure::messaging::PubSubEventMetadata;

    #[test]
    fn generates_v4_identifiers() {
        let envelope = PubSubEventEnvelope::new(
            "ipo.stock.updated",
            "stock-1",
            "IpoStock",
            serde_json::json!({"ok": true}),
            PubSubEventMetadata::new("svc", None),
        );
        assert_eq!(envelope.message_id.get_version(), Some(Version::Random));
        assert_eq!(
            envelope.metadata.correlation_id.get_version(),
            Some(Version::Random)
        );
    }
}
