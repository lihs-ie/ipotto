use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

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

    fn parse_response_body(&self, response_body: &str) -> Result<Vec<ScrapedStock>, DomainError> {
        if let Ok(stocks) = serde_json::from_str::<Vec<ScrapedStock>>(response_body) {
            return Ok(stocks);
        }
        if let Ok(wrapper) = serde_json::from_str::<ScrapedStockResponse>(response_body) {
            return Ok(wrapper.stocks);
        }
        Err(DomainError::ScrapingError {
            scraper_source: "external_site".to_string(),
            reason: "failed to parse external site response".to_string(),
        })
    }
}

#[async_trait]
impl IpoStockScraperPort for ExternalSiteScraperAdapter {
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        let response_body = self
            .client
            .get(&self.base_url)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|error| DomainError::ScrapingError {
                scraper_source: "external_site".to_string(),
                reason: error.to_string(),
            })?
            .text()
            .await
            .map_err(|error| DomainError::ScrapingError {
                scraper_source: "external_site".to_string(),
                reason: error.to_string(),
            })?;
        if response_body.trim().is_empty() {
            return Err(DomainError::ScrapingError {
                scraper_source: "external_site".to_string(),
                reason: "empty response body".to_string(),
            });
        }
        self.parse_response_body(&response_body)
    }
}

#[derive(Debug, Deserialize)]
struct ScrapedStockResponse {
    stocks: Vec<ScrapedStock>,
}
