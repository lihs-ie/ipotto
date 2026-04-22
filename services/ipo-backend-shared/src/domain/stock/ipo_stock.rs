use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{
    CompanyProfile, IpoOffering, IpoPricing, IpoSchedule, MetaSource, StockIdentifier, StockStatus,
};

/// IPO stock aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoStock {
    identifier: StockIdentifier,
    company_profile: CompanyProfile,
    schedule: IpoSchedule,
    pricing: IpoPricing,
    offering: IpoOffering,
    status: StockStatus,
    meta_source: MetaSource,
}

/// Update payload for stock data fetched from upstream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoStockUpdate {
    company_profile: CompanyProfile,
    schedule: IpoSchedule,
    pricing: IpoPricing,
    offering: IpoOffering,
    status: StockStatus,
    meta_source: MetaSource,
}

impl IpoStockUpdate {
    /// Creates a stock update payload.
    pub fn new(
        company_profile: CompanyProfile,
        schedule: IpoSchedule,
        pricing: IpoPricing,
        offering: IpoOffering,
        status: StockStatus,
        meta_source: MetaSource,
    ) -> Self {
        Self {
            company_profile,
            schedule,
            pricing,
            offering,
            status,
            meta_source,
        }
    }
}

impl IpoStock {
    /// Creates a new stock aggregate with a generated identifier.
    pub fn create(
        company_profile: CompanyProfile,
        schedule: IpoSchedule,
        pricing: IpoPricing,
        offering: IpoOffering,
        status: StockStatus,
        meta_source: MetaSource,
    ) -> Result<Self, DomainError> {
        Self::reconstruct(
            StockIdentifier::generate(),
            company_profile,
            schedule,
            pricing,
            offering,
            status,
            meta_source,
        )
    }

    /// Reconstructs a stock aggregate from persisted state.
    pub fn reconstruct(
        identifier: StockIdentifier,
        company_profile: CompanyProfile,
        schedule: IpoSchedule,
        pricing: IpoPricing,
        offering: IpoOffering,
        status: StockStatus,
        meta_source: MetaSource,
    ) -> Result<Self, DomainError> {
        if company_profile.company_name().value().is_empty() {
            return Err(DomainError::InvalidCompanyName {
                reason: "must not be empty".to_string(),
            });
        }
        Ok(Self {
            identifier,
            company_profile,
            schedule,
            pricing,
            offering,
            status,
            meta_source,
        })
    }

    /// Applies fetched source data to the aggregate.
    pub fn update_from_source(&mut self, update: IpoStockUpdate) -> Result<(), DomainError> {
        self.company_profile = update.company_profile;
        self.schedule = update.schedule;
        self.pricing = update.pricing;
        self.offering = update.offering;
        self.status = update.status;
        self.meta_source = update.meta_source;
        Ok(())
    }

    /// Transitions the stock status.
    pub fn transition_status(&mut self, new_status: StockStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(new_status) {
            return Err(DomainError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: new_status.as_str().to_string(),
            });
        }
        self.status = new_status;
        Ok(())
    }

    /// Returns whether the stock is in the book building period on the specified date.
    pub fn is_in_book_building_period(&self, date: NaiveDate) -> bool {
        self.schedule.book_building_period().contains(date)
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &StockIdentifier {
        &self.identifier
    }

    /// Returns the company profile.
    pub fn company_profile(&self) -> &CompanyProfile {
        &self.company_profile
    }

    /// Returns the schedule.
    pub fn schedule(&self) -> &IpoSchedule {
        &self.schedule
    }

    /// Returns the pricing information.
    pub fn pricing(&self) -> &IpoPricing {
        &self.pricing
    }

    /// Returns the offering information.
    pub fn offering(&self) -> &IpoOffering {
        &self.offering
    }

    /// Returns the current status.
    pub fn status(&self) -> StockStatus {
        self.status
    }

    /// Returns the source metadata.
    pub fn meta_source(&self) -> &MetaSource {
        &self.meta_source
    }
}

/// Repository contract for the stock aggregate.
#[async_trait::async_trait]
pub trait IpoStockRepository: Send + Sync {
    /// Finds a stock by identifier.
    async fn find_by_id(
        &self,
        identifier: &StockIdentifier,
    ) -> Result<Option<IpoStock>, DomainError>;

    /// Saves a stock aggregate.
    async fn save(&self, stock: &IpoStock) -> Result<(), DomainError>;

    /// Returns all stocks.
    async fn find_all(&self) -> Result<Vec<IpoStock>, DomainError>;

    /// Returns stocks by status.
    async fn find_by_status(&self, status: StockStatus) -> Result<Vec<IpoStock>, DomainError>;

    /// Returns stocks in the book building period on the given date.
    async fn find_in_book_building_period(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<IpoStock>, DomainError>;
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    use super::IpoStock;
    use crate::domain::stock::{
        BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
        IpoPricing, IpoSchedule, LeadUnderwriter, Market, MetaSource, PriceRange, Shares,
        StockStatus, TickerSymbol, Yen,
    };

    fn build_stock() -> IpoStock {
        let profile = CompanyProfile::new(
            CompanyName::new("テスト株式会社").expect("company name"),
            Some(TickerSymbol::new("1234").expect("ticker")),
            Market::Growth,
            Industry::new("IT").expect("industry"),
        )
        .expect("profile");
        let period = BookBuildingPeriod::new(
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("start date"),
            NaiveDate::from_ymd_opt(2026, 4, 10).expect("end date"),
        )
        .expect("period");
        let schedule = IpoSchedule::new(
            period,
            NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery date"),
            NaiveDate::from_ymd_opt(2026, 4, 20).expect("listing date"),
        )
        .expect("schedule");
        let pricing = IpoPricing::new(
            PriceRange::new(
                Yen::new(1_000).expect("min price"),
                Yen::new(1_200).expect("max price"),
            )
            .expect("range"),
            Some(Yen::new(1_100).expect("offer price")),
        )
        .expect("pricing");
        let offering = IpoOffering::new(
            LeadUnderwriter::new("楽天証券").expect("underwriter"),
            Shares::new(100).expect("shares"),
        )
        .expect("offering");
        IpoStock::create(
            profile,
            schedule,
            pricing,
            offering,
            StockStatus::Fetched,
            MetaSource::new(
                FetchOrigin::ExternalSite,
                Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                    .single()
                    .expect("timestamp"),
            ),
        )
        .expect("stock")
    }

    #[test]
    fn transitions_to_eligible() {
        let mut stock = build_stock();
        let result = stock.transition_status(StockStatus::Eligible);
        assert!(result.is_ok());
        assert_eq!(stock.status(), StockStatus::Eligible);
    }

    #[test]
    fn rejects_invalid_transition() {
        let mut stock = build_stock();
        stock
            .transition_status(StockStatus::Eligible)
            .expect("eligible");
        stock
            .transition_status(StockStatus::Applied)
            .expect("applied");
        stock.transition_status(StockStatus::Lost).expect("lost");
        let result = stock.transition_status(StockStatus::Applied);
        assert!(result.is_err());
    }

    #[test]
    fn matches_book_building_period() {
        let stock = build_stock();
        assert!(stock
            .is_in_book_building_period(NaiveDate::from_ymd_opt(2026, 4, 5).expect("target date")));
    }
}
