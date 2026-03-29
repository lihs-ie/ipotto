use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::{
        exclusion::{Exclusion, ExclusionIdentifier, ExclusionRepository},
        stock::CompanyName,
    },
    errors::DomainError,
    infrastructure::firestore::documents::ExclusionDocument,
};

/// Concrete exclusion repository with Firestore-oriented document mapping.
#[derive(Debug, Clone, Default)]
pub struct FirestoreExclusionRepository {
    documents: Arc<Mutex<BTreeMap<String, ExclusionDocument>>>,
}

impl FirestoreExclusionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExclusionRepository for FirestoreExclusionRepository {
    fn find_by_id(
        &self,
        identifier: &ExclusionIdentifier,
    ) -> Result<Option<Exclusion>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .get(identifier.value())
            .cloned()
            .map(|document| document.to_domain())
            .transpose()
    }

    fn save(&self, exclusion: &Exclusion) -> Result<(), DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .insert(
                exclusion.identifier().value().to_string(),
                ExclusionDocument::from_domain(exclusion),
            );
        Ok(())
    }

    fn delete(&self, identifier: &ExclusionIdentifier) -> Result<(), DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .remove(identifier.value());
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Exclusion>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .map(|document| document.to_domain())
            .collect()
    }

    fn exists_by_company_name(&self, company_name: &CompanyName) -> Result<bool, DomainError> {
        self.find_all().map(|items| {
            items
                .into_iter()
                .any(|item| item.company_name() == company_name)
        })
    }
}
