use async_trait::async_trait;
use ipo_backend_shared::{
    acl::{browser::BrokerBrowserPort, scraping::ScrapedStock},
    domain::{
        account::{AccountCredential, ConnectionTestResult},
        application::LotteryResult,
        stock::IpoStock,
    },
    errors::DomainError,
};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct BrowserServiceClient {
    client: Client,
    base_url: String,
}

impl BrowserServiceClient {
    pub fn new(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl BrokerBrowserPort for BrowserServiceClient {
    async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        self.client
            .get(format!("{}/internal/stocks", self.base_url))
            .send()
            .await
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })?
            .json::<Vec<ScrapedStock>>()
            .await
            .map_err(|error| DomainError::HttpClientError {
                reason: error.to_string(),
            })
    }

    async fn test_connection(
        &self,
        _credential: &AccountCredential,
    ) -> Result<ConnectionTestResult, DomainError> {
        Err(DomainError::HttpClientError {
            reason: "test_connection is not used by ipo-info-fetcher".to_string(),
        })
    }

    async fn check_lottery_result(
        &self,
        _credential: &AccountCredential,
        _stock: &IpoStock,
    ) -> Result<Option<LotteryResult>, DomainError> {
        Err(DomainError::HttpClientError {
            reason: "check_lottery_result is not used by ipo-info-fetcher".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use ipo_backend_shared::acl::scraping::ScrapedStock;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::BrowserServiceClient;

    #[tokio::test]
    async fn fetches_scraped_stocks_from_browser_service() {
        let server = MockServer::start().await;
        let response = vec![ScrapedStock::new(
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
        )];

        Mock::given(method("GET"))
            .and(path("/internal/stocks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = BrowserServiceClient::new(reqwest::Client::new(), server.uri());
        let stocks = ipo_backend_shared::acl::browser::BrokerBrowserPort::fetch_ipo_stocks(&client)
            .await
            .expect("fetch stocks");

        assert_eq!(stocks, response);
    }
}
