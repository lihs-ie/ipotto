use std::sync::Arc;

use async_trait::async_trait;
use firestore::{path, FirestoreDb};

use crate::{
    domain::{
        exclusion::{Exclusion, ExclusionIdentifier, ExclusionRepository},
        stock::CompanyName,
    },
    errors::DomainError,
    infrastructure::firestore::{collections, documents::ExclusionDocument},
};

/// Production Firestore-backed implementation of
/// [`ExclusionRepository`]. Persists aggregates to the `exclusions`
/// collection.
#[derive(Debug, Clone)]
pub struct FirestoreExclusionRepository {
    db: Arc<FirestoreDb>,
}

impl FirestoreExclusionRepository {
    pub fn new(db: Arc<FirestoreDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExclusionRepository for FirestoreExclusionRepository {
    async fn find_by_id(
        &self,
        identifier: &ExclusionIdentifier,
    ) -> Result<Option<Exclusion>, DomainError> {
        let document: Option<ExclusionDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::EXCLUSIONS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        document.map(|document| document.to_domain()).transpose()
    }

    async fn save(&self, exclusion: &Exclusion) -> Result<(), DomainError> {
        let document = ExclusionDocument::from_domain(exclusion);
        self.db
            .fluent()
            .update()
            .in_col(collections::EXCLUSIONS)
            .document_id(exclusion.identifier().value())
            .object(&document)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn delete(&self, identifier: &ExclusionIdentifier) -> Result<(), DomainError> {
        self.db
            .fluent()
            .delete()
            .from(collections::EXCLUSIONS)
            .document_id(identifier.value())
            .execute()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Exclusion>, DomainError> {
        let documents: Vec<ExclusionDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::EXCLUSIONS)
            .obj::<ExclusionDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn exists_by_company_name(
        &self,
        company_name: &CompanyName,
    ) -> Result<bool, DomainError> {
        let value = company_name.value().to_string();
        let documents: Vec<ExclusionDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::EXCLUSIONS)
            .filter(|q| q.for_all([q.field(path!(ExclusionDocument::company_name)).eq(&value)]))
            .obj::<ExclusionDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        Ok(!documents.is_empty())
    }
}

fn map_firestore_error(error: firestore::errors::FirestoreError) -> DomainError {
    DomainError::FirestoreMappingError {
        reason: error.to_string(),
    }
}
