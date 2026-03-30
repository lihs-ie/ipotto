use async_trait::async_trait;

use crate::{
    acl::scraping::ScrapedStock,
    domain::{
        account::{AccountCredential, ConnectionTestResult},
        application::LotteryResult,
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
}
