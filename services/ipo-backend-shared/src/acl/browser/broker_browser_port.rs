use async_trait::async_trait;

use crate::{
    acl::{browser::ApplicationResult, scraping::ScrapedStock},
    domain::{
        account::{AccountCredential, ConnectionTestResult},
        application::{AppliedOrder, LotteryResult},
        stock::IpoStock,
    },
    errors::DomainError,
};

/// Browser integration port for broker-specific pages.
#[async_trait]
pub trait BrokerBrowserPort: Send + Sync {
    /// Fetches IPO stocks visible from the securities site.
    async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError>;

    /// Tests browser login and connection for the given credential.
    async fn test_connection(
        &self,
        credential: &AccountCredential,
    ) -> Result<ConnectionTestResult, DomainError>;

    /// Checks the lottery result for the given stock.
    async fn check_lottery_result(
        &self,
        credential: &AccountCredential,
        stock: &IpoStock,
    ) -> Result<Option<LotteryResult>, DomainError>;

    /// Submits an IPO lottery application for the given stock and applied
    /// order, returning the broker site's decision translated into
    /// [`ApplicationResult`].
    ///
    /// The default implementation errors so that adapters that never need to
    /// apply (e.g. the ipo-info-fetcher stock scraper and ipo-result-checker
    /// result client) can remain untouched. Production adapters such as
    /// `ipo-api`'s `BrowserServiceClient` must override this.
    async fn apply_for_ipo(
        &self,
        _credential: &AccountCredential,
        _stock: &IpoStock,
        _applied_order: &AppliedOrder,
    ) -> Result<ApplicationResult, DomainError> {
        Err(DomainError::HttpClientError {
            reason: "apply_for_ipo is not supported by this adapter".to_string(),
        })
    }
}
