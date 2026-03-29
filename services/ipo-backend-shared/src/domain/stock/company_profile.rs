use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{CompanyName, Industry, Market, TickerSymbol};

/// Company profile for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyProfile {
    company_name: CompanyName,
    ticker_symbol: Option<TickerSymbol>,
    market: Market,
    industry: Industry,
}

impl CompanyProfile {
    /// Creates a company profile.
    pub fn new(
        company_name: CompanyName,
        ticker_symbol: Option<TickerSymbol>,
        market: Market,
        industry: Industry,
    ) -> Result<Self, DomainError> {
        if company_name.value().is_empty() {
            return Err(DomainError::InvalidCompanyName {
                reason: "must not be empty".to_string(),
            });
        }
        Ok(Self {
            company_name,
            ticker_symbol,
            market,
            industry,
        })
    }

    /// Returns the company name.
    pub fn company_name(&self) -> &CompanyName {
        &self.company_name
    }

    /// Returns the optional ticker symbol.
    pub fn ticker_symbol(&self) -> Option<&TickerSymbol> {
        self.ticker_symbol.as_ref()
    }

    /// Returns the market.
    pub fn market(&self) -> Market {
        self.market
    }

    /// Returns the industry.
    pub fn industry(&self) -> &Industry {
        &self.industry
    }
}
