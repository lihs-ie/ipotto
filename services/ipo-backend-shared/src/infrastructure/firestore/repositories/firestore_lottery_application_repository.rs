use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::{
        account::SecuritiesAccountIdentifier,
        application::{
            ApplicationIdentifier, ApplicationStatus, LotteryApplication,
            LotteryApplicationRepository,
        },
        stock::StockIdentifier,
    },
    errors::DomainError,
    infrastructure::firestore::documents::LotteryApplicationDocument,
};

/// Concrete lottery application repository with Firestore-oriented document mapping.
#[derive(Debug, Clone, Default)]
pub struct FirestoreLotteryApplicationRepository {
    documents: Arc<Mutex<BTreeMap<String, LotteryApplicationDocument>>>,
}

impl FirestoreLotteryApplicationRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl LotteryApplicationRepository for FirestoreLotteryApplicationRepository {
    fn find_by_id(
        &self,
        identifier: &ApplicationIdentifier,
    ) -> Result<Option<LotteryApplication>, DomainError> {
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

    fn save(&self, application: &LotteryApplication) -> Result<(), DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .insert(
                application.identifier().value().to_string(),
                LotteryApplicationDocument::from_domain(application),
            );
        Ok(())
    }

    fn find_by_stock(
        &self,
        stock: &StockIdentifier,
    ) -> Result<Vec<LotteryApplication>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .filter(|document| document.stock == stock.value())
            .map(|document| document.to_domain())
            .collect()
    }

    fn find_by_status(
        &self,
        status: ApplicationStatus,
    ) -> Result<Vec<LotteryApplication>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .filter(|document| document.status == status)
            .map(|document| document.to_domain())
            .collect()
    }

    fn exists_by_stock_and_account(
        &self,
        stock: &StockIdentifier,
        account: &SecuritiesAccountIdentifier,
    ) -> Result<bool, DomainError> {
        Ok(self
            .documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .any(|document| {
                document.stock == stock.value() && document.securities_account == account.value()
            }))
    }
}
