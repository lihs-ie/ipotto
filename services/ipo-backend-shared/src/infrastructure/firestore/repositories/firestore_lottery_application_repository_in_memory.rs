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
pub struct FirestoreLotteryApplicationRepositoryInMemory {
    documents: Arc<Mutex<BTreeMap<String, LotteryApplicationDocument>>>,
}

impl FirestoreLotteryApplicationRepositoryInMemory {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl LotteryApplicationRepository for FirestoreLotteryApplicationRepositoryInMemory {
    async fn find_by_id(
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

    async fn save(&self, application: &LotteryApplication) -> Result<(), DomainError> {
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

    async fn find_by_stock(
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

    async fn find_by_status(
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

    async fn exists_by_stock_and_account(
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::FirestoreLotteryApplicationRepositoryInMemory;
    use crate::domain::stock::{Shares, Yen};
    use crate::domain::{
        account::SecuritiesAccountIdentifier,
        application::{ApplicationStatus, LotteryApplication, LotteryApplicationRepository},
        stock::StockIdentifier,
    };

    fn build_application(
        stock: StockIdentifier,
        account: SecuritiesAccountIdentifier,
        applied: bool,
    ) -> LotteryApplication {
        let mut application = LotteryApplication::create_with_values(
            stock,
            account,
            Shares::new(100).expect("shares"),
            Yen::new(1400).expect("price"),
            Utc.with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("ordered at"),
        )
        .expect("application");
        if applied {
            application.apply().expect("apply");
        }
        application
    }

    #[tokio::test]
    async fn filters_lottery_applications() {
        let repository = FirestoreLotteryApplicationRepositoryInMemory::new();
        let stock = StockIdentifier::generate();
        let account = SecuritiesAccountIdentifier::generate();
        let pending = build_application(stock.clone(), account.clone(), false);
        let applied =
            build_application(stock.clone(), SecuritiesAccountIdentifier::generate(), true);

        repository.save(&pending).await.expect("save pending");
        repository.save(&applied).await.expect("save applied");

        assert_eq!(
            repository
                .find_by_stock(&stock)
                .await
                .expect("by stock")
                .len(),
            2
        );
        assert_eq!(
            repository
                .find_by_status(ApplicationStatus::Applied)
                .await
                .expect("by status")
                .len(),
            1
        );
        assert!(repository
            .exists_by_stock_and_account(&stock, &account)
            .await
            .expect("exists"));
    }
}
