use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use chrono::NaiveDate;

use crate::{
    domain::stock::{IpoStock, IpoStockRepository, StockIdentifier, StockStatus},
    errors::DomainError,
    infrastructure::firestore::documents::IpoStockDocument,
};

/// Concrete stock repository with Firestore-oriented document mapping.
#[derive(Debug, Clone, Default)]
pub struct FirestoreIpoStockRepository {
    documents: Arc<Mutex<BTreeMap<String, IpoStockDocument>>>,
}

impl FirestoreIpoStockRepository {
    /// Creates a repository instance.
    pub fn new() -> Self {
        Self::default()
    }
}

impl IpoStockRepository for FirestoreIpoStockRepository {
    fn find_by_id(&self, identifier: &StockIdentifier) -> Result<Option<IpoStock>, DomainError> {
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

    fn save(&self, stock: &IpoStock) -> Result<(), DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .insert(
                stock.identifier().value().to_string(),
                IpoStockDocument::from_domain(stock),
            );
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<IpoStock>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .map(|document| document.to_domain())
            .collect()
    }

    fn find_by_status(&self, status: StockStatus) -> Result<Vec<IpoStock>, DomainError> {
        self.find_all().map(|stocks| {
            stocks
                .into_iter()
                .filter(|stock| stock.status() == status)
                .collect()
        })
    }

    fn find_in_book_building_period(&self, date: NaiveDate) -> Result<Vec<IpoStock>, DomainError> {
        self.find_all().map(|stocks| {
            stocks
                .into_iter()
                .filter(|stock| stock.is_in_book_building_period(date))
                .collect()
        })
    }
}
