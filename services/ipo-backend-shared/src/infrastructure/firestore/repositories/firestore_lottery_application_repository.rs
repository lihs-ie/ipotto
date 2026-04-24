use std::sync::Arc;

use async_trait::async_trait;
use firestore::{path, FirestoreDb};

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
    infrastructure::firestore::{collections, documents::LotteryApplicationDocument},
};

/// Production Firestore-backed implementation of
/// [`LotteryApplicationRepository`]. Persists aggregates to the
/// `lottery_applications` collection.
#[derive(Debug, Clone)]
pub struct FirestoreLotteryApplicationRepository {
    db: Arc<FirestoreDb>,
}

impl FirestoreLotteryApplicationRepository {
    pub fn new(db: Arc<FirestoreDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl LotteryApplicationRepository for FirestoreLotteryApplicationRepository {
    async fn find_by_id(
        &self,
        identifier: &ApplicationIdentifier,
    ) -> Result<Option<LotteryApplication>, DomainError> {
        let document: Option<LotteryApplicationDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(collections::LOTTERY_APPLICATIONS)
            .obj()
            .one(identifier.value().to_string())
            .await
            .map_err(map_firestore_error)?;
        document.map(|document| document.to_domain()).transpose()
    }

    async fn save(&self, application: &LotteryApplication) -> Result<(), DomainError> {
        let document = LotteryApplicationDocument::from_domain(application);
        self.db
            .fluent()
            .update()
            .in_col(collections::LOTTERY_APPLICATIONS)
            .document_id(application.identifier().value())
            .object(&document)
            .execute::<()>()
            .await
            .map_err(map_firestore_error)?;
        Ok(())
    }

    async fn find_by_stock(
        &self,
        stock: &StockIdentifier,
    ) -> Result<Vec<LotteryApplication>, DomainError> {
        let stock_value = stock.value().to_string();
        let documents: Vec<LotteryApplicationDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::LOTTERY_APPLICATIONS)
            .filter(|q| {
                q.for_all([q
                    .field(path!(LotteryApplicationDocument::stock))
                    .eq(&stock_value)])
            })
            .obj::<LotteryApplicationDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_by_status(
        &self,
        status: ApplicationStatus,
    ) -> Result<Vec<LotteryApplication>, DomainError> {
        let status_value = status.as_str().to_string();
        let documents: Vec<LotteryApplicationDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::LOTTERY_APPLICATIONS)
            .filter(|q| {
                q.for_all([q
                    .field(path!(LotteryApplicationDocument::status))
                    .eq(&status_value)])
            })
            .obj::<LotteryApplicationDocument>()
            .query()
            .await
            .map_err(map_firestore_error)?;
        documents
            .into_iter()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn exists_by_stock_and_account(
        &self,
        stock: &StockIdentifier,
        account: &SecuritiesAccountIdentifier,
    ) -> Result<bool, DomainError> {
        let stock_value = stock.value().to_string();
        let account_value = account.value().to_string();
        let documents: Vec<LotteryApplicationDocument> = self
            .db
            .fluent()
            .select()
            .from(collections::LOTTERY_APPLICATIONS)
            .filter(|q| {
                q.for_all([
                    q.field(path!(LotteryApplicationDocument::stock))
                        .eq(&stock_value),
                    q.field(path!(LotteryApplicationDocument::securities_account))
                        .eq(&account_value),
                ])
            })
            .obj::<LotteryApplicationDocument>()
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
