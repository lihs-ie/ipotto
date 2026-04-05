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
        let direct_result = serde_json::from_str::<Vec<ScrapedStock>>(response_body);
        if let Ok(stocks) = direct_result {
            return Ok(stocks);
        }
        let direct_error = direct_result
            .expect_err("direct_result must be Err when Vec<ScrapedStock> parse failed")
            .to_string();

        let wrapped_result = serde_json::from_str::<ScrapedStockResponse>(response_body);
        if let Ok(wrapper) = wrapped_result {
            return Ok(wrapper.stocks);
        }
        let wrapped_error = wrapped_result
            .expect_err("wrapped_result must be Err when wrapper parse failed")
            .to_string();
        Err(DomainError::ScrapingError {
            scraper_source: "external_site".to_string(),
            reason: format!(
                "failed to parse external site response: direct={direct_error}; wrapped={wrapped_error}; response_length={}",
                response_body.len()
            ),
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::ExternalSiteScraperAdapter;
    use crate::acl::scraping::{IpoStockScraperPort, ScrapedStock};

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
    async fn scrapes_direct_json_array_response() {
        let server = MockServer::start().await;
        let expected = vec![build_stock()];

        Mock::given(method("GET"))
            .and(path("/stocks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected))
            .mount(&server)
            .await;

        let adapter = ExternalSiteScraperAdapter::new(
            reqwest::Client::new(),
            format!("{}/stocks", server.uri()),
        );
        let actual = adapter.scrape().await.expect("scrape");

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn scrapes_wrapped_json_response() {
        let server = MockServer::start().await;
        let expected = vec![build_stock()];

        Mock::given(method("GET"))
            .and(path("/wrapped"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "stocks": expected })),
            )
            .mount(&server)
            .await;

        let adapter = ExternalSiteScraperAdapter::new(
            reqwest::Client::new(),
            format!("{}/wrapped", server.uri()),
        );
        let actual = adapter.scrape().await.expect("scrape");

        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].company_name(), "テスト株式会社");
    }
}
