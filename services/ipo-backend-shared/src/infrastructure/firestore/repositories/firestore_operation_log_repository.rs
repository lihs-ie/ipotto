use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use firestore::{path, FirestoreDb};

use crate::{
    domain::operation_log::{OperationEventType, OperationLog, OperationLogRepository},
    errors::DomainError,
    infrastructure::firestore::{collections, documents::OperationLogDocument},
};

/// Production Firestore-backed implementation of
/// [`OperationLogRepository`]. Persists log entries to the
/// `operation_logs` collection.
#[derive(Debug, Clone)]
pub struct FirestoreOperationLogRepository {
    db: Arc<FirestoreDb>,
}

impl FirestoreOperationLogRepository {
    pub fn new(db: Arc<FirestoreDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl OperationLogRepository for FirestoreOperationLogRepository {
    async fn save(&self, log: &OperationLog) -> Result<(), DomainError> {
        let document = OperationLogDocument::from_domain(log);
        self.db
            .fluent()
            .update()
            .in_col(collections::OPERATION_LOGS)
            .document_id(log.identifier().value())
            .object(&document)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<OperationLog>, DomainError> {
        let documents: Vec<OperationLogDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::OPERATION_LOGS)
            .obj::<OperationLogDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<OperationLog>, DomainError> {
        let documents: Vec<OperationLogDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::OPERATION_LOGS)
            .filter(|q| {
                q.for_all([
                    q.field(path!(OperationLogDocument::executed_at))
                        .greater_than_or_equal(start),
                    q.field(path!(OperationLogDocument::executed_at))
                        .less_than_or_equal(end),
                ])
            })
            .obj::<OperationLogDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_by_event_type(
        &self,
        event_type: OperationEventType,
    ) -> Result<Vec<OperationLog>, DomainError> {
        let event_type_value = event_type.as_str().to_string();
        let documents: Vec<OperationLogDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::OPERATION_LOGS)
            .filter(|q| {
                q.for_all([q
                    .field(path!(OperationLogDocument::event_type))
                    .eq(&event_type_value)])
            })
            .obj::<OperationLogDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }
}

fn map_firestore_error(error: firestore::errors::FirestoreError) -> DomainError {
    DomainError::FirestoreMappingError {
        reason: error.to_string(),
    }
}
