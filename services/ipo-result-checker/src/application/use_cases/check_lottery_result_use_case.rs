use std::sync::Arc;

use chrono::Utc;
use ipo_backend_shared::{
    acl::{browser::BrokerBrowserPort, messaging::EventPublisherPort},
    domain::{
        account::SecuritiesAccountRepository,
        application::{ApplicationStatus, LotteryApplicationRepository, LotteryResult},
        operation_log::{
            OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
            OperationStatus,
        },
        stock::IpoStockRepository,
    },
    errors::DomainError,
    events::LotteryResultConfirmed,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultCheckEntry {
    pub application_identifier: String,
    pub stock_identifier: String,
    pub result: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckLotteryResultOutput {
    pub checked_count: u32,
    pub results: Vec<ResultCheckEntry>,
}

pub struct CheckLotteryResultUseCase {
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    browser_port: Arc<dyn BrokerBrowserPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl CheckLotteryResultUseCase {
    pub fn new(
        application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
        stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
        account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
        browser_port: Arc<dyn BrokerBrowserPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
        operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
    ) -> Self {
        Self {
            application_repository,
            stock_repository,
            account_repository,
            browser_port,
            event_publisher,
            operation_log_repository,
        }
    }

    pub async fn execute(&self) -> Result<CheckLotteryResultOutput, DomainError> {
        let applications = self
            .application_repository
            .find_by_status(ApplicationStatus::Applied)
            .await?;
        let mut checked_count = 0_u32;
        let mut results = Vec::new();

        for mut application in applications {
            let stock = match self
                .stock_repository
                .find_by_id(application.stock())
                .await?
            {
                Some(stock) => stock,
                None => continue,
            };
            let account = match self
                .account_repository
                .find_by_id(application.securities_account())
                .await?
            {
                Some(account) => account,
                None => continue,
            };

            match self
                .browser_port
                .check_lottery_result(account.credential(), &stock)
                .await
            {
                Ok(Some(result)) => {
                    application.record_outcome(result, Utc::now())?;
                    self.application_repository.save(&application).await?;
                    let publish_result = self
                        .event_publisher
                        .publish(
                            "ipo-result-updated",
                            application.identifier().value(),
                            "LotteryApplication",
                            serde_json::to_value(LotteryResultConfirmed {
                                identifier: application.identifier().clone(),
                                stock: application.stock().clone(),
                                lottery_result: result,
                                confirmed_at: Utc::now(),
                            })
                            .map_err(|error| {
                                DomainError::PubSubPublishError {
                                    reason: error.to_string(),
                                }
                            })?,
                            None,
                        )
                        .await;

                    match publish_result {
                        Ok(()) => {
                            self.operation_log_repository
                                .save(&OperationLog::create(OperationLogPayload::new(
                                    Some(application.identifier().clone()),
                                    OperationEventType::CheckLotteryResult,
                                    "ipo-result-checker",
                                    OperationStatus::Succeeded,
                                    format!("checked {}", stock.identifier().value()),
                                    None,
                                    Utc::now(),
                                ))?)
                                .await?;
                        }
                        Err(error) => {
                            self.operation_log_repository
                                .save(&OperationLog::create(OperationLogPayload::new(
                                    Some(application.identifier().clone()),
                                    OperationEventType::CheckLotteryResult,
                                    "ipo-result-checker",
                                    OperationStatus::Failed,
                                    format!(
                                        "failed to publish result update for {}",
                                        stock.identifier().value()
                                    ),
                                    Some(error.to_string()),
                                    Utc::now(),
                                ))?)
                                .await?;
                        }
                    }

                    checked_count += 1;
                    results.push(ResultCheckEntry {
                        application_identifier: application.identifier().value().to_string(),
                        stock_identifier: application.stock().value().to_string(),
                        result: Some(lottery_result_to_string(result)),
                        status: application.status().as_str().to_string(),
                    });
                }
                Ok(None) => {
                    results.push(ResultCheckEntry {
                        application_identifier: application.identifier().value().to_string(),
                        stock_identifier: application.stock().value().to_string(),
                        result: None,
                        status: application.status().as_str().to_string(),
                    });
                }
                Err(error) => {
                    self.operation_log_repository
                        .save(&OperationLog::create(OperationLogPayload::new(
                            Some(application.identifier().clone()),
                            OperationEventType::CheckLotteryResult,
                            "ipo-result-checker",
                            OperationStatus::Failed,
                            format!("failed to check {}", stock.identifier().value()),
                            Some(error.to_string()),
                            Utc::now(),
                        ))?)
                        .await?;
                }
            }
        }

        Ok(CheckLotteryResultOutput {
            checked_count,
            results,
        })
    }
}

fn lottery_result_to_string(result: LotteryResult) -> String {
    match result {
        LotteryResult::Won => "Won".to_string(),
        LotteryResult::Lost => "Lost".to_string(),
        LotteryResult::Alternate => "Alternate".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{NaiveDate, TimeZone, Utc};
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, messaging::EventPublisherPort, scraping::ScrapedStock},
        domain::{
            account::{
                AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId,
                LoginPassword, MailAddress, MailCredential, MailPassword, SecuritiesAccount,
                SecuritiesAccountRepository, SecuritiesCompany, TradingPassword,
            },
            application::{
                ApplicationStatus, LotteryApplication, LotteryApplicationRepository, LotteryResult,
            },
            operation_log::{OperationLogRepository, OperationStatus},
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockStatus, Yen,
            },
        },
        errors::DomainError,
        infrastructure::messaging::PubSubEventEnvelope,
        testing::{
            FirestoreIpoStockRepository, FirestoreLotteryApplicationRepository,
            FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            InMemoryCredentialStore, PubSubEventPublisher,
        },
    };
    use uuid::Uuid;

    use super::CheckLotteryResultUseCase;

    #[derive(Debug)]
    struct WinningBrowser;

    #[async_trait]
    impl BrokerBrowserPort for WinningBrowser {
        async fn fetch_ipo_stocks(&self) -> Result<Vec<ScrapedStock>, DomainError> {
            Ok(Vec::new())
        }

        async fn test_connection(
            &self,
            _credential: &AccountCredential,
        ) -> Result<ConnectionTestResult, DomainError> {
            Ok(ConnectionTestResult::new(true, "ok", Utc::now()))
        }

        async fn check_lottery_result(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
        ) -> Result<Option<LotteryResult>, DomainError> {
            Ok(Some(LotteryResult::Won))
        }
    }

    #[derive(Debug)]
    struct FailingPublisher;

    #[async_trait]
    impl EventPublisherPort for FailingPublisher {
        async fn publish(
            &self,
            _event_type: &str,
            _aggregate_id: &str,
            _aggregate_type: &str,
            _payload: serde_json::Value,
            _correlation_id: Option<Uuid>,
        ) -> Result<(), DomainError> {
            Err(DomainError::PubSubPublishError {
                reason: "publisher unavailable".to_string(),
            })
        }
    }

    fn build_account() -> SecuritiesAccount {
        SecuritiesAccount::create(
            SecuritiesCompany::Rakuten,
            AccountCredential::new(
                LoginId::new("login").expect("login id"),
                LoginPassword::new("password").expect("login password"),
                TradingPassword::new("1234").expect("trading password"),
                MailCredential::new(
                    MailAddress::new("test@example.com").expect("mail"),
                    MailPassword::new("mail-password").expect("mail password"),
                    ImapHost::new("imap.example.com").expect("host"),
                    ImapPort::new(993).expect("port"),
                )
                .expect("mail credential"),
            )
            .expect("credential"),
        )
        .expect("account")
    }

    fn build_stock() -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト株式会社").expect("name"),
                None,
                Market::Growth,
                Industry::new("情報・通信業").expect("industry"),
            )
            .expect("profile"),
            IpoSchedule::new(
                BookBuildingPeriod::new(
                    NaiveDate::from_ymd_opt(2026, 4, 1).expect("start"),
                    NaiveDate::from_ymd_opt(2026, 4, 10).expect("end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 15).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 25).expect("listing"),
            )
            .expect("schedule"),
            IpoPricing::new(
                PriceRange::new(Yen::new(1200).expect("min"), Yen::new(1500).expect("max"))
                    .expect("range"),
                Some(Yen::new(1400).expect("offer")),
            )
            .expect("pricing"),
            IpoOffering::new(
                LeadUnderwriter::new("楽天証券").expect("underwriter"),
                Shares::new(100000).expect("shares"),
            )
            .expect("offering"),
            StockStatus::Applied,
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    #[tokio::test]
    async fn records_result_and_publishes_event() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-result-checker"));
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new());

        let account = build_account();
        let stock = build_stock();
        let mut application = LotteryApplication::create_with_values(
            stock.identifier().clone(),
            account.identifier().clone(),
            Shares::new(100).expect("shares"),
            Yen::new(1400).expect("price"),
            Utc.with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("ordered at"),
        )
        .expect("application");
        application.apply().expect("apply");

        account_repository
            .save(&account)
            .await
            .expect("save account");
        stock_repository.save(&stock).await.expect("save stock");
        application_repository
            .save(&application)
            .await
            .expect("save application");

        let output = CheckLotteryResultUseCase::new(
            application_repository.clone(),
            stock_repository,
            account_repository,
            Arc::new(WinningBrowser),
            event_publisher.clone(),
            operation_log_repository,
        )
        .execute()
        .await
        .expect("execute");

        assert_eq!(output.checked_count, 1);
        assert_eq!(
            application_repository
                .find_by_id(application.identifier())
                .await
                .expect("find application")
                .expect("application")
                .status(),
            ApplicationStatus::ResultChecked
        );
        assert_eq!(
            event_publisher
                .published_messages()
                .await
                .expect("messages")
                .len(),
            1
        );
        let messages = event_publisher
            .published_messages()
            .await
            .expect("messages");
        let envelope: PubSubEventEnvelope<serde_json::Value> =
            serde_json::from_str(&messages[0]).expect("envelope");
        assert_eq!(envelope.event_type, "ipo-result-updated");
        assert_eq!(envelope.aggregate_type, "LotteryApplication");
        assert_eq!(envelope.metadata.service_name, "ipo-result-checker");
        assert_eq!(
            envelope.payload["lottery_result"],
            serde_json::Value::String("Won".to_string())
        );
    }

    #[tokio::test]
    async fn continues_when_event_publish_fails_after_persisting_result() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new());

        let account = build_account();
        let stock = build_stock();
        let mut application = LotteryApplication::create_with_values(
            stock.identifier().clone(),
            account.identifier().clone(),
            Shares::new(100).expect("shares"),
            Yen::new(1400).expect("price"),
            Utc.with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("ordered at"),
        )
        .expect("application");
        application.apply().expect("apply");

        account_repository
            .save(&account)
            .await
            .expect("save account");
        stock_repository.save(&stock).await.expect("save stock");
        application_repository
            .save(&application)
            .await
            .expect("save application");

        let output = CheckLotteryResultUseCase::new(
            application_repository.clone(),
            stock_repository,
            account_repository,
            Arc::new(WinningBrowser),
            Arc::new(FailingPublisher),
            operation_log_repository.clone(),
        )
        .execute()
        .await
        .expect("execute");

        assert_eq!(output.checked_count, 1);
        assert_eq!(
            application_repository
                .find_by_id(application.identifier())
                .await
                .expect("find application")
                .expect("application")
                .status(),
            ApplicationStatus::ResultChecked
        );

        let logs = operation_log_repository.find_all().await.expect("logs");
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status(), OperationStatus::Failed);
        assert_eq!(
            logs[0].error_message(),
            Some("pubsub publish error: publisher unavailable")
        );
    }
}
