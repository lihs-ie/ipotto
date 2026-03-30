use axum::{routing::post, Router};
use ipo_backend_shared::http::create_health_check_router;

use crate::{infrastructure::DependencyContainer, presentation::handlers};

/// Creates the info fetcher router.
pub fn create_router(container: DependencyContainer) -> Router {
    Router::<DependencyContainer>::new()
        .merge(create_health_check_router::<DependencyContainer>())
        .route(
            "/internal/pubsub/fetch",
            post(handlers::pubsub_handlers::handle_fetch_message),
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
    use chrono::NaiveDate;
    use ipo_backend_shared::{
        acl::scraping::{IpoStockScraperPort, ScrapedStock},
        domain::stock::IpoStockRepository,
        testing::{
            FirestoreIpoStockRepository, FirestoreOperationLogRepository, PubSubEventPublisher,
        },
    };
    use serde_json::Value;
    use tower::util::ServiceExt;

    use super::create_router;
    use crate::infrastructure::DependencyContainer;

    #[derive(Debug)]
    struct StaticScraper {
        stocks: Vec<ScrapedStock>,
    }

    #[async_trait]
    impl IpoStockScraperPort for StaticScraper {
        async fn scrape(
            &self,
        ) -> Result<Vec<ScrapedStock>, ipo_backend_shared::errors::DomainError> {
            Ok(self.stocks.clone())
        }
    }

    #[tokio::test]
    async fn handles_pubsub_fetch_request() {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        let event_publisher = Arc::new(PubSubEventPublisher::new("ipo-info-fetcher"));
        let app = create_router(DependencyContainer::from_components(
            stock_repository.clone(),
            Arc::new(StaticScraper {
                stocks: vec![ScrapedStock::new(
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
                )],
            }),
            event_publisher.clone(),
            Arc::new(FirestoreOperationLogRepository::new()),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/pubsub/fetch")
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
        assert_eq!(json["fetchedCount"], 1);
        assert_eq!(json["updatedCount"], 0);
        assert_eq!(stock_repository.find_all().expect("find all").len(), 1);
        assert_eq!(
            event_publisher
                .published_messages()
                .await
                .expect("messages")
                .len(),
            1
        );
    }
}
