use axum::{
    routing::{delete, get, post, put},
    Router,
};
use ipo_backend_shared::http::create_health_check_router;

use crate::{infrastructure::DependencyContainer, presentation::handlers};

/// Creates the API router.
pub fn create_router(container: DependencyContainer) -> Router {
    Router::<DependencyContainer>::new()
        .merge(create_health_check_router::<DependencyContainer>())
        .route("/api/v1/stocks", get(handlers::stock_handlers::list_stocks))
        .route(
            "/api/v1/stocks/{stock_id}",
            get(handlers::stock_handlers::get_stock),
        )
        .route(
            "/api/v1/dashboard",
            get(handlers::stock_handlers::get_dashboard),
        )
        .route(
            "/api/v1/exclusions",
            get(handlers::exclusion_handlers::list_exclusions)
                .post(handlers::exclusion_handlers::register_exclusion),
        )
        .route(
            "/api/v1/exclusions/{exclusion_id}",
            delete(handlers::exclusion_handlers::remove_exclusion),
        )
        .route(
            "/api/v1/notifications/settings",
            get(handlers::notification_handlers::get_setting)
                .put(handlers::notification_handlers::update_setting),
        )
        .route(
            "/api/v1/accounts",
            get(handlers::account_handlers::list_accounts)
                .post(handlers::account_handlers::register_account),
        )
        .route(
            "/api/v1/accounts/{account_id}",
            put(handlers::account_handlers::update_account)
                .delete(handlers::account_handlers::delete_account),
        )
        .route(
            "/api/v1/accounts/{account_id}/test",
            post(handlers::account_handlers::test_connection),
        )
        .route("/api/v1/logs", get(handlers::log_handlers::list_logs))
        .route(
            "/internal/pubsub/ipo-info-updated",
            post(handlers::notification_handlers::handle_ipo_info_updated),
        )
        .route(
            "/internal/pubsub/ipo-result-updated",
            post(handlers::notification_handlers::handle_lottery_result_updated),
        )
        .route(
            "/internal/pubsub/ipo-notification",
            post(handlers::notification_handlers::handle_generic_notification),
        )
        .with_state(container)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use async_trait::async_trait;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use chrono::{NaiveDate, TimeZone, Utc};
    use ipo_backend_shared::{
        acl::{
            browser::BrokerBrowserPort, messaging::EventPublisherPort,
            notification::NotificationPort, scraping::ScrapedStock, secrets::CredentialStorePort,
        },
        domain::{
            account::{
                AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId,
                LoginPassword, MailAddress, MailCredential, MailPassword, SecuritiesAccount,
                SecuritiesAccountIdentifier, SecuritiesAccountRepository, SecuritiesCompany,
                TradingPassword,
            },
            application::{ApplicationIdentifier, LotteryApplicationRepository},
            exclusion::ExclusionRepository,
            notification::{
                ChannelDestination, ChannelType, NotificationChannel, NotificationEventType,
                NotificationSetting, NotificationSettingRepository,
            },
            operation_log::OperationLogRepository,
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockIdentifier,
                StockStatus, TickerSymbol, Yen,
            },
        },
        errors::DomainError,
        events::{
            ApplicationCompleted, ApplicationFailed, ImageAuthenticationFailed, IpoInfoUpdated,
            LotteryResultConfirmed,
        },
        infrastructure::messaging::PubSubEventEnvelope,
        infrastructure::{
            notification::EmailNotificationAdapter, secrets::sendgrid_api_key_secret_name,
        },
        testing::{
            FirestoreExclusionRepository, FirestoreIpoStockRepository,
            FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
            FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            InMemoryCredentialStore, PubSubEventPublisher,
        },
    };
    use serde_json::{json, Value};
    use tokio::sync::Mutex;
    use tower::util::ServiceExt;
    use wiremock::{
        matchers::{body_partial_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::create_router;
    use crate::infrastructure::{
        BrowserServiceClient, DependencyContainer, NotificationPortRegistry,
    };

    #[derive(Debug)]
    struct DummyBrowserPort;

    #[async_trait]
    impl BrokerBrowserPort for DummyBrowserPort {
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
        ) -> Result<Option<ipo_backend_shared::domain::application::LotteryResult>, DomainError>
        {
            Ok(None)
        }
    }

    #[derive(Debug, Default)]
    struct CaptureNotificationPort {
        sends: Mutex<Vec<BTreeMap<String, String>>>,
    }

    #[async_trait]
    impl NotificationPort for CaptureNotificationPort {
        async fn send(
            &self,
            _event: &ipo_backend_shared::acl::notification::NotificationEvent,
            destination: &ChannelDestination,
        ) -> Result<(), DomainError> {
            self.sends.lock().await.push(destination.values().clone());
            Ok(())
        }

        fn validate_destination(
            &self,
            _destination: &ChannelDestination,
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    fn build_stock() -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト株式会社").expect("company name"),
                Some(TickerSymbol::new("1234").expect("ticker")),
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
            StockStatus::Eligible,
            MetaSource::new(
                FetchOrigin::ExternalSite,
                Utc.with_ymd_and_hms(2026, 3, 25, 9, 30, 0)
                    .single()
                    .expect("timestamp"),
            ),
        )
        .expect("stock")
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

    fn build_email_notification_app(
        subscription_event_type: NotificationEventType,
        sendgrid_endpoint: String,
    ) -> axum::Router {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let credential_store = InMemoryCredentialStore::new();
        credential_store
            .save(sendgrid_api_key_secret_name(), "sendgrid-token")
            .expect("save api key");
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            credential_store.clone(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(subscription_event_type, true);
        let mut destination_values = BTreeMap::new();
        destination_values.insert("address".to_string(), "notify@example.com".to_string());
        let channel = NotificationChannel::create(
            ChannelType::Email,
            ChannelDestination::new(ChannelType::Email, destination_values).expect("destination"),
            true,
            subscriptions,
        )
        .expect("channel");
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        notification_setting_repository
            .save(&setting)
            .expect("save setting");

        create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository,
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(EmailNotificationAdapter::new_with_endpoint(
                        reqwest::Client::new(),
                        credential_store,
                        "no-reply@example.com",
                        sendgrid_endpoint,
                    )),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        )
    }

    #[tokio::test]
    async fn lists_seeded_stock() {
        let container = DependencyContainer::new().expect("container");
        container
            .stock_repository()
            .save(&build_stock())
            .expect("save stock");
        let app = create_router(container);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stocks")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["totalCount"], 1);
        assert_eq!(json["items"][0]["companyName"], "テスト株式会社");
    }

    #[tokio::test]
    async fn registers_exclusion_via_http() {
        let app = create_router(DependencyContainer::new().expect("container"));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exclusions")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"companyName":"テスト株式会社","reason":"自社"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn dispatches_generic_notification_via_http() {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            InMemoryCredentialStore::new(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(NotificationEventType::ApplicationCompleted, true);
        let mut destination_values = BTreeMap::new();
        destination_values.insert("address".to_string(), "notify@example.com".to_string());
        let channel = NotificationChannel::create(
            ChannelType::Email,
            ChannelDestination::new(ChannelType::Email, destination_values.clone())
                .expect("destination"),
            true,
            subscriptions,
        )
        .expect("channel");
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        notification_setting_repository
            .save(&setting)
            .expect("save setting");

        let line_port = Arc::new(CaptureNotificationPort::default());
        let email_port = Arc::new(CaptureNotificationPort::default());
        let slack_port = Arc::new(CaptureNotificationPort::default());
        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository,
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    line_port,
                    email_port.clone(),
                    slack_port,
                )),
            )
            .expect("container"),
        );

        let event = ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1400).expect("price"),
            applied_at: Utc
                .with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("applied at"),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ApplicationCompleted",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let sends = email_port.sends.lock().await;
        assert_eq!(sends.len(), 1);
        assert_eq!(
            sends[0].get("address").expect("address"),
            "notify@example.com"
        );
    }

    #[tokio::test]
    async fn dispatches_generic_notification_via_http_with_emulated_email_service() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;
        let app = build_email_notification_app(
            NotificationEventType::ApplicationCompleted,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let event = ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1400).expect("price"),
            applied_at: Utc
                .with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("applied at"),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ApplicationCompleted",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn returns_service_unavailable_when_notification_delivery_fails() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&sendgrid_server)
            .await;

        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let credential_store = InMemoryCredentialStore::new();
        credential_store
            .save(sendgrid_api_key_secret_name(), "sendgrid-token")
            .expect("save api key");
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            credential_store.clone(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(NotificationEventType::ApplicationCompleted, true);
        let mut destination_values = BTreeMap::new();
        destination_values.insert("address".to_string(), "notify@example.com".to_string());
        let channel = NotificationChannel::create(
            ChannelType::Email,
            ChannelDestination::new(ChannelType::Email, destination_values).expect("destination"),
            true,
            subscriptions,
        )
        .expect("channel");
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        notification_setting_repository
            .save(&setting)
            .expect("save setting");

        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository,
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(EmailNotificationAdapter::new_with_endpoint(
                        reqwest::Client::new(),
                        credential_store,
                        "no-reply@example.com",
                        format!("{}/mail/send", sendgrid_server.uri()),
                    )),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        );

        let event = ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1400).expect("price"),
            applied_at: Utc
                .with_ymd_and_hms(2026, 4, 5, 10, 0, 0)
                .single()
                .expect("applied at"),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ApplicationCompleted",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["error"]["code"], "SERVICE_UNAVAILABLE");
        assert_eq!(
            json["error"]["message"],
            "外部サービスとの通信に失敗しました"
        );
    }

    #[tokio::test]
    async fn dispatches_ipo_info_updated_via_http_with_emulated_email_service() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;
        let app = build_email_notification_app(
            NotificationEventType::StockUpdated,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let stock = build_stock();
        let event = IpoInfoUpdated {
            identifier: stock.identifier().clone(),
            company_name: stock.company_profile().company_name().clone(),
            book_building_period: stock.schedule().book_building_period().clone(),
            lottery_date: stock.schedule().lottery_date(),
            listing_date: stock.schedule().listing_date(),
            updated_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-info-updated")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).expect("event body")))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_lottery_result_updated_via_http_with_emulated_email_service() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;
        let app = build_email_notification_app(
            NotificationEventType::LotteryResultWon,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let event = LotteryResultConfirmed {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            lottery_result: ipo_backend_shared::domain::application::LotteryResult::Won,
            confirmed_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-result-updated")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event).expect("event body")))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_ipo_info_updated_from_published_envelope_via_http() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;

        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let credential_store = InMemoryCredentialStore::new();
        credential_store
            .save(sendgrid_api_key_secret_name(), "sendgrid-token")
            .expect("save api key");
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            credential_store.clone(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(NotificationEventType::StockUpdated, true);
        let mut destination_values = BTreeMap::new();
        destination_values.insert("address".to_string(), "notify@example.com".to_string());
        let channel = NotificationChannel::create(
            ChannelType::Email,
            ChannelDestination::new(ChannelType::Email, destination_values).expect("destination"),
            true,
            subscriptions,
        )
        .expect("channel");
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        notification_setting_repository
            .save(&setting)
            .expect("save setting");

        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository,
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(EmailNotificationAdapter::new_with_endpoint(
                        reqwest::Client::new(),
                        credential_store,
                        "no-reply@example.com",
                        format!("{}/mail/send", sendgrid_server.uri()),
                    )),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        );

        let stock = build_stock();
        let event = IpoInfoUpdated {
            identifier: stock.identifier().clone(),
            company_name: stock.company_profile().company_name().clone(),
            book_building_period: stock.schedule().book_building_period().clone(),
            lottery_date: stock.schedule().lottery_date(),
            listing_date: stock.schedule().listing_date(),
            updated_at: Utc::now(),
        };
        let publisher = PubSubEventPublisher::new("ipo-info-fetcher");
        publisher
            .publish(
                "ipo-info-updated",
                stock.identifier().value(),
                "IpoStock",
                serde_json::to_value(&event).expect("event json"),
                None,
            )
            .await
            .expect("publish");
        let messages = publisher.published_messages().await.expect("messages");
        let envelope: PubSubEventEnvelope<Value> =
            serde_json::from_str(&messages[0]).expect("envelope");
        assert_eq!(envelope.metadata.service_name, "ipo-info-fetcher");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-info-updated")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&envelope.payload).expect("payload body"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_lottery_result_updated_from_published_envelope_via_http() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;

        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let credential_store = InMemoryCredentialStore::new();
        credential_store
            .save(sendgrid_api_key_secret_name(), "sendgrid-token")
            .expect("save api key");
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            credential_store.clone(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let mut subscriptions = BTreeMap::new();
        subscriptions.insert(NotificationEventType::LotteryResultWon, true);
        let mut destination_values = BTreeMap::new();
        destination_values.insert("address".to_string(), "notify@example.com".to_string());
        let channel = NotificationChannel::create(
            ChannelType::Email,
            ChannelDestination::new(ChannelType::Email, destination_values).expect("destination"),
            true,
            subscriptions,
        )
        .expect("channel");
        let mut setting = NotificationSetting::create(vec![channel]).expect("setting");
        setting.enable().expect("enable");
        notification_setting_repository
            .save(&setting)
            .expect("save setting");

        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository,
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(EmailNotificationAdapter::new_with_endpoint(
                        reqwest::Client::new(),
                        credential_store,
                        "no-reply@example.com",
                        format!("{}/mail/send", sendgrid_server.uri()),
                    )),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        );

        let event = LotteryResultConfirmed {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            lottery_result: ipo_backend_shared::domain::application::LotteryResult::Won,
            confirmed_at: Utc::now(),
        };
        let publisher = PubSubEventPublisher::new("ipo-result-checker");
        publisher
            .publish(
                "ipo-result-updated",
                event.identifier.value(),
                "LotteryApplication",
                serde_json::to_value(&event).expect("event json"),
                None,
            )
            .await
            .expect("publish");
        let messages = publisher.published_messages().await.expect("messages");
        let envelope: PubSubEventEnvelope<Value> =
            serde_json::from_str(&messages[0]).expect("envelope");
        assert_eq!(envelope.metadata.service_name, "ipo-result-checker");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-result-updated")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&envelope.payload).expect("payload body"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_application_completed_from_ipo_browser_contract_via_http() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;

        let app = build_email_notification_app(
            NotificationEventType::ApplicationCompleted,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let event = ApplicationCompleted {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            applied_shares: Shares::new(100).expect("shares"),
            applied_price: Yen::new(1400).expect("price"),
            applied_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ApplicationCompleted",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_application_failed_from_ipo_browser_contract_via_http() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;

        let app = build_email_notification_app(
            NotificationEventType::OperationError,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let event = ApplicationFailed {
            identifier: ApplicationIdentifier::generate(),
            stock: StockIdentifier::generate(),
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            error_message: "IPO抽選の申し込みに失敗しました".to_string(),
            failed_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ApplicationFailed",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn dispatches_image_authentication_failed_from_ipo_browser_contract_via_http() {
        let sendgrid_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mail/send"))
            .and(wiremock::matchers::header(
                "authorization",
                "Bearer sendgrid-token",
            ))
            .and(body_partial_json(json!({
                "from": {"email": "no-reply@example.com"},
                "personalizations": [{"to": [{"email": "notify@example.com"}]}],
            })))
            .respond_with(ResponseTemplate::new(202))
            .mount(&sendgrid_server)
            .await;

        let app = build_email_notification_app(
            NotificationEventType::OperationError,
            format!("{}/mail/send", sendgrid_server.uri()),
        );

        let event = ImageAuthenticationFailed {
            securities_account:
                ipo_backend_shared::domain::account::SecuritiesAccountIdentifier::generate(),
            failure_reason: "画像認証に失敗しました".to_string(),
            attempt_count: 1,
            occurred_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/ipo-notification")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "eventType": "ImageAuthenticationFailed",
                            "payload": serde_json::to_value(event).expect("event json"),
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn tests_account_connection_via_http() {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            InMemoryCredentialStore::new(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let account = build_account();
        let account_id = account.identifier().value().to_string();
        account_repository.save(&account).expect("save account");

        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository.clone(),
                notification_setting_repository,
                operation_log_repository,
                Arc::new(DummyBrowserPort),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/accounts/{account_id}/test"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "ok");

        let stored = account_repository
            .find_by_id(&SecuritiesAccountIdentifier::new(account_id).expect("account identifier"))
            .expect("find account")
            .expect("stored account");
        assert!(stored.connection_test().is_some());
    }

    #[tokio::test]
    async fn tests_account_connection_via_http_with_emulated_browser_service() {
        let browser_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/internal/accounts/test"))
            .and(body_partial_json(json!({
                "loginId": "login",
                "loginPassword": "password",
                "tradingPassword": "1234",
                "mailAddress": "test@example.com",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "message": "connected"
            })))
            .mount(&browser_server)
            .await;

        let stock_repository = Arc::new(FirestoreIpoStockRepository::new())
            as Arc<dyn IpoStockRepository + Send + Sync>;
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new())
            as Arc<dyn ExclusionRepository + Send + Sync>;
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new())
            as Arc<dyn LotteryApplicationRepository + Send + Sync>;
        let account_repository = Arc::new(FirestoreSecuritiesAccountRepository::new(
            InMemoryCredentialStore::new(),
        )) as Arc<dyn SecuritiesAccountRepository + Send + Sync>;
        let notification_setting_repository = Arc::new(FirestoreNotificationSettingRepository::new())
            as Arc<dyn NotificationSettingRepository + Send + Sync>;
        let operation_log_repository = Arc::new(FirestoreOperationLogRepository::new())
            as Arc<dyn OperationLogRepository + Send + Sync>;

        let account = build_account();
        let account_id = account.identifier().value().to_string();
        account_repository.save(&account).expect("save account");

        let app = create_router(
            DependencyContainer::from_components(
                stock_repository,
                exclusion_repository,
                application_repository,
                account_repository.clone(),
                notification_setting_repository,
                operation_log_repository,
                Arc::new(BrowserServiceClient::new(
                    reqwest::Client::new(),
                    browser_server.uri(),
                )),
                Arc::new(NotificationPortRegistry::new(
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(CaptureNotificationPort::default()),
                    Arc::new(CaptureNotificationPort::default()),
                )),
            )
            .expect("container"),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/accounts/{account_id}/test"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "connected");

        let stored = account_repository
            .find_by_id(&SecuritiesAccountIdentifier::new(account_id).expect("account identifier"))
            .expect("find account")
            .expect("stored account");
        assert_eq!(
            stored.connection_test().expect("connection test").message(),
            "connected"
        );
    }
}
