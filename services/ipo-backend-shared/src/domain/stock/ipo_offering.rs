use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

use super::{LeadUnderwriter, Shares};

/// Offering information for an IPO stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpoOffering {
    lead_underwriter: LeadUnderwriter,
    number_of_offered_shares: Shares,
}

impl IpoOffering {
    /// Creates offering information.
    pub fn new(
        lead_underwriter: LeadUnderwriter,
        number_of_offered_shares: Shares,
    ) -> Result<Self, DomainError> {
        if lead_underwriter.value().is_empty() {
            return Err(DomainError::InvalidCompanyName {
                reason: "lead underwriter must not be empty".to_string(),
            });
        }
        Ok(Self {
            lead_underwriter,
            number_of_offered_shares,
        })
    }

    /// Returns the lead underwriter.
    pub fn lead_underwriter(&self) -> &LeadUnderwriter {
        &self.lead_underwriter
    }

    /// Returns the number of offered shares.
    pub fn number_of_offered_shares(&self) -> Shares {
        self.number_of_offered_shares
    }
}
