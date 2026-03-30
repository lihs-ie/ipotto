use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        application::ApplicationIdentifier,
        operation_log::{
            OperationEventType, OperationLog, OperationLogIdentifier, OperationLogPayload,
            OperationStatus,
        },
    },
    errors::DomainError,
};

/// Firestore document model for `OperationLog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationLogDocument {
    pub identifier: String,
    pub application: Option<String>,
    pub event_type: OperationEventType,
    pub service_name: String,
    pub status: OperationStatus,
    pub message: String,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

impl OperationLogDocument {
    pub fn from_domain(log: &OperationLog) -> Self {
        Self {
            identifier: log.identifier().value().to_string(),
            application: log.application().map(|value| value.value().to_string()),
            event_type: log.event_type(),
            service_name: log.service_name().to_string(),
            status: log.status(),
            message: log.message().to_string(),
            error_message: log.error_message().map(ToString::to_string),
            executed_at: log.executed_at(),
        }
    }

    pub fn to_domain(&self) -> Result<OperationLog, DomainError> {
        OperationLog::reconstruct(
            OperationLogIdentifier::new(self.identifier.clone())?,
            OperationLogPayload::new(
                self.application
                    .clone()
                    .map(ApplicationIdentifier::new)
                    .transpose()?,
                self.event_type,
                self.service_name.clone(),
                self.status,
                self.message.clone(),
                self.error_message.clone(),
                self.executed_at,
            ),
        )
    }
}
