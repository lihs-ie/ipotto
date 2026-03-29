use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        account::SecuritiesAccountIdentifier,
        application::{
            ApplicationIdentifier, ApplicationStatus, AppliedOrder, LotteryApplication,
            LotteryOutcome, LotteryResult,
        },
        stock::{Shares, StockIdentifier, Yen},
    },
    errors::DomainError,
};

/// Firestore document model for `LotteryApplication`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LotteryApplicationDocument {
    pub identifier: String,
    pub stock: String,
    pub securities_account: String,
    pub shares: u32,
    pub price: i64,
    pub ordered_at: DateTime<Utc>,
    pub lottery_result: Option<LotteryResult>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub status: ApplicationStatus,
}

impl LotteryApplicationDocument {
    pub fn from_domain(application: &LotteryApplication) -> Self {
        Self {
            identifier: application.identifier().value().to_string(),
            stock: application.stock().value().to_string(),
            securities_account: application.securities_account().value().to_string(),
            shares: application.applied_order().shares().value(),
            price: application.applied_order().price().value(),
            ordered_at: application.applied_order().ordered_at(),
            lottery_result: application.lottery_outcome().map(|value| value.result()),
            confirmed_at: application
                .lottery_outcome()
                .map(|value| value.confirmed_at()),
            status: application.status(),
        }
    }

    pub fn to_domain(&self) -> Result<LotteryApplication, DomainError> {
        LotteryApplication::reconstruct(
            ApplicationIdentifier::new(self.identifier.clone())?,
            StockIdentifier::new(self.stock.clone())?,
            SecuritiesAccountIdentifier::new(self.securities_account.clone())?,
            AppliedOrder::new(
                Shares::new(self.shares)?,
                Yen::new(self.price)?,
                self.ordered_at,
            )?,
            match (self.lottery_result, self.confirmed_at) {
                (Some(result), Some(confirmed_at)) => {
                    Some(LotteryOutcome::new(result, confirmed_at))
                }
                _ => None,
            },
            self.status,
        )
    }
}
