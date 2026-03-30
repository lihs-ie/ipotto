use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    domain::stock::{
        BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
        IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market, MetaSource, PriceRange, Shares,
        StockIdentifier, StockStatus, TickerSymbol, Yen,
    },
    errors::DomainError,
};

/// Firestore document model for `IpoStock`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoStockDocument {
    pub identifier: String,
    pub company_name: String,
    pub ticker_symbol: Option<String>,
    pub market: String,
    pub industry: String,
    pub book_building_start_date: NaiveDate,
    pub book_building_end_date: NaiveDate,
    pub lottery_date: NaiveDate,
    pub listing_date: NaiveDate,
    pub minimum_price: i64,
    pub maximum_price: i64,
    pub offer_price: Option<i64>,
    pub lead_underwriter: String,
    pub number_of_offered_shares: u32,
    pub status: StockStatus,
    pub fetch_origin: FetchOrigin,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

impl IpoStockDocument {
    pub fn from_domain(stock: &IpoStock) -> Self {
        Self {
            identifier: stock.identifier().value().to_string(),
            company_name: stock.company_profile().company_name().value().to_string(),
            ticker_symbol: stock
                .company_profile()
                .ticker_symbol()
                .map(|value| value.value().to_string()),
            market: stock.company_profile().market().as_str().to_string(),
            industry: stock.company_profile().industry().value().to_string(),
            book_building_start_date: stock.schedule().book_building_period().start_date(),
            book_building_end_date: stock.schedule().book_building_period().end_date(),
            lottery_date: stock.schedule().lottery_date(),
            listing_date: stock.schedule().listing_date(),
            minimum_price: stock.pricing().price_range().minimum_price().value(),
            maximum_price: stock.pricing().price_range().maximum_price().value(),
            offer_price: stock.pricing().offer_price().map(|value| value.value()),
            lead_underwriter: stock.offering().lead_underwriter().value().to_string(),
            number_of_offered_shares: stock.offering().number_of_offered_shares().value(),
            status: stock.status(),
            fetch_origin: stock.meta_source().source(),
            fetched_at: stock.meta_source().fetched_at(),
        }
    }

    pub fn to_domain(&self) -> Result<IpoStock, DomainError> {
        IpoStock::reconstruct(
            StockIdentifier::new(self.identifier.clone())?,
            CompanyProfile::new(
                CompanyName::new(self.company_name.clone())?,
                self.ticker_symbol
                    .clone()
                    .map(TickerSymbol::new)
                    .transpose()?,
                Market::new(self.market.clone())?,
                Industry::new(self.industry.clone())?,
            )?,
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    self.book_building_start_date,
                    self.book_building_end_date,
                )?,
                self.lottery_date,
                self.listing_date,
            )?,
            IpoPricing::new(
                PriceRange::new(Yen::new(self.minimum_price)?, Yen::new(self.maximum_price)?)?,
                self.offer_price.map(Yen::new).transpose()?,
            )?,
            IpoOffering::new(
                LeadUnderwriter::new(self.lead_underwriter.clone())?,
                Shares::new(self.number_of_offered_shares)?,
            )?,
            self.status,
            MetaSource::new(self.fetch_origin, self.fetched_at),
        )
    }
}
