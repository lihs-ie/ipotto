use axum::{routing::post, Router};
use ipo_backend_shared::http::create_health_check_router;

use crate::{infrastructure::DependencyContainer, presentation::handlers};

/// Creates the applier router.
pub fn create_router(container: DependencyContainer) -> Router {
    Router::<DependencyContainer>::new()
        .merge(create_health_check_router::<DependencyContainer>())
        .route(
            "/internal/pubsub/apply",
            post(handlers::pubsub_handlers::handle_apply_message),
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
            application::{AppliedOrder, LotteryResult},
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockStatus, Yen,
            },
        },
        errors::DomainError,
        testing::{
            FirestoreExclusionRepository, FirestoreIpoStockRepository,
            FirestoreLotteryApplicationRepository, FirestoreOperationLogRepository,
            FirestoreSecuritiesAccountRepository, InMemoryCredentialStore, PubSubEventPublisher,
        },
    };
    use serde_json::Value;
    use tower::util::ServiceExt;

    use super::create_router;
    use crate::infrastructure::DependencyContainer;

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

    #[tokio::test]
    async fn handles_pubsub_apply_request_with_explicit_target_date() {
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-applier"));

        account_repository
            .save(&build_account())
            .await
            .expect("save account");
        stock_repository
            .save(&build_stock())
            .await
            .expect("save stock");

        let app = create_router(DependencyContainer::from_components(
            application_repository,
            stock_repository,
            account_repository,
            exclusion_repository,
            Arc::new(SucceedingBrowser),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/apply")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"targetDate":"2026-04-22"}"#))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["appliedCount"], 1);
        assert_eq!(json["skippedCount"], 0);
        assert_eq!(json["failedCount"], 0);

        let published = event_publisher
            .published_messages()
            .await
            .expect("messages");
        assert_eq!(published.len(), 1);
    }

    #[tokio::test]
    async fn handles_pubsub_apply_request_with_empty_body_defaults_to_today() {
        // When target_date is absent the handler defaults to "today", which in
        // tests is whatever UTC now returns. Because the fixture's book-building
        // period (2026-04-20 ~ 2026-04-24) almost certainly does not overlap
        // with the current test run date, the use case should simply produce an
        // empty result without failing — that is the contract we verify.
        let credential_store = InMemoryCredentialStore::new();
        let account_repository =
            Arc::new(FirestoreSecuritiesAccountRepository::new(credential_store));
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let application_repository = Arc::new(FirestoreLotteryApplicationRepository::new());
        let exclusion_repository = Arc::new(FirestoreExclusionRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-applier"));

        account_repository
            .save(&build_account())
            .await
            .expect("save account");
        stock_repository
            .save(&build_stock())
            .await
            .expect("save stock");

        let app = create_router(DependencyContainer::from_components(
            application_repository,
            stock_repository,
            account_repository,
            exclusion_repository,
            Arc::new(SucceedingBrowser),
            event_publisher,
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/apply")
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
        assert!(json["appliedCount"].is_number());
        assert!(json["skippedCount"].is_number());
        assert!(json["failedCount"].is_number());
    }
}
