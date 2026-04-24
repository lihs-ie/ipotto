//! Seed data for `lottery_applications`. Produces 6 applications that
//! mirror the Applied / Won / Lost / Alternate stocks in the stock seed
//! set, so dashboards can render completed outcomes alongside pending
//! submissions.

use chrono::{Duration, Utc};
use ipo_backend_shared::{
    domain::{
        account::SecuritiesAccountIdentifier,
        application::{
            ApplicationIdentifier, ApplicationStatus, AppliedOrder, LotteryApplication,
            LotteryOutcome, LotteryResult,
        },
        stock::{IpoStock, Shares, StockStatus},
    },
    errors::DomainError,
};
use ulid::Ulid;

pub fn build_applications(
    account: &SecuritiesAccountIdentifier,
    stocks: &[IpoStock],
) -> Result<Vec<LotteryApplication>, DomainError> {
    let now = Utc::now();
    let mut applications = Vec::new();
    let mut cursor: u32 = 1;

    for stock in stocks {
        let status = stock.status();
        let (application_status, outcome, ordered_offset, confirmed_offset) = match status {
            StockStatus::Applied => (ApplicationStatus::Applied, None, -6, None),
            StockStatus::Won => (
                ApplicationStatus::ResultChecked,
                Some(LotteryResult::Won),
                -13,
                Some(-3),
            ),
            StockStatus::Lost => (
                ApplicationStatus::ResultChecked,
                Some(LotteryResult::Lost),
                -16,
                Some(-6),
            ),
            StockStatus::Alternate => (
                ApplicationStatus::ResultChecked,
                Some(LotteryResult::Alternate),
                -19,
                Some(-9),
            ),
            StockStatus::Purchased => (
                ApplicationStatus::ResultChecked,
                Some(LotteryResult::Won),
                -26,
                Some(-16),
            ),
            StockStatus::Declined => (
                ApplicationStatus::ResultChecked,
                Some(LotteryResult::Won),
                -36,
                Some(-26),
            ),
            _ => continue,
        };

        let shares = Shares::new(100)?;
        let price = stock
            .pricing()
            .offer_price()
            .unwrap_or_else(|| stock.pricing().price_range().maximum_price());
        let ordered_at = now + Duration::days(ordered_offset);
        let applied_order = AppliedOrder::new(shares, price, ordered_at)?;

        let confirmed_outcome = match (outcome, confirmed_offset) {
            (Some(result), Some(offset)) => {
                Some(LotteryOutcome::new(result, now + Duration::days(offset)))
            }
            _ => None,
        };

        applications.push(LotteryApplication::reconstruct(
            application_identifier(cursor)?,
            stock.identifier().clone(),
            account.clone(),
            applied_order,
            confirmed_outcome,
            application_status,
        )?);
        cursor += 1;
    }

    // Ensure we always have at least one Pending application so the
    // dashboard can illustrate the "submitted but not yet applied"
    // transitional state.
    let first_eligible = stocks
        .iter()
        .find(|stock| matches!(stock.status(), StockStatus::Eligible));
    if let Some(stock) = first_eligible {
        let price = stock
            .pricing()
            .offer_price()
            .unwrap_or_else(|| stock.pricing().price_range().maximum_price());
        let ordered_at = now - Duration::hours(4);
        let applied_order = AppliedOrder::new(Shares::new(100)?, price, ordered_at)?;
        applications.push(LotteryApplication::reconstruct(
            application_identifier(cursor)?,
            stock.identifier().clone(),
            account.clone(),
            applied_order,
            None,
            ApplicationStatus::Pending,
        )?);
    }

    Ok(applications)
}

fn application_identifier(index: u32) -> Result<ApplicationIdentifier, DomainError> {
    let ulid = Ulid::from_parts(1_700_000_040_000, u128::from(index));
    ApplicationIdentifier::new(ulid.to_string())
}
