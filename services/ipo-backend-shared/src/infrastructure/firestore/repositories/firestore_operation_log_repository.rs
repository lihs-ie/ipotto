use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};

use crate::{
    domain::operation_log::{OperationEventType, OperationLog, OperationLogRepository},
    errors::DomainError,
    infrastructure::firestore::documents::OperationLogDocument,
};

/// Concrete operation log repository with Firestore-oriented document mapping.
#[derive(Debug, Clone, Default)]
pub struct FirestoreOperationLogRepository {
    documents: Arc<Mutex<BTreeMap<String, OperationLogDocument>>>,
}

impl FirestoreOperationLogRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OperationLogRepository for FirestoreOperationLogRepository {
    fn save(&self, log: &OperationLog) -> Result<(), DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .insert(
                log.identifier().value().to_string(),
                OperationLogDocument::from_domain(log),
            );
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<OperationLog>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .map(|document| document.to_domain())
            .collect()
    }

    fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<OperationLog>, DomainError> {
        self.find_all().map(|logs| {
            logs.into_iter()
                .filter(|log| log.executed_at() >= start && log.executed_at() <= end)
                .collect()
        })
    }

    fn find_by_event_type(
        &self,
        event_type: OperationEventType,
    ) -> Result<Vec<OperationLog>, DomainError> {
        self.find_all().map(|logs| {
            logs.into_iter()
                .filter(|log| log.event_type() == event_type)
                .collect()
        })
    }
}
