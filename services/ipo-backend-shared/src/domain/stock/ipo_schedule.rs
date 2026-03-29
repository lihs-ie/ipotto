use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::BookBuildingPeriod;

/// Schedule information for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoSchedule {
    book_building_period: BookBuildingPeriod,
    lottery_date: NaiveDate,
    listing_date: NaiveDate,
}

impl IpoSchedule {
    /// Creates an IPO schedule.
    pub fn new(
        book_building_period: BookBuildingPeriod,
        lottery_date: NaiveDate,
        listing_date: NaiveDate,
    ) -> Result<Self, DomainError> {
        if book_building_period.end_date() > lottery_date || lottery_date > listing_date {
            return Err(DomainError::InvalidSchedule {
                reason: "book building end date must be on or before lottery date and listing date"
                    .to_string(),
            });
        }
        Ok(Self {
            book_building_period,
            lottery_date,
            listing_date,
        })
    }

    /// Returns the book building period.
    pub fn book_building_period(&self) -> &BookBuildingPeriod {
        &self.book_building_period
    }

    /// Returns the lottery date.
    pub fn lottery_date(&self) -> NaiveDate {
        self.lottery_date
    }

    /// Returns the listing date.
    pub fn listing_date(&self) -> NaiveDate {
        self.listing_date
    }
}
