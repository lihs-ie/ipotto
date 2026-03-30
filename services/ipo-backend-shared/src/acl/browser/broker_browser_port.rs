use async_trait::async_trait;

use crate::{
    acl::scraping::ScrapedStock,
    domain::account::{AccountCredential, ConnectionTestResult},
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
}
