use serde::{Deserialize, Serialize};

/// Raw row extracted from an HTML source before translation into a domain-neutral DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawScrapedEntry {
    pub company_name: String,
    pub ticker_symbol: Option<String>,
    pub market: String,
    pub industry: String,
    pub book_building_period: String,
    pub lottery_date: String,
    pub listing_date: String,
    pub price_range: String,
    pub offer_price: Option<String>,
    pub lead_underwriter: String,
    pub number_of_offered_shares: String,
}
