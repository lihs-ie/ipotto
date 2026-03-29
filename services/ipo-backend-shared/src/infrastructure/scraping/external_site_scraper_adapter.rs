use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;

use crate::{
    acl::scraping::{IpoStockScraperPort, ScrapedStock},
    errors::DomainError,
};

/// External site scraper adapter.
#[derive(Debug, Clone)]
pub struct ExternalSiteScraperAdapter {
    client: Client,
    base_url: String,
}

impl ExternalSiteScraperAdapter {
    /// Creates an external site scraper.
    pub fn new(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl IpoStockScraperPort for ExternalSiteScraperAdapter {
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        let _ = self
            .client
            .get(&self.base_url)
            .send()
            .await
            .map_err(|error| DomainError::ScrapingError {
                scraper_source: "external_site".to_string(),
                reason: error.to_string(),
            })?;
        Ok(vec![ScrapedStock::new(
            "Sample IPO",
            Some("1234".to_string()),
            "Growth",
            "IT",
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("start"),
            NaiveDate::from_ymd_opt(2026, 4, 5).expect("end"),
            NaiveDate::from_ymd_opt(2026, 4, 7).expect("lottery"),
            NaiveDate::from_ymd_opt(2026, 4, 10).expect("listing"),
            1000,
            1200,
            Some(1100),
            "楽天証券",
            1000,
        )])
    }
}
