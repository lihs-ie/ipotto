use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Metadata for a Pub/Sub event envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubSubEventMetadata {
    pub timestamp: DateTime<Utc>,
    pub version: u32,
    pub correlation_id: Uuid,
    pub service_name: String,
}

impl PubSubEventMetadata {
    /// Creates metadata with the current timestamp and default version.
    pub fn new(service_name: impl Into<String>, correlation_id: Option<Uuid>) -> Self {
        Self {
            timestamp: Utc::now(),
            version: 1,
            correlation_id: correlation_id.unwrap_or_else(Uuid::new_v4),
            service_name: service_name.into(),
        }
    }
}
