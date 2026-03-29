use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::stock::{CompanyName, IpoStock},
    errors::DomainError,
};

use super::{ExclusionIdentifier, ExclusionReason};

/// Exclusion aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exclusion {
    identifier: ExclusionIdentifier,
    company_name: CompanyName,
    reason: ExclusionReason,
    registered_at: DateTime<Utc>,
}

impl Exclusion {
    /// Creates a new exclusion with a generated identifier.
    pub fn create(
        company_name: CompanyName,
        reason: ExclusionReason,
        registered_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        Self::reconstruct(
            ExclusionIdentifier::generate(),
            company_name,
            reason,
            registered_at,
        )
    }

    /// Reconstructs an exclusion from persisted state.
    pub fn reconstruct(
        identifier: ExclusionIdentifier,
        company_name: CompanyName,
        reason: ExclusionReason,
        registered_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if company_name.value().is_empty() {
            return Err(DomainError::InvalidCompanyName {
                reason: "must not be empty".to_string(),
            });
        }
        Ok(Self {
            identifier,
            company_name,
            reason,
            registered_at,
        })
    }

    /// Returns whether the exclusion matches the stock.
    pub fn matches(&self, stock: &IpoStock) -> bool {
        self.company_name == *stock.company_profile().company_name()
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &ExclusionIdentifier {
        &self.identifier
    }

    /// Returns the company name.
    pub fn company_name(&self) -> &CompanyName {
        &self.company_name
    }

    /// Returns the exclusion reason.
    pub fn reason(&self) -> &ExclusionReason {
        &self.reason
    }

    /// Returns when the exclusion was registered.
    pub fn registered_at(&self) -> DateTime<Utc> {
        self.registered_at
    }
}

/// Repository contract for exclusions.
pub trait ExclusionRepository {
    /// Finds an exclusion by identifier.
    fn find_by_id(
        &self,
        identifier: &ExclusionIdentifier,
    ) -> Result<Option<Exclusion>, DomainError>;

    /// Saves an exclusion aggregate.
    fn save(&self, exclusion: &Exclusion) -> Result<(), DomainError>;

    /// Deletes an exclusion by identifier.
    fn delete(&self, identifier: &ExclusionIdentifier) -> Result<(), DomainError>;

    /// Returns all exclusions.
    fn find_all(&self) -> Result<Vec<Exclusion>, DomainError>;

    /// Returns whether a company name already exists.
    fn exists_by_company_name(&self, company_name: &CompanyName) -> Result<bool, DomainError>;
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    use super::Exclusion;
    use crate::domain::{
        exclusion::ExclusionReason,
        stock::{
            BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
            IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market, MetaSource, PriceRange,
            Shares, StockStatus, Yen,
        },
    };

    fn build_stock(name: &str) -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new(name).expect("company name"),
                None,
                Market::Growth,
                Industry::new("IT").expect("industry"),
            )
            .expect("profile"),
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    NaiveDate::from_ymd_opt(2026, 4, 1).expect("start"),
                    NaiveDate::from_ymd_opt(2026, 4, 10).expect("end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 20).expect("listing"),
            )
            .expect("schedule"),
            IpoPricing::new(
                PriceRange::new(Yen::new(1_000).expect("min"), Yen::new(1_200).expect("max"))
                    .expect("range"),
                None,
            )
            .expect("pricing"),
            IpoOffering::new(
                LeadUnderwriter::new("楽天証券").expect("underwriter"),
                Shares::new(100).expect("shares"),
            )
            .expect("offering"),
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
    fn matches_exact_company_name() {
        let exclusion = Exclusion::create(
            CompanyName::new("テスト株式会社").expect("name"),
            ExclusionReason::new("rule").expect("reason"),
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("exclusion");
        let stock = build_stock("テスト株式会社");

        assert!(exclusion.matches(&stock));
    }

    #[test]
    fn does_not_match_partial_company_name() {
        let exclusion = Exclusion::create(
            CompanyName::new("テスト").expect("name"),
            ExclusionReason::new("rule").expect("reason"),
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("exclusion");
        let stock = build_stock("テスト株式会社");

        assert!(!exclusion.matches(&stock));
    }
}
