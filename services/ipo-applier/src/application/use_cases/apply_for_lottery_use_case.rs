use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use ipo_backend_shared::{
    acl::{
        browser::{ApplicationResult, BrokerBrowserPort},
        messaging::EventPublisherPort,
    },
    domain::{
        account::SecuritiesAccountRepository,
        application::{ApplicationIdentifier, LotteryApplication, LotteryApplicationRepository},
        exclusion::ExclusionRepository,
        operation_log::{
            OperationEventType, OperationLog, OperationLogPayload, OperationLogRepository,
            OperationStatus,
        },
        stock::{IpoStock, IpoStockRepository, Shares, Yen},
    },
    errors::DomainError,
    events::{ApplicationCompleted, ApplicationFailed},
    services::ApplicationEligibilityService,
};
use serde::{Deserialize, Serialize};

/// Input DTO for DD-101 `ApplyForLotteryUseCase`.
///
/// `target_date` is the reference date used to filter IPO stocks whose
/// book-building period contains that date. Cloud Scheduler passes the
/// execution date; ad-hoc runs (e.g. smoke tests) can pass any date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyForLotteryInput {
    pub target_date: NaiveDate,
}

/// Per-stock result entry surfaced in the use-case output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationResultEntry {
    pub stock_identifier: String,
    pub securities_account: String,
    pub outcome: String,
    pub application_identifier: Option<String>,
    pub reason: Option<String>,
}

/// Aggregated output of DD-101 execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyForLotteryOutput {
    pub applied_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub results: Vec<ApplicationResultEntry>,
}

/// DD-101 `IPO抽選に自動申し込みする` use case. Loops over each active
/// securities account × every stock in the book-building period on the
/// target date, skips excluded / duplicate pairs via
/// `ApplicationEligibilityService`, delegates the submission to the
/// `BrokerBrowserPort`, persists the resulting `LotteryApplication`
/// aggregate, and emits `ApplicationCompleted` / `ApplicationFailed`
/// events for downstream notification fan-out.
pub struct ApplyForLotteryUseCase {
    application_repository: Arc<dyn LotteryApplicationRepository + Send + Sync>,
    stock_repository: Arc<dyn IpoStockRepository + Send + Sync>,
    account_repository: Arc<dyn SecuritiesAccountRepository + Send + Sync>,
    exclusion_repository: Arc<dyn ExclusionRepository + Send + Sync>,
    browser_port: Arc<dyn BrokerBrowserPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
    operation_log_repository: Arc<dyn OperationLogRepository + Send + Sync>,
}

