use std::sync::Arc;

use ipo_backend_shared::{
    acl::{
        browser::BrokerBrowserPort, notification::NotificationPort, secrets::CredentialStorePort,
    },
    domain::exclusion::ExclusionRepository,
    domain::{
        account::SecuritiesAccountRepository,
        application::LotteryApplicationRepository,
        notification::{NotificationSetting, NotificationSettingRepository},
        operation_log::OperationLogRepository,
        stock::IpoStockRepository,
    },
    errors::DomainError,
    infrastructure::{
        firestore::{
            build_firestore_client,
            repositories::{
                FirestoreExclusionRepository, FirestoreIpoStockRepository,
                FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
                FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            },
        },
        http_client::{HttpClientConfig, ReqwestClientFactory},
        notification::{
            EmailNotificationAdapter, LineNotificationAdapter, SlackNotificationAdapter,
        },
        secrets::{build_credential_store, sendgrid_api_key_secret_name},
    },
};

use crate::{config, infrastructure::BrowserServiceClient};

use super::NotificationPortRegistry;

#[derive(Clone)]
pub struct DependencyContainer {
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    exclusion_repository: Arc<dyn ExclusionRepository + Send + Sync>,
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    notification_setting_repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
    browser_port: Arc<dyn BrokerBrowserPort>,
    notification_ports: Arc<NotificationPortRegistry>,
}

impl DependencyContainer {
    pub async fn new() -> Result<Self, DomainError> {
        let client = ReqwestClientFactory::new(HttpClientConfig::default()).build()?;
        let credential_store_port: Arc<dyn CredentialStorePort> =
            build_credential_store(&config::firebase_project_id()).await?;
        if let Some(sendgrid_api_key) = config::sendgrid_api_key() {
            credential_store_port
                .save(sendgrid_api_key_secret_name(), &sendgrid_api_key)
                .await?;
        }
        let db = Arc::new(build_firestore_client(&config::firebase_project_id()).await?);

        let notification_setting_repository =
            Arc::new(FirestoreNotificationSettingRepository::new(db.clone()))
                as Arc<dyn NotificationSettingRepository + Send + Sync>;

        let line_adapter = Arc::new(LineNotificationAdapter::new_with_endpoint(
            client.clone(),
            config::line_notify_endpoint(),
        )) as Arc<dyn NotificationPort + Send + Sync>;
        let email_adapter = Arc::new(EmailNotificationAdapter::new_with_endpoint(
            client.clone(),
            credential_store_port.clone(),
            config::notification_from_address(),
            config::sendgrid_endpoint(),
        )) as Arc<dyn NotificationPort + Send + Sync>;
        let slack_adapter = Arc::new(SlackNotificationAdapter::new(client.clone()))
            as Arc<dyn NotificationPort + Send + Sync>;

        Self::from_components(
            Arc::new(FirestoreIpoStockRepository::new(db.clone())),
            Arc::new(FirestoreExclusionRepository::new(db.clone())),
            Arc::new(FirestoreLotteryApplicationRepository::new(db.clone())),
            Arc::new(FirestoreSecuritiesAccountRepository::new(
                db.clone(),
                credential_store_port,
            )),
            notification_setting_repository,
            Arc::new(FirestoreOperationLogRepository::new(db)),
            Arc::new(BrowserServiceClient::new(
                client,
                config::ipo_browser_base_url(),
            )),
            Arc::new(NotificationPortRegistry::new(
                line_adapter,
                email_adapter,
                slack_adapter,
            )),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn from_components(
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        exclusion_repository: Arc<dyn ExclusionRepository + Send + Sync>,
        application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
        account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
        notification_setting_repository: Arc<dyn NotificationSettingRepository + Send + Sync>,
        operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
        browser_port: Arc<dyn BrokerBrowserPort>,
        notification_ports: Arc<NotificationPortRegistry>,
    ) -> Result<Self, DomainError> {
        if notification_setting_repository
            .find_by_id(
                &ipo_backend_shared::domain::notification::NotificationSettingIdentifier::default_id(),
            )
            .await?
            .is_none()
        {
            notification_setting_repository
                .save(&NotificationSetting::create(Vec::new())?)
                .await?;
        }

        Ok(Self {
            stock_repository,
            exclusion_repository,
            application_repository,
            account_repository,
            notification_setting_repository,
            operation_log_repository,
            browser_port,
            notification_ports,
        })
    }

    pub fn stock_repository(&self) -> Arc<dyn IpoStockRepository + Send + Sync> {
        self.stock_repository.clone()
    }

    pub fn exclusion_repository(&self) -> Arc<dyn ExclusionRepository + Send + Sync> {
        self.exclusion_repository.clone()
    }

    pub fn application_repository(&self) -> Arc<dyn LotteryApplicationRepository + Send + Sync> {
        self.application_repository.clone()
    }

    pub fn account_repository(&self) -> Arc<dyn SecuritiesAccountRepository + Send + Sync> {
        self.account_repository.clone()
    }

    pub fn notification_setting_repository(
        &self,
    ) -> Arc<dyn NotificationSettingRepository + Send + Sync> {
        self.notification_setting_repository.clone()
    }

    pub fn operation_log_repository(&self) -> Arc<dyn OperationLogRepository + Send + Sync> {
        self.operation_log_repository.clone()
    }

    pub fn browser_port(&self) -> Arc<dyn BrokerBrowserPort> {
        self.browser_port.clone()
    }

    pub fn notification_ports(&self) -> Arc<NotificationPortRegistry> {
        self.notification_ports.clone()
    }
}
