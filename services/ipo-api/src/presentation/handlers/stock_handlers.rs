use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    application::use_cases::{
        GetDashboardSummaryUseCase, GetIpoStockInput, GetIpoStockOutput, GetIpoStockUseCase,
        ListIpoStocksInput, ListIpoStocksOutput, ListIpoStocksUseCase,
    },
    infrastructure::DependencyContainer,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpoStocksQuery {
    pub status: Option<String>,
}

pub async fn list_stocks(
    State(container): State<DependencyContainer>,
    Query(query): Query<ListIpoStocksQuery>,
) -> Result<Json<ListIpoStocksOutput>, crate::error::ApiError> {
    Ok(Json(
        ListIpoStocksUseCase::new(container.stock_repository()).execute(ListIpoStocksInput {
            status_filter: query.status,
        })?,
    ))
}

pub async fn get_stock(
    State(container): State<DependencyContainer>,
    Path(stock_id): Path<String>,
) -> Result<Json<GetIpoStockOutput>, crate::error::ApiError> {
    Ok(Json(
        GetIpoStockUseCase::new(
            container.stock_repository(),
            container.application_repository(),
            container.account_repository(),
        )
        .execute(GetIpoStockInput {
            stock_identifier: stock_id,
        })?,
    ))
}

pub async fn get_dashboard(
    State(container): State<DependencyContainer>,
) -> Result<Json<crate::application::use_cases::DashboardSummaryOutput>, crate::error::ApiError> {
    Ok(Json(
        GetDashboardSummaryUseCase::new(
            container.stock_repository(),
            container.application_repository(),
            container.account_repository(),
        )
        .execute()?,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        extract::{Path, Query, State},
        Json,
    };
    use chrono::{NaiveDate, Utc};
    use ipo_backend_shared::{
        acl::{browser::BrokerBrowserPort, notification::NotificationPort},
        domain::{
            account::{AccountCredential, ConnectionTestResult},
            stock::{
                BookBuildingPeriod, CompanyName, CompanyProfile, FetchOrigin, Industry,
                IpoOffering, IpoPricing, IpoSchedule, IpoStock, IpoStockRepository,
                LeadUnderwriter, Market, MetaSource, PriceRange, Shares, StockStatus, TickerSymbol,
                Yen,
            },
        },
        errors::DomainError,
        infrastructure::{
            firestore::repositories::{
                FirestoreExclusionRepository, FirestoreIpoStockRepository,
                FirestoreLotteryApplicationRepository, FirestoreNotificationSettingRepository,
                FirestoreOperationLogRepository, FirestoreSecuritiesAccountRepository,
            },
            notification::SlackNotificationAdapter,
            secrets::InMemoryCredentialStore,
        },
    };
    use reqwest::Client;

    use crate::infrastructure::{DependencyContainer, NotificationPortRegistry};

    use super::{get_dashboard, get_stock, list_stocks, ListIpoStocksQuery};

    #[derive(Debug)]
    struct DummyBrowserPort;

    #[async_trait]
    impl BrokerBrowserPort for DummyBrowserPort {
        async fn fetch_ipo_stocks(
            &self,
        ) -> Result<Vec<ipo_backend_shared::acl::scraping::ScrapedStock>, DomainError> {
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

    fn build_stock() -> IpoStock {
        IpoStock::create(
            CompanyProfile::new(
                CompanyName::new("テスト株式会社").expect("company"),
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
            MetaSource::new(FetchOrigin::ExternalSite, Utc::now()),
        )
        .expect("stock")
    }

    fn container(stock: IpoStock) -> DependencyContainer {
        let stock_repository = Arc::new(FirestoreIpoStockRepository::new());
        stock_repository.save(&stock).expect("save stock");
        DependencyContainer::from_components(
            stock_repository,
            Arc::new(FirestoreExclusionRepository::new()),
            Arc::new(FirestoreLotteryApplicationRepository::new()),
            Arc::new(FirestoreSecuritiesAccountRepository::new(
                InMemoryCredentialStore::new(),
            )),
            Arc::new(FirestoreNotificationSettingRepository::new()),
            Arc::new(FirestoreOperationLogRepository::new()),
            Arc::new(DummyBrowserPort),
            Arc::new(NotificationPortRegistry::new(
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
                Arc::new(SlackNotificationAdapter::new(Client::new()))
                    as Arc<dyn NotificationPort + Send + Sync>,
            )),
        )
        .expect("container")
    }

    #[tokio::test]
    async fn lists_stocks_and_returns_dashboard_summary() {
        let container = container(build_stock());

        let Json(output) = list_stocks(
            State(container.clone()),
            Query(ListIpoStocksQuery {
                status: Some("Eligible".to_string()),
            }),
        )
        .await
        .expect("list stocks");
        assert_eq!(output.total_count, 1);

        let Json(dashboard) = get_dashboard(State(container)).await.expect("dashboard");
        assert_eq!(dashboard.status_counts.get("Eligible"), Some(&1));
    }

    #[tokio::test]
    async fn gets_a_stock_by_identifier() {
        let stock = build_stock();
        let identifier = stock.identifier().value().to_string();
        let container = container(stock);

        let Json(output) = get_stock(State(container), Path(identifier))
            .await
            .expect("get stock");
        assert_eq!(output.company_profile.company_name, "テスト株式会社");
    }
}