impl ApplyForLotteryUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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

    pub async fn execute(
        &self,
        input: ApplyForLotteryInput,
    ) -> Result<ApplyForLotteryOutput, DomainError> {
        let accounts = self.account_repository.find_active().await?;
        let stocks = self
            .stock_repository
            .find_in_book_building_period(input.target_date)
            .await?;
        let exclusions = self.exclusion_repository.find_all().await?;

        let mut applied_count = 0_u32;
        let mut skipped_count = 0_u32;
        let mut failed_count = 0_u32;
        let mut results = Vec::new();

        for account in &accounts {
            for stock in &stocks {
                let already_applied = self
                    .application_repository
                    .exists_by_stock_and_account(stock.identifier(), account.identifier())
                    .await?;

                let eligible = ApplicationEligibilityService::is_eligible(
                    stock,
                    &exclusions,
                    already_applied,
                    input.target_date,
                );
                if !eligible {
                    skipped_count += 1;
                    results.push(ApplicationResultEntry {
                        stock_identifier: stock.identifier().value().to_string(),
                        securities_account: account.identifier().value().to_string(),
                        outcome: "skipped".to_string(),
                        application_identifier: None,
                        reason: Some(if already_applied {
                            "already_applied".to_string()
                        } else {
                            "not_eligible".to_string()
                        }),
                    });
                    continue;
                }

                let (shares, price) = match determine_order_parameters(stock) {
                    Ok(pair) => pair,
                    Err(error) => {
                        failed_count += 1;
                        self.record_log(
                            None,
                            OperationStatus::Failed,
                            format!(
                                "invalid order parameters for {} on {}",
                                account.identifier().value(),
                                stock.identifier().value()
                            ),
                            Some(error.to_string()),
                        )
                        .await?;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "failed".to_string(),
                            application_identifier: None,
                            reason: Some(error.to_string()),
                        });
                        continue;
                    }
                };

                let mut application = LotteryApplication::create_with_values(
                    stock.identifier().clone(),
                    account.identifier().clone(),
                    shares,
                    price,
                    Utc::now(),
                )?;

                let apply_result = self
                    .browser_port
                    .apply_for_ipo(account.credential(), stock, application.applied_order())
                    .await;

                match apply_result {
                    Ok(ApplicationResult::Success) => {
                        application.apply()?;
                        self.application_repository.save(&application).await?;
                        self.publish_application_completed(&application).await;
                        self.record_log(
                            Some(application.identifier().clone()),
                            OperationStatus::Succeeded,
                            format!(
                                "applied {} via broker (application={})",
                                stock.identifier().value(),
                                application.identifier().value()
                            ),
                            None,
                        )
                        .await?;
                        applied_count += 1;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "applied".to_string(),
                            application_identifier: Some(
                                application.identifier().value().to_string(),
                            ),
                            reason: None,
                        });
                    }
                    Ok(ApplicationResult::AlreadyApplied) => {
                        skipped_count += 1;
                        self.record_log(
                            None,
                            OperationStatus::Succeeded,
                            format!(
                                "skipped {} (broker reported already applied)",
                                stock.identifier().value()
                            ),
                            Some("already_applied".to_string()),
                        )
                        .await?;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "skipped".to_string(),
                            application_identifier: None,
                            reason: Some("already_applied".to_string()),
                        });
                    }
                    Ok(ApplicationResult::InsufficientBalance) => {
                        failed_count += 1;
                        self.publish_application_failed(&application, "insufficient_balance")
                            .await;
                        self.record_log(
                            Some(application.identifier().clone()),
                            OperationStatus::Failed,
                            format!("insufficient balance for {}", stock.identifier().value()),
                            Some("insufficient_balance".to_string()),
                        )
                        .await?;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "failed".to_string(),
                            application_identifier: None,
                            reason: Some("insufficient_balance".to_string()),
                        });
                    }
                    Ok(ApplicationResult::Failure { reason }) => {
                        failed_count += 1;
                        self.publish_application_failed(&application, &reason).await;
                        self.record_log(
                            Some(application.identifier().clone()),
                            OperationStatus::Failed,
                            format!("broker reported failure for {}", stock.identifier().value()),
                            Some(reason.clone()),
                        )
                        .await?;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "failed".to_string(),
                            application_identifier: None,
                            reason: Some(reason),
                        });
                    }
                    Err(error) => {
                        failed_count += 1;
                        self.publish_application_failed(&application, &error.to_string())
                            .await;
                        self.record_log(
                            Some(application.identifier().clone()),
                            OperationStatus::Failed,
                            format!("browser_port error for {}", stock.identifier().value()),
                            Some(error.to_string()),
                        )
                        .await?;
                        results.push(ApplicationResultEntry {
                            stock_identifier: stock.identifier().value().to_string(),
                            securities_account: account.identifier().value().to_string(),
                            outcome: "failed".to_string(),
                            application_identifier: None,
                            reason: Some(error.to_string()),
                        });
                    }
                }
            }
        }

        Ok(ApplyForLotteryOutput {
            applied_count,
            skipped_count,
            failed_count,
            results,
        })
    }

    async fn publish_application_completed(&self, application: &LotteryApplication) {
        let event = ApplicationCompleted {
            identifier: application.identifier().clone(),
            stock: application.stock().clone(),
            securities_account: application.securities_account().clone(),
            applied_shares: application.applied_order().shares(),
            applied_price: application.applied_order().price(),
            applied_at: application.applied_order().ordered_at(),
        };
        let payload = match serde_json::to_value(event) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(error = %error, "failed to serialize ApplicationCompleted");
                return;
            }
        };
        if let Err(error) = self
            .event_publisher
            .publish(
                "ipo-notification",
                application.identifier().value(),
                "LotteryApplication",
                payload,
                None,
            )
            .await
        {
            tracing::warn!(error = %error, "failed to publish ApplicationCompleted");
        }
    }

    async fn publish_application_failed(
        &self,
        application: &LotteryApplication,
        error_message: &str,
    ) {
        let event = ApplicationFailed {
            identifier: application.identifier().clone(),
            stock: application.stock().clone(),
            securities_account: application.securities_account().clone(),
            error_message: error_message.to_string(),
            failed_at: Utc::now(),
        };
        let payload = match serde_json::to_value(event) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(error = %error, "failed to serialize ApplicationFailed");
                return;
            }
        };
        if let Err(error) = self
            .event_publisher
            .publish(
                "ipo-notification",
                application.identifier().value(),
                "LotteryApplication",
                payload,
                None,
            )
            .await
        {
            tracing::warn!(error = %error, "failed to publish ApplicationFailed");
        }
    }

    async fn record_log(
        &self,
        application: Option<ApplicationIdentifier>,
        status: OperationStatus,
        message: String,
        error_message: Option<String>,
    ) -> Result<(), DomainError> {
        let log = OperationLog::create(OperationLogPayload::new(
            application,
            OperationEventType::ApplyLottery,
            "ipo-applier",
            status,
            message,
            error_message,
            Utc::now(),
        ))?;
        self.operation_log_repository.save(&log).await
    }
}

