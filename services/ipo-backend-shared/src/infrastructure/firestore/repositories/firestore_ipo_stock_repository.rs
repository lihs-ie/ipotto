use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use firestore::{path, FirestoreDb};

use crate::{
    domain::stock::{IpoStock, IpoStockRepository, StockIdentifier, StockStatus},
    errors::DomainError,
    infrastructure::firestore::{collections, documents::IpoStockDocument},
};

/// Production Firestore-backed implementation of [`IpoStockRepository`].
/// Persists stocks to the `ipo_stocks` collection via the `firestore`
/// crate's fluent API. Honours the `FIRESTORE_EMULATOR_HOST` environment
/// variable through [`crate::infrastructure::firestore::build_firestore_client`].
#[derive(Debug, Clone)]
pub struct FirestoreIpoStockRepository {
    db: Arc<FirestoreDb>,
}

impl FirestoreIpoStockRepository {
    pub fn new(db: Arc<FirestoreDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IpoStockRepository for FirestoreIpoStockRepository {
    async fn find_by_id(
        &self,
        identifier: &StockIdentifier,
    ) -> Result<Option<IpoStock>, DomainError> {
        let document: Option<IpoStockDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::IPO_STOCKS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        document.map(|document| document.to_domain()).transpose()
    }

    async fn save(&self, stock: &IpoStock) -> Result<(), DomainError> {
        let document = IpoStockDocument::from_domain(stock);
        self.db
            .fluent()
            .update()
            .in_col(collections::IPO_STOCKS)
            .document_id(stock.identifier().value())
            .object(&document)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<IpoStock>, DomainError> {
        let documents: Vec<IpoStockDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::IPO_STOCKS)
            .obj::<IpoStockDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_by_status(&self, status: StockStatus) -> Result<Vec<IpoStock>, DomainError> {
        let status_value = status.as_str().to_string();
        let documents: Vec<IpoStockDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::IPO_STOCKS)
            .filter(|q| q.for_all([q.field(path!(IpoStockDocument::status)).eq(&status_value)]))
            .obj::<IpoStockDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_in_book_building_period(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<IpoStock>, DomainError> {
        // Firestore does not support range-filters on two different
        // fields simultaneously, so we narrow by end_date >= date on the
        // server and filter out start_date > date in-process. In practice
        // the number of active IPOs is tiny (≦ 10) so the client-side
        // filter is negligible.
        let documents: Vec<IpoStockDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::IPO_STOCKS)
            .filter(|q| {
                q.for_all([q
                    .field(path!(IpoStockDocument::book_building_end_date))
                    .greater_than_or_equal(date)])
            })
            .obj::<IpoStockDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .filter(|document| document.book_building_start_date <= date)
            .map(|document| document.to_domain())
            .collect()
    }
}

fn map_firestore_error(error: firestore::errors::FirestoreError) -> DomainError {
    DomainError::FirestoreMappingError {
        reason: error.to_string(),
    }
}
