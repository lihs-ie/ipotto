use async_trait::async_trait;

use crate::{
    acl::{
        browser::BrokerBrowserPort,
        scraping::{IpoStockScraperPort, ScrapedStock},
    },
    errors::DomainError,
};

/// Securities site scraper adapter backed by a broker browser port.
#[derive(Debug)]
pub struct SecuritiesSiteScraperAdapter<B>
where
    B: BrokerBrowserPort,
{
    browser_port: B,
}

impl<B> SecuritiesSiteScraperAdapter<B>
where
    B: BrokerBrowserPort,
{
    /// Creates a securities site scraper.
    pub fn new(browser_port: B) -> Self {
        Self { browser_port }
    }
}

#[async_trait]
impl<B> IpoStockScraperPort for SecuritiesSiteScraperAdapter<B>
where
    B: BrokerBrowserPort,
{
    async fn scrape(&self) -> Result<Vec<ScrapedStock>, DomainError> {
        self.browser_port.fetch_ipo_stocks().await
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use super::SecuritiesSiteScraperAdapter;
    use crate::{
        acl::{
            browser::BrokerBrowserPort,
            scraping::{IpoStockScraperPort, ScrapedStock},
        },
        domain::{
            account::{AccountCredential, ConnectionTestResult},
            application::LotteryResult,
            stock::IpoStock,
        },
        errors::DomainError,
    };

    #[derive(Debug)]
    struct BrowserStub;

    #[async_trait]
    impl BrokerBrowserPort for BrowserStub {
        async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError> {
            Ok(vec![ScrapedStock::new(
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
            )])
        }

        async fn test_connection(
            &self,
            _credential: &AccountCredential,
        ) -> Result<ConnectionTestResult, DomainError> {
            Err(DomainError::HttpClientError {
                reason: "unused".to_string(),
            })
        }

        async fn check_lottery_result(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
        ) -> Result<Option<LotteryResult>, DomainError> {
            Err(DomainError::HttpClientError {
                reason: "unused".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn delegates_scrape_to_browser_port() {
        let adapter = SecuritiesSiteScraperAdapter::new(BrowserStub);
        let stocks = adapter.scrape().await.expect("scrape");

        assert_eq!(stocks.len(), 1);
        assert_eq!(stocks[0].company_name(), "テスト株式会社");
    }
}