/// Determines shares / price to submit. Rule (see
/// `docs/03-detailed-design/use-case.md` DD-101 implementation note):
/// `shares = 100` (single lot; Japanese IPO minimum), `price =
/// offer_price.unwrap_or(price_range.maximum_price())` so we submit at
/// the upper bound of the book-building range when the official offer
/// price has not yet been fixed.
fn determine_order_parameters(stock: &IpoStock) -> Result<(Shares, Yen), DomainError> {
    let shares = Shares::new(100)?;
    let price = stock
        .pricing()
        .offer_price()
        .unwrap_or_else(|| stock.pricing().price_range().maximum_price());
    Ok((shares, price))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use ipo_backend_shared::{
        acl::{
            browser::{ApplicationResult, BrokerBrowserPort},
            scraping::ScrapedStock,
        },
        domain::{
            account::{
                AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId,
                LoginPassword, MailAddress, MailCredential, MailPassword, SecuritiesAccount,
                SecuritiesAccountRepository, SecuritiesCompany, TradingPassword,
            },
            application::{
                ApplicationStatus, AppliedOrder, LotteryApplicationRepository, LotteryResult,
            },
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockStatus, Yen,
            },
        },
        errors::DomainError,
        testing::{
            FirestoreExclusionRepositoryInMemory, FirestoreIpoStockRepositoryInMemory,
            FirestoreLotteryApplicationRepositoryInMemory, FirestoreOperationLogRepositoryInMemory,
            FirestoreSecuritiesAccountRepositoryInMemory, InMemoryCredentialStore,
            PubSubEventPublisherInMemory,
        },
    };

    use super::{ApplyForLotteryInput, ApplyForLotteryUseCase};

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
                    NaiveDate::from_ymd_opt(2026, 4, 20).expect("start"),
                    NaiveDate::from_ymd_opt(2026, 4, 24).expect("end"),
                )
                .expect("period"),
                NaiveDate::from_ymd_opt(2026, 4, 27).expect("lottery"),
                NaiveDate::from_ymd_opt(2026, 4, 30).expect("listing"),
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
                Shares::new(100_000).expect("shares"),
            )
            .expect("offering"),
            StockStatus::Eligible,
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    #[derive(Debug)]
    struct SucceedingBrowser;

    #[async_trait]
    impl BrokerBrowserPort for SucceedingBrowser {
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
            Ok(None)
        }

        async fn apply_for_ipo(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
            _applied_order: &AppliedOrder,
        ) -> Result<ApplicationResult, DomainError> {
            Ok(ApplicationResult::Success)
        }
    }

    #[derive(Debug)]
    struct FailingBrowser {
        reason: String,
    }

    #[async_trait]
    impl BrokerBrowserPort for FailingBrowser {
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
            Ok(None)
        }

        async fn apply_for_ipo(
            &self,
            _credential: &AccountCredential,
            _stock: &IpoStock,
            _applied_order: &AppliedOrder,
        ) -> Result<ApplicationResult, DomainError> {
            Ok(ApplicationResult::Failure {
                reason: self.reason.clone(),
            })
        }
    }

    #[tokio::test]
    async fn applies_successfully_and_publishes_event() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepositoryInMemory::new(
            credential_store,
        ));
        let stock_repository = Arc::new(FirestoreIpoStockRepositoryInMemory::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepositoryInMemory::new());
        let exclusion_repository = Arc::new(FirestoreExclusionRepositoryInMemory::new());
        let event_publisher = Arc::new(PubSubEventPublisherInMemory::new("ipo-applier"));

        account_repository
            .save(&build_account())
            .await
            .expect("save account");
        stock_repository
            .save(&build_stock())
            .await
            .expect("save stock");

        let use_case = ApplyForLotteryUseCase::new(
            application_repository.clone(),
            stock_repository,
            account_repository,
            exclusion_repository,
            Arc::new(SucceedingBrowser),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepositoryInMemory::new()),
        );

        let output = use_case
            .execute(ApplyForLotteryInput {
                target_date: NaiveDate::from_ymd_opt(2026, 4, 22).expect("date"),
            })
            .await
            .expect("execute");

        assert_eq!(output.applied_count, 1);
        assert_eq!(output.skipped_count, 0);
        assert_eq!(output.failed_count, 0);
        assert_eq!(output.results.len(), 1);
        assert_eq!(output.results[0].outcome, "applied");

        let saved = application_repository
            .find_by_status(ApplicationStatus::Applied)
            .await
            .expect("find by status");
        assert_eq!(saved.len(), 1);

        let published = event_publisher
            .published_messages()
            .await
            .expect("messages");
        assert_eq!(published.len(), 1);
    }

    #[tokio::test]
    async fn skips_when_outside_book_building_period() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepositoryInMemory::new(
            credential_store,
        ));
        let stock_repository = Arc::new(FirestoreIpoStockRepositoryInMemory::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepositoryInMemory::new());
        let exclusion_repository = Arc::new(FirestoreExclusionRepositoryInMemory::new());
        let event_publisher = Arc::new(PubSubEventPublisherInMemory::new("ipo-applier"));

        account_repository
            .save(&build_account())
            .await
            .expect("save account");
        // Intentionally do not save any stocks so find_in_book_building_period returns empty.

        let use_case = ApplyForLotteryUseCase::new(
            application_repository,
            stock_repository,
            account_repository,
            exclusion_repository,
            Arc::new(SucceedingBrowser),
            event_publisher,
            Arc::new(FirestoreOperationLogRepositoryInMemory::new()),
        );

        let output = use_case
            .execute(ApplyForLotteryInput {
                target_date: NaiveDate::from_ymd_opt(2026, 4, 22).expect("date"),
            })
            .await
            .expect("execute");

        assert_eq!(output.applied_count, 0);
        assert_eq!(output.skipped_count, 0);
        assert_eq!(output.failed_count, 0);
        assert!(output.results.is_empty());
    }

    #[tokio::test]
    async fn emits_application_failed_on_browser_failure() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepositoryInMemory::new(
            credential_store,
        ));
        let stock_repository = Arc::new(FirestoreIpoStockRepositoryInMemory::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepositoryInMemory::new());
        let exclusion_repository = Arc::new(FirestoreExclusionRepositoryInMemory::new());
        let event_publisher = Arc::new(PubSubEventPublisherInMemory::new("ipo-applier"));

        account_repository
            .save(&build_account())
            .await
            .expect("save account");
        stock_repository
            .save(&build_stock())
            .await
            .expect("save stock");

        let use_case = ApplyForLotteryUseCase::new(
            application_repository,
            stock_repository,
            account_repository,
            exclusion_repository,
            Arc::new(FailingBrowser {
                reason: "broker 500".to_string(),
            }),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepositoryInMemory::new()),
        );

        let output = use_case
            .execute(ApplyForLotteryInput {
                target_date: NaiveDate::from_ymd_opt(2026, 4, 22).expect("date"),
            })
            .await
            .expect("execute");

        assert_eq!(output.applied_count, 0);
        assert_eq!(output.failed_count, 1);
        let published = event_publisher
            .published_messages()
            .await
            .expect("messages");
        assert_eq!(published.len(), 1);
        assert_eq!(output.results[0].outcome, "failed");
    }
}
