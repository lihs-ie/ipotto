use async_trait::async_trait;

use crate::{
    acl::scraping::{IpoStockScraperPort, ScrapedStock},
    errors::DomainError,
};

/// Scraper adapter that retries with a fallback source.
#[derive(Debug)]
pub struct FallbackScraperAdapter<P, F>
where
    P: IpoStockScraperPort,
    F: IpoStockScraperPort,
{
    primary: P,
    fallback: F,
}

impl<P, F> FallbackScraperAdapter<P, F>
where
    P: IpoStockScraperPort,
    F: IpoStockScraperPort,
{
    /// Creates a fallback scraper adapter.
    pub fn new(primary: P, fallback: F) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl<P, F> IpoStockScraperPort for FallbackScraperAdapter<P, F>
where
    P: IpoStockScraperPort,
    F: IpoStockScraperPort,
{
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        match self.primary.scrape().await {
            Ok(stocks) => Ok(stocks),
            Err(_) => self.fallback.scrape().await,
        }
    }
}
