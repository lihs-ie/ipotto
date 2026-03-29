use chrono::NaiveDate;

use crate::domain::{exclusion::Exclusion, stock::IpoStock};

/// Service for determining whether a stock is eligible for application.
pub struct ApplicationEligibilityService;

impl ApplicationEligibilityService {
    /// Returns whether a stock is eligible for application.
    pub fn is_eligible(
        stock: &IpoStock,
        exclusions: &[Exclusion],
        already_applied: bool,
        date: NaiveDate,
    ) -> bool {
        stock.is_in_book_building_period(date)
            && !already_applied
            && !exclusions.iter().any(|exclusion| exclusion.matches(stock))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    use super::ApplicationEligibilityService;
    use crate::domain::{
        exclusion::{Exclusion, ExclusionReason},
        stock::{
            BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry, IpoOffering,
            IpoPricing, IpoSchedule, IpoStock, LeadUnderwriter, Market, MetaSource, PriceRange,
            Shares, StockStatus, Yen,
        },
    };

    fn build_stock(name: &str) -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new(name).expect("name"),
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
    fn returns_true_for_eligible_stock() {
        let stock = build_stock("テスト株式会社");
        let eligible = ApplicationEligibilityService::is_eligible(
            &stock,
            &[],
            false,
            NaiveDate::from_ymd_opt(2026, 4, 5).expect("date"),
        );
        assert!(eligible);
    }

    #[test]
    fn returns_false_for_excluded_stock() {
        let stock = build_stock("テスト株式会社");
        let exclusions = vec![Exclusion::create(
            CompanyName::new("テスト株式会社").expect("name"),
            ExclusionReason::new("rule").expect("reason"),
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("exclusion")];
        let eligible = ApplicationEligibilityService::is_eligible(
            &stock,
            &exclusions,
            false,
            NaiveDate::from_ymd_opt(2026, 4, 5).expect("date"),
        );
        assert!(!eligible);
    }
}
