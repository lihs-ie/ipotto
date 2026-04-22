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
pub struct FirestoreIpoStockRepositoryInMemory {
    documents: Arc<Mutex<BTreeMap<String, IpoStockDocument>>>,
}

impl FirestoreIpoStockRepositoryInMemory {
    /// Creates a repository instance.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl IpoStockRepository for FirestoreIpoStockRepositoryInMemory {
    async fn find_by_id(
        &self,
        identifier: &StockIdentifier,
    ) -> Result<Option<IpoStock>, DomainError> {
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

    async fn save(&self, stock: &IpoStock) -> Result<(), DomainError> {
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

    async fn find_all(&self) -> Result<Vec<IpoStock>, DomainError> {
        self.documents
            .lock()
            .map_err(|error| DomainError::FirestoreMappingError {
                reason: error.to_string(),
            })?
            .values()
            .map(|document| document.to_domain())
            .collect()
    }

    async fn find_by_status(&self, status: StockStatus) -> Result<Vec<IpoStock>, DomainError> {
        self.find_all().await.map(|stocks| {
            stocks
                .into_iter()
                .filter(|stock| stock.status() == status)
                .collect()
        })
    }

    async fn find_in_book_building_period(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<IpoStock>, DomainError> {
        self.find_all().await.map(|stocks| {
            stocks
                .into_iter()
                .filter(|stock| stock.is_in_book_building_period(date))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

    use super::FirestoreIpoStockRepositoryInMemory;
    use crate::domain::stock::{
        BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
        IpoPricing, IpoSchedule, IpoStock, IpoStockRepository, LeadUnderwriter, Market, MetaSource,
        PriceRange, Shares, StockStatus, TickerSymbol, Yen,
    };

    fn build_stock(status: StockStatus) -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト株式会社").expect("company"),
                Some(TickerSymbol::new("1234").expect("ticker")),
                Market::Growth,
                Industry::new("情報・通信業").expect("industry"),
            )
            .expect("profile"),
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    NaiveDate::from_ymd_opt(2026, 4, 1).expect("bb start"),
                    NaiveDate::from_ymd_opt(2026, 4, 10).expect("bb end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
            )
            .expect("schedule"),
            IpoPricing::new(
                PriceRange::new(Yen::new(1200).expect("min"), Yen::new(1500).expect("max"))
                    .expect("range"),
                Some(Yen::new(1400).expect("offer")),
            )
            .expect("pricing"),
            IpoOffering::new(
                LeadUnderwriter::new("楽天証券").expect("underwriter"),
                Shares::new(100000).expect("shares"),
            )
            .expect("offering"),
            status,
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    #[tokio::test]
    async fn saves_and_filters_stocks() {
        let repository = FirestoreIpoStockRepositoryInMemory::new();
        let eligible = build_stock(StockStatus::Eligible);
        let applied = build_stock(StockStatus::Applied);

        repository.save(&eligible).await.expect("save eligible");
        repository.save(&applied).await.expect("save applied");

        let found = repository
            .find_by_id(eligible.identifier())
            .await
            .expect("find by id")
            .expect("stock");
        assert_eq!(found.identifier(), eligible.identifier());
        assert_eq!(
            repository
                .find_by_status(StockStatus::Eligible)
                .await
                .expect("by status")
                .len(),
            1
        );
        assert_eq!(
            repository
                .find_in_book_building_period(
                    NaiveDate::from_ymd_opt(2026, 4, 5).expect("within period")
                )
                .await
                .expect("by period")
                .len(),
            2
        );
    }
}
