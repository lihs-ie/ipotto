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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use super::FallbackScraperAdapter;
    use crate::{
        acl::scraping::{IpoStockScraperPort, ScrapedStock},
        errors::DomainError,
    };

    #[derive(Debug)]
    struct StaticScraper(Result<Vec<ScrapedStock>, DomainError>);

    #[async_trait]
    impl IpoStockScraperPort for StaticScraper {
        async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
            self.0.clone()
        }
    }

    fn build_stock() -> ScrapedStock {
        ScrapedStock::new(
            "テスト株式会社",
            Some("1234".to_string()),
            "Growth",
            "情報・通信業",
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("bb start"),
            NaiveDate::from_ymd_opt(2026, 4, 10).expect("bb end"),
            NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
            NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
            1200,
            1500,
            Some(1400),
            "楽天証券",
            100000,
        )
    }

    #[tokio::test]
    async fn uses_fallback_when_primary_fails() {
        let adapter = FallbackScraperAdapter::new(
            StaticScraper(Err(DomainError::ScrapingError {
                scraper_source: "primary".to_string(),
                reason: "failed".to_string(),
            })),
            StaticScraper(Ok(vec![build_stock()])),
        );

        let result = adapter.scrape().await.expect("fallback result");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].company_name(), "テスト株式会社");
    }
}
