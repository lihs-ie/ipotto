use async_trait::async_trait;

use crate::{acl::scraping::ScrapedStock, errors::DomainError};

/// Scraper port for IPO stock acquisition.
#[async_trait]
pub trait IpoStockScraperPort: Send + Sync {
    /// Scrapes IPO stock rows and returns translated DTOs.
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError>;
}
