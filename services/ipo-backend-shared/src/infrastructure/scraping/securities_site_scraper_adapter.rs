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
