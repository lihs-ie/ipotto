use axum::{routing::post, Router};
use ipo_backend_shared::http::create_health_check_router;

use crate::{infrastructure::DependencyContainer, presentation::handlers};

/// Creates the result checker router.
pub fn create_router(container: DependencyContainer) -> Router {
    Router::<DependencyContainer>::new()
        .merge(create_health_check_router::<DependencyContainer>())
        .route(
            "/internal/pubsub/check",
            post(handlers::pubsub_handlers::handle_check_message),
        )
        .with_state(container)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use chrono::{NaiveDate, TimeZone, Utc};
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, scraping::ScrapedStock},
        domain::{
            account::{
                AccountCredential, ConnectionTestResult, ImapHost, ImapPort, LoginId,
                LoginPassword, MailAddress, MailCredential, MailPassword, SecuritiesAccount,
                SecuritiesAccountRepository, SecuritiesCompany, TradingPassword,
            },
            application::{
                ApplicationStatus, LotteryApplication, LotteryApplicationRepository, LotteryResult,
            },
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockStatus, Yen,
            },
        },
        errors::DomainError,
        testing::{
            FirestoreIpoStockRepository, FirestoreLotteryApplicationRepository,
            FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            InMemoryCredentialStore, PubSubEventPublisher,
        },
    };
    use serde_json::Value;
    use tower::util::ServiceExt;
    use wiremock::{
        matchers::{body_partial_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::create_router;
    use crate::infrastructure::{BrowserServiceClient, DependencyContainer};

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
    async fn handles_pubsub_check_request() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-result-checker"));

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

        account_repository.save(&account).expect("save account");
        stock_repository.save(&stock).expect("save stock");
        application_repository
            .save(&application)
            .expect("save application");

        let app = create_router(DependencyContainer::from_components(
            application_repository.clone(),
            stock_repository,
            account_repository,
            Arc::new(WinningBrowser),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/check")
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
        assert_eq!(json["checkedCount"], 1);
        assert_eq!(
            application_repository
                .find_by_id(application.identifier())
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
    }

    #[tokio::test]
    async fn handles_pubsub_check_request_with_emulated_browser_service() {
        let browser_server = MockServer::start().await;
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-result-checker"));

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

        Mock::given(method("POST"))
            .and(path("/internal/lottery-results/check"))
            .and(body_partial_json(serde_json::json!({
                "stockIdentifier": stock.identifier().value(),
                "credential": {
                    "loginId": "login",
                    "loginPassword": "password",
                    "tradingPassword": "1234"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": "Won"
            })))
            .mount(&browser_server)
            .await;

        account_repository.save(&account).expect("save account");
        stock_repository.save(&stock).expect("save stock");
        application_repository
            .save(&application)
            .expect("save application");

        let app = create_router(DependencyContainer::from_components(
            application_repository.clone(),
            stock_repository,
            account_repository,
            Arc::new(BrowserServiceClient::new(
                reqwest::Client::new(),
                browser_server.uri(),
            )),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/check")
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
        assert_eq!(json["checkedCount"], 1);
        assert_eq!(
            application_repository
                .find_by_id(application.identifier())
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
    }

    #[tokio::test]
    async fn continues_without_publish_when_browser_service_fails() {
        let browser_server = MockServer::start().await;
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-result-checker"));

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

        Mock::given(method("POST"))
            .and(path("/internal/lottery-results/check"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&browser_server)
            .await;

        account_repository.save(&account).expect("save account");
        stock_repository.save(&stock).expect("save stock");
        application_repository
            .save(&application)
            .expect("save application");

        let app = create_router(DependencyContainer::from_components(
            application_repository.clone(),
            stock_repository,
            account_repository,
            Arc::new(BrowserServiceClient::new(
                reqwest::Client::new(),
                browser_server.uri(),
            )),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/check")
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
        assert_eq!(json["checkedCount"], 0);
        assert_eq!(json["results"], serde_json::json!([]));
        assert_eq!(
            application_repository
                .find_by_id(application.identifier())
                .expect("find application")
                .expect("application")
                .status(),
            ApplicationStatus::Applied
        );
        assert_eq!(
            event_publisher
                .published_messages()
                .await
                .expect("messages")
                .len(),
            0
        );
    }
}
