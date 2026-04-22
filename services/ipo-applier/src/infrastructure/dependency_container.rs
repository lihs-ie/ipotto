use std::sync::Arc;

use ipo_backend_shared::{
    acl::{browser::BrokerBrowserPort, messaging::EventPublisherPort},
    domain::{
        account::SecuritiesAccountRepository, application::LotteryApplicationRepository,
        exclusion::ExclusionRepository, operation_log::OperationLogRepository,
        stock::IpoStockRepository,
    },
    errors::DomainError,
    infrastructure::{
        firestore::repositories::{
            FirestoreExclusionRepository, FirestoreIpoStockRepository,
            FirestoreLotteryApplicationRepository, FirestoreOperationLogRepository,
            FirestoreSecuritiesAccountRepository,
        },
        http_client::{HttpClientConfig, ReqwestClientFactory},
        messaging::PubSubEventPublisher,
        secrets::InMemoryCredentialStore,
    },
};

use crate::{config, infrastructure::BrowserServiceClient};

#[derive(Clone)]
pub struct DependencyContainer {
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    exclusion_repository: Arc<dyn ExclusionRepository + Send + Sync>,
    browser_port: Arc<dyn BrokerBrowserPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl DependencyContainer {
    pub fn new() -> Result<Self, DomainError> {
        let client = ReqwestClientFactory::new(HttpClientConfig::default()).build()?;
        let credential_store = InMemoryCredentialStore::new();
        Ok(Self {
            application_repository: Arc::new(FirestoreLotteryApplicationRepository::new()),
            stock_repository: Arc::new(FirestoreIpoStockRepository::new()),
            account_repository: Arc::new(FirestoreSecuritiesAccountRepository::new(
                credential_store,
            )),
            exclusion_repository: Arc::new(FirestoreExclusionRepository::new()),
            browser_port: Arc::new(BrowserServiceClient::new(
                client,
                config::ipo_browser_base_url(),
            )),
            event_publisher: Arc::new(PubSubEventPublisher::new("ipo-applier")),
            operation_log_repository: Arc::new(FirestoreOperationLogRepository::new()),
        })
    }

    #[cfg(test)]
    pub fn from_components(
        application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
        exclusion_repository: Arc<dyn ExclusionRepository + Send + Sync>,
        browser_port: Arc<dyn BrokerBrowserPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
        operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
    ) -> Self {
        Self {
            application_repository,
            stock_repository,
            account_repository,
            exclusion_repository,
            browser_port,
            event_publisher,
            operation_log_repository,
        }
    }

    pub fn application_repository(&self) -> Arc<dyn LotteryApplicationRepository + Send + Sync> {
        self.application_repository.clone()
    }

    pub fn stock_repository(&self) -> Arc<dyn IpoStockRepository + Send + Sync> {
        self.stock_repository.clone()
    }

    pub fn account_repository(&self) -> Arc<dyn SecuritiesAccountRepository + Send + Sync> {
        self.account_repository.clone()
    }

    pub fn exclusion_repository(&self) -> Arc<dyn ExclusionRepository + Send + Sync> {
        self.exclusion_repository.clone()
    }

    pub fn browser_port(&self) -> Arc<dyn BrokerBrowserPort> {
        self.browser_port.clone()
    }

    pub fn event_publisher(&self) -> Arc<dyn EventPublisherPort> {
        self.event_publisher.clone()
    }

    pub fn operation_log_repository(&self) -> Arc<dyn OperationLogRepository + Send + Sync> {
        self.operation_log_repository.clone()
    }
}
