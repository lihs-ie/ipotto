use async_trait::async_trait;
use tracing::warn;

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
            Err(primary_error) => {
                warn!(
                    error = ?primary_error,
                    "primary scraper failed; attempting fallback"
                );
                self.fallback.scrape().await.map_err(|fallback_error| {
                    DomainError::ScrapingError {
                        scraper_source: "fallback".to_string(),
                        reason: format!(
                            "primary scrape failed: {primary_error}; fallback scrape failed: {fallback_error}"
                        ),
                    }
                })
            }
        }
    }
}
