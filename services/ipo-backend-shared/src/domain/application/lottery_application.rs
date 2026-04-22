use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        account::SecuritiesAccountIdentifier,
        stock::{Shares, StockIdentifier, Yen},
    },
    errors::DomainError,
};

use super::{
    ApplicationIdentifier, ApplicationStatus, AppliedOrder, LotteryOutcome, LotteryResult,
};

/// Lottery application aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LotteryApplication {
    identifier: ApplicationIdentifier,
    stock: StockIdentifier,
    securities_account: SecuritiesAccountIdentifier,
    applied_order: AppliedOrder,
    lottery_outcome: Option<LotteryOutcome>,
    status: ApplicationStatus,
}

impl LotteryApplication {
    /// Creates a new lottery application.
    pub fn create(
        stock: StockIdentifier,
        securities_account: SecuritiesAccountIdentifier,
        applied_order: AppliedOrder,
    ) -> Result<Self, DomainError> {
        Self::reconstruct(
            ApplicationIdentifier::generate(),
            stock,
            securities_account,
            applied_order,
            None,
            ApplicationStatus::Pending,
        )
    }

    /// Reconstructs a lottery application from persisted state.
    pub fn reconstruct(
        identifier: ApplicationIdentifier,
        stock: StockIdentifier,
        securities_account: SecuritiesAccountIdentifier,
        applied_order: AppliedOrder,
        lottery_outcome: Option<LotteryOutcome>,
        status: ApplicationStatus,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            identifier,
            stock,
            securities_account,
            applied_order,
            lottery_outcome,
            status,
        })
    }

    /// Marks the application as submitted.
    pub fn apply(&mut self) -> Result<(), DomainError> {
        self.transition_status(ApplicationStatus::Applied)
    }

    /// Records the lottery result.
    pub fn record_outcome(
        &mut self,
        result: LotteryResult,
        confirmed_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if self.status != ApplicationStatus::Applied {
            return Err(DomainError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: ApplicationStatus::ResultChecked.as_str().to_string(),
            });
        }
        self.lottery_outcome = Some(LotteryOutcome::new(result, confirmed_at));
        self.status = ApplicationStatus::ResultChecked;
        Ok(())
    }

    /// Returns whether the application is completed.
    pub fn is_completed(&self) -> bool {
        self.status == ApplicationStatus::ResultChecked
    }

    /// Returns the identifier.
    pub fn identifier(&self) -> &ApplicationIdentifier {
        &self.identifier
    }

    /// Returns the stock identifier.
    pub fn stock(&self) -> &StockIdentifier {
        &self.stock
    }

    /// Returns the account identifier.
    pub fn securities_account(&self) -> &SecuritiesAccountIdentifier {
        &self.securities_account
    }

    /// Returns the applied order.
    pub fn applied_order(&self) -> &AppliedOrder {
        &self.applied_order
    }

    /// Returns the optional lottery outcome.
    pub fn lottery_outcome(&self) -> Option<&LotteryOutcome> {
        self.lottery_outcome.as_ref()
    }

    /// Returns the status.
    pub fn status(&self) -> ApplicationStatus {
        self.status
    }

    fn transition_status(&mut self, next: ApplicationStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(next) {
            return Err(DomainError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: next.as_str().to_string(),
            });
        }
        self.status = next;
        Ok(())
    }
}

/// Repository contract for lottery applications.
#[async_trait::async_trait]
pub trait LotteryApplicationRepository: Send + Sync {
    /// Finds an application by identifier.
    async fn find_by_id(
        &self,
        identifier: &ApplicationIdentifier,
    ) -> Result<Option<LotteryApplication>, DomainError>;

    /// Saves an application aggregate.
    async fn save(&self, application: &LotteryApplication) -> Result<(), DomainError>;

    /// Returns applications for a stock.
    async fn find_by_stock(
        &self,
        stock: &StockIdentifier,
    ) -> Result<Vec<LotteryApplication>, DomainError>;

    /// Returns applications by status.
    async fn find_by_status(
        &self,
        status: ApplicationStatus,
    ) -> Result<Vec<LotteryApplication>, DomainError>;

    /// Returns whether an application exists for the stock and account combination.
    async fn exists_by_stock_and_account(
        &self,
        stock: &StockIdentifier,
        account: &SecuritiesAccountIdentifier,
    ) -> Result<bool, DomainError>;
}

impl LotteryApplication {
    /// Creates a pending application with raw order values.
    pub fn create_with_values(
        stock: StockIdentifier,
        securities_account: SecuritiesAccountIdentifier,
        shares: Shares,
        price: Yen,
        ordered_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let order = AppliedOrder::new(shares, price, ordered_at)?;
        Self::create(stock, securities_account, order)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::LotteryApplication;
    use crate::domain::{
        account::SecuritiesAccountIdentifier,
        application::{ApplicationStatus, LotteryResult},
        stock::{Shares, StockIdentifier, Yen},
    };

    #[test]
    fn applies_and_completes() {
        let mut application = LotteryApplication::create_with_values(
            StockIdentifier::generate(),
            SecuritiesAccountIdentifier::generate(),
            Shares::new(100).expect("shares"),
            Yen::new(1_000).expect("price"),
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("application");

        application.apply().expect("apply");
        application
            .record_outcome(
                LotteryResult::Won,
                Utc.with_ymd_and_hms(2026, 4, 15, 10, 0, 0)
                    .single()
                    .expect("timestamp"),
            )
            .expect("record outcome");

        assert!(application.is_completed());
        assert_eq!(application.status(), ApplicationStatus::ResultChecked);
    }

    #[test]
    fn rejects_recording_outcome_before_apply() {
        let mut application = LotteryApplication::create_with_values(
            StockIdentifier::generate(),
            SecuritiesAccountIdentifier::generate(),
            Shares::new(100).expect("shares"),
            Yen::new(1_000).expect("price"),
            Utc.with_ymd_and_hms(2026, 3, 29, 10, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("application");

        let result = application.record_outcome(
            LotteryResult::Lost,
            Utc.with_ymd_and_hms(2026, 4, 15, 10, 0, 0)
                .single()
                .expect("timestamp"),
        );

        assert!(result.is_err());
    }
}
