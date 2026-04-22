use std::sync::Arc;

use ipo_backend_shared::{
    acl::{messaging::EventPublisherPort, scraping::IpoStockScraperPort},
    domain::{operation_log::OperationLogRepository, stock::IpoStockRepository},
    errors::DomainError,
    infrastructure::{
        firestore::{
            build_firestore_client,
            repositories::{FirestoreIpoStockRepository, FirestoreOperationLogRepository},
        },
        http_client::{HttpClientConfig, ReqwestClientFactory},
        messaging::PubSubEventPublisher,
        scraping::{
            ExternalSiteScraperAdapter, FallbackScraperAdapter, SecuritiesSiteScraperAdapter,
        },
    },
};

use crate::{config, infrastructure::BrowserServiceClient};

#[derive(Clone)]
pub struct DependencyContainer {
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    scraper: Arc<dyn IpoStockScraperPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl DependencyContainer {
    pub async fn new() -> Result<Self, DomainError> {
        let client = ReqwestClientFactory::new(HttpClientConfig::default()).build()?;
        let browser_client =
            BrowserServiceClient::new(client.clone(), config::ipo_browser_base_url());
        let scraper = FallbackScraperAdapter::new(
            ExternalSiteScraperAdapter::new(client.clone(), config::external_scraper_base_url()),
            SecuritiesSiteScraperAdapter::new(browser_client),
        );
        let db = Arc::new(build_firestore_client(&config::firebase_project_id()).await?);

        Ok(Self {
            stock_repository: Arc::new(FirestoreIpoStockRepository::new(db.clone())),
            scraper: Arc::new(scraper),
            event_publisher: Arc::new(PubSubEventPublisher::new("ipo-info-fetcher")),
            operation_log_repository: Arc::new(FirestoreOperationLogRepository::new(db)),
        })
    }

    #[cfg(test)]
    pub fn from_components(
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        scraper: Arc<dyn IpoStockScraperPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
        operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
    ) -> Self {
        Self {
            stock_repository,
            scraper,
            event_publisher,
            operation_log_repository,
        }
    }

    pub fn stock_repository(&self) -> Arc<dyn IpoStockRepository + Send + Sync> {
        self.stock_repository.clone()
    }

    pub fn scraper(&self) -> Arc<dyn IpoStockScraperPort> {
        self.scraper.clone()
    }

    pub fn event_publisher(&self) -> Arc<dyn EventPublisherPort> {
        self.event_publisher.clone()
    }

    pub fn operation_log_repository(&self) -> Arc<dyn OperationLogRepository + Send + Sync> {
        self.operation_log_repository.clone()
    }
}
