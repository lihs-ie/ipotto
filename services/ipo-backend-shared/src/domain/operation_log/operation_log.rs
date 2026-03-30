use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{domain::application::ApplicationIdentifier, errors::DomainError};

use super::{OperationEventType, OperationLogIdentifier, OperationStatus};

/// Operation log aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationLog {
    identifier: OperationLogIdentifier,
    application: Option<ApplicationIdentifier>,
    event_type: OperationEventType,
    service_name: String,
    status: OperationStatus,
    message: String,
    error_message: Option<String>,
    executed_at: DateTime<Utc>,
}

/// Payload used to create or reconstruct an operation log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationLogPayload {
    application: Option<ApplicationIdentifier>,
    event_type: OperationEventType,
    service_name: String,
    status: OperationStatus,
    message: String,
    error_message: Option<String>,
    executed_at: DateTime<Utc>,
}

impl OperationLogPayload {
    /// Creates an operation log payload.
    pub fn new(
        application: Option<ApplicationIdentifier>,
        event_type: OperationEventType,
        service_name: impl Into<String>,
        status: OperationStatus,
        message: impl Into<String>,
        error_message: Option<String>,
        executed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            application,
            event_type,
            service_name: service_name.into(),
            status,
            message: message.into(),
            error_message,
            executed_at,
        }
    }
}

impl OperationLog {
    /// Creates a new operation log entry.
    pub fn create(payload: OperationLogPayload) -> Result<Self, DomainError> {
        Self::reconstruct(OperationLogIdentifier::generate(), payload)
    }

    /// Reconstructs an operation log entry from persisted state.
    pub fn reconstruct(
        identifier: OperationLogIdentifier,
        payload: OperationLogPayload,
    ) -> Result<Self, DomainError> {
        let service_name = payload.service_name.trim().to_string();
        let message = payload.message.trim().to_string();
        if service_name.is_empty() {
            return Err(DomainError::OperationLogValidationError {
                reason: "service_name must not be empty".to_string(),
            });
        }
        if message.is_empty() {
            return Err(DomainError::OperationLogValidationError {
                reason: "message must not be empty".to_string(),
            });
        }
        Ok(Self {
            identifier,
            application: payload.application,
            event_type: payload.event_type,
            service_name,
            status: payload.status,
            message,
            error_message: payload
                .error_message
                .filter(|value| !value.trim().is_empty()),
            executed_at: payload.executed_at,
        })
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &OperationLogIdentifier {
        &self.identifier
    }

    /// Returns the application identifier.
    pub fn application(&self) -> Option<&ApplicationIdentifier> {
        self.application.as_ref()
    }

    /// Returns the event type.
    pub fn event_type(&self) -> OperationEventType {
        self.event_type
    }

    /// Returns the service name.
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Returns the execution status.
    pub fn status(&self) -> OperationStatus {
        self.status
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the optional error message.
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Returns the execution timestamp.
    pub fn executed_at(&self) -> DateTime<Utc> {
        self.executed_at
    }
}

/// Repository contract for operation logs.
pub trait OperationLogRepository {
    /// Saves an operation log entry.
    fn save(&self, log: &OperationLog) -> Result<(), DomainError>;

    /// Returns all operation log entries.
    fn find_all(&self) -> Result<Vec<OperationLog>, DomainError>;

    /// Returns entries executed in the provided date range.
    fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<OperationLog>, DomainError>;

    /// Returns entries filtered by event type.
    fn find_by_event_type(
        &self,
        event_type: OperationEventType,
    ) -> Result<Vec<OperationLog>, DomainError>;
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{OperationLog, OperationLogPayload};
    use crate::domain::operation_log::{OperationEventType, OperationStatus};

    #[test]
    fn creates_valid_operation_log() {
        let log = OperationLog::create(OperationLogPayload::new(
            None,
            OperationEventType::FetchStocks,
            "ipo-info-fetcher",
            OperationStatus::Succeeded,
            "fetched stocks",
            None,
            Utc.with_ymd_and_hms(2026, 3, 29, 12, 0, 0)
                .single()
                .expect("timestamp"),
        ))
        .expect("log");

        assert_eq!(log.service_name(), "ipo-info-fetcher");
        assert_eq!(log.message(), "fetched stocks");
    }

    #[test]
    fn rejects_empty_service_name() {
        let result = OperationLog::create(OperationLogPayload::new(
            None,
            OperationEventType::FetchStocks,
            "",
            OperationStatus::Succeeded,
            "fetched stocks",
            None,
            Utc::now(),
        ));
        assert!(result.is_err());
    }
}
