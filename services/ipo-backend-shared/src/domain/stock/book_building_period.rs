use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// Book building period of an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookBuildingPeriod {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl BookBuildingPeriod {
    /// Creates a book building period.
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Result<Self, DomainError> {
        if start_date > end_date {
            return Err(DomainError::InvalidSchedule {
                reason: "book building start date must be on or before end date".to_string(),
            });
        }
        Ok(Self {
            start_date,
            end_date,
        })
    }

    /// Returns true when the date is within the period.
    pub fn contains(&self, date: NaiveDate) -> bool {
        self.start_date <= date && date <= self.end_date
    }

    /// Returns the start date.
    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    /// Returns the end date.
    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }
}
