use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::stock::{BookBuildingPeriod, CompanyName, StockIdentifier};

/// Event emitted when IPO stock information is updated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoInfoUpdated {
    pub identifier: StockIdentifier,
    pub company_name: CompanyName,
    pub book_building_period: BookBuildingPeriod,
    pub lottery_date: NaiveDate,
    pub listing_date: NaiveDate,
    pub updated_at: DateTime<Utc>,
}
