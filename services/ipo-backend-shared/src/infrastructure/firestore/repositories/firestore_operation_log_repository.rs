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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::FirestoreOperationLogRepository;
    use crate::domain::operation_log::{
        OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
        OperationStatus,
    };

    fn build_log(
        event_type: OperationEventType,
        status: OperationStatus,
        executed_at: chrono::DateTime<Utc>,
    ) -> OperationLog {
        OperationLog::create(OperationLogPayload::new(
            None,
            event_type,
            "ipo-service",
            status,
            "executed",
            None,
            executed_at,
        ))
        .expect("log")
    }

    #[test]
    fn filters_operation_logs() {
        let repository = FirestoreOperationLogRepository::new();
        let first = build_log(
            OperationEventType::FetchStocks,
            OperationStatus::Succeeded,
            Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0)
                .single()
                .expect("first"),
        );
        let second = build_log(
            OperationEventType::ConnectionTest,
            OperationStatus::Failed,
            Utc.with_ymd_and_hms(2026, 4, 3, 9, 0, 0)
                .single()
                .expect("second"),
        );

        repository.save(&first).expect("save first");
        repository.save(&second).expect("save second");

        assert_eq!(
            repository
                .find_by_event_type(OperationEventType::ConnectionTest)
                .expect("by event")
                .len(),
            1
        );
        assert_eq!(
            repository
                .find_by_date_range(
                    Utc.with_ymd_and_hms(2026, 4, 2, 0, 0, 0)
                        .single()
                        .expect("start"),
                    Utc.with_ymd_and_hms(2026, 4, 4, 0, 0, 0)
                        .single()
                        .expect("end"),
                )
                .expect("by range")
                .len(),
            1
        );
    }
}
