use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Raw stock data transferred from scraping adapters into the application layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrapedStock {
    company_name: String,
    ticker_symbol: Option<String>,
    market: String,
    industry: String,
    book_building_start_date: NaiveDate,
    book_building_end_date: NaiveDate,
    lottery_date: NaiveDate,
    listing_date: NaiveDate,
    price_range_min: i64,
    price_range_max: i64,
    offer_price: Option<i64>,
    lead_underwriter: String,
    number_of_offered_shares: u32,
}

impl ScrapedStock {
    /// Creates a scraped stock DTO.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        company_name: impl Into<String>,
        ticker_symbol: Option<String>,
        market: impl Into<String>,
        industry: impl Into<String>,
        book_building_start_date: NaiveDate,
        book_building_end_date: NaiveDate,
        lottery_date: NaiveDate,
        listing_date: NaiveDate,
        price_range_min: i64,
        price_range_max: i64,
        offer_price: Option<i64>,
        lead_underwriter: impl Into<String>,
        number_of_offered_shares: u32,
    ) -> Self {
        Self {
            company_name: company_name.into(),
            ticker_symbol,
            market: market.into(),
            industry: industry.into(),
            book_building_start_date,
            book_building_end_date,
            lottery_date,
            listing_date,
            price_range_min,
            price_range_max,
            offer_price,
            lead_underwriter: lead_underwriter.into(),
            number_of_offered_shares,
        }
    }

    pub fn company_name(&self) -> &str {
        &self.company_name
    }
    pub fn ticker_symbol(&self) -> Option<&str> {
        self.ticker_symbol.as_deref()
    }
    pub fn market(&self) -> &str {
        &self.market
    }
    pub fn industry(&self) -> &str {
        &self.industry
    }
    pub fn book_building_start_date(&self) -> NaiveDate {
        self.book_building_start_date
    }
    pub fn book_building_end_date(&self) -> NaiveDate {
        self.book_building_end_date
    }
    pub fn lottery_date(&self) -> NaiveDate {
        self.lottery_date
    }
    pub fn listing_date(&self) -> NaiveDate {
        self.listing_date
    }
    pub fn price_range_min(&self) -> i64 {
        self.price_range_min
    }
    pub fn price_range_max(&self) -> i64 {
        self.price_range_max
    }
    pub fn offer_price(&self) -> Option<i64> {
        self.offer_price
    }
    pub fn lead_underwriter(&self) -> &str {
        &self.lead_underwriter
    }
    pub fn number_of_offered_shares(&self) -> u32 {
        self.number_of_offered_shares
    }
}
